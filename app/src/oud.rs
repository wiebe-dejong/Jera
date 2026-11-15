//! De oude sessies van een tab, uit `~/.claude/projects/<map>/<sessie-id>.jsonl` van Claude Code.
//! `claude --resume <sessie-id>` start er een opnieuw. Zie `research/sessies.md`.

use std::{
    collections::HashSet,
    io::{BufRead, BufReader, Read},
    path::{Path, PathBuf},
    time::SystemTime,
};

use crate::claude;

pub struct Oud {
    pub id: String,
    pub map: PathBuf,
    pub titel: String,
    /// Het laatste bericht.
    pub tijd: SystemTime,
}

/// Zo ver lees ik in een gesprek naar de titel en de map. Een gesprek kan tientallen MB zijn.
const KOP: u64 = 2 << 20;

/// De oude sessies onder `root`, de nieuwste eerst. Sessies die nu draaien en sessies
/// zonder titel staan er niet in.
pub fn lees(root: &Path) -> Vec<Oud> {
    // De sessies die nu draaien, in Jera of elders.
    let draait: HashSet<String> =
        claude::sessies().iter().filter_map(|json| json["sessionId"].as_str().map(str::to_string)).collect();
    // De map van een submap begint met die van `root`; de map zelf staat in het gesprek.
    let code = claude::projectcode(root);
    let mut uit = Vec::new();
    let Ok(projecten) = std::fs::read_dir(claude::map().join("projects")) else { return uit };
    for project in projecten.flatten() {
        if !project.file_name().to_string_lossy().starts_with(&code) {
            continue;
        }
        let Ok(gesprekken) = std::fs::read_dir(project.path()) else { continue };
        for gesprek in gesprekken.flatten() {
            let pad = gesprek.path();
            if pad.extension().is_none_or(|e| e != "jsonl") {
                continue;
            }
            let id = pad.file_stem().unwrap_or_default().to_string_lossy().into_owned();
            if draait.contains(&id) {
                continue;
            }
            let Some((map, titel)) = kop(&pad) else { continue };
            if !map.starts_with(root) {
                continue;
            }
            let tijd = gesprek.metadata().and_then(|m| m.modified()).unwrap_or(SystemTime::UNIX_EPOCH);
            uit.push(Oud { id, map, titel, tijd });
        }
    }
    uit.sort_by(|a, b| b.tijd.cmp(&a.tijd));
    uit
}

/// De map en de titel uit het begin van een gesprek.
fn kop(pad: &Path) -> Option<(PathBuf, String)> {
    let bestand = std::fs::File::open(pad).ok()?;
    let (mut map, mut titel) = (None, None);
    for regel in BufReader::new(bestand.take(KOP)).lines() {
        let Ok(regel) = regel else { break };
        // Een regel met `cwd` als JSON lezen: in de tekst kan een pad escapes hebben.
        if map.is_none() && regel.contains("\"cwd\":") {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&regel) {
                map = json["cwd"].as_str().map(PathBuf::from);
            }
        }
        if titel.is_none() && regel.contains("\"ai-title\"") {
            let json: serde_json::Value = serde_json::from_str(&regel).ok()?;
            titel = json["aiTitle"].as_str().map(str::to_string);
        }
        if map.is_some() && titel.is_some() {
            break;
        }
    }
    Some((map?, titel?))
}
