//! De agents van een sessie: subagents en teamleden, uit `~/.claude/projects/<map>/<sessie-id>/subagents/`
//! en `~/.claude/teams/<team>/config.json` van Claude Code. Het formaat staat niet in de
//! documentatie; de meting staat in `research/agents.md`.

use std::{
    collections::HashMap,
    io::{Read, Seek, SeekFrom},
    path::Path,
    time::SystemTime,
};

use crate::claude;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Stand {
    Bezig,
    Klaar,
    Gestopt,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Agent {
    /// De naam van de agent, of zonder naam zijn omschrijving.
    pub tekst: String,
    /// De omschrijving die de agent bij zijn start kreeg.
    pub opdracht: String,
    /// Van een teamlid de plek van zijn kleur in `Thema::team`.
    pub kleur: Option<usize>,
    pub stand: Stand,
    /// Van een agent die bezig is de laatste tool met zijn doel, zoals `Read agents.rs`.
    pub bezig_met: Option<String>,
}

/// De kleuren die Claude Code een teamlid geeft, in de volgorde van `Thema::team`.
const KLEUREN: [&str; 8] = ["blue", "green", "yellow", "purple", "orange", "pink", "cyan", "red"];

/// Zo ver van het eind lees ik in een gesprek naar de laatste regel.
const STAART: u64 = 16 << 10;

/// Zo ver van het eind lees ik in het gesprek van een agent die bezig is naar zijn laatste tool.
/// Een regel met een tool die een bestand schrijft, kan lang zijn.
const STAART_TOOL: u64 = 64 << 10;

/// De agents van sessie `id` die in `cwd` is gestart, in de volgorde van starten: de subagents
/// die bezig zijn en de teamleden die nog in hun team staan. Een subagent die klaar of gestopt is, valt weg.
pub fn lees(id: &str, cwd: &Path) -> Vec<Agent> {
    let claude = claude::map();
    let map = claude.join("projects").join(claude::projectcode(cwd)).join(id).join("subagents");
    let Ok(bestanden) = std::fs::read_dir(&map) else { return Vec::new() };
    // Per team de namen die er nog in staan. Een sessie kan naast haar eigen team een ander team hebben.
    let mut teams: HashMap<String, Vec<String>> = HashMap::new();
    let mut uit: Vec<(SystemTime, Agent)> = Vec::new();
    for bestand in bestanden.flatten() {
        let pad = bestand.path();
        let Some(stam) = pad.file_name().and_then(|n| n.to_str()).and_then(|n| n.strip_suffix(".meta.json")) else {
            continue;
        };
        let Ok(tekst) = std::fs::read_to_string(&pad) else { continue };
        let Ok(meta) = serde_json::from_str::<serde_json::Value>(&tekst) else { continue };
        let gesprek = map.join(format!("{stam}.jsonl"));
        let opdracht = meta["description"].as_str().unwrap_or_default().to_string();
        // Ook een subagent kan een naam hebben. Een teamlid herken ik aan zijn team.
        let tekst = meta["name"].as_str().map_or_else(|| opdracht.clone(), str::to_string);
        let (kleur, stand) = match meta["teamName"].as_str() {
            Some(team) => {
                // Een teamlid dat gestopt is, staat niet meer in het team.
                let leden = teams.entry(team.to_string()).or_insert_with(|| leden(&claude, team));
                if !leden.contains(&tekst) {
                    continue;
                }
                (meta["color"].as_str().and_then(|k| KLEUREN.iter().position(|c| *c == k)), laatste(&gesprek))
            }
            None => {
                // Een subagent kan zijn beurt afsluiten terwijl zijn eigen achtergrondtaak nog loopt.
                // Klaar is hij pas met `finished`.
                if meta["finished"].as_bool() == Some(true) || laatste(&gesprek) == Stand::Gestopt {
                    continue;
                }
                (None, Stand::Bezig)
            }
        };
        let bezig_met = (stand == Stand::Bezig).then(|| tool(&staart(&gesprek, STAART_TOOL))).flatten();
        // Het gesprek groeit alleen aan; `meta.json` schrijft Claude Code opnieuw als een agent klaar is.
        let start = std::fs::metadata(&gesprek).and_then(|m| m.created()).unwrap_or(SystemTime::UNIX_EPOCH);
        uit.push((start, Agent { tekst, opdracht, kleur, stand, bezig_met }));
    }
    uit.sort_by_key(|(start, _)| *start);
    uit.into_iter().map(|(_, agent)| agent).collect()
}

/// De namen in team `naam`, zonder `team-lead`.
fn leden(claude: &Path, naam: &str) -> Vec<String> {
    let Ok(tekst) = std::fs::read_to_string(claude.join("teams").join(naam).join("config.json")) else {
        return Vec::new();
    };
    let Ok(config) = serde_json::from_str::<serde_json::Value>(&tekst) else { return Vec::new() };
    config["members"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|lid| lid["name"].as_str())
        .filter(|naam| *naam != "team-lead")
        .map(str::to_string)
        .collect()
}

/// De laatste `lengte` bytes van het gesprek.
fn staart(gesprek: &Path, lengte: u64) -> String {
    let Ok(mut bestand) = std::fs::File::open(gesprek) else { return String::new() };
    let totaal = bestand.metadata().map(|m| m.len()).unwrap_or(0);
    let _ = bestand.seek(SeekFrom::Start(totaal.saturating_sub(lengte)));
    let mut staart = Vec::new();
    let _ = bestand.read_to_end(&mut staart);
    String::from_utf8_lossy(&staart).into_owned()
}

/// De stand uit de laatste regel van het gesprek. Een regel kan langer zijn dan wat ik lees,
/// maar `type` en `stop_reason` staan aan het eind.
fn laatste(gesprek: &Path) -> Stand {
    if !gesprek.exists() {
        return Stand::Bezig;
    }
    let staart = staart(gesprek, STAART);
    let regel = staart.trim_end().rsplit('\n').next().unwrap_or_default();
    if regel.contains("\"type\":\"assistant\"") && regel.contains("\"stop_reason\":\"end_turn\"") {
        Stand::Klaar
    } else if regel.contains("\"type\":\"user\"") && regel.contains("[Request interrupted by user") {
        Stand::Gestopt
    } else {
        Stand::Bezig
    }
}

/// De laatste tool in de staart van een gesprek, met zijn doel: `Read agents.rs`.
/// Een regel die niet helemaal in de staart past, sla ik over.
fn tool(staart: &str) -> Option<String> {
    staart.lines().rev().find_map(|regel| {
        let regel: serde_json::Value = serde_json::from_str(regel).ok()?;
        if regel["type"] != "assistant" {
            return None;
        }
        let gebruik = regel["message"]["content"].as_array()?.iter().rev().find(|c| c["type"] == "tool_use")?;
        let naam = gebruik["name"].as_str()?;
        let invoer = &gebruik["input"];
        let doel = match naam {
            "Read" | "Edit" | "Write" | "NotebookEdit" => invoer["file_path"]
                .as_str()
                .or(invoer["notebook_path"].as_str())
                .map(|pad| Path::new(pad).file_name().map_or(pad.into(), |n| n.to_string_lossy())),
            "Bash" => invoer["description"].as_str().or(invoer["command"].as_str()).map(Into::into),
            "Grep" | "Glob" => invoer["pattern"].as_str().map(Into::into),
            "WebFetch" => invoer["url"].as_str().map(Into::into),
            "WebSearch" => invoer["query"].as_str().map(Into::into),
            "Agent" | "Task" => invoer["description"].as_str().map(Into::into),
            _ => None,
        };
        Some(match doel {
            Some(doel) => format!("{naam} {}", doel.lines().next().unwrap_or_default()),
            None => naam.to_string(),
        })
    })
}

#[cfg(test)]
mod tests {
    use super::tool;

    #[test]
    fn tool_met_doel() {
        let staart = concat!(
            r#"{"type":"assistant","message":{"content":[{"type":"tool_use","name":"Read","input":{"file_path":"/a/b/agents.rs"}}]}}"#,
            "\n",
            r#"{"type":"user","message":{"content":[{"type":"tool_result","content":"..."}]}}"#,
            "\n",
            r#"{"type":"api-request"}"#,
        );
        assert_eq!(tool(staart).as_deref(), Some("Read agents.rs"));
    }

    #[test]
    fn laatste_tool_telt() {
        let staart = concat!(
            r#"{"type":"assistant","message":{"content":[{"type":"tool_use","name":"Grep","input":{"pattern":"status"}}]}}"#,
            "\n",
            r#"{"type":"assistant","message":{"content":[{"type":"tool_use","name":"Bash","input":{"command":"cargo test\necho klaar","description":"Run tests"}}]}}"#,
        );
        assert_eq!(tool(staart).as_deref(), Some("Bash Run tests"));
    }

    #[test]
    fn afgekapte_regel_en_onbekende_tool() {
        // De eerste regel begint midden in, zoals aan het begin van een staart.
        let staart = concat!(
            r#"nt":[{"type":"tool_use","name":"Read","input":{"file_path":"x"}}]}}"#,
            "\n",
            r#"{"type":"assistant","message":{"content":[{"type":"tool_use","name":"mcp__x__y","input":{}}]}}"#,
            "\n",
            r#"{"type":"assistant","message":{"content":[{"type":"text","text":"Klaar."}]}}"#,
        );
        assert_eq!(tool(staart).as_deref(), Some("mcp__x__y"));
        assert_eq!(tool(r#"{"type":"assistant","message":{"content":[{"type":"text","text":"Hoi"}]}}"#), None);
    }
}
