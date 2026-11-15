//! Het venster: tabs in de titelbalk, de linkerkolom, de Render-regel, de terminal en het git-paneel.

use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};

use gpui_kit::{
    base::input::{Input, InputEvent, InputState},
    prelude::FluentBuilder,
    *,
};

mod bestandsboom;
mod naamveld;
mod onderdelen;
mod poortenpaneel;
mod sessielijst;

use bestandsboom::{Gesleept, Item, Menu, kort_pad, lees_map};
#[cfg(feature = "schermafdruk")]
use bestandsboom::{Keuze, Sub};
use naamveld::{Doel, NaamVeld};
pub(crate) use onderdelen::{PIJL_DICHT, PIJL_OPEN, knop_vak, lijn, pijl, tab_vak};
use onderdelen::*;
use sessielijst::titel;

use crate::agents::{self, Agent};
use crate::artifacts::{self, Artifact};
use crate::bestandsicoon;
use crate::bewerk;
use crate::gitpaneel::GitPaneel;
use crate::openen::{NieuwProject, OpenProject, Openen, OpenenEvent};
use crate::oud::{self, Oud};
use crate::poorten;
use crate::status::{Gelezen, Lezer, Status};
use crate::taal::t;
use crate::terminal::{TerminalEvent, TerminalView};
use crate::thema::{self as t, Thema};

/// Het commando waarmee elke sessie start, via de eigen shell.
pub const STARTCOMMANDO: &str = "claude";

/// Tussen een pijl, icoon of woord en wat ernaast staat: krapper dan `GAP`.
const GAP_KLEIN: f32 = 6.0;
/// De breedte van de + rechts in een regel. Het pijltje van een dag in oud staat eronder.
const B_PLUS: f32 = 32.0;

/// Zo vaak leest Jera de sessiebestanden.
const STATUS_ELKE: Duration = Duration::from_millis(500);
/// Zonder sessiebestand na deze tijd staat Claude Code op een scherm bij de start, zoals
/// de vraag of ik de map vertrouw. Daarvoor is het nog aan het starten.
const START_DUUR: Duration = Duration::from_secs(3);
/// Zo vaak leest Jera de poorten en agent-browsers.
const POORTEN_ELKE: Duration = Duration::from_secs(2);
/// Zolang het paneel open is, een nieuw plaatje van elke agent-browser na zoveel keer lezen.
const PLAATJE_RONDES: u64 = 2;

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
    /// De naam die ik de sessie gaf, in plaats van de titel van Claude.
    naam: Option<String>,
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

pub struct Jera {
    tabs: Vec<Tab>,
    actief: usize,
    stand: Stand,
    git_open: bool,
    volgende_id: usize,
    menu: Option<Menu>,
    /// Het menu van een twee-vingerklik op de sessie met dit id, op die plek.
    sessie_menu: Option<(usize, Point<Pixels>)>,
    /// De tab met deze hoofdmap wacht op het antwoord of hij dicht mag.
    sluiten: Option<PathBuf>,
    naam_veld: Option<NaamVeld>,
    openen: Entity<Openen>,
    openen_open: bool,
    /// De poorten en agent-browsers van de hele computer, en of hun paneel openstaat.
    poorten: poorten::Stand,
    poorten_open: bool,
    /// Het laatste plaatje van elke agent-browser.
    plaatjes: HashMap<String, Arc<RenderImage>>,
    /// Per sessie-id de artifacts die de sessie publiceerde.
    artifacts: HashMap<String, Vec<Artifact>>,
}

impl Jera {
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
            menu: None,
            sessie_menu: None,
            sluiten: None,
            naam_veld: None,
            openen,
            openen_open: false,
            poorten: poorten::Stand::default(),
            poorten_open: false,
            plaatjes: HashMap::new(),
            artifacts: HashMap::new(),
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
                // De branches staan klaar voordat ik het git-paneel open.
                let git = cx.new(|cx| {
                    let mut git = GitPaneel::new(root.clone(), cx);
                    git.ververs(cx);
                    git
                });
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
            naam: None,
        });
        tab.gekozen = tab.sessies.len() - 1;
        self.actief = index;
        self.stand = Stand::Sessies;
        self.focus(window, cx);
        cx.notify();
    }

    pub fn gekozen_terminal(&self) -> Option<&Entity<TerminalView>> {
        let tab = self.tabs.get(self.actief)?;
        Some(&tab.sessies.get(tab.gekozen)?.terminal)
    }

    /// Leest de status van de sessies, steeds opnieuw. De bestanden leest Jera op de
    /// achtergrond; alleen de uitkomst komt op de main thread.
    pub fn volg(window: &mut Window, cx: &mut Context<Self>) {
        cx.spawn_in(window, async move |this, cx| {
            let mut lezer = Lezer::default();
            loop {
                let Ok(pids) = this.update(cx, |this, cx| this.pids(cx)) else { break };
                let (gelezen, terug) =
                    cx.background_executor().spawn(async move { (lees_sessies(&mut lezer, pids), lezer) }).await;
                lezer = terug;
                if this.update(cx, |this, cx| this.zet_status(gelezen, cx)).is_err() {
                    break;
                }
                cx.background_executor().timer(STATUS_ELKE).await;
            }
        })
        .detach();
        Self::volg_poorten(window, cx);
    }

    /// Leest de poorten en agent-browsers, steeds opnieuw. Zolang het paneel open is, ook
    /// een plaatje van elke agent-browser.
    fn volg_poorten(window: &mut Window, cx: &mut Context<Self>) {
        cx.spawn_in(window, async move |this, cx| {
            for ronde in 0u64.. {
                let (stand, gepubliceerd) =
                    cx.background_executor().spawn(async { (poorten::lees(), artifacts::lees()) }).await;
                let Ok(namen) = this.update_in(cx, |this, window, cx| {
                    if this.poorten != stand {
                        this.poorten = stand;
                        cx.notify();
                    }
                    if this.artifacts != gepubliceerd {
                        this.artifacts = gepubliceerd;
                        cx.notify();
                    }
                    // Het plaatje van een browser die er niet meer is, gaat weg.
                    let browsers = &this.poorten.browsers;
                    let weg: Vec<String> =
                        this.plaatjes.keys().filter(|naam| !browsers.iter().any(|b| &b.naam == *naam)).cloned().collect();
                    for naam in weg {
                        if let Some(beeld) = this.plaatjes.remove(&naam) {
                            cx.drop_image(beeld, Some(window));
                        }
                    }
                    match this.poorten_open && ronde % PLAATJE_RONDES == 0 {
                        true => this.poorten.browsers.iter().map(|b| b.naam.clone()).collect(),
                        false => Vec::new(),
                    }
                }) else {
                    break;
                };
                if !namen.is_empty() {
                    let beelden: Vec<(String, Arc<RenderImage>)> = cx
                        .background_executor()
                        .spawn(async move {
                            namen
                                .into_iter()
                                .filter_map(|naam| {
                                    let beeld = poorten::plaatje(&naam)?;
                                    Some((naam, Arc::new(RenderImage::new(vec![image::Frame::new(beeld)]))))
                                })
                                .collect()
                        })
                        .await;
                    let klaar = this.update_in(cx, |this, window, cx| {
                        for (naam, beeld) in beelden {
                            if let Some(oud) = this.plaatjes.insert(naam, beeld) {
                                cx.drop_image(oud, Some(window));
                            }
                        }
                        cx.notify();
                    });
                    if klaar.is_err() {
                        break;
                    }
                }
                cx.background_executor().timer(POORTEN_ELKE).await;
            }
        })
        .detach();
    }

    /// De pids van de shells van alle sessies.
    fn pids(&self, cx: &App) -> Vec<u32> {
        self.tabs.iter().flat_map(|tab| &tab.sessies).filter_map(|s| s.terminal.read(cx).pid).collect()
    }

    fn zet_status(&mut self, mut gelezen: HashMap<u32, (Gelezen, Vec<Agent>)>, cx: &mut Context<Self>) {
        let open = self.gekozen_id();
        let mut veranderd = false;
        for sessie in self.tabs.iter_mut().flat_map(|tab| tab.sessies.iter_mut()) {
            let pid = sessie.terminal.read(cx).pid;
            let (stand, mut agents) = match pid.and_then(|pid| gelezen.remove(&pid)) {
                Some((stand, agents)) => (Some(stand), agents),
                None => (None, Vec::new()),
            };
            let status = match stand {
                Some(Gelezen::Bezig) => Status::Bezig,
                Some(Gelezen::Antwoord) => Status::Antwoord,
                Some(Gelezen::Toestemming) => Status::Toestemming,
                Some(Gelezen::Klaar) => {
                    sessie.opgestart = true;
                    let net = std::mem::take(&mut sessie.gewerkt) && open != Some(sessie.id);
                    if net || sessie.status == Status::NetKlaar { Status::NetKlaar } else { Status::Idle }
                }
                // De terminal is niet gestart: er draait niets.
                None if pid.is_none() => Status::Idle,
                None if sessie.gestart.elapsed() > START_DUUR => Status::Toestemming,
                None => Status::Bezig,
            };
            if sessie.opgestart && matches!(status, Status::Bezig | Status::Antwoord | Status::Toestemming) {
                sessie.gewerkt = true;
            }
            // Een idle sessie heeft geen agent meer aan het werk: een die nog bezig lijkt, is gestopt.
            if stand == Some(Gelezen::Klaar) {
                agents.retain(|a| a.stand != agents::Stand::Bezig);
            }
            veranderd |= status != sessie.status || agents != sessie.agents;
            sessie.status = status;
            sessie.agents = agents;
        }
        if veranderd {
            cx.notify();
        }
        if let Some(tab) = self.tabs.get(self.actief) {
            let open = self.git_open;
            tab.git.update(cx, |git, cx| {
                git.zet_zichtbaar(open);
                git.ververs(cx);
            });
        }
    }

    /// Voor de testhulp: "kies N" kiest sessie N van de tab, "sessies" en "bestanden" de stand,
    /// "klik pad" klikt in de bestandsboom op dat pad, vanaf de hoofdmap, "zed" en "kleur" kiezen
    /// de iconen zoals het menu, "nl" en "en" de taal, "oud" klapt het blok oud open of dicht,
    /// "dag D" klapt die dag in het blok oud open of dicht, "hervat N" hervat de oude sessie op plek N en "git" klapt het
    /// git-paneel open of dicht. "menu pad" opent het menu van dat pad, "keuze woord" kiest
    /// daarin ("pad" opent de lijst naast pad kopiëren, "vanaf" en "volledig" kiezen daaruit), "naam tekst" typt die naam in het veld en drukt op Enter. "openen" opent het scherm
    /// Openen, "nieuw" opent het met het naamveld, "zoek tekst" typt in het veld, "github" toont de
    /// repo's van GitHub. "agents" klapt
    /// de agents van de gekozen sessie open of dicht. "sessiemenu" opent
    /// het menu van de gekozen sessie, "sluit" sluit de actieve tab, of vraagt dat eerst.
    #[cfg(feature = "schermafdruk")]
    pub fn test_stap(&mut self, stap: &str, window: &mut Window, cx: &mut Context<Self>) {
        match stap {
            "sessies" => self.stand = Stand::Sessies,
            "bestanden" => self.stand = Stand::Bestanden,
            "git" => self.git_open = !self.git_open,
            "poorten" => self.poorten_open = !self.poorten_open,
            "oud" => self.wissel_oud(cx),
            "agents" => {
                let Some(tab) = self.tabs.get(self.actief) else { return };
                let Some(sessie) = tab.sessies.get(tab.gekozen) else { return };
                let id = sessie.id;
                self.wissel_agents(id, sessie.agents_open.unwrap_or(true), cx);
            }
            "sessiemenu" => self.sessie_menu = self.gekozen_id().map(|id| (id, point(px(160.0), px(110.0)))),
            "sluit" => {
                if let Some(root) = self.tabs.get(self.actief).map(|tab| tab.root.clone()) {
                    self.sluit_tab(root, window, cx);
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
            "nl" => window.dispatch_action(Box::new(crate::instellingen::TaalNederlands), cx),
            "en" => window.dispatch_action(Box::new(crate::instellingen::TaalEngels), cx),
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
                        self.zet_sub(Some(Sub::Nieuw), cx);
                        return;
                    }
                    "pad" => {
                        self.zet_sub(Some(Sub::Pad), cx);
                        return;
                    }
                    "kopieer" => Keuze::Kopieer,
                    "volledig" => Keuze::Pad { vanaf_root: false },
                    "vanaf" => Keuze::Pad { vanaf_root: true },
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
            break;
        }
        cx.notify();
    }

    /// Stopt de sessie `id`, met de poorten en browsers die ze startte. Daarna staat hij bij oud.
    fn stop_sessie(&mut self, id: usize, window: &mut Window, cx: &mut Context<Self>) {
        self.sessie_menu = None;
        if let Some(sessie) = self.tabs.iter().flat_map(|tab| &tab.sessies).find(|s| s.id == id) {
            self.ruim_op(sessie, cx);
            sessie.terminal.read(cx).stop();
        }
        self.verwijder(id, cx);
        self.focus(window, cx);
    }

    /// Sluit de tab met `root`, met de poorten en browsers die de sessies daarin startten.
    /// Draait er nog een sessie, dan vraagt Jera het eerst.
    fn sluit_tab(&mut self, root: PathBuf, window: &mut Window, cx: &mut Context<Self>) {
        let Some(index) = self.tabs.iter().position(|tab| tab.root == root) else { return };
        let vragen = self.sluiten.is_none() && !self.tabs[index].sessies.is_empty();
        if vragen {
            self.sluiten = Some(root);
            cx.notify();
            return;
        }
        self.sluiten = None;
        let tab = self.tabs.remove(index);
        for sessie in &tab.sessies {
            self.ruim_op(sessie, cx);
            sessie.terminal.read(cx).stop();
        }
        if self.actief > index || self.actief >= self.tabs.len() {
            self.actief = self.actief.saturating_sub(1);
        }
        self.focus(window, cx);
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

    fn titelbalk(&self, th: &Thema, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let tabs = self.tabs.iter().enumerate().map(|(index, tab)| {
            let actief = index == self.actief;
            // De slechtste status van de sessies in de tab: rood boven geel boven groen.
            let slechtste = tab.sessies.iter().map(|s| s.status).min();
            let root = tab.root.clone();
            // Onder de muis staat een kruisje op de plek van het aantal. Beide staan er altijd,
            // zodat het kruisje er nog is als de muis tijdens de klik iets beweegt.
            let groep = SharedString::from(format!("tab-{index}"));
            let rechts = div()
                .relative()
                .flex()
                .justify_center()
                .min_w(px(t::ICOON))
                .child(
                    div()
                        .text_color(rgb(th.mute))
                        .group_hover(groep.clone(), |s| s.invisible())
                        .child(tab.sessies.len().to_string()),
                )
                .child(
                    div()
                        .id(("sluit", index))
                        .absolute()
                        .inset_0()
                        .flex()
                        .justify_center()
                        .text_color(rgb(th.mute))
                        .invisible()
                        .group_hover(groep.clone(), |s| s.visible())
                        .hover(|d| d.text_color(rgb(th.ink)))
                        .child("×")
                        .on_click(cx.listener(move |this, _, window, cx| {
                            cx.stop_propagation();
                            this.sluit_tab(root.clone(), window, cx);
                        })),
                );
            tab_vak(th, actief)
                .id(("tab", index))
                .group(groep)
                .children(slechtste.map(|status| {
                    let woord = status.woord();
                    div()
                        .id(("tabstatus", index))
                        .flex_none()
                        .child(icoon(status, ("tabdraai", index), th))
                        .tooltip(move |_, cx| cx.new(|_| Tip(vec![woord.into()])).into())
                }))
                .child(naam(&tab.root))
                .child(div().flex().justify_center().min_w(px(t::ICOON)).font_weight(FontWeight::NORMAL).child(rechts))
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
                    .child(t("+ map", "+ folder"))
                    .on_click(cx.listener(|this, _, window, cx| this.toon_openen(false, window, cx)))
                    // Openen hangt onder deze tab. De randen vallen samen met de lijnen van de titelbalk.
                    .when(self.openen_open, |d| {
                        let vak = self.openen.update(cx, |openen, cx| openen.vak(window, cx));
                        d.child(
                            div()
                                .absolute()
                                .top_full()
                                .left(px(-t::LIJN))
                                .mt(px(-t::LIJN))
                                .child(deferred(anchored().snap_to_window().child(vak)).with_priority(2)),
                        )
                    }),
            )
            .child(lijn(th))
            .child(div().flex_1())
            .child(lijn(th))
            .child(
                div()
                    .flex()
                    .items_center()
                    .px(px(t::PAD))
                    .child(svg().data(LOGO).size(px(t::ICOON)).text_color(rgb(th.ink))),
            )
    }

    fn kolom(&self, th: &Thema, cx: &mut Context<Self>) -> impl IntoElement {
        // De sessie die het dringendst op mij wacht, voor de stip op "sessies".
        let wacht = self.tabs.get(self.actief).and_then(|tab| {
            tab.sessies.iter().map(|s| s.status).filter(|s| s.wacht()).min()
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
                .when(!actief, |d| d.hover(|d| d.text_color(rgb(th.ink))))
                .gap(px(GAP_KLEIN))
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
            .child(knop(Stand::Sessies, t("sessies", "sessions")))
            .child(knop(Stand::Bestanden, t("bestanden", "files")));

        let inhoud = match (self.stand, self.tabs.get(self.actief)) {
            (Stand::Sessies, Some(tab)) => self.sessielijst(tab, th, cx).into_any_element(),
            (Stand::Bestanden, Some(tab)) => {
                let hoofdmap = Item { pad: tab.root.clone(), map: true };
                let mut rijen = vec![self.boom_rij(tab, &hoofdmap, 0, Some(kort_pad(&tab.root)), th, cx)];
                self.boom(tab, &tab.root, 1, th, cx, &mut rijen);
                // Neerzetten onder de regels gaat in de hoofdmap.
                let root = tab.root.clone();
                div()
                    .id("boom")
                    .min_h_full()
                    .children(rijen)
                    .on_drop::<Gesleept>({
                        let root = root.clone();
                        cx.listener(move |this, gesleept: &Gesleept, _, cx| this.zet_neer(&gesleept.0, &root, root.clone(), cx))
                    })
                    .on_drop::<ExternalPaths>(
                        cx.listener(move |this, paden: &ExternalPaths, _, cx| this.plak_in(paden.paths().to_vec(), root.clone(), true, cx)),
                    )
                    .into_any_element()
            }
            (Stand::Sessies | Stand::Bestanden, None) => lege_staat(
                th,
                t("Nog geen sessie.", "No session yet."),
                knop_vak(th)
                    .id("leeg-map")
                    .child(t("+ map", "+ folder"))
                    .on_click(cx.listener(|this, _, window, cx| this.toon_openen(false, window, cx))),
            )
            .into_any_element(),
        };
        div()
            .flex()
            .flex_col()
            .flex_none()
            .w(px(t::B_KOLOM))
            .border_r(px(t::LIJN))
            .border_color(rgb(th.line))
            .child(schakelaar)
            .child(
                div()
                    .relative()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_h_0()
                    .child(div().id("kolom").flex_1().overflow_y_scroll().child(inhoud))
                    .when(self.stand == Stand::Sessies && !self.tabs.is_empty(), |d| d.child(toevoegen_vlak(th))),
            )
            .when_some(self.tabs.get(self.actief).filter(|_| self.stand == Stand::Sessies), |d, tab| {
                d.child(self.oud_blok(tab, th, cx))
            })
    }

    /// De vraag of de tab dicht mag terwijl er sessies draaien, met die sessies.
    fn sluit_vraag(&self, root: &Path, th: &Thema, cx: &mut Context<Self>) -> Option<impl IntoElement> {
        let tab = self.tabs.iter().find(|tab| tab.root == root)?;
        let rijen = tab.sessies.iter().map(|sessie| {
            div()
                .flex()
                .items_center()
                .gap(px(t::GAP))
                .h(px(t::H_RIJ))
                .px(px(t::PAD))
                .child(icoon(sessie.status, ("vraagdraai", sessie.id), th))
                .child(div().flex_1().min_w_0().truncate().child(titel(sessie, cx)))
        });
        let root = root.to_path_buf();
        Some(
            deferred(
                div()
                    .id("sluit-achter")
                    .absolute()
                    .inset_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .occlude()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.sluiten = None;
                            cx.notify();
                        }),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .w(px(420.0))
                            .bg(rgb(th.bg))
                            .border(px(t::LIJN))
                            .border_color(rgb(th.line_strong))
                            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .h(px(t::H_TITELBALK))
                                    .px(px(t::PAD))
                                    .bg(rgb(th.bar))
                                    .border_b(px(t::LIJN))
                                    .border_color(rgb(th.line))
                                    .font_weight(FontWeight::BOLD)
                                    .child(match crate::taal::engels() {
                                        true => format!("Close {}?", naam(&tab.root)),
                                        false => format!("{} sluiten?", naam(&tab.root)),
                                    }),
                            )
                            .child(div().py(px(8.0)).children(rijen))
                            .child(
                                div()
                                    .flex()
                                    .justify_end()
                                    .gap(px(8.0))
                                    .p(px(t::PAD))
                                    .border_t(px(t::LIJN))
                                    .border_color(rgb(th.line))
                                    .child(knop_vak(th).id("annuleren").child(t("Annuleren", "Cancel")).on_click(cx.listener(
                                        |this, _, _, cx| {
                                            this.sluiten = None;
                                            cx.notify();
                                        },
                                    )))
                                    .child(
                                        // De hoofdknop: vlak in `ink`, tekst in `bg`.
                                        div()
                                            .id("toch-sluiten")
                                            .flex()
                                            .items_center()
                                            .h(px(t::H_KNOP))
                                            .px(px(8.0))
                                            .border(px(t::LIJN))
                                            .bg(rgb(th.ink))
                                            .border_color(rgb(th.ink))
                                            .text_color(rgb(th.bg))
                                            .font_weight(FontWeight::BOLD)
                                            .child(t("Toch sluiten", "Close anyway"))
                                            .on_click(cx.listener(move |this, _, window, cx| {
                                                this.sluit_tab(root.clone(), window, cx)
                                            })),
                                    ),
                            ),
                    ),
            )
            .with_priority(3),
        )
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
                    .when(!self.git_open, |d| d.hover(|d| d.text_color(rgb(th.ink))))
                    .child("git")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.git_open = !this.git_open;
                        cx.notify();
                    })),
            )
    }
}

impl Render for Jera {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let th = t::thema(window);
        if let Some(tab) = self.tabs.get(self.actief) {
            window.set_window_title(&naam(&tab.root));
        }
        let terminal = self.gekozen_terminal().cloned();
        // Een tab zonder sessies blijft open, met in plaats van de terminal een knop voor een nieuwe.
        let leeg = match (&terminal, self.tabs.get(self.actief)) {
            (None, Some(tab)) => {
                let root = tab.root.clone();
                Some(lege_staat(
                    th,
                    t("Geen sessies.", "No sessions."),
                    knop_vak(th)
                        .id("leeg-sessie")
                        .child(t("Sessie starten", "Start session"))
                        .on_click(cx.listener(move |this, _, window, cx| this.start(root.clone(), window, cx))),
                ))
            }
            _ => None,
        };
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
            .child(self.titelbalk(th, window, cx))
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
                                    .child(div().flex_1().min_w_0().children(terminal).children(leeg))
                                    .children(self.rechts(th)),
                            ),
                    ),
            )
            .child(self.statusbalk(th, cx))
            .when(self.poorten_open, |d| d.child(self.lade(th, cx)))
            .children(self.menu.as_ref().map(|menu| self.menu_vak(menu, th, cx)))
            .children(self.sessie_menu.and_then(|(id, plek)| self.sessie_menu_vak(id, plek, th, cx)))
            .children(self.sluiten.as_deref().and_then(|root| self.sluit_vraag(root, th, cx)))
            .when(self.openen_open, |d| {
                d.child(deferred(self.openen.update(cx, |openen, cx| openen.achter(cx))).with_priority(1))
            })
    }
}

impl Jera {
    /// De kolom rechts van de terminal: het git-paneel.
    fn rechts(&self, th: &Thema) -> Option<impl IntoElement> {
        let git = self.tabs.get(self.actief).filter(|_| self.git_open).map(|tab| tab.git.clone())?;
        Some(div().flex_none().h_full().border_l(px(t::LIJN)).border_color(rgb(th.line)).child(git))
    }

    /// De balk onder het hele venster, met rechts de teller.
    fn statusbalk(&self, th: &Thema, cx: &mut Context<Self>) -> impl IntoElement {
        let teller = self.teller(th, cx);
        div()
            .flex()
            .flex_none()
            .h(px(t::H_STATUSBALK))
            .bg(rgb(th.bar))
            .border_t(px(t::LIJN))
            .border_color(rgb(th.line))
            .child(div().flex_1())
            .when_some(teller, |d, teller| d.child(lijn(th)).child(teller))
    }
}

/// Wat er staat als er niets is: één regel in `mute` en de knop die het laat verschijnen.
/// Over de sessielijst zolang ik een bestand uit Finder erboven sleep. Het vlak vangt de drop
/// niet zelf: die gaat naar de map eronder.
fn toevoegen_vlak(th: &Thema) -> impl IntoElement {
    let rand = px(8.0);
    div()
        .id("toevoegen")
        .absolute()
        .top(rand)
        .bottom(rand)
        .left(rand)
        .right(rand)
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(t::GAP))
        .bg(rgba((th.bg << 8) | 0xE6))
        .border(px(t::LIJN))
        .border_dashed()
        .border_color(rgb(th.accent))
        .text_color(rgb(th.ink))
        .invisible()
        .drag_over::<ExternalPaths>(|d, _, _, _| d.visible())
        .child(svg().data(NIEUW).size(px(t::ICOON)).text_color(rgb(th.accent)))
        .child(t("bestand toevoegen", "add file"))
}

fn lege_staat(th: &Thema, regel: &'static str, knop: impl IntoElement) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .items_start()
        .gap(px(t::PAD))
        .p(px(t::PAD))
        .text_color(rgb(th.mute))
        .child(regel)
        .child(knop)
}

pub(crate) fn naam(map: &Path) -> String {
    map.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| map.display().to_string())
}

/// Wat de bestanden van Claude Code zeggen over de sessies met deze pids: de stand en de agents.
fn lees_sessies(lezer: &mut Lezer, pids: Vec<u32>) -> HashMap<u32, (Gelezen, Vec<Agent>)> {
    let mut bestanden = lezer.lees();
    pids.into_iter()
        .filter_map(|pid| {
            let bestand = bestanden.remove(&pid)?;
            Some((pid, (bestand.gelezen, agents::lees(&bestand.id, &bestand.cwd))))
        })
        .collect()
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
