//! Het git-paneel: de branches van de repo als graaf, rechts naast de terminal. Een klik op een branch
//! opent zijn menu: commit, push, update, checkout en verwijderen; bij de hoofdbranch ook opruimen.
//! Commit opent de commit view op de plek van de graaf, zie `commit.rs`.
//! De stijl komt uit `stijl/design-system/components/GitPaneel`:
//! recht, de namen getrapt boven de lijnen, de dagen rechts op hun lijn.

use std::{
    cell::{Cell, RefCell},
    collections::{BTreeMap, HashMap, HashSet},
    path::PathBuf,
    rc::Rc,
    sync::Arc,
    time::{Duration, Instant},
};

use chrono::{Datelike, Local, TimeZone, Timelike};
use gpui_kit::{prelude::FluentBuilder, *};

use crate::git::{self, Branch, Commit, Repo};
use crate::openen::{GitHub, gh};
use crate::taal;
use crate::thema::{self as t, Thema};

mod commit;

/// Zo oud mogen de gelezen branches zijn als het paneel open is.
const VERVERS: Duration = Duration::from_secs(20);
/// Zo vaak haalt het paneel op bij GitHub, zolang het open is.
const OPHALEN: Duration = Duration::from_secs(300);

// de maten van de graaf
const LW: f32 = 43.0; // afstand tussen twee lijnen
const X0: f32 = 22.0; // de eerste lijn
const RHMIN: f32 = 26.0; // kleinste afstand tussen twee rijen
const RN: f32 = 18.0; // een regel in de kop met de namen
const RUIMTE: f32 = 4.0; // kleinste afstand tussen twee randen
const LIJN: f32 = 1.5;
const PUNT: f32 = 3.0;
const KOP: f32 = 6.5;
const KOP_LIJN: f32 = 2.0;
/// Breedte van een teken in JetBrains Mono, per px lettergrootte.
const TEKEN: f32 = 0.6;

pub struct GitPaneel {
    root: PathBuf,
    repo: Option<Arc<Repo>>,
    geen_git: bool,
    leest: bool,
    gelezen: Option<Instant>,
    agents: bool,
    open: HashSet<String>,
    oorsprong: Rc<Cell<Point<Pixels>>>,
    doelen: Rc<RefCell<Vec<Doel>>>,
    /// `Some(false)` als de repo geen remote heeft: dan staat bovenaan Op GitHub.
    remote: Option<bool>,
    /// Waar de repo op GitHub kan komen, en welke gekozen is.
    eigenaars: Vec<String>,
    eigenaar: usize,
    maak: Maak,
    zichtbaar: bool,
    opgehaald: Option<Instant>,
    haalt_op: bool,
    /// Claude is bezig in deze repo: dan geen checkout, want die verandert de bestanden onder Claude.
    claude_bezig: bool,
    /// Het menu van een branch: de naam en waar ik klikte.
    menu: Option<(String, Point<Pixels>)>,
    /// De commit view, zolang hij open is: dan staat hij op de plek van de graaf.
    commit: Option<commit::CommitView>,
    /// Wat Jera nu doet, zoals "pushen feat/x".
    bezig: Option<String>,
    fout: Option<String>,
}

#[derive(Clone, Copy, PartialEq)]
enum Actie {
    /// In het menu opent dit de commit view; daarin commit het, en pusht het daarna als `push`.
    Commit { push: bool },
    Push { nieuw: bool },
    Update,
    Checkout,
    Verwijder,
    Opruimen,
}

#[derive(Clone, Copy, PartialEq)]
enum Maak {
    Niet,
    Bezig,
    Mislukt,
}

impl GitPaneel {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            repo: None,
            geen_git: false,
            leest: false,
            gelezen: None,
            agents: false,
            open: HashSet::new(),
            oorsprong: Rc::default(),
            doelen: Rc::default(),
            remote: None,
            eigenaars: Vec::new(),
            eigenaar: 0,
            maak: Maak::Niet,
            zichtbaar: false,
            opgehaald: None,
            haalt_op: false,
            claude_bezig: false,
            menu: None,
            commit: None,
            bezig: None,
            fout: None,
        }
    }

    pub fn zet_claude_bezig(&mut self, bezig: bool, cx: &mut Context<Self>) {
        if bezig != self.claude_bezig {
            self.claude_bezig = bezig;
            cx.notify();
        }
    }

    fn sluit_menu(&mut self, cx: &mut Context<Self>) {
        self.menu = None;
        cx.notify();
    }

    /// Doet een actie uit het menu op de achtergrond, en leest daarna opnieuw.
    fn doe(&mut self, actie: Actie, naam: String, cx: &mut Context<Self>) {
        let Some(repo) = self.repo.clone() else { return };
        let Some(b) = repo.branches.iter().find(|b| b.naam == naam) else { return };
        if self.bezig.is_some() {
            return;
        }
        let root = self.root.clone();
        let map = b.map.clone();
        let tak = naam.clone();
        // commit en push: de push volgt op een gelukte commit
        let daarna = (actie == Actie::Commit { push: true }).then_some(Actie::Push { nieuw: b.origin.is_none() });
        let taak: Box<dyn FnOnce() -> Result<(), String> + Send> = match actie {
            Actie::Commit { .. } => {
                let Some(view) = &self.commit else { return };
                let (bericht, paden, map) = (view.invoer.read(cx).value().trim().to_string(), view.gekozen(), view.map.clone());
                if bericht.is_empty() || paden.is_empty() {
                    return;
                }
                Box::new(move || git::commit(&map, &bericht, &paden))
            }
            Actie::Push { nieuw } => Box::new(move || git::push(&root, &naam, nieuw)),
            Actie::Update => Box::new(move || git::update(&root, &naam, map.as_deref())),
            Actie::Checkout => Box::new(move || git::checkout(&root, &naam)),
            Actie::Verwijder => {
                let (lokaal, op_github) = (!b.alleen_origin, b.alleen_origin || b.origin.is_some());
                Box::new(move || git::verwijder(&root, &naam, lokaal, op_github))
            }
            Actie::Opruimen => {
                let weg: Vec<(String, bool, bool)> =
                    repo.branches.iter().filter(|b| gemerged(b)).map(|b| (b.naam.clone(), !b.alleen_origin, b.alleen_origin || b.origin.is_some())).collect();
                Box::new(move || weg.iter().try_for_each(|(n, lokaal, op_github)| git::verwijder(&root, n, *lokaal, *op_github)))
            }
        };
        let woord = match actie {
            Actie::Commit { .. } => taal::t("committen", "committing"),
            Actie::Push { .. } => taal::t("pushen", "pushing"),
            Actie::Update => taal::t("updaten", "updating"),
            Actie::Checkout => taal::t("checkout", "checking out"),
            Actie::Verwijder => taal::t("verwijderen", "deleting"),
            Actie::Opruimen => taal::t("opruimen", "cleaning up"),
        };
        self.bezig = Some(if actie == Actie::Opruimen { woord.to_string() } else { format!("{woord} {}", kort(&b.naam)) });
        self.fout = None;
        self.menu = None;
        cx.notify();
        cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let uitkomst = cx.background_executor().spawn(async move { taak() }).await;
            let _ = this.update(cx, |p, cx| {
                p.bezig = None;
                let gelukt = uitkomst.is_ok();
                match uitkomst {
                    Ok(()) => p.commit = None,
                    Err(fout) => p.fout = Some(fout),
                }
                match daarna.filter(|_| gelukt) {
                    Some(push) => p.doe(push, tak, cx),
                    None => {
                        p.gelezen = None;
                        p.ververs(cx);
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    /// Het menu van een branch, op de plek van de klik: alleen wat bij die branch kan.
    fn menu_vak(&self, th: &Thema, cx: &mut Context<Self>) -> Option<AnyElement> {
        let (naam, plek) = self.menu.clone()?;
        let repo = self.repo.clone()?;
        let b = repo.branches.iter().find(|b| b.naam == naam)?;
        let regel = |id: &'static str| div().id(id).flex().items_center().gap(px(t::GAP)).h(px(t::H_RIJ)).px(px(t::PAD));
        let mut vak = crate::venster::menuvak(th).id("branchmenu").min_w(px(200.0)).on_mouse_down_out(cx.listener(|p, _, _, cx| {
            if p.bezig.is_none() {
                p.sluit_menu(cx);
            }
        }));
        let lijst = acties(&repo, b, self.remote == Some(true));
        if lijst.is_empty() {
            vak = vak.child(regel("niets").text_color(rgb(th.mute)).child(taal::t("niets te doen", "nothing to do")));
        }
        let mut vorige_gevaarlijk = true;
        for (actie, rechts) in lijst {
            let gevaarlijk = matches!(actie, Actie::Verwijder | Actie::Opruimen);
            if gevaarlijk && !vorige_gevaarlijk {
                vak = vak.child(crate::venster::scheiding(th));
            }
            vorige_gevaarlijk = gevaarlijk;
            let (id, woord): (&'static str, &'static str) = match actie {
                Actie::Commit { .. } => ("commit", "commit"),
                Actie::Push { .. } => ("push", "push"),
                Actie::Update => ("update", "update"),
                Actie::Checkout => ("checkout", "checkout"),
                Actie::Verwijder => ("verwijder", taal::t("verwijder", "delete")),
                Actie::Opruimen => ("opruimen", taal::t("opruimen", "clean up")),
            };
            // geen checkout zolang Claude bezig is
            let kan = !(actie == Actie::Checkout && self.claude_bezig);
            let rechts = if kan { rechts } else { taal::t("Claude bezig", "Claude working").to_string() };
            let tak = naam.clone();
            vak = vak.child(
                regel(id)
                    .text_color(rgb(if !kan { th.mute } else if gevaarlijk { th.bad } else { th.ink }))
                    .child(div().flex_1().child(woord))
                    .child(div().text_color(rgb(th.mute)).child(rechts))
                    .when(kan, |d| {
                        d.hover(|d| d.bg(rgb(th.sel))).on_click(cx.listener(move |p, _, window, cx| match actie {
                            Actie::Commit { .. } => p.open_commit(tak.clone(), window, cx),
                            _ => p.doe(actie, tak.clone(), cx),
                        }))
                    }),
            );
        }
        Some(deferred(anchored().position(plek).snap_to_window_with_margin(px(8.0)).child(vak)).with_priority(1).into_any_element())
    }

    /// Bovenaan het paneel, zolang Jera iets doet of als het mislukte.
    fn melding_rij(&self, th: &Thema) -> Option<impl IntoElement> {
        let (tekst, kleur) = match (&self.bezig, &self.fout) {
            (Some(bezig), _) => (bezig.clone(), th.mute),
            (None, Some(fout)) => (fout.clone(), th.ink),
            (None, None) => return None,
        };
        Some(
            div()
                .flex()
                .flex_none()
                .items_center()
                .h(px(t::H_RIJ + 8.0))
                .px(px(t::PAD))
                .border_b(px(t::LIJN))
                .border_color(rgb(th.line))
                .text_color(rgb(kleur))
                .child(div().w_full().min_w_0().truncate().child(tekst)),
        )
    }

    /// Of het paneel open staat. Bij het openen haalt het meteen op.
    pub fn zet_zichtbaar(&mut self, open: bool) {
        if open && !self.zichtbaar {
            self.opgehaald = None;
        }
        self.zichtbaar = open;
    }

    /// Haalt op bij GitHub als het paneel open is en het langer dan `OPHALEN` geleden is.
    fn haal_op(&mut self, cx: &mut Context<Self>) {
        if !self.zichtbaar || self.haalt_op || self.remote != Some(true) {
            return;
        }
        if self.opgehaald.is_some_and(|t| t.elapsed() < OPHALEN) {
            return;
        }
        self.haalt_op = true;
        let root = self.root.clone();
        cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            cx.background_executor().spawn(async move { git::haal_op(&root) }).await;
            let _ = this.update(cx, |p, cx| {
                p.haalt_op = false;
                p.opgehaald = Some(Instant::now());
                p.gelezen = None;
                p.ververs(cx);
            });
        })
        .detach();
    }

    /// Leest de branches opnieuw als ze ouder zijn dan `VERVERS`.
    pub fn ververs(&mut self, cx: &mut Context<Self>) {
        self.haal_op(cx);
        if self.leest || self.gelezen.is_some_and(|t| t.elapsed() < VERVERS) {
            return;
        }
        self.leest = true;
        let root = self.root.clone();
        cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let (repo, remote, eigenaars) = cx
                .background_executor()
                .spawn(async move {
                    let remote = git::heeft_remote(&root);
                    let eigenaars = if remote == Some(false) { GitHub::eigenaars() } else { Vec::new() };
                    (git::lees(&root), remote, eigenaars)
                })
                .await;
            let _ = this.update(cx, |p, cx| {
                p.remote = remote;
                if !eigenaars.is_empty() && p.eigenaars.is_empty() {
                    p.eigenaars = eigenaars;
                }
                p.geen_git = repo.is_none();
                p.repo = repo.map(Arc::new);
                p.leest = false;
                p.gelezen = Some(Instant::now());
                cx.notify();
            });
        })
        .detach();
    }

    /// Maakt een privé repo op GitHub bij de gekozen eigenaar en zet de code erop.
    fn op_github(&mut self, cx: &mut Context<Self>) {
        let Some(eigenaar) = self.eigenaars.get(self.eigenaar).cloned() else { return };
        let root = self.root.clone();
        self.maak = Maak::Bezig;
        cx.notify();
        cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let gelukt = cx
                .background_executor()
                .spawn(async move {
                    let naam = root.file_name().unwrap_or_default().to_string_lossy().into_owned();
                    let commit = std::process::Command::new("git")
                        .arg("-C")
                        .arg(&root)
                        .args(["rev-parse", "--verify", "-q", "HEAD"])
                        .output()
                        .is_ok_and(|uit| uit.status.success());
                    let mut maak = gh();
                    maak.args(["repo", "create", &format!("{eigenaar}/{naam}"), "--private", "--remote", "origin", "--source"]);
                    maak.arg(&root);
                    if commit {
                        maak.arg("--push");
                    }
                    match maak.output() {
                        Ok(uit) if uit.status.success() => true,
                        Ok(uit) => {
                            eprintln!("op GitHub: {}", String::from_utf8_lossy(&uit.stderr));
                            false
                        }
                        Err(_) => false,
                    }
                })
                .await;
            let _ = this.update(cx, |p, cx| {
                p.maak = if gelukt { Maak::Niet } else { Maak::Mislukt };
                p.gelezen = None;
                p.ververs(cx);
                cx.notify();
            });
        })
        .detach();
    }

    /// Bovenaan het paneel, als de repo geen remote heeft: de eigenaar en Op GitHub.
    fn op_github_rij(&self, th: &Thema, cx: &mut Context<Self>) -> impl IntoElement {
        let eigenaar = self.eigenaars.get(self.eigenaar).cloned().unwrap_or_default();
        let knop = |label: &'static str| crate::venster::knop_vak(th).id("op-github").child(label);
        div()
            .flex()
            .flex_none()
            .items_center()
            .gap(px(t::GAP))
            .h(px(t::H_RIJ + 8.0))
            .px(px(t::PAD))
            .border_b(px(t::LIJN))
            .border_color(rgb(th.line))
            .child(
                div()
                    .id("eigenaar")
                    .flex()
                    .flex_1()
                    .min_w_0()
                    .items_center()
                    // Onder de muis een vlak zo hoog als de knop ernaast; het woord blijft op `pad` van de rand.
                    .h(px(t::H_KNOP))
                    .px(px(8.0))
                    .ml(px(-8.0))
                    .when(self.eigenaars.len() > 1 && self.maak != Maak::Bezig, |d| d.hover(|d| d.bg(rgb(th.sel))))
                    .child(div().flex_1().min_w_0().truncate().child(eigenaar))
                    .on_click(cx.listener(|p, _, _, cx| {
                        if !p.eigenaars.is_empty() && p.maak != Maak::Bezig {
                            p.eigenaar = (p.eigenaar + 1) % p.eigenaars.len();
                            cx.notify();
                        }
                    })),
            )
            .when(self.maak == Maak::Mislukt, |d| d.child(div().text_color(rgb(th.mute)).child(taal::t("mislukt", "failed"))))
            .child(match self.maak {
                Maak::Bezig => div().text_color(rgb(th.mute)).child(taal::t("maken", "creating")).into_any_element(),
                Maak::Niet | Maak::Mislukt if self.eigenaars.is_empty() => div().into_any_element(),
                Maak::Niet => knop(taal::t("Op GitHub", "To GitHub")).on_click(cx.listener(|p, _, _, cx| p.op_github(cx))).into_any_element(),
                Maak::Mislukt => knop(taal::t("Opnieuw", "Retry")).on_click(cx.listener(|p, _, _, cx| p.op_github(cx))).into_any_element(),
            })
    }

    /// Een knop als de muis erin staat, anders het dichtstbijzijnde punt binnen 30 px opzij en 20 px
    /// boven of onder. Punten op een lijn liggen dichter bij elkaar, dus langs een lijn zit geen gat.
    fn doel_op(&self, positie: Point<Pixels>) -> Option<usize> {
        let p = positie - self.oorsprong.get();
        let (x, y) = (f32::from(p.x), f32::from(p.y));
        let doelen = self.doelen.borrow();
        if let Some(i) = doelen.iter().rposition(|d| !d.punt && (x - d.x).abs() <= d.rx && (y - d.y).abs() <= d.ry) {
            return Some(i);
        }
        doelen
            .iter()
            .enumerate()
            .filter(|(_, d)| d.punt)
            .filter_map(|(i, d)| {
                let (dx, dy) = ((x - d.x).abs(), (y - d.y).abs());
                (dx <= 30.0 && dy <= 20.0).then_some((i, dx * dx + 2.0 * dy * dy))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(i, _)| i)
    }
}

impl Render for GitPaneel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let th = t::thema(window);
        let inhoud = match self.commit_vak(th, cx) {
            Some(vak) => vak,
            None => self.graaf(window, cx),
        };
        div()
            .flex()
            .flex_col()
            .h_full()
            .when(self.remote == Some(false), |d| d.min_w(px(270.0)).child(self.op_github_rij(th, cx)))
            .children(self.melding_rij(th))
            .child(inhoud)
            .children(self.menu_vak(th, cx))
    }
}

impl GitPaneel {
    /// De branches als graaf.
    fn graaf(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let th = t::thema(window);
        let Some(repo) = self.repo.clone() else {
            return div()
                .flex_none()
                .w(px(270.0))
                .flex_1()
                .min_h_0()
                .p(px(t::PAD))
                .text_color(rgb(th.mute))
                .when(self.geen_git, |d| d.child(taal::t("Geen git in deze map", "No git in this folder")))
                .into_any_element();
        };
        let staat = Staat { agents: self.agents, open: self.open.clone() };
        let breedte = breedte(&repo, staat.agents);
        let (oorsprong, doelen) = (self.oorsprong.clone(), self.doelen.clone());
        div()
            .id("git-paneel")
            .flex_none()
            .w(px(breedte))
            .flex_1()
            .min_h_0()
            .overflow_hidden()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, e: &MouseDownEvent, window, cx| {
                    if this.bezig.is_none() {
                        this.fout = None;
                    }
                    let Some(i) = this.doel_op(e.position) else {
                        cx.notify();
                        return;
                    };
                    let klik = this.doelen.borrow()[i].klik.clone();
                    match klik {
                        Klik::Groep(key) => {
                            if !this.open.remove(&key) {
                                this.open.insert(key);
                            }
                        }
                        Klik::Agents => this.agents = !this.agents,
                        Klik::Branch(naam) => this.menu = Some((naam, e.position)),
                        // een klik op het open bolletje opent meteen de commit view
                        Klik::Commit(naam) => this.open_commit(naam, window, cx),
                    }
                    cx.notify();
                }),
            )
            .child(
                canvas(
                    |_, _, _| (),
                    move |bounds, _, window, cx| {
                        oorsprong.set(bounds.origin);
                        let tek = teken(&repo, &staat, f32::from(bounds.size.width), f32::from(bounds.size.height), th);
                        verf(&tek, bounds, th, window, cx);
                        *doelen.borrow_mut() = tek.doelen;
                    },
                )
                .size_full(),
            )
            .into_any_element()
    }
}

struct Staat {
    agents: bool,
    open: HashSet<String>,
}

#[derive(Clone)]
enum Klik {
    Groep(String),
    Agents,
    /// Het menu van een branch.
    Branch(String),
    /// De commit view van een branch.
    Commit(String),
}

/// Een punt of knop waar een klik iets doet.
struct Doel {
    x: f32,
    y: f32,
    rx: f32,
    ry: f32,
    /// Een punt op een lijn. Een knop telt alleen als de muis erin staat.
    punt: bool,
    klik: Klik,
}

enum Vorm {
    Lijn { punten: Vec<(f32, f32)>, dik: f32, kleur: Hsla, streep: Option<[f32; 2]> },
    Bol { x: f32, y: f32, r: f32, kleur: Hsla },
    Ring { x: f32, y: f32, r: f32, kleur: Hsla, vul: Hsla },
    Hol { x: f32, y: f32, z: f32, rand: f32, kleur: Hsla, vul: Hsla },
    Wolk { x: f32, y: f32, kleur: Hsla, vul: Hsla },
    Knop { x: f32, y: f32, w: f32, h: f32, rand: Hsla, vul: Hsla },
    Tekst { x: f32, y: f32, tekst: String, grootte: f32, vet: bool, kleur: Hsla, rechts: bool },
}

#[derive(Default)]
struct Tekening {
    onder: Vec<Vorm>,
    boven: Vec<Vorm>,
    doelen: Vec<Doel>,
}

fn kleur(c: u32, a: f32) -> Hsla {
    let mut h: Hsla = rgb(c).into();
    h.a = a;
    h
}

fn agent(b: &Branch) -> bool {
    b.naam.starts_with("worktree-agent-")
}

fn kort(n: &str) -> String {
    if n.chars().count() > 24 { n.chars().take(23).chain(['…']).collect() } else { n.to_string() }
}

fn middernacht(ts: i64) -> i64 {
    let d = Local.timestamp_opt(ts, 0).single().map(|d| d.date_naive()).unwrap_or_default();
    Local.from_local_datetime(&d.and_hms_opt(0, 0, 0).unwrap_or_default()).earliest().map_or(ts, |d| d.timestamp())
}

fn datum(ts: i64) -> String {
    let d = Local.timestamp_opt(ts, 0).single().unwrap_or_default();
    format!("{} {}, {:02}:{:02}", d.day(), taal::maand(d.month0() as usize), d.hour(), d.minute())
}

fn dag_naam(ts: i64, nu: i64) -> String {
    let (v, d) = (middernacht(nu), middernacht(ts));
    if d == v {
        return taal::t("vandaag", "today").into();
    }
    if d == middernacht(v - 3600) {
        return taal::t("gisteren", "yesterday").into();
    }
    let t = Local.timestamp_opt(ts, 0).single().unwrap_or_default();
    format!("{} {} {}", taal::weekdag(t.weekday().num_days_from_monday() as usize), t.day(), taal::maand(t.month0() as usize))
}

fn geleden(ts: i64, nu: i64) -> String {
    let s = nu - ts;
    for (n, naam) in [(2592000, taal::t("mnd", "mo")), (604800, "wk"), (86400, "d"), (3600, taal::t("u", "h"))] {
        if s >= n {
            return format!("{} {naam}", s / n);
        }
    }
    taal::t("nu", "now").into()
}

/// De volgorde van de lijnen: de hoofdbranch, dan steeds wat erop aansluit.
fn volgorde(repo: &Repo, agents: bool) -> Vec<usize> {
    let zicht: Vec<usize> = (0..repo.branches.len()).filter(|&i| agents || !agent(&repo.branches[i])).collect();
    let is_basis = |i: usize| {
        let n = &repo.branches[i].naam;
        *n == repo.hoofd || zicht.iter().any(|&c| repo.branches[c].basis.as_ref() == Some(n))
    };
    fn lus(n: &str, repo: &Repo, zicht: &[usize], uit: &mut Vec<usize>, is_basis: &dyn Fn(usize) -> bool) {
        let Some(&i) = zicht.iter().find(|&&i| repo.branches[i].naam == n) else { return };
        if uit.contains(&i) {
            return;
        }
        uit.push(i);
        let mut kinderen: Vec<usize> = zicht.iter().copied().filter(|&c| repo.branches[c].basis.as_deref() == Some(n)).collect();
        kinderen.sort_by_key(|&c| (!is_basis(c), !repo.branches[c].vast));
        for c in kinderen {
            lus(&repo.branches[c].naam.clone(), repo, zicht, uit, is_basis);
        }
    }
    let mut uit = Vec::new();
    lus(&repo.hoofd, repo, &zicht, &mut uit, &is_basis);
    let rest: Vec<usize> = zicht.iter().copied().filter(|i| !uit.contains(i)).collect();
    uit.extend(rest);
    uit
}

fn x(i: usize) -> f32 {
    X0 + i as f32 * LW
}

/// Zo breed als de lijnen en de getrapte namen vragen.
fn breedte(repo: &Repo, agents: bool) -> f32 {
    volgorde(repo, agents)
        .iter()
        .enumerate()
        .map(|(i, &b)| x(i) + 7.0 + kort(&repo.branches[b].naam).chars().count() as f32 * 12.0 * TEKEN + 50.0)
        .fold(270.0, f32::max)
}

enum Item<'a> {
    Groep { run: Vec<(&'a Commit, bool)>, key: String, vaag: bool },
    Los { c: &'a Commit, vaag: bool, key: Option<String> },
}

struct Vak<'a> {
    vast: Vec<&'a Commit>,
    vakken: BTreeMap<usize, Vec<(&'a Commit, bool)>>,
}

struct Rij {
    h: String,
    ts: i64,
    dag: bool,
}

/// Waar alles in de graaf staat: de volgorde van de lijnen, de rijen op tijd met hun plek
/// vanaf boven, en per lijn de commits in de vakken tussen twee rijen.
struct Indeling<'a> {
    repo: &'a Repo,
    /// De branches in de volgorde van de lijnen.
    l: Vec<usize>,
    lane: HashMap<&'a str, usize>,
    rijen: Vec<Rij>,
    rij: HashMap<String, usize>,
    /// De plek van elke rij vanaf boven: de nieuwste rij staat bovenaan.
    ys: Vec<f32>,
    vakken: Vec<Vak<'a>>,
    kop_h: f32,
}

impl<'a> Indeling<'a> {
    fn new(repo: &'a Repo, staat: &Staat) -> Self {
        let br = &repo.branches;
        let l = volgorde(repo, staat.agents);
        let lane: HashMap<&str, usize> = l.iter().enumerate().map(|(i, &b)| (br[b].naam.as_str(), i)).collect();
        let in_l = |n: &str| lane.contains_key(n);
        let doorlopend = |b: &Branch| doorlopend(repo, &l, b);
        let kop_h = 16.0 + l.len() as f32 * RN;
        let top = kop_h + 26.0;

        // de rijen: de punten die ertoe doen, het moment van ophalen, en de middernacht van elke dag
        let commits: Vec<Vec<&Commit>> = l
            .iter()
            .map(|&b| if doorlopend(&br[b]) { br[b].eigen.iter().chain(&br[b].oud).collect() } else { br[b].eigen.iter().collect() })
            .collect();
        let mut rijen: Vec<Rij> = Vec::new();
        let mut gezien: HashSet<String> = HashSet::new();
        let mut pt = |h: &str, ts: i64, rijen: &mut Vec<Rij>| {
            if gezien.insert(h.to_string()) {
                rijen.push(Rij { h: h.to_string(), ts, dag: false });
            }
        };
        let mut dagen: Vec<i64> = vec![middernacht(repo.nu)];
        for (i, &bi) in l.iter().enumerate() {
            let b = &br[bi];
            pt(&b.h, b.ts, &mut rijen);
            if let Some(o) = b.origin.as_ref().filter(|o| o.behind > 0 || o.ahead > 0) {
                pt(&o.h, o.ts, &mut rijen);
            }
            if !doorlopend(b) {
                if let Some(mb) = &b.mb {
                    pt(&mb.h, mb.ts, &mut rijen);
                }
            }
            if let (Some(basis), true) = (&b.basis, doorlopend(b)) {
                if let Some(c) = commits[i].iter().find(|c| c.van.as_ref() == Some(basis)) {
                    pt(&c.h, c.ts, &mut rijen);
                }
            }
            dagen.extend(commits[i].iter().map(|c| middernacht(c.ts)));
            dagen.extend(b.op_origin.iter().map(|c| middernacht(c.ts)));
        }
        // de merge van een gemergde branch: daar loopt zijn lijn naartoe
        for &bi in &l {
            if let (Some(m), true) = (&br[bi].merged_into, br[bi].basis.as_ref().is_some_and(|n| lane.contains_key(n.as_str()))) {
                pt(&m.h, m.ts, &mut rijen);
            }
        }
        if let Some(f) = repo.opgehaald {
            pt("fetch", f, &mut rijen);
        }
        dagen.extend(rijen.iter().map(|r| middernacht(r.ts)));
        dagen.sort();
        dagen.dedup();
        for m in dagen {
            rijen.push(Rij { h: format!("dag:{m}"), ts: m, dag: true });
        }
        rijen.sort_by(|a, b| a.ts.cmp(&b.ts).then(b.dag.cmp(&a.dag)));
        let rij: HashMap<String, usize> = rijen.iter().enumerate().map(|(i, r)| (r.h.clone(), i)).collect();

        // commits per branch, verdeeld over de vakken tussen twee rijen, op tijd
        let vakken: Vec<Vak> = l
            .iter()
            .enumerate()
            .map(|(i, &bi)| {
                let b = &br[bi];
                let mut v = Vak { vast: Vec::new(), vakken: BTreeMap::new() };
                let tr = rij[b.h.as_str()] as isize;
                if let Some(o) = b.origin.as_ref().filter(|o| o.behind > 0) {
                    let op: Vec<&Commit> = b.op_origin.iter().collect();
                    leg(&op, tr, rij[o.h.as_str()] as isize - 1, true, &mut v, b, &rij, &rijen);
                }
                let lo = if doorlopend(b) { 0 } else { b.mb.as_ref().map_or(0, |mb| rij[mb.h.as_str()] as isize) };
                leg(&commits[i], lo, tr - 1, false, &mut v, b, &rij, &rijen);
                v
            })
            .collect();

        // de hoogte van elk vak, en de plek van elke rij vanaf boven
        let mut gaps = vec![RHMIN; rijen.len()];
        for (i, v) in vakken.iter().enumerate() {
            let b = &br[l[i]];
            for (&q, lijst) in &v.vakken {
                let it = groepeer(b, lijst, &staat.open, &in_l);
                let mut moet = straal_op_rij(&rijen, b, v, q) + straal_op_rij(&rijen, b, v, q + 1) + (it.len() + 1) as f32 * RUIMTE;
                moet += it.len() as f32 * 2.0 * PUNT;
                if q + 1 < gaps.len() && gaps[q + 1] < moet {
                    gaps[q + 1] = moet;
                }
            }
        }
        let mut ys = vec![0.0; rijen.len()];
        let mut acc = top;
        for q in (0..rijen.len()).rev() {
            ys[q] = acc;
            acc += gaps[q];
        }
        Indeling { repo, l, lane, rijen, rij, ys, vakken, kop_h }
    }

    fn y(&self, h: &str) -> f32 {
        self.ys[self.rij[h]]
    }

    fn in_l(&self, n: &str) -> bool {
        self.lane.contains_key(n)
    }

    fn doorlopend(&self, b: &Branch) -> bool {
        doorlopend(self.repo, &self.l, b)
    }
}

/// Een lijn die doorloopt tot onderaan: een basis, een vaste branch, een branch zonder basis,
/// of een met commits van voor zijn merge-base.
fn doorlopend(repo: &Repo, l: &[usize], b: &Branch) -> bool {
    let br = &repo.branches;
    let is_basis = b.naam == repo.hoofd || l.iter().any(|&c| br[c].basis.as_ref() == Some(&b.naam));
    is_basis
        || b.vast
        || b.basis.as_ref().is_none_or(|n| !br.iter().any(|c| c.naam == *n))
        || b.mb.as_ref().is_some_and(|mb| b.eigen.iter().any(|c| c.ts < mb.ts))
}

/// Een branch zonder eigen werk: zijn naam en eindpunt zijn vaag.
fn leeg(b: &Branch) -> bool {
    !b.vast && b.basis.is_some() && b.uniek == 0 && !(b.hier || b.gewijzigd)
}

/// Wat het menu van een branch toont, met wat er rechts bij staat.
fn acties(repo: &Repo, b: &Branch, remote: bool) -> Vec<(Actie, String)> {
    let mut v = Vec::new();
    if b.map.is_some() && b.wijzigingen > 0 {
        let n = b.wijzigingen;
        v.push((Actie::Commit { push: false }, format!("{n} {}", if n == 1 { taal::t("bestand", "file") } else { taal::t("bestanden", "files") })));
    }
    if remote && !b.alleen_origin {
        match &b.origin {
            None => v.push((Actie::Push { nieuw: true }, taal::t("nieuw", "new").to_string())),
            // push pas als er niets te halen is: anders weigert GitHub hem
            Some(o) if o.ahead > 0 && o.behind == 0 => v.push((Actie::Push { nieuw: false }, format!("↑{}", o.ahead))),
            _ => {}
        }
    }
    // beide kanten verder kan alleen met een rebase, en dus alleen waar de branch uitgecheckt staat
    if let Some(o) = b.origin.as_ref().filter(|o| o.behind > 0 && (o.ahead == 0 || b.map.is_some())) {
        v.push((Actie::Update, format!("↓{}", o.behind)));
    }
    if !b.hier && b.map.is_none() {
        v.push((Actie::Checkout, String::new()));
    }
    if gemerged(b) {
        v.push((Actie::Verwijder, String::new()));
    }
    if b.naam == repo.hoofd {
        let n = repo.branches.iter().filter(|b| gemerged(b)).count();
        if n > 0 {
            v.push((Actie::Opruimen, format!("{n} {}", taal::t("gemerged", "merged"))));
        }
    }
    v
}

/// Een branch die weg kan: gemerged, nergens uitgecheckt, niets geparkeerd en niets dat alleen hier staat.
fn gemerged(b: &Branch) -> bool {
    b.merged_into.is_some() && !b.vast && b.map.is_none() && b.geparkeerd == 0 && b.origin.as_ref().is_none_or(|o| o.ahead == 0)
}

/// De kleur van lijn `i`.
fn lk(th: &Thema, i: usize) -> u32 {
    th.lanes[i % 4]
}

/// Hoe groot wat er van branch `b` op rij `r` staat.
fn straal_op_rij(rijen: &[Rij], b: &Branch, v: &Vak, r: usize) -> f32 {
    let h = &rijen[r].h;
    if *h == b.h {
        return KOP + KOP_LIJN / 2.0;
    }
    if b.origin.as_ref().is_some_and(|o| o.h == *h) {
        return PUNT;
    }
    if v.vast.iter().any(|c| c.h == *h) { PUNT } else { 0.0 }
}

#[allow(clippy::too_many_arguments)]
fn tekst(x: f32, y: f32, tekst: String, grootte: f32, vet: bool, kleur: Hsla, rechts: bool) -> Vorm {
    Vorm::Tekst { x, y, tekst, grootte, vet, kleur, rechts }
}

fn lijn(punten: Vec<(f32, f32)>, dik: f32, kleur: Hsla, streep: Option<[f32; 2]>) -> Vorm {
    Vorm::Lijn { punten, dik, kleur, streep }
}

fn doel(x: f32, y: f32, r: f32, klik: Klik) -> Doel {
    Doel { x, y, rx: r, ry: r, punt: true, klik }
}

/// Een merge: haaks van de branch af en met een bocht de lijn in waar hij in ging, precies op de merge.
fn pijl(onder: &mut Vec<Vorm>, x_van: f32, x_naar: f32, yy: f32, k: u32) {
    let d = if x_van > x_naar { 1.0 } else { -1.0 };
    let mut punten = vec![(x_van, yy + BOCHT)];
    for s in 0..=12 {
        let t = s as f32 / 12.0 * std::f32::consts::FRAC_PI_2;
        punten.push((x_naar + d * BOCHT - d * BOCHT * t.sin(), yy + BOCHT * t.cos()));
    }
    onder.push(lijn(punten, LIJN, kleur(k, 1.0), None));
}

/// De straal van de bocht waarmee een merge de andere lijn in gaat.
const BOCHT: f32 = 10.0;

/// Het paneel als SVG, zonder venster: dezelfde vormen als `verf` tekent.
#[cfg(feature = "schermafdruk")]
pub fn svg(map: &std::path::Path, licht: bool, hh: f32) -> Option<String> {
    let th = if licht { &t::LICHT } else { &t::DONKER };
    let repo = git::lees(map)?;
    let staat = Staat { agents: false, open: HashSet::new() };
    let wd = breedte(&repo, false);
    let tek = teken(&repo, &staat, wd, hh, th);
    let k = |c: &Hsla| {
        let r = Rgba::from(*c);
        format!("rgba({},{},{},{:.3})", (r.r * 255.0).round(), (r.g * 255.0).round(), (r.b * 255.0).round(), r.a)
    };
    let mut s = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{wd}\" height=\"{hh}\" viewBox=\"0 0 {wd} {hh}\" font-family=\"JetBrains Mono, ui-monospace, Menlo, monospace\"><rect width=\"{wd}\" height=\"{hh}\" fill=\"#{:06x}\"/>",
        th.bg
    );
    for vorm in tek.onder.iter().chain(&tek.boven) {
        s += &match vorm {
            Vorm::Lijn { punten, dik, kleur, streep } => {
                let p: Vec<String> = punten.iter().map(|(x, y)| format!("{x:.2},{y:.2}")).collect();
                let st = streep.map_or(String::new(), |[a, b]| format!(" stroke-dasharray=\"{a} {b}\""));
                format!("<polyline points=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{dik}\"{st}/>", p.join(" "), k(kleur))
            }
            Vorm::Bol { x, y, r, kleur } => format!("<circle cx=\"{x}\" cy=\"{y}\" r=\"{r}\" fill=\"{}\"/>", k(kleur)),
            Vorm::Ring { x, y, r, kleur, vul } => {
                format!("<circle cx=\"{x}\" cy=\"{y}\" r=\"{r}\" fill=\"{}\" stroke=\"{}\" stroke-width=\"{LIJN}\"/>", k(vul), k(kleur))
            }
            Vorm::Hol { x, y, z: r, rand, kleur, vul } => {
                format!("<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"{}\" stroke=\"{}\" stroke-width=\"{rand}\"/>", x - r, y - r, 2.0 * r, 2.0 * r, k(vul), k(kleur))
            }
            Vorm::Wolk { x, y, kleur, vul } => {
                let s = 0.95;
                let q = |a: f32, b: f32| format!("{:.2} {:.2}", x + a * s, y + 1.5 + b * s);
                format!(
                    "<path d=\"M{} A{r1} {r1} 0 0 1 {} A{r2} {r2} 0 0 1 {} A{r3} {r3} 0 0 1 {} Z\" fill=\"{}\" stroke=\"{}\" stroke-width=\"{KOP_LIJN}\"/>",
                    q(-5.5, 4.5),
                    q(-6.1, -2.65),
                    q(2.7, -3.75),
                    q(5.0, 4.5),
                    k(vul),
                    k(kleur),
                    r1 = 3.6 * s,
                    r2 = 4.6 * s,
                    r3 = 3.9 * s
                )
            }
            Vorm::Knop { x, y, w, h, rand, vul } => {
                format!("<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"{}\" stroke=\"{}\" stroke-width=\"1\"/>", x + 0.5, y + 0.5, w - 1.0, h - 1.0, k(vul), k(rand))
            }
            Vorm::Tekst { x, y, tekst, grootte, vet, kleur, rechts } => {
                let t = tekst.replace('&', "&amp;").replace('<', "&lt;");
                format!(
                    "<text x=\"{x}\" y=\"{}\" font-size=\"{grootte}\" font-weight=\"{}\" fill=\"{}\" dominant-baseline=\"central\"{}>{t}</text>",
                    y + RN / 2.0,
                    if *vet { 600 } else { 400 },
                    k(kleur),
                    if *rechts { " text-anchor=\"end\"" } else { "" }
                )
            }
        };
    }
    s += "</svg>";
    Some(s)
}

/// De lijn zegt waar een stuk staat: vol = ook op GitHub, gestreept = alleen hier, vaag gestippeld = alleen op GitHub.
fn baan(onder: &mut Vec<Vorm>, punten: Vec<(f32, f32)>, k: u32, soort: &str) {
    match soort {
        "origin" => onder.push(lijn(punten, LIJN, kleur(k, VAAG), Some([1.5, 4.0]))),
        "lokaal" => onder.push(lijn(punten, LIJN, kleur(k, 1.0), Some([4.0, 3.0]))),
        _ => onder.push(lijn(punten, LIJN, kleur(k, 1.0), None)),
    }
}

/// Wat alleen op GitHub staat en nog niet hier.
const VAAG: f32 = 0.45;

/// Waar de naam van lijn `i` komt: de lijn loopt er dun naartoe tot `tot`.
struct Naam {
    i: usize,
    xi: f32,
    tot: f32,
    /// Bij een stompje onderaan: hoe oud de branch is.
    oud: Option<String>,
    leeg: bool,
}

fn teken(repo: &Repo, staat: &Staat, wd: f32, hh: f32, th: &Thema) -> Tekening {
    let ind = Indeling::new(repo, staat);
    let mut tek = Tekening::default();
    teken_tijd(&mut tek, &ind, wd, hh, th);
    let mut namen = Vec::new();
    for i in 0..ind.l.len() {
        namen.push(teken_branch(&mut tek, &ind, staat, i, hh, th));
    }
    teken_namen(&mut tek, &ind, namen, th);
    teken_agents_knop(&mut tek, repo, staat, wd, th);
    tek
}

/// De kop, de dagen rechts op hun lijn en het moment van ophalen.
fn teken_tijd(tek: &mut Tekening, ind: &Indeling, wd: f32, hh: f32, th: &Thema) {
    let repo = ind.repo;
    let (onder, boven) = (&mut tek.onder, &mut tek.boven);
    onder.push(lijn(vec![(0.0, ind.kop_h + 0.5), (wd, ind.kop_h + 0.5)], 1.0, kleur(th.line, 1.0), None));
    let mut vorige = ind.kop_h;
    for q in (0..ind.rijen.len()).rev() {
        if vorige >= hh {
            break;
        }
        if !ind.rijen[q].dag {
            continue;
        }
        boven.push(tekst(wd - 8.0, vorige + 4.0, dag_naam(ind.rijen[q].ts, repo.nu), t::KLEIN, false, kleur(th.mute, 1.0), true));
        onder.push(lijn(vec![(0.0, ind.ys[q] + 0.5), (wd, ind.ys[q] + 0.5)], 1.0, kleur(th.line, 1.0), None));
        vorige = ind.ys[q];
    }
    if let Some(f) = repo.opgehaald.filter(|_| ind.y("fetch") < hh) {
        let yf = ind.y("fetch");
        let oud = repo.nu - f > 86400;
        let wanneer = if middernacht(f) == middernacht(repo.nu) {
            let d = Local.timestamp_opt(f, 0).single().unwrap_or_default();
            format!("{:02}:{:02}", d.hour(), d.minute())
        } else {
            datum(f)
        };
        let k = kleur(if oud { th.warn } else { th.line_strong }, 1.0);
        onder.push(lijn(vec![(0.0, yf + 0.5), (wd, yf + 0.5)], 1.0, k, Some([3.0, 4.0])));
        boven.push(tekst(wd - 8.0, yf + 3.0, format!("{} {wanneer}", taal::t("opgehaald", "fetched")), t::KLEIN, oud, if oud { k } else { kleur(th.mute, 1.0) }, true));
    }
}

/// Lijn `i`: de lijn zelf, waar hij aftakt of in ging, zijn commits en zijn eindpunt.
fn teken_branch(tek: &mut Tekening, ind: &Indeling, staat: &Staat, i: usize, hh: f32, th: &Thema) -> Naam {
    let repo = ind.repo;
    let br = &repo.branches;
    let b = &br[ind.l[i]];
    let (onder, boven, doelen) = (&mut tek.onder, &mut tek.boven, &mut tek.doelen);
    let bg = kleur(th.bg, 1.0);
    let y = |h: &str| ind.y(h);
    let (xi, yt, k, v, door) = (x(i), y(&b.h), lk(th, i), &ind.vakken[i], ind.doorlopend(b));
    let hoogste = match &b.origin {
        Some(o) if o.behind > 0 => y(&o.h),
        _ => yt,
    };
    // valt de hele branch onder de onderrand, dan een stompje onderaan met zijn naam en hoe oud hij is
    if hoogste > hh - 6.0 {
        onder.push(lijn(vec![(xi, hh), (xi, hh - 18.0)], LIJN, kleur(k, 1.0), None));
        boven.push(Vorm::Hol { x: xi, y: hh - 18.0, z: KOP * 0.9, rand: KOP_LIJN, kleur: kleur(k, 1.0), vul: if b.hier { kleur(k, 1.0) } else { bg } });
        doelen.push(doel(xi, hh - 18.0, 9.0, Klik::Branch(b.naam.clone())));
        return Naam { i, xi, tot: hh - 18.0 - KOP, oud: Some(geleden(b.ts, repo.nu)), leeg: leeg(b) };
    }
    let soort = if b.alleen_origin {
        "origin"
    } else if b.origin.is_none() {
        "lokaal"
    } else {
        "vol"
    };
    let voet = if door { hh } else { b.mb.as_ref().map_or(hh, |mb| y(&mb.h).min(hh)) };
    let yp = match &b.origin {
        Some(o) if soort == "vol" && o.ahead > 0 && o.behind == 0 => y(&o.h),
        _ => yt,
    };
    // een gemergde branch loopt door tot zijn merge
    if let Some(m) = b.merged_into.as_ref().filter(|m| ind.rij.contains_key(m.h.as_str())) {
        let tot = y(&m.h) + BOCHT;
        if tot < yt {
            onder.push(lijn(vec![(xi, yt), (xi, tot)], LIJN, kleur(k, 1.0), None));
        }
    }
    if voet > yp {
        baan(onder, vec![(xi, voet), (xi, yp)], k, soort);
    }
    // staat GitHub lager dan het eindpunt, dan is het stuk erboven nog niet gepusht; het wolkje is GitHub
    if yp > yt {
        baan(onder, vec![(xi, yp), (xi, yt)], k, "lokaal");
        boven.push(Vorm::Wolk { x: xi, y: yp, kleur: kleur(k, 1.0), vul: bg });
    }
    if let Some(&j) = b.basis.as_deref().and_then(|n| ind.lane.get(n)) {
        let xj = x(j);
        let basis = &br[ind.l[j]];
        let dikke_merge = b.merged_into.as_ref().filter(|m| ind.rij.contains_key(m.h.as_str()) && basis.h == m.h);
        if let Some(m) = dikke_merge {
            pijl(onder, xi, xj, y(&m.h), k);
        }
        if !door {
            // een gewone branch takt hier af van zijn basis, met een rechte hoek
            if let Some(mb) = &b.mb {
                let ym = y(&mb.h);
                let d = if xi > xj { 1.0 } else { -1.0 };
                baan(onder, vec![(xj, ym), (xi + d * LIJN / 2.0, ym)], k, soort);
            }
        }
    }
    if let Some(o) = b.origin.as_ref().filter(|o| o.behind > 0) {
        let yo = y(&o.h);
        baan(onder, vec![(xi, yt), (xi, yo)], k, "origin");
        boven.push(Vorm::Wolk { x: xi, y: yo, kleur: kleur(k, 1.0), vul: bg });
    }
    // een commit die alleen op GitHub staat is vaag
    let achter = b.origin.as_ref().is_some_and(|o| o.behind > 0);
    let bolletje = |boven: &mut Vec<Vorm>, yy: f32, vaag: bool| {
        let a = if b.alleen_origin || vaag || (achter && yy < yt - 0.5) { VAAG } else { 1.0 };
        boven.push(Vorm::Bol { x: xi, y: yy, r: PUNT, kleur: kleur(k, a) });
    };

    // één bolletje per commit
    let punt = |onder: &mut Vec<Vorm>, boven: &mut Vec<Vorm>, doelen: &mut Vec<Doel>, c: &Commit, yy: f32, vaag: bool, key: Option<String>| {
        if yy > hh + 10.0 {
            return;
        }
        let bron = c.van.as_deref().filter(|v| *v != b.naam).and_then(|v| ind.lane.get(v)).copied().or_else(|| {
            ind.l.iter().position(|&o| repo.branches[o].merged_into.as_ref().is_some_and(|m| m.h == c.h) && repo.branches[o].naam != b.naam)
        });
        if let Some(bron) = bron {
            pijl(onder, x(bron), xi, yy, lk(th, bron));
        }
        bolletje(boven, yy, vaag);
        // een klik op een losse commit uit een uitgeklapte groep klapt hem weer in
        if let Some(key) = key {
            doelen.push(doel(xi, yy, 6.0, Klik::Groep(key)));
        }
    };
    for c in &v.vast {
        punt(onder, boven, doelen, c, y(&c.h), false, None);
    }
    for (&q, lijst) in &v.vakken {
        let (y_laag, y_hoog) = (ind.ys[q], ind.ys[q + 1]);
        let mut it = groepeer(b, lijst, &staat.open, &|n| ind.in_l(n));
        it.reverse();
        // gelijke afstand van rand tot rand, ook tot het bolletje op de rij eronder en erboven
        let onderkant = y_laag - straal_op_rij(&ind.rijen, b, v, q);
        let bovenkant = y_hoog + straal_op_rij(&ind.rijen, b, v, q + 1);
        let dik = it.len() as f32 * 2.0 * PUNT;
        let tussen = (onderkant - bovenkant - dik) / (it.len() + 1) as f32;
        let mut yy = onderkant;
        for o in it {
            let r = PUNT;
            yy -= tussen + r;
            match o {
                Item::Groep { run, key, vaag } if yy <= hh + 10.0 => {
                    bolletje(boven, yy, vaag);
                    let n = run.len();
                    boven.push(tekst(xi + PUNT + 6.0, yy - 9.0, n.to_string(), t::KLEIN, true, kleur(k, 1.0), false));
                    doelen.push(doel(xi, yy, 11.0, Klik::Groep(key)));
                }
                Item::Groep { .. } => {}
                Item::Los { c, vaag, key } => punt(onder, boven, doelen, c, yy, vaag, key),
            }
            yy -= r;
        }
    }
    // het eindpunt van de branch; vol als dit de map van de tab is
    let rk = KOP;
    let ka = if b.alleen_origin { VAAG } else { 1.0 };
    boven.push(Vorm::Hol { x: xi, y: yt, z: rk * 0.9, rand: KOP_LIJN, kleur: kleur(k, ka), vul: if b.hier { kleur(k, 1.0) } else { bg } });
    doelen.push(doel(xi, yt, 9.0, Klik::Branch(b.naam.clone())));
    // wat nog niet gecommit is: een open bolletje boven het eindpunt, met het aantal bestanden
    let open = b.wijzigingen + b.geparkeerd;
    if open > 0 {
        let yw = yt - RHMIN;
        onder.push(lijn(vec![(xi, yt - rk), (xi, yw)], LIJN, kleur(k, 1.0), Some([1.5, 3.0])));
        boven.push(Vorm::Ring { x: xi, y: yw, r: PUNT + 0.5, kleur: kleur(k, 1.0), vul: bg });
        boven.push(tekst(xi + PUNT + 6.0, yw - 9.0, open.to_string(), t::KLEIN, true, kleur(k, 1.0), false));
        // geparkeerd: eerst een checkout, dus het menu
        let klik = if b.wijzigingen > 0 { Klik::Commit(b.naam.clone()) } else { Klik::Branch(b.naam.clone()) };
        doelen.push(doel(xi, yw, 9.0, klik));
        return Naam { i, xi, tot: yw - PUNT - 2.0, oud: None, leeg: leeg(b) };
    }
    Naam { i, xi, tot: yt - rk - 1.0, oud: None, leeg: leeg(b) }
}

/// De namen, getrapt boven de lijnen: elke naam op een eigen regel, met de lijn er dun naartoe.
fn teken_namen(tek: &mut Tekening, ind: &Indeling, namen: Vec<Naam>, th: &Thema) {
    for Naam { i, xi, tot, oud, leeg } in namen {
        let rn = 8.0 + i as f32 * RN;
        let k = lk(th, i);
        if tot > rn + 4.0 {
            tek.onder.push(lijn(vec![(xi, rn + 4.0), (xi, tot)], LIJN, kleur(k, if leeg { 0.2 } else { 0.4 }), None));
        }
        let naam = kort(&ind.repo.branches[ind.l[i]].naam);
        let breed = naam.chars().count() as f32 * 12.0 * TEKEN;
        tek.boven.push(tekst(xi + 7.0, rn, naam, 12.0, true, kleur(k, if leeg { 0.45 } else { 1.0 }), false));
        let volledig = ind.repo.branches[ind.l[i]].naam.clone();
        tek.doelen.push(Doel { x: xi + 7.0 + breed / 2.0, y: rn + RN / 2.0, rx: breed / 2.0 + 4.0, ry: RN / 2.0, punt: false, klik: Klik::Branch(volledig) });
        if let Some(oud) = oud {
            tek.boven.push(tekst(xi + 7.0 + breed + 7.0, rn + 1.0, oud, t::KLEIN, false, kleur(th.mute, 1.0), false));
        }
    }
}

/// De knop voor de branches van agents.
fn teken_agents_knop(tek: &mut Tekening, repo: &Repo, staat: &Staat, wd: f32, th: &Thema) {
    let n_agent = repo.branches.iter().filter(|b| agent(b)).count();
    if n_agent == 0 {
        return;
    }
    let label = format!("{n_agent} agents {}", if staat.agents { "▾" } else { "▸" });
    let w = label.chars().count() as f32 * t::KLEIN * TEKEN + 16.0;
    let (cx_, cy) = (wd - 8.0 - w / 2.0, 6.0 + 10.0);
    tek.boven.push(Vorm::Knop { x: cx_ - w / 2.0, y: cy - 10.0, w, h: t::H_KNOP, rand: kleur(th.line_strong, 1.0), vul: kleur(th.bg, 1.0) });
    tek.boven.push(tekst(cx_ - w / 2.0 + 8.0, cy - 9.0, label, t::KLEIN, false, kleur(th.ink, 1.0), false));
    tek.doelen.push(Doel { x: cx_, y: cy, rx: w / 2.0, ry: 10.0, punt: false, klik: Klik::Agents });
}

#[allow(clippy::too_many_arguments)]
fn leg<'a>(lijst: &[&'a Commit], lo: isize, hi: isize, vaag: bool, v: &mut Vak<'a>, b: &Branch, rij: &HashMap<String, usize>, rijen: &[Rij]) {
    for &c in lijst {
        if rij.contains_key(c.h.as_str()) {
            if c.h != b.h && b.origin.as_ref().is_none_or(|o| c.h != o.h) {
                v.vast.push(c);
            }
            continue;
        }
        let idx = rijen.iter().rposition(|r| r.ts <= c.ts).unwrap_or(0) as isize;
        v.vakken.entry(idx.min(hi).max(lo) as usize).or_default().push((c, vaag));
    }
}

/// Drie of meer gewone commits op een rij worden één groep: een vierkantje met het aantal ernaast.
/// Een merge uit een andere lijn (met pijl) blijft los en breekt de rij. Een klik klapt een groep uit of in.
fn groepeer<'a>(b: &Branch, lijst: &[(&'a Commit, bool)], open: &HashSet<String>, in_l: &dyn Fn(&str) -> bool) -> Vec<Item<'a>> {
    let mut uit = Vec::new();
    let mut run: Vec<(&'a Commit, bool)> = Vec::new();
    let sluit = |run: &mut Vec<(&'a Commit, bool)>, uit: &mut Vec<Item<'a>>| {
        if run.is_empty() {
            return;
        }
        let key = format!("{}:{}", b.naam, run[0].0.h);
        let veel = run.len() >= 3;
        if veel && !open.contains(&key) {
            let vaag = run[0].1;
            uit.push(Item::Groep { run: std::mem::take(run), key, vaag });
        } else {
            for (c, vaag) in run.drain(..) {
                uit.push(Item::Los { c, vaag, key: veel.then(|| key.clone()) });
            }
        }
    };
    for &(c, vaag) in lijst {
        if c.van.as_ref().is_some_and(|v| *v != b.naam && in_l(v)) {
            sluit(&mut run, &mut uit);
            uit.push(Item::Los { c, vaag, key: None });
        } else {
            run.push((c, vaag));
        }
    }
    sluit(&mut run, &mut uit);
    uit
}

fn verf(tek: &Tekening, bounds: Bounds<Pixels>, th: &Thema, window: &mut Window, cx: &mut App) {
    let o = bounds.origin;
    let p = |x: f32, y: f32| point(o.x + px(x), o.y + px(y));
    window.paint_quad(fill(bounds, rgb(th.bg)));
    for vorm in tek.onder.iter().chain(&tek.boven) {
        match vorm {
            Vorm::Lijn { punten, dik, kleur, streep } => {
                let mut pad = PathBuilder::stroke(px(*dik));
                if let Some([a, b]) = streep {
                    pad = pad.dash_array(&[px(*a), px(*b)]);
                }
                pad.move_to(p(punten[0].0, punten[0].1));
                for &(x, y) in &punten[1..] {
                    pad.line_to(p(x, y));
                }
                if let Ok(pad) = pad.build() {
                    window.paint_path(pad, *kleur);
                }
            }
            Vorm::Bol { x, y, r, kleur } => {
                window.paint_quad(fill(Bounds::new(p(x - r, y - r), size(px(2.0 * r), px(2.0 * r))), *kleur).corner_radii(px(*r)));
            }
            Vorm::Ring { x, y, r, kleur, vul } => {
                let zz = r + LIJN / 2.0;
                window.paint_quad(quad(Bounds::new(p(x - zz, y - zz), size(px(2.0 * zz), px(2.0 * zz))), px(zz), *vul, px(LIJN), *kleur, BorderStyle::default()));
            }
            Vorm::Hol { x, y, z: r, rand, kleur, vul } => {
                let zz = r + rand / 2.0;
                if zz > 0.0 {
                    let b = Bounds::new(p(x - zz, y - zz), size(px(2.0 * zz), px(2.0 * zz)));
                    window.paint_quad(quad(b, px(0.0), *vul, px(*rand), *kleur, BorderStyle::default()));
                }
            }
            Vorm::Knop { x, y, w, h, rand, vul } => {
                window.paint_quad(quad(Bounds::new(p(*x, *y), size(px(*w), px(*h))), px(0.0), *vul, px(1.0), *rand, BorderStyle::default()));
            }
            Vorm::Wolk { x, y, kleur, vul } => {
                // de stand op GitHub: een wolkje op de lijn
                let s = 0.95;
                let q = |a: f32, b: f32| p(x + a * s, y + 1.5 + b * s);
                for (mut pad, k) in [(PathBuilder::fill(), *vul), (PathBuilder::stroke(px(KOP_LIJN)), *kleur)] {
                    pad.move_to(q(-5.5, 4.5));
                    pad.arc_to(point(px(3.6 * s), px(3.6 * s)), px(0.0), false, true, q(-6.1, -2.65));
                    pad.arc_to(point(px(4.6 * s), px(4.6 * s)), px(0.0), false, true, q(2.7, -3.75));
                    pad.arc_to(point(px(3.9 * s), px(3.9 * s)), px(0.0), false, true, q(5.0, 4.5));
                    pad.close();
                    if let Ok(pad) = pad.build() {
                        window.paint_path(pad, k);
                    }
                }
            }
            Vorm::Tekst { x, y, tekst, grootte, vet, kleur, rechts } => {
                schrijf(tekst, *grootte, *vet, *kleur, p(*x, *y), *rechts, window, cx);
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn schrijf(s: &str, grootte: f32, vet: bool, kleur: Hsla, plek: Point<Pixels>, rechts: bool, window: &mut Window, cx: &mut App) {
    let run = TextRun {
        len: s.len(),
        font: Font { weight: if vet { FontWeight::SEMIBOLD } else { FontWeight::NORMAL }, ..font(t::LETTER) },
        color: kleur,
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    let regel = window.text_system().shape_line(s.to_string().into(), px(grootte), &[run], None);
    let plek = if rechts { point(plek.x - regel.width, plek.y) } else { plek };
    let _ = regel.paint(plek, px(RN), TextAlign::Left, None, window, cx);
}
