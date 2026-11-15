//! De poorten waarop iets luistert en de agent-browsers die draaien, met de sessie die ze startte.
//! De meting staat in `research/poorten.md`.

use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    process::Command,
};

use crate::claude;

#[derive(Clone, PartialEq, Debug)]
pub struct Poort {
    pub nummer: u16,
    pub pid: u32,
    /// Het commando zonder het programma ervoor, zoals `next dev` of `uvicorn app:main`.
    pub commando: String,
    /// De map waaruit het proces is gestart.
    pub map: PathBuf,
    /// Het id van de Claude-sessie die het startte. Dat blijft hetzelfde als het gesprek
    /// wordt voortgezet, ook na een herstart van Jera.
    pub sessie: Option<String>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Browser {
    pub naam: String,
    pub pid: u32,
    pub sessie: Option<String>,
}

/// Wat er nu draait.
#[derive(Clone, PartialEq, Default, Debug)]
pub struct Stand {
    pub poorten: Vec<Poort>,
    pub browsers: Vec<Browser>,
    /// De Claude-sessies die draaien: sessie-id naar het pid van de ouder van Claude en de naam uit het sessiebestand.
    pub sessies: HashMap<String, (u32, String)>,
}

impl Stand {
    /// Hoort `sessie` bij een sessie die niet meer draait.
    pub fn wees(&self, sessie: &Option<String>) -> bool {
        sessie.as_ref().is_some_and(|id| !self.sessies.contains_key(id))
    }

    /// De poorten van de standaardlijst: gestart in de thuismap, of door een sessie.
    pub fn standaard(&self) -> impl Iterator<Item = &Poort> {
        let thuis = crate::thuis();
        self.poorten.iter().filter(move |p| p.sessie.is_some() || p.map.starts_with(&thuis))
    }
}

/// `~/.agent-browser`.
fn browsermap() -> PathBuf {
    crate::thuis().join(".agent-browser")
}

pub fn lees() -> Stand {
    let ouders = ouders();
    let mut claude_pids = HashSet::new();
    let sessies: HashMap<String, (u32, String)> = claude::sessies()
        .into_iter()
        .filter_map(|json| {
            let pid = json["pid"].as_u64()? as u32;
            let ouder = *ouders.get(&pid)?;
            claude_pids.insert(pid);
            let id = json["sessionId"].as_str()?.to_string();
            Some((id, (ouder, json["name"].as_str().unwrap_or_default().to_string())))
        })
        .collect();

    let browsers: Vec<(String, u32)> = std::fs::read_dir(browsermap())
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|bestand| {
            let pad = bestand.path();
            if pad.extension()? != "pid" {
                return None;
            }
            let pid: u32 = std::fs::read_to_string(&pad).ok()?.trim().parse().ok()?;
            let naam = pad.file_stem()?.to_string_lossy().into_owned();
            ouders.contains_key(&pid).then_some((naam, pid))
        })
        .collect();
    let browser_pids: HashSet<u32> = browsers.iter().map(|(_, pid)| *pid).collect();

    // De poorten van Claude Code zelf en van agent-browser en zijn Chrome horen er niet bij.
    let luistert: Vec<(u32, u16)> = luisteraars()
        .into_iter()
        .filter(|(pid, _)| !claude_pids.contains(pid))
        .filter(|(pid, _)| !boven(*pid, &ouders).any(|p| browser_pids.contains(&p)))
        .collect();

    // Het sessie-id staat in de omgeving van het proces. Een proces dat zijn naam overschrijft
    // (zoals next-server) wist die; dan staat hij bij een ouder.
    let mut rij: Vec<u32> = luistert.iter().map(|(pid, _)| *pid).chain(browser_pids.iter().copied()).collect();
    rij.extend(rij.clone().iter().flat_map(|pid| boven(*pid, &ouders).skip(1)));
    rij.sort();
    rij.dedup();
    let omgeving = sessie_ids(&rij);
    let commandos = commandos(&rij);
    // Een app die vanuit een sessie is geopend (zoals Raycast) erft het sessienummer ook,
    // maar is geen server van die sessie.
    let van = |pid: u32| {
        for p in boven(pid, &ouders) {
            if commandos.get(&p).is_some_and(|c| c.contains(".app/Contents/")) {
                return None;
            }
            if let Some(id) = omgeving.get(&p) {
                return Some(id.clone());
            }
        }
        None
    };

    let pids: Vec<u32> = luistert.iter().map(|(pid, _)| *pid).collect();
    let mappen = mappen(&pids);
    let mut poorten: Vec<Poort> = luistert
        .into_iter()
        .map(|(pid, nummer)| Poort {
            nummer,
            pid,
            commando: commandos.get(&pid).map(|c| kort(c)).unwrap_or_default(),
            map: mappen.get(&pid).cloned().unwrap_or_default(),
            sessie: van(pid),
        })
        .collect();
    poorten.sort_by_key(|p| p.nummer);
    let mut browsers: Vec<Browser> =
        browsers.into_iter().map(|(naam, pid)| Browser { naam, pid, sessie: van(pid) }).collect();
    browsers.sort_by(|a, b| a.naam.cmp(&b.naam));
    Stand { poorten, browsers, sessies }
}

/// `pid` en zijn ouders, tot launchd.
fn boven(pid: u32, ouders: &HashMap<u32, u32>) -> impl Iterator<Item = u32> + '_ {
    std::iter::successors(Some(pid), |p| ouders.get(p).copied().filter(|o| *o > 1)).take(32)
}

/// Pid naar het pid van de ouder, voor alle processen.
fn ouders() -> HashMap<u32, u32> {
    uitvoer("ps", &["-A", "-o", "pid=,ppid="])
        .lines()
        .filter_map(|regel| {
            let mut delen = regel.split_whitespace().map(|d| d.parse().ok());
            Some((delen.next()??, delen.next()??))
        })
        .collect()
}

/// De poorten waarop iets luistert, met het pid. IPv4 en IPv6 tellen één keer.
fn luisteraars() -> Vec<(u32, u16)> {
    let mut uit = Vec::new();
    let mut pid = 0;
    for regel in uitvoer("lsof", &["-nP", "-iTCP", "-sTCP:LISTEN", "-Fpn"]).lines() {
        if let Some(p) = regel.strip_prefix('p') {
            pid = p.parse().unwrap_or(0);
        } else if let Some(adres) = regel.strip_prefix('n') {
            if let Some(nummer) = adres.rsplit(':').next().and_then(|n| n.parse().ok()) {
                uit.push((pid, nummer));
            }
        }
    }
    uit.sort();
    uit.dedup();
    uit
}

/// `CLAUDE_CODE_SESSION_ID` uit de omgeving van deze processen.
fn sessie_ids(pids: &[u32]) -> HashMap<u32, String> {
    if pids.is_empty() {
        return HashMap::new();
    }
    let lijst = lijst(pids);
    uitvoer("ps", &["-E", "-ww", "-o", "pid=,command=", "-p", &lijst])
        .lines()
        .filter_map(|regel| {
            let regel = regel.trim_start();
            let pid = regel.split_whitespace().next()?.parse().ok()?;
            let na = regel.split(" CLAUDE_CODE_SESSION_ID=").nth(1)?;
            Some((pid, na.split_whitespace().next()?.to_string()))
        })
        .collect()
}

fn commandos(pids: &[u32]) -> HashMap<u32, String> {
    if pids.is_empty() {
        return HashMap::new();
    }
    uitvoer("ps", &["-ww", "-o", "pid=,command=", "-p", &lijst(pids)])
        .lines()
        .filter_map(|regel| {
            let (pid, commando) = regel.trim_start().split_once(' ')?;
            Some((pid.parse().ok()?, commando.trim().to_string()))
        })
        .collect()
}

/// Het commando zonder de tolk ervoor en zonder paden: `node …/.bin/next dev` wordt `next dev`.
pub fn kort(commando: &str) -> String {
    const TOLKEN: [&str; 8] = ["node", "python", "python3", "Python", "bun", "deno", "ruby", "uv"];
    let mut delen: Vec<&str> = commando.split_whitespace().collect();
    let tolk = |d: &str| {
        let naam = Path::new(d).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        TOLKEN.iter().any(|t| naam == *t || naam.starts_with(&format!("{t}3.")) || naam.starts_with(&format!("{t}@")))
    };
    if delen.len() > 1 && tolk(delen[0]) {
        delen.remove(0);
        if delen.first() == Some(&"-m") || delen.first() == Some(&"run") {
            delen.remove(0);
        }
    }
    delen
        .iter()
        .enumerate()
        .map(|(i, d)| match i {
            0 => Path::new(d).file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| d.to_string()),
            _ => d.to_string(),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn mappen(pids: &[u32]) -> HashMap<u32, PathBuf> {
    if pids.is_empty() {
        return HashMap::new();
    }
    let mut uit = HashMap::new();
    let mut pid = 0;
    for regel in uitvoer("lsof", &["-a", "-d", "cwd", "-Fpn", "-p", &lijst(pids)]).lines() {
        if let Some(p) = regel.strip_prefix('p') {
            pid = p.parse().unwrap_or(0);
        } else if let Some(map) = regel.strip_prefix('n') {
            uit.insert(pid, PathBuf::from(map));
        }
    }
    uit
}

fn lijst(pids: &[u32]) -> String {
    pids.iter().map(u32::to_string).collect::<Vec<_>>().join(",")
}

fn uitvoer(programma: &str, args: &[&str]) -> String {
    Command::new(programma)
        .args(args)
        .output()
        .map(|uit| String::from_utf8_lossy(&uit.stdout).into_owned())
        .unwrap_or_default()
}

/// Stopt het proces.
pub fn stop(pid: u32) {
    let _ = Command::new("kill").args(["-TERM", &pid.to_string()]).status();
}

/// Opent de poort in de standaardbrowser.
pub fn open(nummer: u16) {
    let _ = Command::new("open").arg(format!("http://localhost:{nummer}")).status();
}

/// Sluit de agent-browser met deze naam.
pub fn sluit(naam: &str) {
    let _ = Command::new("agent-browser").args(["--session", naam, "close"]).output();
}

/// Zo breed wordt een plaatje hoogstens bewaard, in pixels.
const B_PLAATJE: u32 = 720;

/// Een plaatje van wat de agent-browser `naam` nu toont, in BGRA zoals GPUI het wil.
pub fn plaatje(naam: &str) -> Option<image::RgbaImage> {
    let pad = std::env::temp_dir().join(format!("jera-plaatje-{naam}.png"));
    let uit = Command::new("agent-browser").args(["--session", naam, "screenshot"]).arg(&pad).output().ok()?;
    let bytes = uit.status.success().then(|| std::fs::read(&pad).ok()).flatten();
    let _ = std::fs::remove_file(&pad);
    let beeld = image::load_from_memory_with_format(&bytes?, image::ImageFormat::Png).ok()?;
    let beeld = match beeld.width() > B_PLAATJE {
        true => beeld.resize(B_PLAATJE, u32::MAX, image::imageops::FilterType::Triangle),
        false => beeld,
    };
    let mut data = beeld.into_rgba8();
    for pixel in data.chunks_exact_mut(4) {
        pixel.swap(0, 2);
    }
    Some(data)
}

#[cfg(test)]
mod tests {
    use super::kort;

    #[test]
    fn commando_kort() {
        assert_eq!(kort("/opt/homebrew/bin/node /x/node_modules/.bin/next dev"), "next dev");
        assert_eq!(kort("/usr/bin/python3 -m uvicorn app:main"), "uvicorn app:main");
        assert_eq!(kort("python3.13 -m http.server 8798"), "http.server 8798");
        assert_eq!(kort("/opt/homebrew/opt/postgresql@16/bin/postgres -D /var"), "postgres -D /var");
        assert_eq!(kort("next-server (v15)"), "next-server (v15)");
    }
}
