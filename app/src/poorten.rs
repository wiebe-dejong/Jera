//! De poorten waarop iets luistert en de agent-browsers die draaien, met de sessie die ze startte.
//! De meting staat in `research/poorten.md`.

use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    process::Command,
    sync::Mutex,
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
    /// De map waarin elke sessie hierboven liep, ook die niet meer draaien.
    pub mappen: HashMap<String, PathBuf>,
}

impl Stand {
    /// Hoort `sessie` bij een sessie die niet meer draait.
    pub fn wees(&self, sessie: &Option<String>) -> bool {
        sessie.as_ref().is_some_and(|id| !self.sessies.contains_key(id))
    }

    /// Liep `sessie` in het project in `root`.
    pub fn van_project(&self, sessie: Option<&str>, root: &Path) -> bool {
        sessie.and_then(|id| self.mappen.get(id)).is_some_and(|map| map.starts_with(root))
    }

    /// De poorten van het project in `root`: van een sessie daar, of gestart in die map.
    pub fn poorten_van<'a>(&'a self, root: &'a Path) -> impl Iterator<Item = &'a Poort> {
        self.poorten.iter().filter(move |p| self.van_project(p.sessie.as_deref(), root) || p.map.starts_with(root))
    }

    /// De agent-browsers van een sessie in het project in `root`.
    pub fn browsers_van<'a>(&'a self, root: &'a Path) -> impl Iterator<Item = &'a Browser> {
        self.browsers.iter().filter(move |b| self.van_project(b.sessie.as_deref(), root))
    }
}

/// De map van een sessie die niet meer draait, eenmaal uit haar gesprek gelezen.
static MAPPEN: Mutex<Vec<(String, Option<PathBuf>)>> = Mutex::new(Vec::new());

/// `~/.agent-browser`.
fn browsermap() -> PathBuf {
    crate::thuis().join(".agent-browser")
}

pub fn lees() -> Stand {
    let ouders = ouders();
    let mut claude_pids = HashSet::new();
    let mut sessiemappen = HashMap::new();
    let sessies: HashMap<String, (u32, String)> = claude::sessies()
        .into_iter()
        .filter_map(|json| {
            let pid = json["pid"].as_u64()? as u32;
            let ouder = *ouders.get(&pid)?;
            claude_pids.insert(pid);
            let id = json["sessionId"].as_str()?.to_string();
            if let Some(cwd) = json["cwd"].as_str() {
                sessiemappen.insert(id.clone(), PathBuf::from(cwd));
            }
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

    // De poorten van macOS zelf (AirPlay in ControlCenter, rapportd) en van een app (Raycast
    // Backend, met Raycast als ouder) horen er ook niet bij, ook niet onder Alles. Niet op
    // `.app/` zoeken: Python draait zelf vanuit `Python.framework/…/Python.app`.
    let apps = crate::thuis().join("Applications/");
    let mac = ["/Applications/", "/System/", "/usr/libexec/", "/usr/sbin/", &apps.to_string_lossy()].map(String::from);
    let draait_uit = |pid: u32, mappen: &[String]| {
        boven(pid, &ouders).take(2).any(|p| commandos.get(&p).is_some_and(|c| mappen.iter().any(|m| c.starts_with(m.as_str()))))
    };
    // Behalve de poorten van Docker-containers: die opent Docker Desktop zelf.
    let docker = match luistert.iter().any(|(pid, _)| draait_uit(*pid, &[DOCKER_APP.to_string()])) {
        true => docker(),
        false => HashMap::new(),
    };
    let luistert: Vec<(u32, u16)> =
        luistert.into_iter().filter(|(pid, nummer)| docker.contains_key(nummer) || !draait_uit(*pid, &mac)).collect();

    let pids: Vec<u32> = luistert.iter().map(|(pid, _)| *pid).collect();
    let mappen = mappen(&pids);
    let mut poorten: Vec<Poort> = luistert
        .into_iter()
        .map(|(pid, nummer)| match docker.get(&nummer) {
            // Een container heet naar zichzelf en hoort bij de map van docker compose.
            Some((naam, map)) => Poort { nummer, pid, commando: naam.clone(), map: map.clone(), sessie: None },
            None => Poort {
                nummer,
                pid,
                commando: commandos.get(&pid).map(|c| kort(c)).unwrap_or_default(),
                map: mappen.get(&pid).cloned().unwrap_or_default(),
                sessie: van(pid),
            },
        })
        .collect();
    poorten.sort_by_key(|p| p.nummer);
    let mut browsers: Vec<Browser> =
        browsers.into_iter().map(|(naam, pid)| Browser { naam, pid, sessie: van(pid) }).collect();
    browsers.sort_by(|a, b| a.naam.cmp(&b.naam));

    // De map van een sessie die niet meer draait, staat in haar gesprek.
    let mut gelezen = MAPPEN.lock().unwrap();
    let wezen = poorten.iter().map(|p| &p.sessie).chain(browsers.iter().map(|b| &b.sessie)).flatten();
    for id in wezen.filter(|id| !sessiemappen.contains_key(*id)).cloned().collect::<Vec<_>>() {
        let map = match gelezen.iter().find(|(i, _)| *i == id) {
            Some((_, map)) => map.clone(),
            None => {
                let map = claude::map_van(&id);
                gelezen.push((id.clone(), map.clone()));
                map
            }
        };
        if let Some(map) = map {
            sessiemappen.insert(id, map);
        }
    }
    Stand { poorten, browsers, sessies, mappen: sessiemappen }
}

/// Docker Desktop. De poorten van containers opent `com.docker.backend` daarin.
const DOCKER_APP: &str = "/Applications/Docker.app/";

/// De poorten van de containers die draaien: poort naar de naam van de container en de map
/// waaruit docker compose hem startte. Een container zonder compose heeft geen map.
fn docker() -> HashMap<u16, (String, PathBuf)> {
    let opmaak = "{{.Names}}\t{{.Label \"com.docker.compose.project.working_dir\"}}\t{{.Ports}}";
    let docker = format!("{DOCKER_APP}Contents/Resources/bin/docker");
    let mut uit = HashMap::new();
    for regel in uitvoer(&docker, &["ps", "--format", opmaak]).lines() {
        let mut delen = regel.split('\t');
        let (Some(naam), Some(map), Some(poorten)) = (delen.next(), delen.next(), delen.next()) else { continue };
        // Zoals `0.0.0.0:5432->5432/tcp, [::]:5432->5432/tcp`.
        for deel in poorten.split(", ") {
            let nummer = deel.split_once("->").and_then(|(buiten, _)| buiten.rsplit(':').next()?.parse().ok());
            if let Some(nummer) = nummer {
                uit.insert(nummer, (naam.to_string(), PathBuf::from(map)));
            }
        }
    }
    uit
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

/// `agent-browser`, ook als Jera buiten de shell start en Homebrew niet in het PATH staat.
fn agent_browser() -> Command {
    let pad = ["/opt/homebrew/bin/agent-browser", "/usr/local/bin/agent-browser"]
        .into_iter()
        .find(|p| Path::new(p).exists())
        .unwrap_or("agent-browser");
    Command::new(pad)
}

/// Sluit de agent-browser met deze naam.
pub fn sluit(naam: &str) {
    let _ = agent_browser().args(["--session", naam, "close"]).output();
}

/// Zo breed wordt een plaatje hoogstens bewaard, in pixels.
const B_PLAATJE: u32 = 720;

/// Een plaatje van wat de agent-browser `naam` nu toont, in BGRA zoals GPUI het wil.
pub fn plaatje(naam: &str) -> Option<image::RgbaImage> {
    let pad = std::env::temp_dir().join(format!("jera-plaatje-{naam}.png"));
    let uit = agent_browser().args(["--session", naam, "screenshot"]).arg(&pad).output().ok()?;
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
