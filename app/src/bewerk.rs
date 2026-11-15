//! Wat het menu in de bestandsboom met bestanden en mappen doet.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::taal::t;

/// Zet `pad` op het klembord van macOS, zoals Kopieer in Finder.
pub fn kopieer(pad: &Path) {
    let _ = Command::new("osascript")
        .args(["-l", "JavaScript", "-e", KOPIEER])
        .arg(pad)
        .status();
}

const KOPIEER: &str = r#"function run(argv) {
    ObjC.import("AppKit");
    const pb = $.NSPasteboard.generalPasteboard;
    pb.clearContents;
    pb.writeObjects($(argv.map(p => $.NSURL.fileURLWithPath(p))));
}"#;

/// Kopieert `bronnen` naar `map`. Een naam die er al is, krijgt "kopie" erachter, zoals in Finder.
/// Geeft de nieuwe paden.
pub fn plak(bronnen: &[PathBuf], map: &Path) -> Vec<PathBuf> {
    let mut nieuw = Vec::new();
    for bron in bronnen {
        // Een map in zichzelf kopiëren houdt niet op.
        if map.starts_with(bron) && bron.is_dir() {
            continue;
        }
        let Some(naam) = bron.file_name() else { continue };
        let doel = vrije_naam(map, &naam.to_string_lossy());
        if Command::new("cp").arg("-Rp").arg(bron).arg(&doel).status().is_ok_and(|s| s.success()) {
            nieuw.push(doel);
        }
    }
    nieuw
}

/// Verplaatst `bron` naar `map`. Een naam die er al is, krijgt "kopie" erachter. Geeft het nieuwe pad.
pub fn verplaats(bron: &Path, map: &Path) -> Option<PathBuf> {
    // Al in die map, of een map in zichzelf.
    if bron.parent() == Some(map) || map.starts_with(bron) {
        return None;
    }
    let doel = vrije_naam(map, &bron.file_name()?.to_string_lossy());
    std::fs::rename(bron, &doel).ok().map(|_| doel)
}

/// `pad` vanaf `root`, de hoofdmap van het project. De hoofdmap zelf is `.`.
pub fn vanaf(pad: &Path, root: &Path) -> String {
    match pad.strip_prefix(root) {
        Ok(rest) if rest.as_os_str().is_empty() => ".".to_string(),
        Ok(rest) => rest.display().to_string(),
        Err(_) => pad.display().to_string(),
    }
}

/// Verplaatst `pad` naar de prullenmand.
pub fn prullenmand(pad: &Path) {
    let _ = Command::new("trash").arg(pad).status();
}

/// Geeft `pad` de naam `naam`. Geeft het nieuwe pad.
pub fn hernoem(pad: &Path, naam: &str) -> Result<PathBuf, &'static str> {
    let nieuw = pad.with_file_name(geldig(naam)?);
    if nieuw == pad {
        return Ok(nieuw);
    }
    // Alleen hoofdletters anders is op macOS hetzelfde bestand.
    if nieuw.exists() && !naam.eq_ignore_ascii_case(&pad.file_name().unwrap_or_default().to_string_lossy()) {
        return Err(t("bestaat al", "already exists"));
    }
    std::fs::rename(pad, &nieuw).map_err(|_| t("lukt niet", "failed"))?;
    Ok(nieuw)
}

/// Maakt een leeg bestand of een lege map `naam` in `map`. Geeft het nieuwe pad.
pub fn maak(map: &Path, naam: &str, is_map: bool) -> Result<PathBuf, &'static str> {
    let nieuw = map.join(geldig(naam)?);
    if nieuw.exists() {
        return Err(t("bestaat al", "already exists"));
    }
    let gelukt = match is_map {
        true => std::fs::create_dir(&nieuw).is_ok(),
        false => std::fs::File::create_new(&nieuw).is_ok(),
    };
    if gelukt { Ok(nieuw) } else { Err(t("lukt niet", "failed")) }
}

fn geldig(naam: &str) -> Result<&str, &'static str> {
    match naam {
        "" | "." | ".." => Err(t("geen naam", "no name")),
        _ if naam.contains('/') => Err(t("geen / in een naam", "no / in a name")),
        _ => Ok(naam),
    }
}

/// `naam` in `map`, of "naam kopie", "naam kopie 2", enzovoort als die er al is.
fn vrije_naam(map: &Path, naam: &str) -> PathBuf {
    let (stam, ext) = match naam.rfind('.') {
        Some(i) if i > 0 => naam.split_at(i),
        _ => (naam, ""),
    };
    (0..)
        .map(|n| match n {
            0 => naam.to_string(),
            1 => format!("{stam} {}{ext}", t("kopie", "copy")),
            _ => format!("{stam} {} {n}{ext}", t("kopie", "copy")),
        })
        .map(|naam| map.join(naam))
        .find(|pad| !pad.exists())
        .expect("een vrije naam")
}
