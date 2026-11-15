//! Het scherm Openen: bovenaan "open" en "clone repository", daaronder het zoekveld en de
//! mappen die ik het laatst in Jera opende. Een nieuw project gaat via ⌘N.
//! De recente mappen staan in `~/Library/Application Support/Jera/recent.json`,
//! de repo's van GitHub komen uit `gh`.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::Command,
};

use gpui_kit::{
    base::input::{Input, InputEvent, InputState},
    prelude::FluentBuilder,
    *,
};

use crate::bestandsicoon;
use crate::thema::{self as t, Thema};
use crate::venster::naam;

actions!(jera, [OpenProject, NieuwProject]);

/// De regelhoogte van het invoerveld. Met een regelhoogte in verhouding tot de letter tekent
/// het veld zijn tekst te laag.
const REGEL: f32 = t::TEKST * t::REGEL;
/// Een regel met de naam en klein het pad eronder.
const H_REGEL: f32 = 42.0;
/// De twee regels boven het zoekveld: "open" en "clone repository".
const VAST: usize = 2;
/// Zoveel mappen onthoudt Jera.
const MAX_RECENT: usize = 30;

pub enum OpenenEvent {
    Gekozen(PathBuf),
    AndereMap,
    Sluit,
}

#[derive(Clone, Copy, PartialEq)]
enum Stand {
    Recent,
    Github,
    Nieuw,
}

#[derive(Clone)]
enum Regel {
    Github,
    AndereMap,
    Map(PathBuf),
    /// eigenaar/naam op GitHub.
    Repo(String),
}

pub struct Openen {
    stand: Stand,
    /// Het zoekveld, of het naamveld bij een nieuw project.
    veld: Entity<InputState>,
    recent: Vec<PathBuf>,
    repos: Vec<String>,
    github_fout: bool,
    /// Wat er gekloond wordt: `None` zolang het bezig is, anders wat er misging.
    klonen: HashMap<String, Option<&'static str>>,
    /// Wat er mis is met de naam van een nieuw project.
    fout: Option<&'static str>,
    gekozen: usize,
    scroll: ScrollHandle,
    _volg: Subscription,
}

impl EventEmitter<OpenenEvent> for Openen {}

impl Openen {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let veld = cx.new(|cx| InputState::new(window, cx));
        let volg = cx.subscribe_in(&veld, window, |this, _, event, window, cx| match event {
            InputEvent::Change => {
                this.gekozen = VAST;
                this.fout = None;
                this.scroll.scroll_to_item(0);
                cx.notify();
            }
            InputEvent::PressEnter { .. } if this.stand == Stand::Nieuw => this.maak(cx),
            InputEvent::PressEnter { .. } => this.kies(this.gekozen, window, cx),
            _ => {}
        });
        Self {
            stand: Stand::Recent,
            veld,
            recent: Vec::new(),
            repos: Vec::new(),
            github_fout: false,
            klonen: HashMap::new(),
            fout: None,
            gekozen: 0,
            scroll: ScrollHandle::new(),
            _volg: volg,
        }
    }

    /// Toont de recente mappen, of met `nieuw` het naamveld voor een nieuw project.
    pub fn toon(&mut self, nieuw: bool, window: &mut Window, cx: &mut Context<Self>) {
        self.recent = lees_recent();
        self.zet_stand(if nieuw { Stand::Nieuw } else { Stand::Recent }, window, cx);
    }

    fn zet_stand(&mut self, stand: Stand, window: &mut Window, cx: &mut Context<Self>) {
        self.stand = stand;
        self.gekozen = VAST;
        self.fout = None;
        let plaats = match stand {
            Stand::Recent => "zoek",
            Stand::Github => "zoek in github",
            Stand::Nieuw => "naam van het nieuwe project",
        };
        self.veld.update(cx, |veld, cx| {
            veld.set_value("", window, cx);
            veld.set_placeholder(plaats, window, cx);
            veld.focus(window, cx);
        });
        if stand == Stand::Github {
            self.lees_github(cx);
        }
        cx.notify();
    }

    /// Toont de repo's uit de vorige keer en leest GitHub opnieuw.
    fn lees_github(&mut self, cx: &mut Context<Self>) {
        if let Some(gh) = GitHub::uit_cache() {
            self.repos = gh.repo_namen();
        }
        cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let vers = cx.background_executor().spawn(async { GitHub::lees() }).await;
            let _ = this.update(cx, |this, cx| {
                this.github_fout = vers.is_none();
                if let Some(gh) = vers {
                    this.repos = gh.repo_namen();
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// De regels die nu te zien zijn: de twee vaste, en de lijst met alleen de namen waar elk
    /// woord uit het zoekveld in staat.
    fn regels(&self, cx: &App) -> Vec<Regel> {
        let zoek = self.veld.read(cx).value().to_lowercase();
        let woorden: Vec<&str> = zoek.split_whitespace().collect();
        let past = |naam: &str| {
            let naam = naam.to_lowercase();
            woorden.iter().all(|w| naam.contains(w))
        };
        let lijst: Vec<Regel> = match self.stand {
            Stand::Recent => self.recent.iter().filter(|p| past(&naam(p))).cloned().map(Regel::Map).collect(),
            Stand::Github => self.repos.iter().filter(|r| past(repo_naam(r))).cloned().map(Regel::Repo).collect(),
            Stand::Nieuw => return Vec::new(),
        };
        [Regel::AndereMap, Regel::Github].into_iter().chain(lijst).collect()
    }

    fn kies(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(regel) = self.regels(cx).into_iter().nth(index) else { return };
        match regel {
            Regel::Github => {
                let stand = if self.stand == Stand::Github { Stand::Recent } else { Stand::Github };
                self.zet_stand(stand, window, cx)
            }
            Regel::AndereMap => cx.emit(OpenenEvent::AndereMap),
            Regel::Map(map) => cx.emit(OpenenEvent::Gekozen(map)),
            Regel::Repo(repo) => self.kloon(repo, cx),
        }
    }

    /// Vraagt waar het nieuwe project komt, maakt de map met `git init` en kiest hem.
    fn maak(&mut self, cx: &mut Context<Self>) {
        let naam = self.veld.read(cx).value().trim().to_string();
        self.fout = if naam.is_empty() {
            Some("geen naam")
        } else if naam.contains('/') {
            Some("geen / in een naam")
        } else {
            None
        };
        cx.notify();
        if self.fout.is_some() {
            return;
        }
        let ouder = kies_plek("Maak hier", cx);
        cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let Some(ouder) = ouder.await else { return };
            let map = ouder.join(&naam);
            let uitkomst = if map.exists() {
                Err("bestaat al")
            } else {
                std::fs::create_dir_all(&map)
                    .ok()
                    .and_then(|_| Command::new("git").arg("init").arg("-q").arg(&map).status().ok())
                    .filter(|s| s.success())
                    .map(|_| ())
                    .ok_or("mislukt")
            };
            let _ = this.update(cx, |this, cx| match uitkomst {
                Ok(()) => cx.emit(OpenenEvent::Gekozen(map)),
                Err(fout) => {
                    this.fout = Some(fout);
                    cx.notify();
                }
            });
        })
        .detach();
    }

    /// Vraagt waar de repo komt, kloont hem en kiest hem. Staat hij daar al, dan opent hij die.
    fn kloon(&mut self, repo: String, cx: &mut Context<Self>) {
        if self.klonen.get(&repo).is_some_and(Option::is_none) {
            return;
        }
        let ouder = kies_plek("Kloon hier", cx);
        cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let Some(ouder) = ouder.await else { return };
            let map = ouder.join(repo_naam(&repo));
            let _ = this.update(cx, |this, cx| {
                this.klonen.insert(repo.clone(), None);
                cx.notify();
            });
            let (r, m) = (repo.clone(), map.clone());
            let uitkomst = cx
                .background_executor()
                .spawn(async move {
                    if m.exists() {
                        let zelfde = remote(&m).is_some_and(|o| o.eq_ignore_ascii_case(&r));
                        return if zelfde { Ok(()) } else { Err("bestaat al") };
                    }
                    let uit = gh().args(["repo", "clone", &r]).arg(&m).output().map_err(|_| "mislukt")?;
                    if !uit.status.success() {
                        eprintln!("klonen van {r}: {}", String::from_utf8_lossy(&uit.stderr));
                        return Err("mislukt");
                    }
                    Ok(())
                })
                .await;
            let _ = this.update(cx, |this, cx| {
                match uitkomst {
                    Ok(()) => {
                        this.klonen.remove(&repo);
                        cx.emit(OpenenEvent::Gekozen(map));
                    }
                    Err(fout) => {
                        this.klonen.insert(repo, Some(fout));
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn verschuif(&mut self, stap: isize, cx: &mut Context<Self>) {
        let aantal = self.regels(cx).len();
        if aantal == 0 {
            return;
        }
        self.gekozen = (self.gekozen as isize + stap).clamp(0, aantal as isize - 1) as usize;
        self.scroll.scroll_to_item(self.gekozen.saturating_sub(VAST));
        cx.notify();
    }

    /// Eén regel: de naam, en klein het pad eronder.
    fn regel(&self, index: usize, regel: &Regel, th: &Thema, cx: &mut Context<Self>) -> AnyElement {
        let (titel, onder) = match regel {
            Regel::AndereMap => ("open".to_string(), None),
            Regel::Github => ("clone repository".to_string(), None),
            Regel::Map(map) => (naam(map), Some(tilde(map))),
            Regel::Repo(repo) => {
                let onder = match self.klonen.get(repo) {
                    Some(None) => "klonen".to_string(),
                    Some(Some(fout)) => fout.to_string(),
                    None => repo.clone(),
                };
                (repo_naam(repo).to_string(), Some(onder))
            }
        };
        let icoon = match regel {
            Regel::AndereMap => Some(bestandsicoon::map(false, th)),
            Regel::Github => Some(bestandsicoon::git(th)),
            Regel::Map(_) | Regel::Repo(_) => None,
        };
        let gekozen = index == self.gekozen;
        // In de lijst van GitHub is "clone repository" de actieve regel.
        let actief = matches!(regel, Regel::Github) && self.stand == Stand::Github;
        if let Some(icoon) = icoon {
            return div()
                .id(("regel", index))
                .flex()
                .items_center()
                .gap(px(t::GAP))
                .h(px(t::H_RIJ))
                .pl(px(t::PAD - t::STREEP))
                .pr(px(t::PAD))
                .border_l(px(t::STREEP))
                .border_color(transparent_black())
                .when(gekozen || actief, |d| d.font_weight(FontWeight::BOLD))
                .when(gekozen, |d| d.bg(rgb(th.sel)).border_color(rgb(th.accent)))
                .when(!gekozen, |d| d.hover(|d| d.bg(rgb(th.sel))))
                .child(icoon)
                .child(titel)
                .on_click(cx.listener(move |this, _, window, cx| this.kies(index, window, cx)))
                .into_any_element();
        }
        div()
            .id(("regel", index))
            .flex()
            .flex_col()
            .justify_center()
            .h(px(if onder.is_some() { H_REGEL } else { t::H_RIJ }))
            .pl(px(t::PAD - t::STREEP))
            .pr(px(t::PAD))
            .border_l(px(t::STREEP))
            .border_color(transparent_black())
            .when(gekozen, |d| d.bg(rgb(th.sel)).border_color(rgb(th.accent)))
            .when(!gekozen, |d| d.hover(|d| d.bg(rgb(th.sel))))
            .child(div().truncate().when(gekozen, |d| d.font_weight(FontWeight::BOLD)).child(titel))
            .children(onder.map(|onder| div().truncate().text_size(px(11.0)).text_color(rgb(th.mute)).child(onder)))
            .on_click(cx.listener(move |this, _, window, cx| this.kies(index, window, cx)))
            .into_any_element()
    }

    /// Het scherm zelf. Het venster hangt het onder de tab "+ map".
    pub fn vak(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let th = t::thema(window);
        let regels = self.regels(cx);
        let mut boven = div().flex_none();
        let mut lijst = div().id("openen-lijst").flex_1().min_h_0().overflow_y_scroll().track_scroll(&self.scroll);
        for (index, regel) in regels.iter().enumerate() {
            let rij = self.regel(index, regel, th, cx);
            if index < VAST {
                boven = boven.child(rij);
            } else {
                lijst = lijst.child(rij);
            }
        }
        let leeg = match self.stand {
            _ if regels.len() > VAST => None,
            Stand::Github if self.github_fout && self.repos.is_empty() => {
                Some("GitHub niet bereikbaar. Log in met gh auth login.")
            }
            Stand::Github => Some("Geen repo met deze naam."),
            Stand::Recent if !self.veld.read(cx).value().trim().is_empty() => Some("Geen map met deze naam."),
            _ => None,
        };
        let lijst = lijst.children(leeg.map(|tekst| {
            div().flex().items_center().h(px(t::H_RIJ)).px(px(t::PAD)).text_color(rgb(th.mute)).child(tekst)
        }));
        let lijn = || div().flex_none().h(px(t::LIJN)).bg(rgb(th.line));
        let veld = div()
            .flex_none()
            .flex()
            .items_center()
            .gap(px(6.0))
            .h(px(t::H_RIJ + 8.0))
            .px(px(t::PAD))
            .child(div().flex_1().min_w_0().h_full().line_height(px(REGEL)).child(Input::new(&self.veld)))
            .children(self.fout.map(|fout| div().flex_none().text_color(rgb(th.mute)).child(fout)));

        let lijst_open = self.stand != Stand::Nieuw;
        let onder = lijst_open && (regels.len() > VAST || leeg.is_some());
        let hoogte = f32::from(window.viewport_size().height) - 2.0 * 64.0;
        div()
            .id("openen")
            .flex()
            .flex_col()
            .w(px(560.0))
            .max_h(px(hoogte.max(200.0)))
            .bg(rgb(th.bg))
            .border(px(t::LIJN))
            .border_color(rgb(th.line_strong))
            .text_color(rgb(th.ink))
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .capture_key_down(cx.listener(|this, e: &KeyDownEvent, window, cx| {
                match e.keystroke.key.as_str() {
                    "escape" if this.stand == Stand::Github => this.zet_stand(Stand::Recent, window, cx),
                    "escape" => cx.emit(OpenenEvent::Sluit),
                    "up" => this.verschuif(-1, cx),
                    "down" => this.verschuif(1, cx),
                    _ => return,
                }
                cx.stop_propagation();
            }))
            .when(lijst_open, |d| d.child(boven).child(lijn()))
            .child(veld)
            .when(onder, |d| d.child(lijn()).child(lijst))
            .into_any_element()
    }

    /// Het vlak achter het scherm: een klik ernaast sluit het.
    pub fn achter(&self, cx: &mut Context<Self>) -> AnyElement {
        div()
            .id("openen-achter")
            .absolute()
            .inset_0()
            .occlude()
            .on_mouse_down(MouseButton::Left, cx.listener(|_, _, _, cx| cx.emit(OpenenEvent::Sluit)))
            .into_any_element()
    }
}

fn recent_pad() -> PathBuf {
    crate::thuis().join("Library/Application Support/Jera/recent.json")
}

/// De mappen die ik in Jera opende, de laatste eerst. Een map die er niet meer is, valt weg.
fn lees_recent() -> Vec<PathBuf> {
    let tekst = std::fs::read_to_string(recent_pad()).unwrap_or_default();
    let paden: Vec<String> = serde_json::from_str(&tekst).unwrap_or_default();
    paden.into_iter().map(PathBuf::from).filter(|p| p.is_dir()).collect()
}

/// Zet `map` bovenaan de recente mappen.
pub fn onthoud(map: &Path) {
    let mut recent = lees_recent();
    recent.retain(|p| p != map);
    recent.insert(0, map.to_path_buf());
    recent.truncate(MAX_RECENT);
    let pad = recent_pad();
    if let Some(ouder) = pad.parent() {
        let _ = std::fs::create_dir_all(ouder);
    }
    let paden: Vec<String> = recent.iter().map(|p| p.to_string_lossy().into_owned()).collect();
    let _ = std::fs::write(pad, serde_json::json!(paden).to_string());
}

/// Het pad met ~ voor de thuismap.
fn tilde(map: &Path) -> String {
    let thuis = crate::thuis();
    match map.strip_prefix(&thuis) {
        Ok(rest) => format!("~/{}", rest.display()),
        Err(_) => map.display().to_string(),
    }
}

/// Opent de mapkeuze van macOS met `knop` als knop, en geeft de gekozen map.
fn kies_plek(knop: &'static str, cx: &mut App) -> impl std::future::Future<Output = Option<PathBuf>> + use<> {
    let paden =
        cx.prompt_for_paths(PathPromptOptions { files: false, directories: true, multiple: false, prompt: Some(knop.into()) });
    async move { paden.await.ok()?.ok()??.into_iter().next() }
}

/// `gh`, ook als Jera buiten de shell start en Homebrew niet in het PATH staat.
pub(crate) fn gh() -> Command {
    let pad = ["/opt/homebrew/bin/gh", "/usr/local/bin/gh"].into_iter().find(|p| Path::new(p).exists()).unwrap_or("gh");
    Command::new(pad)
}

/// De naam uit eigenaar/naam.
fn repo_naam(repo: &str) -> &str {
    repo.rsplit('/').next().unwrap_or(repo)
}

/// eigenaar/naam van de origin op GitHub.
fn remote(map: &Path) -> Option<String> {
    let uit = Command::new("git").arg("-C").arg(map).args(["config", "--get", "remote.origin.url"]).output().ok()?;
    let url = String::from_utf8_lossy(&uit.stdout).trim().to_string();
    let rest = url.split_once("github.com")?.1.trim_start_matches([':', '/']);
    Some(rest.trim_end_matches('/').trim_end_matches(".git").to_string())
}

/// Mijn account en organisaties op GitHub, met hun repo's.
pub struct GitHub {
    login: String,
    orgs: Vec<String>,
    /// eigenaar/naam en de laatste push, in seconden. Zonder de gearchiveerde.
    repos: Vec<(String, i64)>,
}

fn cache_pad() -> PathBuf {
    crate::thuis().join("Library/Application Support/Jera/github.json")
}

impl GitHub {
    /// Leest GitHub met `gh` en bewaart het voor de volgende keer.
    fn lees() -> Option<Self> {
        let vraag = |args: &[&str]| -> Option<String> {
            let uit = gh().arg("api").args(args).output().ok()?;
            uit.status.success().then(|| String::from_utf8_lossy(&uit.stdout).into_owned())
        };
        let login = vraag(&["user", "-q", ".login"])?.trim().to_string();
        let orgs = vraag(&["user/orgs", "-q", ".[].login"])?.lines().map(str::to_string).collect();
        let repos = vraag(&[
            "user/repos?per_page=100",
            "--paginate",
            "-q",
            ".[] | select(.archived | not) | [.full_name, .pushed_at] | @tsv",
        ])?
        .lines()
        .filter_map(|regel| {
            let (naam, tijd) = regel.split_once('\t')?;
            let tijd = chrono::DateTime::parse_from_rfc3339(tijd).map(|t| t.timestamp()).unwrap_or(0);
            Some((naam.to_string(), tijd))
        })
        .collect();
        let gh = GitHub { login, orgs, repos };
        gh.bewaar();
        Some(gh)
    }

    fn bewaar(&self) {
        let json = serde_json::json!({ "login": self.login, "orgs": self.orgs, "repos": self.repos });
        let pad = cache_pad();
        if let Some(map) = pad.parent() {
            let _ = std::fs::create_dir_all(map);
        }
        let _ = std::fs::write(pad, json.to_string());
    }

    fn uit_cache() -> Option<Self> {
        let json: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(cache_pad()).ok()?).ok()?;
        let tekst = |v: &serde_json::Value| v.as_str().map(str::to_string);
        Some(GitHub {
            login: tekst(&json["login"])?,
            orgs: json["orgs"].as_array()?.iter().filter_map(tekst).collect(),
            repos: json["repos"].as_array()?.iter().filter_map(|r| Some((tekst(&r[0])?, r[1].as_i64()?))).collect(),
        })
    }

    /// De repo's, de laatst gepushte eerst.
    fn repo_namen(&self) -> Vec<String> {
        let mut repos = self.repos.clone();
        repos.sort_by(|a, b| b.1.cmp(&a.1));
        repos.into_iter().map(|(naam, _)| naam).collect()
    }

    /// Waar een nieuwe repo kan komen: mijn account en mijn organisaties. De eigenaar met de
    /// meeste repo's staat eerst.
    pub fn eigenaars() -> Vec<String> {
        let Some(gh) = Self::uit_cache().or_else(Self::lees) else { return Vec::new() };
        let mut eigenaars: Vec<String> = std::iter::once(gh.login.clone()).chain(gh.orgs.clone()).collect();
        let aantal = |e: &str| gh.repos.iter().filter(|(r, _)| r.split('/').next() == Some(e)).count();
        eigenaars.sort_by_key(|e| std::cmp::Reverse(aantal(e)));
        eigenaars
    }
}

#[cfg(feature = "schermafdruk")]
impl Openen {
    pub fn test_zoek(&mut self, tekst: String, window: &mut Window, cx: &mut Context<Self>) {
        self.veld.update(cx, |veld, cx| veld.set_value(tekst, window, cx));
        cx.notify();
    }

    pub fn test_github(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.zet_stand(Stand::Github, window, cx);
    }
}
