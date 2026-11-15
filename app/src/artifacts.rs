//! De artifacts die een sessie publiceerde, uit haar gesprek in
//! `~/.claude/projects/<map>/<sessie-id>.jsonl`. Claude Code zet het resultaat van de tool
//! Artifact in `toolUseResult`: bij publish en open met `url` en `title`, bij read alleen de link.
//! Gelezen artifacts tellen niet mee.

use std::{
    collections::HashMap,
    io::{Read, Seek, SeekFrom},
    path::PathBuf,
    sync::Mutex,
};

use crate::claude;

#[derive(Clone, PartialEq, Debug)]
pub struct Artifact {
    pub url: String,
    pub titel: String,
}

/// Per gesprek hoe ver het gelezen is en wat erin stond, zodat elke ronde alleen het nieuwe leest.
static GELEZEN: Mutex<Vec<(PathBuf, u64, Vec<Artifact>)>> = Mutex::new(Vec::new());

/// Per sessie-id de artifacts van de sessies die nu draaien, in de volgorde van de eerste keer.
pub fn lees() -> HashMap<String, Vec<Artifact>> {
    let mut gelezen = GELEZEN.lock().unwrap();
    let mut uit = HashMap::new();
    let mut nu = Vec::new();
    for json in claude::sessies() {
        let (Some(id), Some(cwd)) = (json["sessionId"].as_str(), json["cwd"].as_str()) else { continue };
        let pad = claude::map().join("projects").join(claude::projectcode(cwd.as_ref())).join(format!("{id}.jsonl"));
        let (mut tot, mut lijst) = match gelezen.iter().position(|(p, ..)| *p == pad) {
            Some(i) => {
                let (_, tot, lijst) = gelezen.swap_remove(i);
                (tot, lijst)
            }
            None => (0, Vec::new()),
        };
        tot = lees_vanaf(&pad, tot, &mut lijst);
        if !lijst.is_empty() {
            uit.insert(id.to_string(), lijst.clone());
        }
        nu.push((pad, tot, lijst));
    }
    *gelezen = nu;
    uit
}

/// Leest het gesprek vanaf `tot` tot de laatste hele regel, en geeft terug hoe ver het nu is.
fn lees_vanaf(pad: &PathBuf, tot: u64, lijst: &mut Vec<Artifact>) -> u64 {
    let Ok(mut bestand) = std::fs::File::open(pad) else { return tot };
    // Korter dan gelezen: het bestand is opnieuw begonnen.
    let (tot, opnieuw) = match bestand.metadata().map(|m| m.len()).unwrap_or(0) < tot {
        true => (0, true),
        false => (tot, false),
    };
    if opnieuw {
        lijst.clear();
    }
    let mut tekst = Vec::new();
    if bestand.seek(SeekFrom::Start(tot)).is_err() || bestand.read_to_end(&mut tekst).is_err() {
        return tot;
    }
    let Some(einde) = tekst.iter().rposition(|b| *b == b'\n') else { return tot };
    for regel in tekst[..einde].split(|b| *b == b'\n') {
        voeg_toe(regel, lijst);
    }
    tot + einde as u64 + 1
}

/// Neemt een artifact uit één regel van het gesprek op. Een nieuwe versie telt niet opnieuw,
/// maar zijn titel wint.
fn voeg_toe(regel: &[u8], lijst: &mut Vec<Artifact>) {
    let teken = b"\"toolUseResult\"";
    if !regel.windows(teken.len()).any(|w| w == teken) {
        return;
    }
    let Ok(json) = serde_json::from_slice::<serde_json::Value>(regel) else { return };
    let r = &json["toolUseResult"];
    let (Some(url), Some(titel)) = (r["url"].as_str(), r["title"].as_str()) else { return };
    if !url.starts_with("https://claude.ai/") || r.get("read").is_some() {
        return;
    }
    match lijst.iter_mut().find(|a| a.url == url) {
        Some(a) => a.titel = titel.to_string(),
        None => lijst.push(Artifact { url: url.to_string(), titel: titel.to_string() }),
    }
}

/// Opent het artifact in de standaardbrowser.
pub fn open(url: &str) {
    let _ = std::process::Command::new("open").arg(url).status();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lijst(regels: &[&str]) -> Vec<Artifact> {
        let mut uit = Vec::new();
        regels.iter().for_each(|r| voeg_toe(r.as_bytes(), &mut uit));
        uit
    }

    #[test]
    fn gepubliceerd_en_geopend_tellen_gelezen_niet() {
        let uit = lijst(&[
            r#"{"type":"user","toolUseResult":{"url":"https://claude.ai/artifact/A","title":"Uitnodigingsmail","updated":false,"artifact_id":"x"}}"#,
            r#"{"type":"user","toolUseResult":{"url":"https://claude.ai/artifact/A","title":"Uitnodigingsmail, ronde 2","updated":true}}"#,
            r#"{"type":"user","toolUseResult":{"opened":true,"url":"https://claude.ai/artifact/B","title":"Stand"}}"#,
            r#"{"type":"user","toolUseResult":{"read":{"code":200,"url":"https://claude.ai/artifact/C"}}}"#,
            r#"{"type":"user","toolUseResult":null}"#,
            r#"{"type":"assistant","message":{"content":"https://claude.ai/artifact/D"}}"#,
        ]);
        assert_eq!(
            uit,
            vec![
                Artifact { url: "https://claude.ai/artifact/A".into(), titel: "Uitnodigingsmail, ronde 2".into() },
                Artifact { url: "https://claude.ai/artifact/B".into(), titel: "Stand".into() },
            ]
        );
    }

    #[test]
    fn leest_alleen_hele_regels_en_daarna_het_nieuwe() {
        let pad = std::env::temp_dir().join(format!("jera-artifacts-{}.jsonl", std::process::id()));
        let a = r#"{"toolUseResult":{"url":"https://claude.ai/artifact/A","title":"A"}}"#;
        let b = r#"{"toolUseResult":{"url":"https://claude.ai/artifact/B","title":"B"}}"#;
        std::fs::write(&pad, format!("{a}\n{}", &b[..20])).unwrap();
        let mut uit = Vec::new();
        let tot = lees_vanaf(&pad, 0, &mut uit);
        assert_eq!(uit.len(), 1);
        std::fs::write(&pad, format!("{a}\n{b}\n")).unwrap();
        lees_vanaf(&pad, tot, &mut uit);
        assert_eq!(uit.iter().map(|a| a.titel.as_str()).collect::<Vec<_>>(), ["A", "B"]);
        let _ = std::fs::remove_file(&pad);
    }
}
