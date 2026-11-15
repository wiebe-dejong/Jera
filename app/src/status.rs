//! De status van een sessie, gelezen uit `~/.claude/sessions/<pid>.json` van Claude Code.
//! Het formaat staat niet in de documentatie; de meting staat in `research/zelf-bouwen.md`.

use std::{collections::HashMap, path::PathBuf, process::Command};

/// Van dringend naar niet dringend.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Status {
    Toestemming,
    Antwoord,
    NetKlaar,
    Bezig,
    Idle,
}

impl Status {
    /// Wacht de sessie op mij.
    pub fn wacht(self) -> bool {
        matches!(self, Status::Antwoord | Status::Toestemming | Status::NetKlaar)
    }

    /// Claude werkt, of wacht midden in het werk op mij.
    pub fn bezig(self) -> bool {
        matches!(self, Status::Bezig | Status::Antwoord | Status::Toestemming)
    }

    /// Het woord van de status, voor onder de muis.
    pub fn woord(self) -> &'static str {
        match self {
            Status::Bezig => "bezig",
            Status::Antwoord => "wacht op antwoord",
            Status::Toestemming => "wacht op toestemming",
            Status::NetKlaar => "net klaar",
            Status::Idle => "idle",
        }
    }
}

/// Wat het sessiebestand zegt. Of ik een klare sessie al gezien heb, weet alleen Jera.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Gelezen {
    Bezig,
    Antwoord,
    Toestemming,
    Klaar,
}

/// Eén sessiebestand: de stand, en waar de agents van de sessie staan.
pub struct Bestand {
    pub gelezen: Gelezen,
    pub id: String,
    pub cwd: PathBuf,
}

/// Leest de sessiebestanden. De sleutel is het pid van de ouder van Claude,
/// bij een sessie van Jera de shell die de terminal start.
#[derive(Default)]
pub struct Lezer {
    ouders: HashMap<u32, u32>,
}

impl Lezer {
    pub fn lees(&mut self) -> HashMap<u32, Bestand> {
        let mut uit = HashMap::new();
        let Ok(map) = std::fs::read_dir(sessiemap()) else { return uit };
        for bestand in map.flatten() {
            let pad = bestand.path();
            if pad.extension().is_none_or(|e| e != "json") {
                continue;
            }
            let Ok(tekst) = std::fs::read_to_string(&pad) else { continue };
            let Ok(json) = serde_json::from_str::<serde_json::Value>(&tekst) else { continue };
            let Some(pid) = json["pid"].as_u64().map(|p| p as u32) else { continue };
            let gelezen = match (json["status"].as_str(), json["waitingFor"].as_str()) {
                (Some("idle"), _) => Gelezen::Klaar,
                (Some("waiting"), Some("permission prompt")) => Gelezen::Toestemming,
                (Some("waiting"), _) => Gelezen::Antwoord,
                // busy, en shell als Claude een commando draait.
                _ => Gelezen::Bezig,
            };
            if let Some(ouder) = self.ouder(pid) {
                let id = json["sessionId"].as_str().unwrap_or_default().to_string();
                let cwd = PathBuf::from(json["cwd"].as_str().unwrap_or_default());
                uit.insert(ouder, Bestand { gelezen, id, cwd });
            }
        }
        uit
    }

    /// De ouder van `pid`. `ps` draait alleen als er een onbekend pid bij komt.
    fn ouder(&mut self, pid: u32) -> Option<u32> {
        if !self.ouders.contains_key(&pid) {
            let uit = Command::new("ps").args(["-A", "-o", "pid=,ppid="]).output().ok()?;
            self.ouders = String::from_utf8_lossy(&uit.stdout)
                .lines()
                .filter_map(|regel| {
                    let mut delen = regel.split_whitespace().map(|d| d.parse().ok());
                    Some((delen.next()??, delen.next()??))
                })
                .collect();
        }
        self.ouders.get(&pid).copied()
    }
}

fn sessiemap() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".claude/sessions")
}
