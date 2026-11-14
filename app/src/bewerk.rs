//! Wat het menu in de bestandsboom met bestanden en mappen doet.

use std::path::{Path, PathBuf};
use std::process::Command;

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
pub fn plak(bronnen: &[PathBuf], map: &Path) {
    for bron in bronnen {
        // Een map in zichzelf kopiëren houdt niet op.
        if map.starts_with(bron) && bron.is_dir() {
            continue;
        }
        let Some(naam) = bron.file_name() else { continue };
        let doel = vrije_naam(map, &naam.to_string_lossy());
        let _ = Command::new("cp").arg("-Rp").arg(bron).arg(doel).status();
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
        return Err("bestaat al");
    }
    std::fs::rename(pad, &nieuw).map_err(|_| "lukt niet")?;
    Ok(nieuw)
}

/// Maakt een leeg bestand of een lege map `naam` in `map`. Geeft het nieuwe pad.
pub fn maak(map: &Path, naam: &str, is_map: bool) -> Result<PathBuf, &'static str> {
    let nieuw = map.join(geldig(naam)?);
    if nieuw.exists() {
        return Err("bestaat al");
    }
    let gelukt = match is_map {
        true => std::fs::create_dir(&nieuw).is_ok(),
        false => std::fs::File::create_new(&nieuw).is_ok(),
    };
    if gelukt { Ok(nieuw) } else { Err("lukt niet") }
}

fn geldig(naam: &str) -> Result<&str, &'static str> {
    match naam {
        "" | "." | ".." => Err("geen naam"),
        _ if naam.contains('/') => Err("geen / in een naam"),
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
            1 => format!("{stam} kopie{ext}"),
            _ => format!("{stam} kopie {n}{ext}"),
        })
        .map(|naam| map.join(naam))
        .find(|pad| !pad.exists())
        .expect("een vrije naam")
}
