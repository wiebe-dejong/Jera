//! Waar Claude Code zijn bestanden bewaart. Het formaat staat niet in de documentatie;
//! de metingen staan in `research/sessies.md` en `research/agents.md`.

use std::path::{Path, PathBuf};

/// `~/.claude`.
pub fn map() -> PathBuf {
    crate::thuis().join(".claude")
}

/// De naam van de map onder `~/.claude/projects` voor `pad`: Claude Code zet een streepje
/// voor elk ander teken dan een letter of cijfer. Dat is niet terug te rekenen.
pub fn projectcode(pad: &Path) -> String {
    pad.to_string_lossy().chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '-' }).collect()
}

/// De sessiebestanden in `~/.claude/sessions`: één per sessie die nu draait, in Jera of elders.
pub fn sessies() -> Vec<serde_json::Value> {
    let Ok(bestanden) = std::fs::read_dir(map().join("sessions")) else { return Vec::new() };
    bestanden
        .flatten()
        .map(|bestand| bestand.path())
        .filter(|pad| pad.extension().is_some_and(|e| e == "json"))
        .filter_map(|pad| std::fs::read_to_string(pad).ok())
        .filter_map(|tekst| serde_json::from_str(&tekst).ok())
        .collect()
}
