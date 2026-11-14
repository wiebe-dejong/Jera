//! De agents van een sessie: subagents en teamleden, uit `~/.claude/projects/<map>/<sessie-id>/subagents/`
//! en `~/.claude/teams/<team>/config.json` van Claude Code. Het formaat staat niet in de
//! documentatie; de meting staat in `research/agents.md`.

use std::{
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    time::SystemTime,
};

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Stand {
    Bezig,
    Klaar,
    Gestopt,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Agent {
    /// Van een teamlid de naam, van een subagent de omschrijving.
    pub tekst: String,
    /// Van een teamlid de plek van zijn kleur in `Thema::team`.
    pub kleur: Option<usize>,
    pub stand: Stand,
}

/// De kleuren die Claude Code een teamlid geeft, in de volgorde van `Thema::team`.
const KLEUREN: [&str; 8] = ["blue", "green", "yellow", "purple", "orange", "pink", "cyan", "red"];

/// Zo ver van het eind lees ik in een gesprek naar de laatste regel.
const STAART: u64 = 16 << 10;

/// De agents van sessie `id` die in `cwd` is gestart, in de volgorde van starten.
pub fn lees(id: &str, cwd: &Path) -> Vec<Agent> {
    let claude = PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".claude");
    // Claude Code noemt de map naar het pad, met een streepje voor elk ander teken dan een letter of cijfer.
    let code: String = cwd.to_string_lossy().chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '-' }).collect();
    let map = claude.join("projects").join(code).join(id).join("subagents");
    let Ok(bestanden) = std::fs::read_dir(&map) else { return Vec::new() };
    let mut leden: Option<Vec<String>> = None;
    let mut uit: Vec<(SystemTime, Agent)> = Vec::new();
    for bestand in bestanden.flatten() {
        let pad = bestand.path();
        let Some(stam) = pad.file_name().and_then(|n| n.to_str()).and_then(|n| n.strip_suffix(".meta.json")) else {
            continue;
        };
        let Ok(tekst) = std::fs::read_to_string(&pad) else { continue };
        let Ok(meta) = serde_json::from_str::<serde_json::Value>(&tekst) else { continue };
        let gesprek = map.join(format!("{stam}.jsonl"));
        let agent = match meta["name"].as_str() {
            Some(naam) => {
                // Een teamlid dat gestopt is, staat niet meer in het team.
                let leden = leden.get_or_insert_with(|| team(&claude, meta["teamName"].as_str().unwrap_or_default()));
                let stand = if leden.iter().any(|l| l == naam) { laatste(&gesprek) } else { Stand::Gestopt };
                let kleur = meta["color"].as_str().and_then(|k| KLEUREN.iter().position(|c| *c == k));
                Agent { tekst: naam.to_string(), kleur, stand }
            }
            None => {
                // Een subagent kan zijn beurt afsluiten terwijl zijn eigen achtergrondtaak nog loopt.
                // Klaar is hij pas met `finished`.
                let stand = match (meta["finished"].as_bool() == Some(true), laatste(&gesprek)) {
                    (true, _) => Stand::Klaar,
                    (false, Stand::Gestopt) => Stand::Gestopt,
                    (false, _) => Stand::Bezig,
                };
                Agent { tekst: meta["description"].as_str().unwrap_or_default().to_string(), kleur: None, stand }
            }
        };
        // Het gesprek groeit alleen aan; `meta.json` schrijft Claude Code opnieuw als een agent klaar is.
        let start = std::fs::metadata(&gesprek).and_then(|m| m.created()).unwrap_or(SystemTime::UNIX_EPOCH);
        uit.push((start, agent));
    }
    uit.sort_by_key(|(start, _)| *start);
    uit.into_iter().map(|(_, agent)| agent).collect()
}

/// De namen in het team, zonder `team-lead`.
fn team(claude: &Path, naam: &str) -> Vec<String> {
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

/// De stand uit de laatste regel van het gesprek. Een regel kan langer zijn dan wat ik lees,
/// maar `type` en `stop_reason` staan aan het eind.
fn laatste(gesprek: &Path) -> Stand {
    let Ok(mut bestand) = std::fs::File::open(gesprek) else { return Stand::Bezig };
    let lengte = bestand.metadata().map(|m| m.len()).unwrap_or(0);
    let _ = bestand.seek(SeekFrom::Start(lengte.saturating_sub(STAART)));
    let mut staart = Vec::new();
    let _ = bestand.read_to_end(&mut staart);
    let staart = String::from_utf8_lossy(&staart);
    let regel = staart.trim_end().rsplit('\n').next().unwrap_or_default();
    if regel.contains("\"type\":\"assistant\"") && regel.contains("\"stop_reason\":\"end_turn\"") {
        Stand::Klaar
    } else if regel.contains("\"type\":\"user\"") && regel.contains("[Request interrupted by user") {
        Stand::Gestopt
    } else {
        Stand::Bezig
    }
}
