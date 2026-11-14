//! Het venster: tabs in de titelbalk, de linkerkolom, de Render-regel, de terminal en het git-paneel.

use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use gpui_kit::{
    base::input::{Input, InputEvent, InputState},
    prelude::FluentBuilder,
    *,
};

use crate::agents::{self, Agent};
use crate::bestandsicoon;
use crate::bewerk;
use crate::gitpaneel::GitPaneel;
use crate::openen::{NieuwProject, OpenProject, Openen, OpenenEvent};
use crate::oud::{self, Oud};
use crate::status::{Gelezen, Lezer, Status};
use crate::terminal::{TerminalEvent, TerminalView};
use crate::thema::{self as t, Thema};

/// Het commando waarmee elke sessie start, via de eigen shell.
pub const STARTCOMMANDO: &str = "claude";

/// Zo vaak leest Cockpit de sessiebestanden.
const STATUS_ELKE: Duration = Duration::from_millis(500);
/// Zonder sessiebestand na deze tijd staat Claude Code op een scherm bij de start, zoals
/// de vraag of ik de map vertrouw. Daarvoor is het nog aan het starten.
const START_DUUR: Duration = Duration::from_secs(3);

#[derive(Clone, Copy, PartialEq)]
enum Stand {
    Sessies,
    Bestanden,
}

struct Sessie {
    id: usize,
    map: PathBuf,
    terminal: Entity<TerminalView>,
    status: Status,
    /// Claude heeft gewerkt sinds de laatste keer dat het klaar was.
    gewerkt: bool,
    /// Claude was een keer klaar. Daarvoor is het bezig met opstarten, en dat is geen werk.
    opgestart: bool,
    gestart: Instant,
    agents: Vec<Agent>,
    /// Agents onder de sessieregel open of dicht. Zolang ik niets koos, is alleen de gekozen sessie open.
    agents_open: Option<bool>,
    team_dicht: bool,
}

/// Een map of bestand in de bestandsboom.
struct Item {
    pad: PathBuf,
    map: bool,
}

/// Eén tab per git-repo. Een map zonder git is zijn eigen tab.
struct Tab {
    root: PathBuf,
    sessies: Vec<Sessie>,
    gekozen: usize,
    /// De mappen die openstaan in Bestanden, met wat erin staat. De hoofdmap staat eerst open.
    open: HashMap<PathBuf, Vec<Item>>,
    /// Het bestand dat ik het laatst heb geopend.
    bestand: Option<PathBuf>,
    /// De oude sessies, of het blok oud openstaat en welke dagen erin dicht zijn.
    oud: Vec<Oud>,
    oud_open: bool,
    oud_dicht: HashSet<String>,
    git: Entity<GitPaneel>,
    /// De mappen waarin nu geplakt wordt, een keer per keer plakken.
    plakken: Vec<PathBuf>,
}

impl Tab {
    fn new(root: PathBuf, git: Entity<GitPaneel>) -> Self {
        let open = HashMap::from([(root.clone(), lees_map(&root))]);
        Self { root, sessies: Vec::new(), gekozen: 0, open, bestand: None, oud: Vec::new(), oud_open: false, oud_dicht: HashSet::new(), git, plakken: Vec::new() }
    }

    /// Leest de open mappen opnieuw. Een map die er niet meer is, gaat dicht.
    fn ververs(&mut self) {
        self.open.retain(|map, _| map.is_dir());
        for (map, items) in &mut self.open {
            *items = lees_map(map);
        }
    }
}

/// Het menu van een twee-vingerklik op een regel in de bestandsboom.
struct Menu {
    pad: PathBuf,
    plek: Point<Pixels>,
    /// Wat er op het klembord van macOS staat om te plakken.
    plakbaar: Vec<PathBuf>,
    /// De lijst met soorten naast nieuw staat open.
    nieuw_open: bool,
}

/// Wat ik onder nieuw kan maken, na de map: het woord en de extensie.
const SOORTEN: [(&str, &str); 13] = [
    ("bestand", ""),
    ("markdown", ".md"),
    ("rust", ".rs"),
    ("python", ".py"),
    ("typescript", ".ts"),
    ("javascript", ".js"),
    ("html", ".html"),
    ("css", ".css"),
    ("json", ".json"),
    ("toml", ".toml"),
    ("yaml", ".yml"),
    ("shell", ".sh"),
    ("tekst", ".txt"),
];

/// Een naam die ik in de bestandsboom typ: een nieuwe naam, of die van iets nieuws.
struct NaamVeld {
    doel: Doel,
    invoer: Entity<InputState>,
    fout: Option<&'static str>,
    _volg: Subscription,
}

enum Doel {
    Hernoem(PathBuf),
    /// Een map, of een bestand met deze extensie.
    Nieuw { map: PathBuf, ext: Option<&'static str> },
}

#[derive(Clone, Copy)]
enum Keuze {
    Nieuw(Option<&'static str>),
    Kopieer,
    Plak,
    Hernoem,
    Prullenmand,
}

pub struct Cockpit {
    tabs: Vec<Tab>,
    actief: usize,
    stand: Stand,
    git_open: bool,
    volgende_id: usize,
    lezer: Lezer,
    menu: Option<Menu>,
    naam_veld: Option<NaamVeld>,
    openen: Entity<Openen>,
    openen_open: bool,
}

impl Cockpit {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let openen = cx.new(|cx| Openen::new(window, cx));
        cx.observe(&openen, |_, _, cx| cx.notify()).detach();
        cx.subscribe_in(&openen, window, |this, _, event, window, cx| match event {
            OpenenEvent::Gekozen(map) => {
                this.openen_open = false;
                match this.tabs.iter().position(|tab| map.starts_with(&tab.root)) {
                    Some(index) => {
                        this.actief = index;
                        this.focus(window, cx);
                    }
                    None => this.start(map.clone(), window, cx),
                }
                cx.notify();
            }
            OpenenEvent::AndereMap => {
                this.openen_open = false;
                this.kies_map(window, cx);
            }
            OpenenEvent::Sluit => {
                this.openen_open = false;
                this.focus(window, cx);
                cx.notify();
            }
        })
        .detach();
        Self {
            tabs: Vec::new(),
            actief: 0,
            stand: Stand::Sessies,
            git_open: false,
            volgende_id: 0,
            lezer: Lezer::default(),
            menu: None,
            naam_veld: None,
            openen,
            openen_open: false,
        }
    }

    /// Opent het scherm Openen, met een naamveld als het om een nieuw project gaat.
    pub fn toon_openen(&mut self, nieuw: bool, window: &mut Window, cx: &mut Context<Self>) {
        self.openen.update(cx, |openen, cx| openen.toon(nieuw, window, cx));
        self.openen_open = true;
        cx.notify();
    }

    /// Start een sessie in `map`, in de tab van zijn repo.
    pub fn start(&mut self, map: PathBuf, window: &mut Window, cx: &mut Context<Self>) {
        self.start_met(map, STARTCOMMANDO, window, cx);
    }

    fn start_met(&mut self, map: PathBuf, commando: &str, window: &mut Window, cx: &mut Context<Self>) {
        let root = repo_root(&map);
        crate::openen::onthoud(&root);
        let terminal = cx.new(|cx| TerminalView::new(map.clone(), commando, cx));
        let id = self.volgende_id;
        self.volgende_id += 1;
        cx.subscribe(&terminal, move |this, _, event, cx| match event {
            TerminalEvent::Titel => cx.notify(),
            TerminalEvent::Gestopt => this.verwijder(id, cx),
        })
        .detach();

        let index = match self.tabs.iter().position(|tab| tab.root == root) {
            Some(index) => index,
            None => {
                let git = cx.new(|_| GitPaneel::new(root.clone()));
                self.tabs.push(Tab::new(root.clone(), git));
                self.laad_oud(root, cx);
                self.tabs.len() - 1
            }
        };
        let tab = &mut self.tabs[index];
        tab.sessies.push(Sessie {
            id,
            map,
            terminal,
            status: Status::Bezig,
            gewerkt: false,
            opgestart: false,
            gestart: Instant::now(),
            agents: Vec::new(),
            agents_open: None,
            team_dicht: false,
        });
        tab.gekozen = tab.sessies.len() - 1;
        self.actief = index;
        self.stand = Stand::Sessies;
        self.focus(window, cx);
        cx.notify();
    }

    /// Leest de oude sessies van de tab met `root` opnieuw, op de achtergrond.
    fn laad_oud(&mut self, root: PathBuf, cx: &mut Context<Self>) {
        cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let r = root.clone();
            let oud = cx.background_executor().spawn(async move { oud::lees(&r) }).await;
            let _ = this.update(cx, |this, cx| {
                if let Some(tab) = this.tabs.iter_mut().find(|tab| tab.root == root) {
                    tab.oud = oud;
                    cx.notify();
                }
            });
        })
        .detach();
    }

    /// Klapt het blok oud open of dicht. Open leest het de oude sessies opnieuw.
    fn wissel_oud(&mut self, cx: &mut Context<Self>) {
        let Some(tab) = self.tabs.get_mut(self.actief) else { return };
        tab.oud_open = !tab.oud_open;
        if tab.oud_open {
            let root = tab.root.clone();
            self.laad_oud(root, cx);
        }
        cx.notify();
    }

    /// Klapt een dag in het blok oud open of dicht.
    fn wissel_dag(&mut self, dag: String, cx: &mut Context<Self>) {
        let Some(tab) = self.tabs.get_mut(self.actief) else { return };
        if !tab.oud_dicht.remove(&dag) {
            tab.oud_dicht.insert(dag);
        }
        cx.notify();
    }

    /// Zet een oude sessie voort in een nieuwe terminal.
    fn hervat(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(tab) = self.tabs.get_mut(self.actief) else { return };
        if index >= tab.oud.len() {
            return;
        }
        let oud = tab.oud.remove(index);
        self.start_met(oud.map, &format!("{STARTCOMMANDO} --resume {}", oud.id), window, cx);
    }

    pub fn gekozen_terminal(&self) -> Option<&Entity<TerminalView>> {
        let tab = self.tabs.get(self.actief)?;
        Some(&tab.sessies.get(tab.gekozen)?.terminal)
    }

    /// Leest de status van de sessies, steeds opnieuw.
    pub fn volg(window: &mut Window, cx: &mut Context<Self>) {
        cx.spawn_in(window, async move |this, cx| {
            loop {
                if this.update(cx, |this, cx| this.lees_status(cx)).is_err() {
                    break;
                }
                cx.background_executor().timer(STATUS_ELKE).await;
            }
        })
        .detach();
    }

    fn lees_status(&mut self, cx: &mut Context<Self>) {
        let gelezen = self.lezer.lees();
        let open = self.gekozen_id();
        let mut veranderd = false;
        for sessie in self.tabs.iter_mut().flat_map(|tab| tab.sessies.iter_mut()) {
            let bestand = gelezen.get(&sessie.terminal.read(cx).pid);
            let mut agents = bestand.map(|b| agents::lees(&b.id, &b.cwd)).unwrap_or_default();
            let status = match bestand.map(|b| b.gelezen) {
                Some(Gelezen::Bezig) => Status::Bezig,
                Some(Gelezen::Antwoord) => Status::Antwoord,
                Some(Gelezen::Toestemming) => Status::Toestemming,
                Some(Gelezen::Klaar) => {
                    sessie.opgestart = true;
                    let net = std::mem::take(&mut sessie.gewerkt) && open != Some(sessie.id);
                    if net || sessie.status == Status::NetKlaar { Status::NetKlaar } else { Status::Idle }
                }
                None if sessie.gestart.elapsed() > START_DUUR => Status::Toestemming,
                None => Status::Bezig,
            };
            if sessie.opgestart && matches!(status, Status::Bezig | Status::Antwoord | Status::Toestemming) {
                sessie.gewerkt = true;
            }
            // Een idle sessie heeft geen agent meer aan het werk: een die nog bezig lijkt, is gestopt.
            if bestand.is_some_and(|b| b.gelezen == Gelezen::Klaar) {
                for agent in agents.iter_mut().filter(|a| a.stand == agents::Stand::Bezig) {
                    agent.stand = agents::Stand::Gestopt;
                }
            }
            veranderd |= status != sessie.status || agents != sessie.agents;
            sessie.status = status;
            sessie.agents = agents;
        }
        if veranderd {
            cx.notify();
        }
        if let Some(tab) = self.tabs.get(self.actief).filter(|_| self.git_open) {
            tab.git.update(cx, |git, cx| git.ververs(cx));
        }
    }

    /// Voor de testhulp: "kies N" kiest sessie N van de tab, "sessies" en "bestanden" de stand,
    /// "klik pad" klikt in de bestandsboom op dat pad, vanaf de hoofdmap, "zed" en "kleur" kiezen
    /// de iconen zoals het menu, "oud" klapt het blok oud open of dicht, "dag D" klapt die dag
    /// in het blok oud open of dicht, "hervat N" hervat de oude sessie op plek N en "git" klapt het
    /// git-paneel open of dicht. "menu pad" opent het menu van dat pad, "keuze woord" kiest
    /// daarin, "naam tekst" typt die naam in het veld en drukt op Enter. "openen" opent het scherm
    /// Openen, "nieuw" opent het met het naamveld, "zoek tekst" typt in het veld, "github" toont de
    /// repo's van GitHub. "agents" klapt
    /// de agents van de gekozen sessie open of dicht, "team" het team daarvan.
    #[cfg(feature = "schermafdruk")]
    pub fn test_stap(&mut self, stap: &str, window: &mut Window, cx: &mut Context<Self>) {
        match stap {
            "sessies" => self.stand = Stand::Sessies,
            "bestanden" => self.stand = Stand::Bestanden,
            "git" => self.git_open = !self.git_open,
            "oud" => self.wissel_oud(cx),
            "agents" | "team" => {
                let Some(tab) = self.tabs.get(self.actief) else { return };
                let Some(sessie) = tab.sessies.get(tab.gekozen) else { return };
                let id = sessie.id;
                match stap {
                    "agents" => self.wissel_agents(id, sessie.agents_open.unwrap_or(true), cx),
                    _ => self.wissel_team(id, cx),
                }
            }
            "openen" => self.toon_openen(false, window, cx),
            "nieuw" => self.toon_openen(true, window, cx),
            _ if stap.starts_with("zoek ") => {
                let zoek = stap[5..].to_string();
                self.openen.update(cx, |openen, cx| openen.test_zoek(zoek, window, cx));
            }
            "github" => self.openen.update(cx, |openen, cx| openen.test_github(window, cx)),
            _ if stap.starts_with("dag ") => self.wissel_dag(stap[4..].to_string(), cx),
            _ if stap.starts_with("hervat ") => {
                if let Ok(n) = stap[7..].parse() {
                    self.hervat(n, window, cx);
                }
            }
            "zed" => window.dispatch_action(Box::new(crate::instellingen::IconenZed), cx),
            "kleur" => window.dispatch_action(Box::new(crate::instellingen::IconenKleur), cx),
            _ if stap.starts_with("menu ") => {
                if let Some(root) = self.tabs.get(self.actief).map(|tab| tab.root.clone()) {
                    self.open_menu(root.join(&stap[5..]), point(px(160.0), px(120.0)), cx);
                }
            }
            _ if stap.starts_with("keuze ") => {
                let keuze = match &stap[6..] {
                    "nieuw" => Keuze::Nieuw(Some(".md")),
                    "map" => Keuze::Nieuw(None),
                    "soorten" => {
                        self.zet_nieuw_open(true, cx);
                        return;
                    }
                    "kopieer" => Keuze::Kopieer,
                    "plak" => Keuze::Plak,
                    "hernoem" => Keuze::Hernoem,
                    _ => Keuze::Prullenmand,
                };
                self.kies(keuze, window, cx);
            }
            _ if stap.starts_with("naam ") => {
                if let Some(veld) = &self.naam_veld {
                    veld.invoer.update(cx, |invoer, cx| invoer.set_value(stap[5..].to_string(), window, cx));
                    self.bevestig_naam(false, window, cx);
                }
            }
            _ if stap.starts_with("klik ") => {
                if let Some(root) = self.tabs.get(self.actief).map(|tab| tab.root.clone()) {
                    self.klik_boom(&root.join(&stap[5..]), cx);
                }
            }
            _ => {
                if let (Some(n), Some(tab)) =
                    (stap.strip_prefix("kies ").and_then(|n| n.parse().ok()), self.tabs.get_mut(self.actief))
                {
                    tab.gekozen = n;
                    self.focus(window, cx);
                }
            }
        }
        cx.notify();
    }

    fn gekozen_id(&self) -> Option<usize> {
        let tab = self.tabs.get(self.actief)?;
        Some(tab.sessies.get(tab.gekozen)?.id)
    }

    /// Geeft de gekozen sessie de focus. Was die net klaar, dan heb ik hem nu gezien.
    fn focus(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(tab) = self.tabs.get_mut(self.actief) {
            if let Some(sessie) = tab.sessies.get_mut(tab.gekozen) {
                if sessie.status == Status::NetKlaar {
                    sessie.status = Status::Idle;
                }
            }
        }
        if let Some(terminal) = self.gekozen_terminal() {
            let focus = terminal.read(cx).focus.clone();
            window.focus(&focus, cx);
        }
    }

    fn verwijder(&mut self, id: usize, cx: &mut Context<Self>) {
        for index in 0..self.tabs.len() {
            let tab = &mut self.tabs[index];
            let Some(pos) = tab.sessies.iter().position(|s| s.id == id) else { continue };
            // Een gestopte sessie is nu een oude sessie.
            let root = tab.root.clone();
            self.laad_oud(root, cx);
            let tab = &mut self.tabs[index];
            tab.sessies.remove(pos);
            if tab.gekozen > pos || tab.gekozen >= tab.sessies.len() {
                tab.gekozen = tab.gekozen.saturating_sub(1);
            }
            if tab.sessies.is_empty() {
                self.tabs.remove(index);
                if self.actief >= index {
                    self.actief = self.actief.saturating_sub(1);
                }
            }
            break;
        }
        cx.notify();
    }

    fn kies_map(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let paden = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: None,
        });
        cx.spawn_in(window, async move |this, cx| {
            let Ok(Ok(Some(paden))) = paden.await else { return };
            let Some(map) = paden.into_iter().next() else { return };
            let _ = this.update_in(cx, |this, window, cx| this.start(map, window, cx));
        })
        .detach();
    }

    fn titelbalk(&self, th: &Thema, cx: &mut Context<Self>) -> impl IntoElement {
        let tabs = self.tabs.iter().enumerate().map(|(index, tab)| {
            let actief = index == self.actief;
            tab_vak(th, actief)
                .id(("tab", index))
                .child(naam(&tab.root))
                .child(div().font_weight(FontWeight::NORMAL).text_color(rgb(th.mute)).child(tab.sessies.len().to_string()))
                .on_click(cx.listener(move |this, _, window, cx| {
                    this.actief = index;
                    this.focus(window, cx);
                    cx.notify();
                }))
        });
        div()
            .flex()
            .flex_none()
            .h(px(t::H_TITELBALK))
            .pl(px(t::B_LICHTEN))
            .bg(rgb(th.bar))
            .border_b(px(t::LIJN))
            .border_color(rgb(th.line))
            .children(tabs.flat_map(|tab| [lijn(th).into_any_element(), tab.into_any_element()]))
            .child(lijn(th))
            .child(
                tab_vak(th, false)
                    .id("map")
                    .child("+ map")
                    .on_click(cx.listener(|this, _, window, cx| this.toon_openen(false, window, cx))),
            )
            .child(lijn(th))
    }

    fn kolom(&self, th: &Thema, cx: &mut Context<Self>) -> impl IntoElement {
        // De sessie die het dringendst op mij wacht, voor de stip op "sessies".
        let wacht = self.tabs.get(self.actief).and_then(|tab| {
            tab.sessies.iter().map(|s| s.status).filter(|s| s.wacht()).min_by_key(|s| *s as u8)
        });
        let knop = |stand: Stand, label: &'static str| {
            let actief = self.stand == stand;
            let stip = match wacht {
                Some(status) if stand == Stand::Sessies && !actief => Some(kleur(status, th)),
                _ => None,
            };
            div()
                .id(label)
                .flex()
                .items_center()
                .pt(px(t::STREEP))
                .border_b(px(t::STREEP))
                .border_color(transparent_black())
                .text_color(rgb(th.mute))
                .when(actief, |d| {
                    d.text_color(rgb(th.ink)).font_weight(FontWeight::BOLD).border_color(rgb(th.accent))
                })
                .gap(px(6.0))
                .child(label)
                .children(stip.map(|kleur| div().size(px(6.0)).rounded_full().bg(rgb(kleur))))
                .on_click(cx.listener(move |this, _, _, cx| {
                    if stand == Stand::Bestanden {
                        if let Some(tab) = this.tabs.get_mut(this.actief) {
                            tab.ververs();
                        }
                    }
                    this.stand = stand;
                    cx.notify();
                }))
        };
        let schakelaar = div()
            .flex()
            .flex_none()
            .gap(px(18.0))
            .h(px(t::H_SCHAKELAAR))
            .px(px(t::PAD))
            .border_b(px(t::LIJN))
            .border_color(rgb(th.line))
            .child(knop(Stand::Sessies, "sessies"))
            .child(knop(Stand::Bestanden, "bestanden"));

        let inhoud = match (self.stand, self.tabs.get(self.actief)) {
            (Stand::Sessies, Some(tab)) => self.sessielijst(tab, th, cx).into_any_element(),
            (Stand::Bestanden, Some(tab)) => {
                let hoofdmap = Item { pad: tab.root.clone(), map: true };
                let mut rijen = vec![self.boom_rij(tab, &hoofdmap, 0, Some(kort_pad(&tab.root)), th, cx)];
                self.boom(tab, &tab.root, 1, th, cx, &mut rijen);
                div().children(rijen).into_any_element()
            }
            (Stand::Sessies | Stand::Bestanden, None) => lege_staat(th, cx).into_any_element(),
        };
        div()
            .flex()
            .flex_col()
            .flex_none()
            .w(px(t::B_KOLOM))
            .border_r(px(t::LIJN))
            .border_color(rgb(th.line))
            .child(schakelaar)
            .child(div().id("kolom").flex_1().overflow_y_scroll().child(inhoud))
            .when_some(self.tabs.get(self.actief).filter(|_| self.stand == Stand::Sessies), |d, tab| {
                d.child(self.oud_blok(tab, th, cx))
            })
    }

    /// De sessies van de tab, per map: de hoofdmap eerst, dan de submappen.
    fn sessielijst(&self, tab: &Tab, th: &Thema, cx: &mut Context<Self>) -> impl IntoElement {
        let mut mappen: Vec<&Path> = tab.sessies.iter().map(|s| s.map.as_path()).collect();
        mappen.sort_by_key(|map| (*map != tab.root, map.to_path_buf()));
        mappen.dedup();

        let groepen = mappen.into_iter().map(|map| {
            let label = map_label(map, &tab.root);
            let map_buf = map.to_path_buf();
            let kop = div()
                .flex()
                .items_center()
                .h(px(t::H_RIJ + 2.0))
                .mt(px(8.0))
                .pl(px(t::PAD))
                .text_color(rgb(th.mute))
                .child(div().flex_1().truncate().child(label))
                .child(
                    div()
                        .id(SharedString::from(format!("plus-{}", map.display())))
                        .flex()
                        .items_center()
                        .justify_center()
                        .w(px(32.0))
                        .h_full()
                        .child("+")
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.start(map_buf.clone(), window, cx)
                        })),
                );
            let rijen = tab.sessies.iter().enumerate().filter(|(_, s)| s.map == map).map(
                |(index, sessie)| {
                    let gekozen = index == tab.gekozen;
                    let open = sessie.agents_open.unwrap_or(gekozen);
                    let id = sessie.id;
                    let regel = div()
                        .id(("sessie", sessie.id))
                        .flex()
                        .items_center()
                        .gap(px(t::GAP))
                        .h(px(t::H_RIJ))
                        .pl(px(t::PAD - t::STREEP))
                        .pr(px(t::PAD))
                        .border_l(px(t::STREEP))
                        .border_color(transparent_black())
                        .when(sessie.status == Status::Idle, |d| d.text_color(rgb(th.mute)))
                        .when(gekozen, |d| {
                            d.bg(rgb(th.sel)).font_weight(FontWeight::BOLD).border_color(rgb(th.accent))
                        })
                        .child(icoon(sessie.status, ("draai", sessie.id), th))
                        .child(div().flex_1().min_w_0().truncate().child(titel(sessie, cx)))
                        .when(!sessie.agents.is_empty(), |d| {
                            let bezig = sessie.agents.iter().filter(|a| a.stand == agents::Stand::Bezig).count();
                            d.child(
                                div()
                                    .id(("agents", id))
                                    .flex()
                                    .flex_none()
                                    .child(pijl(if open { PIJL_OPEN } else { PIJL_DICHT }, th))
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        cx.stop_propagation();
                                        this.wissel_agents(id, open, cx);
                                    })),
                            )
                            .when(!open, |d| {
                                d.child(
                                    div()
                                        .flex_none()
                                        .font_weight(FontWeight::NORMAL)
                                        .text_color(rgb(th.mute))
                                        .child(format!("{bezig}/{}", sessie.agents.len())),
                                )
                            })
                        })
                        .on_click(cx.listener(move |this, _, window, cx| this.kies_sessie(index, window, cx)));
                    div().child(regel).when(open, |d| d.children(agent_rijen(sessie, index, th, cx)))
                },
            );
            div().child(kop).children(rijen)
        });
        div().children(groepen)
    }

    /// Kiest sessie `index` van de tab.
    fn kies_sessie(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(tab) = self.tabs.get_mut(self.actief) {
            tab.gekozen = index;
        }
        self.focus(window, cx);
        cx.notify();
    }

    /// Klapt de agents van sessie `id` open of dicht.
    fn wissel_agents(&mut self, id: usize, open: bool, cx: &mut Context<Self>) {
        if let Some(sessie) = self.tabs.iter_mut().flat_map(|tab| tab.sessies.iter_mut()).find(|s| s.id == id) {
            sessie.agents_open = Some(!open);
        }
        cx.notify();
    }

    /// Klapt het team van sessie `id` open of dicht.
    fn wissel_team(&mut self, id: usize, cx: &mut Context<Self>) {
        if let Some(sessie) = self.tabs.iter_mut().flat_map(|tab| tab.sessies.iter_mut()).find(|s| s.id == id) {
            sessie.team_dicht = !sessie.team_dicht;
        }
        cx.notify();
    }

    /// Onderaan de kolom: het blok oud, standaard dicht. Open schuift de kop omhoog en
    /// toont eronder de oude sessies per dag van het laatste bericht, de nieuwste eerst.
    /// Een dag klapt dicht; een klik op een sessie zet die voort.
    /// Er zijn `OUD_ZICHTBAAR` regels te zien; de rest scrolt binnen het blok.
    fn oud_blok(&self, tab: &Tab, th: &Thema, cx: &mut Context<Self>) -> impl IntoElement {
        let kop = div()
            .id("oud")
            .flex()
            .items_center()
            .gap(px(6.0))
            .flex_none()
            .h(px(t::H_RIJ))
            .pl(px(t::PAD))
            .text_color(rgb(th.mute))
            .child(pijl(if tab.oud_open { PIJL_OPEN } else { PIJL_DICHT }, th))
            .child("oud")
            .child(div().ml(px(2.0)).child(tab.oud.len().to_string()))
            .on_click(cx.listener(|this, _, _, cx| this.wissel_oud(cx)));
        let mut lijst = div().id("oud-lijst").overflow_y_scroll();
        let mut regels = 0;
        if tab.oud_open {
            let mut vorige = None;
            for (index, oud) in tab.oud.iter().enumerate() {
                let dag = dag(oud.tijd);
                let dicht = tab.oud_dicht.contains(&dag);
                if vorige.as_ref() != Some(&dag) {
                    regels += 1;
                    let d = dag.clone();
                    lijst = lijst.child(
                        div()
                            .id(SharedString::from(format!("dag-{dag}")))
                            .flex()
                            .items_center()
                            .h(px(t::H_RIJ))
                            .pl(px(t::PAD))
                            .text_size(px(11.0))
                            .text_color(rgb(th.mute))
                            .child(div().flex_1().child(dag.clone()))
                            // Rechts, onder de + van de mappen in de sessielijst.
                            .child(
                                div()
                                    .flex()
                                    .justify_center()
                                    .w(px(32.0))
                                    .child(pijl(if dicht { PIJL_DICHT } else { PIJL_OPEN }, th)),
                            )
                            .on_click(cx.listener(move |this, _, _, cx| this.wissel_dag(d.clone(), cx))),
                    );
                    vorige = Some(dag);
                }
                if dicht {
                    continue;
                }
                regels += 1;
                lijst = lijst.child(
                    div()
                        .id(("hervat", index))
                        .flex()
                        .items_center()
                        .h(px(t::H_RIJ))
                        .px(px(t::PAD))
                        .hover(|d| d.bg(rgb(th.sel)))
                        .child(div().flex_1().min_w_0().truncate().child(oud.titel.clone()))
                        .on_click(cx.listener(move |this, _, window, cx| this.hervat(index, window, cx))),
                );
            }
        }
        div()
            .flex_none()
            .border_t(px(t::LIJN))
            .border_color(rgb(th.line))
            .child(kop)
            .when(regels > 0, |d| d.child(lijst.h(px(t::H_RIJ * regels.min(OUD_ZICHTBAAR) as f32))))
    }

    /// Een map gaat open of dicht, een bestand opent in de app die macOS ervoor heeft.
    fn klik_boom(&mut self, pad: &Path, cx: &mut Context<Self>) {
        let Some(tab) = self.tabs.get_mut(self.actief) else { return };
        if !pad.is_dir() {
            cx.open_with_system(pad);
            tab.bestand = Some(pad.to_path_buf());
        } else if tab.open.contains_key(pad) {
            // Dicht, met alles eronder.
            tab.open.retain(|map, _| !map.starts_with(pad));
        } else {
            tab.open.insert(pad.to_path_buf(), lees_map(pad));
        }
        cx.notify();
    }

    fn open_menu(&mut self, pad: PathBuf, plek: Point<Pixels>, cx: &mut Context<Self>) {
        let plakbaar = cx
            .read_from_clipboard()
            .and_then(|item| {
                item.entries().iter().find_map(|entry| match entry {
                    ClipboardEntry::ExternalPaths(paden) => Some(paden.paths().to_vec()),
                    _ => None,
                })
            })
            .unwrap_or_default();
        self.menu = Some(Menu { pad, plek, plakbaar, nieuw_open: false });
        cx.notify();
    }

    fn kies(&mut self, keuze: Keuze, window: &mut Window, cx: &mut Context<Self>) {
        let Some(Menu { pad, plakbaar, .. }) = self.menu.take() else { return };
        let Some(tab) = self.tabs.get_mut(self.actief) else { return };
        // Nieuw en plakken gaan in de map zelf, of in de map waar het bestand in staat.
        let map = if pad.is_dir() { pad.clone() } else { pad.parent().map(Path::to_path_buf).unwrap_or_default() };
        match keuze {
            Keuze::Nieuw(ext) => {
                tab.open.entry(map.clone()).or_insert_with(|| lees_map(&map));
                // De cursor staat voor de extensie.
                self.start_naam(Doel::Nieuw { map, ext }, ext.unwrap_or(""), 0..0, window, cx);
            }
            Keuze::Hernoem => {
                let naam = naam(&pad);
                let stam = match naam.rfind('.') {
                    Some(i) if i > 0 => i,
                    _ => naam.len(),
                };
                self.start_naam(Doel::Hernoem(pad), &naam, 0..stam, window, cx);
            }
            Keuze::Kopieer => cx.background_executor().spawn(async move { bewerk::kopieer(&pad) }).detach(),
            // Een grote map kopiëren duurt even; het venster wacht er niet op.
            Keuze::Plak => {
                tab.open.entry(map.clone()).or_insert_with(|| lees_map(&map));
                tab.plakken.push(map.clone());
                let root = tab.root.clone();
                cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
                    let m = map.clone();
                    cx.background_executor().spawn(async move { bewerk::plak(&plakbaar, &m) }).await;
                    let _ = this.update(cx, |this, cx| {
                        if let Some(tab) = this.tabs.iter_mut().find(|tab| tab.root == root) {
                            if let Some(i) = tab.plakken.iter().position(|p| *p == map) {
                                tab.plakken.remove(i);
                            }
                            tab.ververs();
                            cx.notify();
                        }
                    });
                })
                .detach();
            }
            Keuze::Prullenmand => {
                bewerk::prullenmand(&pad);
                if tab.bestand.as_ref().is_some_and(|b| b.starts_with(&pad)) {
                    tab.bestand = None;
                }
                tab.ververs();
            }
        }
        cx.notify();
    }

    fn zet_nieuw_open(&mut self, open: bool, cx: &mut Context<Self>) {
        if let Some(menu) = self.menu.as_mut().filter(|m| m.nieuw_open != open) {
            menu.nieuw_open = open;
            cx.notify();
        }
    }

    /// Opent het veld voor een naam, met `naam` erin en `kies` daarvan gekozen.
    fn start_naam(
        &mut self,
        doel: Doel,
        naam: &str,
        kies: std::ops::Range<usize>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let invoer = cx.new(|cx| {
            let mut invoer = InputState::new(window, cx);
            invoer.set_value(naam.to_string(), window, cx);
            invoer.set_selected_range(kies, cx);
            invoer
        });
        invoer.update(cx, |invoer, cx| invoer.focus(window, cx));
        let volg = cx.subscribe_in(&invoer, window, |this, _, event, window, cx| match event {
            InputEvent::PressEnter { .. } => this.bevestig_naam(false, window, cx),
            InputEvent::Blur => this.bevestig_naam(true, window, cx),
            _ => {}
        });
        self.naam_veld = Some(NaamVeld { doel, invoer, fout: None, _volg: volg });
        cx.notify();
    }

    /// Hernoemt of maakt wat in het veld staat. Bij een fout blijft het veld open, of gaat
    /// het dicht als ik ergens anders heb geklikt (`weg`).
    fn bevestig_naam(&mut self, weg: bool, window: &mut Window, cx: &mut Context<Self>) {
        let Some(veld) = self.naam_veld.as_mut() else { return };
        let naam = veld.invoer.read(cx).value().trim().to_string();
        let uitkomst = match &veld.doel {
            Doel::Nieuw { ext, .. } if naam == ext.unwrap_or("") => Err("geen naam"),
            Doel::Hernoem(pad) => bewerk::hernoem(pad, &naam).map(|nieuw| (pad.clone(), nieuw)),
            Doel::Nieuw { map, ext } => bewerk::maak(map, &naam, ext.is_none()).map(|nieuw| (nieuw.clone(), nieuw)),
        };
        match uitkomst {
            Err(fout) if !weg => {
                veld.fout = Some(fout);
                cx.notify();
                return;
            }
            Err(_) => {}
            Ok((oud, nieuw)) => {
                if let Some(tab) = self.tabs.get_mut(self.actief) {
                    // Open mappen en het gekozen bestand verhuizen mee.
                    let verhuis = |p: PathBuf| match p.strip_prefix(&oud) {
                        Ok(rest) if rest.as_os_str().is_empty() => nieuw.clone(),
                        Ok(rest) => nieuw.join(rest),
                        Err(_) => p,
                    };
                    tab.open = tab.open.drain().map(|(map, items)| (verhuis(map), items)).collect();
                    tab.bestand = tab.bestand.take().map(verhuis);
                    tab.ververs();
                }
            }
        }
        self.stop_naam(window, cx);
    }

    fn stop_naam(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.naam_veld.take().is_some() {
            self.focus(window, cx);
        }
        cx.notify();
    }

    /// Het menu op de plek van de klik. Onder de muis gaat naast nieuw de lijst met soorten open.
    fn menu_vak(&self, menu: &Menu, th: &Thema, cx: &mut Context<Self>) -> impl IntoElement {
        let hoofdmap = self.tabs.get(self.actief).is_some_and(|tab| tab.root == menu.pad);
        // Een regel: icoon, woord en rechts eventueel iets in `mute`.
        let regel = |id: &'static str, icoon: AnyElement, label: &'static str, rechts: Option<AnyElement>| {
            div()
                .id(id)
                .flex()
                .items_center()
                .gap(px(t::GAP))
                .h(px(t::H_RIJ))
                .px(px(t::PAD))
                .child(icoon)
                .child(div().flex_1().child(label))
                .children(rechts)
        };
        let actie = |keuze: Keuze, icoon: &'static [u8], label: &'static str, kan: bool| {
            regel(label, actie_icoon(icoon, th), label, None)
                .on_hover(cx.listener(|this, hover: &bool, _, cx| {
                    if *hover {
                        this.zet_nieuw_open(false, cx);
                    }
                }))
                .when(kan, |d| {
                    d.hover(|d| d.bg(rgb(th.sel)))
                        .on_click(cx.listener(move |this, _, window, cx| this.kies(keuze, window, cx)))
                })
                .when(!kan, |d| d.text_color(rgb(th.mute)))
        };
        let nieuw = regel("nieuw", actie_icoon(NIEUW, th), "nieuw", Some(pijl(PIJL_DICHT, th).into_any_element()))
            .when(menu.nieuw_open, |d| d.bg(rgb(th.sel)))
            .on_hover(cx.listener(|this, hover: &bool, _, cx| {
                if *hover {
                    this.zet_nieuw_open(true, cx);
                }
            }))
            .on_click(cx.listener(|this, _, _, cx| this.zet_nieuw_open(true, cx)));
        let scheiding = || div().h(px(t::LIJN)).my(px(4.0)).bg(rgb(th.line));
        let vak = || {
            div()
                .flex()
                .flex_col()
                .min_w(px(180.0))
                .py(px(4.0))
                .bg(rgb(th.bg))
                .border(px(t::LIJN))
                .border_color(rgb(th.line_strong))
                // De muis gaat niet door het menu heen naar de bestandsboom.
                .occlude()
        };
        let soorten = menu.nieuw_open.then(|| {
            let soort = |id: &'static str, icoon: AnyElement, label: &'static str, ext: Option<&'static str>| {
                let rechts = ext.filter(|e| !e.is_empty()).map(|e| div().text_color(rgb(th.mute)).child(e).into_any_element());
                regel(id, icoon, label, rechts)
                    .hover(|d| d.bg(rgb(th.sel)))
                    .on_click(cx.listener(move |this, _, window, cx| this.kies(Keuze::Nieuw(ext), window, cx)))
            };
            vak()
                // De randen van beide vakken vallen samen.
                .ml(px(-t::LIJN))
                .child(soort("soort-map", bestandsicoon::map(false, th), "map", None))
                .child(scheiding())
                .children(SOORTEN.iter().map(|(label, ext)| {
                    let icoon = bestandsicoon::bestand(Path::new(&format!("naam{ext}")), th);
                    soort(label, icoon, label, Some(*ext))
                }))
        });
        deferred(
            anchored().position(menu.plek).snap_to_window_with_margin(px(8.0)).child(
                div()
                    .id("menu")
                    .flex()
                    .items_start()
                    .on_mouse_down_out(cx.listener(|this, _, _, cx| {
                        this.menu = None;
                        cx.notify();
                    }))
                    .child(
                        vak()
                            .child(nieuw)
                            .child(scheiding())
                            .child(actie(Keuze::Kopieer, KOPIEER, "kopiëren", true))
                            .child(actie(Keuze::Plak, PLAK, "plakken", !menu.plakbaar.is_empty()))
                            .when(!hoofdmap, |d| {
                                d.child(scheiding())
                                    .child(actie(Keuze::Hernoem, HERNOEM, "naam wijzigen", true))
                                    .child(actie(Keuze::Prullenmand, PRULLENMAND, "naar prullenmand", true))
                            }),
                    )
                    .children(soorten),
            ),
        )
        .with_priority(1)
    }

    /// Het veld waarin ik een naam typ, met rechts wat er mis is.
    fn naam_invoer(&self, veld: &NaamVeld, th: &Thema, cx: &mut Context<Self>) -> AnyElement {
        div()
            .flex()
            .flex_1()
            .min_w_0()
            .items_center()
            .gap(px(6.0))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .px(px(4.0))
                    .border(px(t::LIJN))
                    .border_color(rgb(th.line_strong))
                    .child(Input::new(&veld.invoer)),
            )
            .children(veld.fout.map(|fout| div().flex_none().text_color(rgb(th.mute)).child(fout)))
            // Een klik in het veld is geen klik op de regel.
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .capture_key_down(cx.listener(|this, e: &KeyDownEvent, window, cx| {
                if e.keystroke.key == "escape" {
                    cx.stop_propagation();
                    this.stop_naam(window, cx);
                }
            }))
            .into_any_element()
    }

    /// De regels van de bestandsboom onder `map`, met de open mappen erin.
    fn boom(
        &self,
        tab: &Tab,
        map: &Path,
        diepte: usize,
        th: &Thema,
        cx: &mut Context<Self>,
        rijen: &mut Vec<AnyElement>,
    ) {
        let Some(items) = tab.open.get(map) else { return };
        if let Some(veld) = &self.naam_veld {
            if let Doel::Nieuw { map: m, ext } = &veld.doel {
                if m == map {
                    let icoon = match ext {
                        None => bestandsicoon::map(false, th),
                        Some(ext) => bestandsicoon::bestand(Path::new(&format!("naam{ext}")), th),
                    };
                    rijen.push(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(6.0))
                            .h(px(t::H_RIJ))
                            .pl(px(t::PAD + t::INSPRING * diepte as f32 + t::ICOON + 6.0))
                            .pr(px(t::PAD))
                            .child(icoon)
                            .child(self.naam_invoer(veld, th, cx))
                            .into_any_element(),
                    );
                }
            }
        }
        for item in items {
            rijen.push(self.boom_rij(tab, item, diepte, None, th, cx));
            if item.map && tab.open.contains_key(&item.pad) {
                self.boom(tab, &item.pad, diepte + 1, th, cx, rijen);
            }
        }
    }

    /// Eén regel van de bestandsboom. Bij de hoofdmap staat `waar` naast de naam.
    fn boom_rij(
        &self,
        tab: &Tab,
        item: &Item,
        diepte: usize,
        waar: Option<String>,
        th: &Thema,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let open = item.map && tab.open.contains_key(&item.pad);
        // Met het menu open is de regel van het menu gekozen, anders het laatst geopende bestand.
        let gekozen = match &self.menu {
            Some(menu) => menu.pad == item.pad,
            None => tab.bestand.as_ref() == Some(&item.pad),
        };
        let pad = item.pad.clone();
        let mut naam = naam(&item.pad);
        if item.map {
            naam.push('/');
        }
        let pijl = match (item.map, open) {
            (false, _) => div().flex_none().size(px(t::ICOON)).into_any_element(),
            (true, false) => pijl(PIJL_DICHT, th).into_any_element(),
            (true, true) => pijl(PIJL_OPEN, th).into_any_element(),
        };
        let icoon = match item.map {
            true => bestandsicoon::map(open, th),
            false => bestandsicoon::bestand(&item.pad, th),
        };
        let hernoem = self.naam_veld.as_ref().filter(|v| matches!(&v.doel, Doel::Hernoem(p) if *p == item.pad));
        let tekst = match (hernoem, waar) {
            (Some(veld), _) => self.naam_invoer(veld, th, cx),
            (None, None) => div().flex_1().min_w_0().truncate().child(naam).into_any_element(),
            (None, Some(waar)) => div()
                .flex()
                .flex_1()
                .min_w_0()
                .gap(px(6.0))
                .child(div().flex_none().child(naam))
                .child(div().min_w_0().truncate().text_color(rgb(th.mute)).child(waar))
                .into_any_element(),
        };
        div()
            .id(SharedString::from(format!("boom-{}", item.pad.display())))
            .flex()
            .items_center()
            .gap(px(6.0))
            .h(px(t::H_RIJ))
            .pl(px(t::PAD - t::STREEP + t::INSPRING * diepte as f32))
            .when(!item.map, |d| d.pr(px(t::PAD)))
            .border_l(px(t::STREEP))
            .border_color(transparent_black())
            .hover(|d| d.bg(rgb(th.sel)))
            .when(gekozen, |d| {
                d.bg(rgb(th.sel)).font_weight(FontWeight::BOLD).border_color(rgb(th.accent))
            })
            .child(pijl)
            .child(icoon)
            .child(tekst)
            // Het boogje van bezig zolang er in deze map geplakt wordt.
            .when(tab.plakken.contains(&item.pad), |d| {
                d.child(
                    svg()
                        .data(BOOG)
                        .flex_none()
                        .size(px(t::ICOON))
                        .text_color(rgb(th.ok))
                        .with_animation(
                            SharedString::from(format!("plakt-{}", item.pad.display())),
                            Animation::new(Duration::from_millis(1100)).repeat(),
                            |s, d| s.with_transformation(Transformation::rotate(percentage(d))),
                        ),
                )
            })
            .when(item.map, |d| {
                let map = pad.clone();
                d.child(
                    div()
                        .id(SharedString::from(format!("boom-plus-{}", pad.display())))
                        .flex()
                        .flex_none()
                        .items_center()
                        .justify_center()
                        .w(px(32.0))
                        .h_full()
                        .text_color(rgb(th.mute))
                        .child("+")
                        .on_click(cx.listener(move |this, _, window, cx| {
                            cx.stop_propagation();
                            this.start(map.clone(), window, cx);
                        })),
                )
            })
            .on_mouse_down(MouseButton::Right, {
                let pad = pad.clone();
                cx.listener(move |this, e: &MouseDownEvent, _, cx| this.open_menu(pad.clone(), e.position, cx))
            })
            .on_click(cx.listener(move |this, _, _, cx| this.klik_boom(&pad, cx)))
            .into_any_element()
    }

    fn render_regel(&self, th: &Thema, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_none()
            .h(px(t::H_RENDER))
            .border_b(px(t::LIJN))
            .border_color(rgb(th.line))
            .child(div().flex_1().min_w_0())
            .child(lijn(th))
            .child(
                div()
                    .id("git")
                    .flex()
                    .flex_none()
                    .items_center()
                    .px(px(t::TAB_PAD))
                    .pt(px(t::STREEP))
                    .border_b(px(t::STREEP))
                    .border_color(transparent_black())
                    .text_color(rgb(th.mute))
                    .when(self.git_open, |d| d.text_color(rgb(th.ink)).border_color(rgb(th.accent)))
                    .child("git")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.git_open = !this.git_open;
                        cx.notify();
                    })),
            )
    }
}

impl Render for Cockpit {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let th = t::thema(window);
        if let Some(tab) = self.tabs.get(self.actief) {
            window.set_window_title(&naam(&tab.root));
        }
        let terminal = self.gekozen_terminal().cloned();
        div()
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(th.bg))
            .on_action(cx.listener(|this, _: &OpenProject, window, cx| this.toon_openen(false, window, cx)))
            .on_action(cx.listener(|this, _: &NieuwProject, window, cx| this.toon_openen(true, window, cx)))
            .text_color(rgb(th.ink))
            .font_family(t::LETTER)
            .text_size(px(t::TEKST))
            .line_height(relative(t::REGEL))
            .whitespace_nowrap()
            .child(self.titelbalk(th, cx))
            .child(
                div()
                    .flex()
                    .flex_1()
                    .min_h_0()
                    .child(self.kolom(th, cx))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .flex_1()
                            .min_w_0()
                            .child(self.render_regel(th, cx))
                            .child(
                                div()
                                    .flex()
                                    .flex_1()
                                    .min_h_0()
                                    .child(div().flex_1().min_w_0().children(terminal))
                                    .when_some(self.tabs.get(self.actief).filter(|_| self.git_open), |d, tab| {
                                        d.child(
                                            div()
                                                .flex_none()
                                                .h_full()
                                                .border_l(px(t::LIJN))
                                                .border_color(rgb(th.line))
                                                .child(tab.git.clone()),
                                        )
                                    }),
                            ),
                    ),
            )
            .children(self.menu.as_ref().map(|menu| self.menu_vak(menu, th, cx)))
            .when(self.openen_open, |d| {
                d.child(deferred(self.openen.update(cx, |openen, cx| openen.vak(window, cx))).with_priority(2))
            })
    }
}

/// Een vak in de titelbalk: vlak met een streep erboven als het actief is.
fn tab_vak(th: &Thema, actief: bool) -> Div {
    div()
        .flex()
        .items_center()
        .gap(px(8.0))
        .mb(px(-t::LIJN))
        .px(px(t::TAB_PAD))
        .border_t(px(t::STREEP))
        .border_color(transparent_black())
        .text_color(rgb(th.mute))
        .when(actief, |d| {
            d.bg(rgb(th.bg)).text_color(rgb(th.ink)).font_weight(FontWeight::BOLD).border_color(rgb(th.accent))
        })
}

const BOOG: &[u8] = include_bytes!("../iconen/bezig.svg");
const SLOT: &[u8] = include_bytes!("../iconen/toestemming.svg");
const VINK: &[u8] = include_bytes!("../iconen/klaar.svg");
const STREEPJE: &[u8] = include_bytes!("../iconen/idle.svg");
const PIJL_DICHT: &[u8] = include_bytes!("../iconen/pijl-dicht.svg");
const PIJL_OPEN: &[u8] = include_bytes!("../iconen/pijl-open.svg");

const NIEUW: &[u8] = include_bytes!("../iconen/nieuw.svg");
const KOPIEER: &[u8] = include_bytes!("../iconen/kopieer.svg");
const PLAK: &[u8] = include_bytes!("../iconen/plak.svg");
const HERNOEM: &[u8] = include_bytes!("../iconen/hernoem.svg");
const PRULLENMAND: &[u8] = include_bytes!("../iconen/prullenmand.svg");

/// Het icoon van een actie in het menu, in `mute`.
fn actie_icoon(data: &'static [u8], th: &Thema) -> AnyElement {
    svg().data(data).flex_none().size(px(t::ICOON)).text_color(rgb(th.mute)).into_any_element()
}

fn pijl(data: &[u8], th: &Thema) -> Svg {
    svg().data(data).flex_none().size(px(t::ICOON)).text_color(rgb(th.mute))
}

/// Deze namen staan niet in de bestandsboom.
const VERBORGEN: [&str; 2] = [".git", ".DS_Store"];

/// Wat er in `map` staat: eerst de mappen, dan de bestanden, op naam.
fn lees_map(map: &Path) -> Vec<Item> {
    let mut items: Vec<Item> = std::fs::read_dir(map)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| !VERBORGEN.contains(&e.file_name().to_string_lossy().as_ref()))
        .map(|e| Item { map: e.path().is_dir(), pad: e.path() })
        .collect();
    items.sort_by_cached_key(|i| (!i.map, naam(&i.pad).to_lowercase()));
    items
}

fn kleur(status: Status, th: &Thema) -> u32 {
    match status {
        Status::Bezig | Status::NetKlaar => th.ok,
        Status::Antwoord => th.warn,
        Status::Toestemming => th.bad,
        Status::Idle => th.mute,
    }
}

/// Het statusicoon uit het design system, zonder woord.
fn icoon(status: Status, id: impl Into<ElementId>, th: &Thema) -> AnyElement {
    let maat = px(t::ICOON);
    let beeld = |data: &[u8]| svg().data(data).flex_none().size(maat).text_color(rgb(kleur(status, th)));
    match status {
        Status::Bezig => beeld(BOOG)
            .with_animation(id, Animation::new(Duration::from_millis(1100)).repeat(), |s, d| {
                s.with_transformation(Transformation::rotate(percentage(d)))
            })
            .into_any_element(),
        Status::Antwoord => div()
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .size(maat)
            .rounded_full()
            .bg(rgb(th.warn))
            .text_color(rgb(th.bg))
            .text_size(px(11.0))
            .font_weight(FontWeight::BOLD)
            .child("?")
            .into_any_element(),
        Status::Toestemming => beeld(SLOT).into_any_element(),
        Status::NetKlaar => beeld(VINK).into_any_element(),
        Status::Idle => beeld(STREEPJE).into_any_element(),
    }
}

/// De agents onder een sessieregel: het team onder een kopje dat dichtklapt, met elk teamlid,
/// en daaronder een kopje met alleen het aantal subagents. Een klik op een teamlid kiest de sessie,
/// die op plek `plek` van de tab staat.
fn agent_rijen(sessie: &Sessie, plek: usize, th: &Thema, cx: &mut Context<Cockpit>) -> Vec<AnyElement> {
    let id = sessie.id;
    let inspring = px(t::PAD + t::ICOON + t::GAP);
    let kopje = |label: &'static str| {
        div()
            .flex()
            .items_center()
            .gap(px(6.0))
            .h(px(20.0))
            .pl(inspring)
            .pr(px(t::PAD))
            .text_size(px(11.0))
            .text_color(rgb(th.mute))
            .child(div().flex_1().child(label))
    };
    let mut rijen = Vec::new();
    let team: Vec<(usize, &Agent)> = sessie.agents.iter().enumerate().filter(|(_, a)| a.kleur.is_some()).collect();
    if !team.is_empty() {
        let dicht = sessie.team_dicht;
        rijen.push(
            kopje("team")
                .id(("team", id))
                .child(pijl(if dicht { PIJL_DICHT } else { PIJL_OPEN }, th))
                .child(team.len().to_string())
                .on_click(cx.listener(move |this, _, _, cx| this.wissel_team(id, cx)))
                .into_any_element(),
        );
        if !dicht {
            for (index, agent) in team {
                rijen.push(
                    div()
                        .id(SharedString::from(format!("teamlid-{id}-{index}")))
                        .flex()
                        .items_center()
                        .gap(px(t::GAP))
                        .h(px(t::H_RIJ))
                        .pl(inspring)
                        .pr(px(t::PAD))
                        .child(agent_icoon(agent.stand, SharedString::from(format!("draai-{id}-{index}")), th))
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .truncate()
                                .text_color(rgb(th.team[agent.kleur.unwrap_or(0)]))
                                .child(agent.tekst.clone()),
                        )
                        .on_click(cx.listener(move |this, _, window, cx| this.kies_sessie(plek, window, cx)))
                        .into_any_element(),
                );
            }
        }
    }
    let subagents = sessie.agents.iter().filter(|a| a.kleur.is_none()).count();
    if subagents > 0 {
        rijen.push(kopje("subagents").child(subagents.to_string()).into_any_element());
    }
    rijen
}

/// Het statusicoon van een agent: bezig, klaar of gestopt.
/// Dezelfde vormen als bij een sessie: het boogje, het vinkje en het streepje.
fn agent_icoon(stand: agents::Stand, id: SharedString, th: &Thema) -> AnyElement {
    let status = match stand {
        agents::Stand::Bezig => Status::Bezig,
        agents::Stand::Klaar => Status::NetKlaar,
        agents::Stand::Gestopt => Status::Idle,
    };
    icoon(status, id, th)
}

/// Een staande lijn tussen twee vakken.
fn lijn(th: &Thema) -> Div {
    div().flex_none().w(px(t::LIJN)).h_full().bg(rgb(th.line))
}

/// De naam van een groep: de map ten opzichte van de hoofdmap, of de naam van de hoofdmap.
fn map_label(map: &Path, root: &Path) -> String {
    match map.strip_prefix(root) {
        Ok(rel) if !rel.as_os_str().is_empty() => format!("{}/", rel.display()),
        _ => format!("{}/", naam(map)),
    }
}

/// Zoveel regels van het blok oud zijn tegelijk te zien, onder de kop.
const OUD_ZICHTBAAR: usize = 8;

const MAANDEN: [&str; 12] = ["jan", "feb", "mrt", "apr", "mei", "jun", "jul", "aug", "sep", "okt", "nov", "dec"];

/// De dag van `tijd`: vandaag, gisteren of bijvoorbeeld 9 nov.
fn dag(tijd: std::time::SystemTime) -> String {
    use chrono::Datelike;
    let datum = chrono::DateTime::<chrono::Local>::from(tijd).date_naive();
    let vandaag = chrono::Local::now().date_naive();
    match (vandaag - datum).num_days() {
        0 => "vandaag".into(),
        1 => "gisteren".into(),
        _ if datum.year() == vandaag.year() => format!("{} {}", datum.day(), MAANDEN[datum.month0() as usize]),
        _ => format!("{} {} {}", datum.day(), MAANDEN[datum.month0() as usize], datum.year()),
    }
}

/// Een gewone knop uit het design system: een woord met een rand.
pub(crate) fn knop_vak(th: &Thema) -> Div {
    div()
        .flex()
        .flex_none()
        .items_center()
        .h(px(20.0))
        .px(px(8.0))
        .border(px(t::LIJN))
        .border_color(rgb(th.line_strong))
        .text_color(rgb(th.ink))
}

fn lege_staat(th: &Thema, cx: &mut Context<Cockpit>) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .items_start()
        .gap(px(t::PAD))
        .p(px(t::PAD))
        .text_color(rgb(th.mute))
        .child("Nog geen sessie.")
        .child(
            knop_vak(th)
                .id("leeg-map")
                .child("+ map")
                .on_click(cx.listener(|this, _, window, cx| this.toon_openen(false, window, cx))),
        )
}

/// De titel die Claude Code zet, zonder het tekentje ervoor.
/// Zolang die "Claude Code" is, de naam van de map.
fn titel(sessie: &Sessie, cx: &App) -> String {
    let titel = sessie.terminal.read(cx).title.trim_start_matches(|c: char| !c.is_alphanumeric());
    if titel.is_empty() || titel == "Claude Code" { naam(&sessie.map) } else { titel.to_string() }
}

/// De map waar `map` in staat, met ~ voor de thuismap.
fn kort_pad(map: &Path) -> String {
    let ouder = map.parent().unwrap_or(map);
    let thuis = PathBuf::from(std::env::var("HOME").unwrap_or_default());
    match ouder.strip_prefix(&thuis) {
        Ok(rest) if rest.as_os_str().is_empty() => "~".to_string(),
        Ok(rest) => format!("~/{}", rest.display()),
        Err(_) => ouder.display().to_string(),
    }
}

pub(crate) fn naam(map: &Path) -> String {
    map.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| map.display().to_string())
}

/// De hoofdmap van de repo van `map`. Een worktree valt onder zijn repo.
fn repo_root(map: &Path) -> PathBuf {
    let uit = std::process::Command::new("git")
        .args(["rev-parse", "--path-format=absolute", "--git-common-dir"])
        .current_dir(map)
        .output();
    match uit {
        Ok(uit) if uit.status.success() => {
            let git_dir = PathBuf::from(String::from_utf8_lossy(&uit.stdout).trim());
            match git_dir.file_name() {
                Some(n) if n == ".git" => git_dir.parent().map(Path::to_path_buf).unwrap_or(git_dir),
                _ => git_dir,
            }
        }
        _ => map.to_path_buf(),
    }
}
