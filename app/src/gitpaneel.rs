//! Het git-paneel: de branches van de repo als graaf, rechts naast de terminal.
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

use crate::git::{self, Branch, Commit, Repo, Tov};
use crate::openen::{GitHub, gh};
use crate::taal;
use crate::thema::{self as t, Thema};

/// Zo oud mogen de gelezen branches zijn als het paneel open is.
const VERVERS: Duration = Duration::from_secs(20);

// de maten van de graaf
const LW: f32 = 43.0; // afstand tussen twee lijnen
const X0: f32 = 22.0; // de eerste lijn
const RHMIN: f32 = 26.0; // kleinste afstand tussen twee rijen
const RN: f32 = 18.0; // een regel in de kop met de namen
const RUIMTE: f32 = 4.0; // kleinste afstand tussen twee randen
const LIJN: f32 = 2.0;
const PUNT: f32 = 3.0;
const DUB: f32 = 2.0;
const MERGE: f32 = 3.4;
const GROEP: f32 = 4.8;
const KOP: f32 = 6.5;
const KOP_LIJN: f32 = 2.0;
const HIER: f32 = 8.0;
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
    hover: Option<usize>,
    oorsprong: Rc<Cell<Point<Pixels>>>,
    doelen: Rc<RefCell<Vec<Doel>>>,
    /// `Some(false)` als de repo geen remote heeft: dan staat bovenaan Op GitHub.
    remote: Option<bool>,
    /// Waar de repo op GitHub kan komen, en welke gekozen is.
    eigenaars: Vec<String>,
    eigenaar: usize,
    maak: Maak,
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
            hover: None,
            oorsprong: Rc::default(),
            doelen: Rc::default(),
            remote: None,
            eigenaars: Vec::new(),
            eigenaar: 0,
            maak: Maak::Niet,
        }
    }

    /// Leest de branches opnieuw als ze ouder zijn dan `VERVERS`.
    pub fn ververs(&mut self, cx: &mut Context<Self>) {
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

    fn doel_op(&self, positie: Point<Pixels>) -> Option<usize> {
        let p = positie - self.oorsprong.get();
        let (x, y) = (f32::from(p.x), f32::from(p.y));
        self.doelen.borrow().iter().rposition(|d| (x - d.x).abs() <= d.rx && (y - d.y).abs() <= d.ry)
    }
}

impl Render for GitPaneel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let th = t::thema(window);
        div()
            .flex()
            .flex_col()
            .h_full()
            .when(self.remote == Some(false), |d| d.min_w(px(270.0)).child(self.op_github_rij(th, cx)))
            .child(self.graaf(window, cx))
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
        let staat = Staat { agents: self.agents, open: self.open.clone(), hover: self.hover };
        let breedte = breedte(&repo, staat.agents);
        let (oorsprong, doelen) = (self.oorsprong.clone(), self.doelen.clone());
        div()
            .id("git-paneel")
            .flex_none()
            .w(px(breedte))
            .flex_1()
            .min_h_0()
            .overflow_hidden()
            .on_mouse_move(cx.listener(|this, e: &MouseMoveEvent, _, cx| {
                let hover = this.doel_op(e.position);
                if hover != this.hover {
                    this.hover = hover;
                    cx.notify();
                }
            }))
            .on_hover(cx.listener(|this, binnen: &bool, _, cx| {
                if !*binnen && this.hover.take().is_some() {
                    cx.notify();
                }
            }))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, e: &MouseDownEvent, _, cx| {
                    let klik = this.doel_op(e.position).and_then(|i| this.doelen.borrow()[i].klik.clone());
                    match klik {
                        Some(Klik::Groep(key)) => {
                            if !this.open.remove(&key) {
                                this.open.insert(key);
                            }
                        }
                        Some(Klik::Agents) => this.agents = !this.agents,
                        None => return,
                    }
                    this.hover = None;
                    cx.notify();
                }),
            )
            .child(
                canvas(
                    |_, _, _| (),
                    move |bounds, _, window, cx| {
                        oorsprong.set(bounds.origin);
                        let tek = teken(&repo, &staat, f32::from(bounds.size.width), f32::from(bounds.size.height), th);
                        verf(&tek, staat.hover, bounds, th, window, cx);
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
    hover: Option<usize>,
}

#[derive(Clone)]
enum Klik {
    Groep(String),
    Agents,
}

#[derive(Clone, Copy)]
enum Soort {
    Kop,
    Gewoon,
    Mute,
}

/// Een plek die iets laat zien als de muis erop staat, of iets doet bij een klik.
struct Doel {
    x: f32,
    y: f32,
    rx: f32,
    ry: f32,
    tip: Vec<(String, Soort)>,
    klik: Option<Klik>,
}

enum Vorm {
    Lijn { punten: Vec<(f32, f32)>, dik: f32, kleur: Hsla, streep: Option<[f32; 2]> },
    Vlak { x: f32, y: f32, z: f32, kleur: Hsla },
    Hol { x: f32, y: f32, z: f32, rand: f32, kleur: Hsla, vul: Hsla },
    Driehoek { punten: [(f32, f32); 3], kleur: Hsla },
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
                moet += it.iter().map(|o| 2.0 * straal_van(b, o)).sum::<f32>();
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

/// De kleur van lijn `i`.
fn lk(th: &Thema, i: usize) -> u32 {
    th.lanes[i % 4]
}

fn straal_van(b: &Branch, o: &Item) -> f32 {
    match o {
        Item::Groep { .. } => GROEP,
        Item::Los { c, .. } if c.n > 0 => MERGE + 0.9,
        Item::Los { c, .. } if b.dubbel.contains(&c.h) => DUB,
        Item::Los { .. } => PUNT,
    }
}

/// Hoe groot wat er van branch `b` op rij `r` staat.
fn straal_op_rij(rijen: &[Rij], b: &Branch, v: &Vak, r: usize) -> f32 {
    let h = &rijen[r].h;
    if *h == b.h {
        return if b.hier { HIER + 1.3 } else if b.alleen_origin { KOP } else { KOP + KOP_LIJN / 2.0 };
    }
    if b.origin.as_ref().is_some_and(|o| o.h == *h) {
        return 7.0;
    }
    v.vast.iter().find(|c| c.h == *h).map_or(0.0, |&c| straal_van(b, &Item::Los { c, vaag: false, key: None }))
}

#[allow(clippy::too_many_arguments)]
fn tekst(x: f32, y: f32, tekst: String, grootte: f32, vet: bool, kleur: Hsla, rechts: bool) -> Vorm {
    Vorm::Tekst { x, y, tekst, grootte, vet, kleur, rechts }
}

fn lijn(punten: Vec<(f32, f32)>, dik: f32, kleur: Hsla, streep: Option<[f32; 2]>) -> Vorm {
    Vorm::Lijn { punten, dik, kleur, streep }
}

fn doel(x: f32, y: f32, r: f32, tip: Vec<(String, Soort)>, klik: Option<Klik>) -> Doel {
    Doel { x, y, rx: r, ry: r, tip, klik }
}

fn titel(c: &Commit) -> String {
    match c.pr {
        Some(pr) => format!("#{pr} {}", c.titel),
        None => c.titel.clone(),
    }
}

/// Wat een eindpunt extra heeft en mist ten opzichte van de andere lijnen.
fn tov_tip(ind: &Indeling, naam: &str, lijst: &[Tov]) -> Vec<(String, Soort)> {
    let rijen: Vec<&Tov> = lijst.iter().filter(|o| ind.in_l(o.naam.trim_start_matches("origin/"))).collect();
    let w = rijen.iter().map(|o| o.naam.chars().count()).max().unwrap_or(0).min(26);
    let mist = taal::t("mist", "missing");
    let m = mist.chars().count();
    let mut uit = vec![(naam.to_string(), Soort::Kop), (format!("{:w$}  extra  {mist}", ""), Soort::Mute)];
    for o in rijen {
        let n = kort(&o.naam);
        let regel = if o.extra == 0 && o.mist == 0 {
            format!("{n:w$}  {}", taal::t("gelijk", "same"))
        } else {
            let s = |v: u32| if v == 0 { "–".to_string() } else { v.to_string() };
            format!("{n:w$}  {:>5}  {:>m$}", s(o.extra), s(o.mist))
        };
        uit.push((regel, Soort::Gewoon));
    }
    uit
}

/// Een pijl van een merge: van de lijn waar hij vandaan kwam naar de lijn waar hij in ging.
fn pijl(onder: &mut Vec<Vorm>, x_van: f32, x_naar: f32, yy: f32, k: u32, dik: bool) {
    let d = if x_van > x_naar { 1.0 } else { -1.0 };
    let (a, f) = if dik { (1.0, 1.0) } else { (0.65, 0.8) };
    let c = kleur(k, a);
    onder.push(lijn(vec![(x_van, yy), (x_naar + d * 14.0 * f, yy)], if dik { LIJN } else { LIJN * 0.72 }, c, None));
    let p = x_naar + d * 8.0 * f;
    onder.push(Vorm::Driehoek { punten: [(p, yy), (p + d * 7.0 * f, yy - 4.5 * f), (p + d * 7.0 * f, yy + 4.5 * f)], kleur: c });
    onder.push(Vorm::Vlak { x: x_van, y: yy, z: if dik { 2.8 } else { 2.3 } * 0.9, kleur: c });
}

/// De lijn zelf zegt waar een stuk staat: vol = lokaal en op origin, gestippeld = alleen op origin, hol = alleen lokaal.
fn baan(onder: &mut Vec<Vorm>, punten: Vec<(f32, f32)>, k: u32, soort: &str, th: &Thema) {
    match soort {
        "origin" => onder.push(lijn(punten, LIJN, kleur(k, 0.7), Some([2.0, 6.0]))),
        "lokaal" => {
            onder.push(lijn(punten.clone(), LIJN * 2.0 + 1.0, kleur(k, 1.0), None));
            onder.push(lijn(punten, LIJN * 0.8, kleur(th.bg, 1.0), None));
        }
        _ => onder.push(lijn(punten, LIJN, kleur(k, 1.0), None)),
    }
}

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
    let namen: Vec<Naam> = (0..ind.l.len()).map(|i| teken_branch(&mut tek, &ind, staat, i, hh, th)).collect();
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
    let tip = tov_tip(ind, &b.naam, &b.tov);
    // valt de hele branch onder de onderrand, dan een stompje onderaan met zijn naam en hoe oud hij is
    if hoogste > hh - 6.0 {
        onder.push(lijn(vec![(xi, hh), (xi, hh - 18.0)], LIJN, kleur(k, 1.0), Some([1.0, 5.0])));
        let rand = kleur(k, if leeg(b) { 0.4 } else { 1.0 });
        boven.push(Vorm::Hol { x: xi, y: hh - 18.0, z: (KOP - 2.0) * 0.9, rand: KOP_LIJN, kleur: rand, vul: bg });
        doelen.push(doel(xi, hh - 18.0, 11.0, tip, None));
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
    if voet > yp {
        baan(onder, vec![(xi, voet), (xi, yp)], k, soort, th);
    }
    if yp > yt {
        baan(onder, vec![(xi, yp), (xi, yt)], k, "lokaal", th);
        boven.push(Vorm::Wolk { x: xi, y: yp, kleur: kleur(k, 1.0), vul: bg });
        if let Some(ot) = &b.origin_tov {
            doelen.push(doel(xi, yp, 10.0, tov_tip(ind, &format!("origin/{}", b.naam), ot), None));
        }
    }
    if let Some(&j) = b.basis.as_deref().and_then(|n| ind.lane.get(n)) {
        let xj = x(j);
        let basis = &br[ind.l[j]];
        let dikke_merge = b.merged_into.as_ref().filter(|m| ind.rij.contains_key(m.h.as_str()) && basis.h == m.h);
        if let Some(m) = dikke_merge {
            pijl(onder, xi, xj, y(&m.h), k, true);
        } else if !door {
            // een gewone branch takt hier af van zijn basis, met een rechte hoek
            if let Some(mb) = &b.mb {
                let ym = y(&mb.h);
                let d = if xi > xj { 1.0 } else { -1.0 };
                baan(onder, vec![(xj, ym), (xi + d * LIJN / 2.0, ym)], k, soort, th);
                onder.push(Vorm::Vlak { x: xj, y: ym, z: PUNT * 0.9, kleur: kleur(lk(th, j), 1.0) });
            }
        }
    }
    if let Some(o) = b.origin.as_ref().filter(|o| o.behind > 0) {
        let yo = y(&o.h);
        onder.push(lijn(vec![(xi, yt), (xi, yo)], LIJN, kleur(k, 0.7), Some([2.0, 6.0])));
        boven.push(Vorm::Wolk { x: xi, y: yo, kleur: kleur(k, 1.0), vul: bg });
        if let Some(ot) = &b.origin_tov {
            doelen.push(doel(xi, yo, 10.0, tov_tip(ind, &format!("origin/{}", b.naam), ot), None));
        }
    }

    // één puntje per commit; een merge (PR) is hol; een wijziging die ook op de basis staat is klein
    let punt = |onder: &mut Vec<Vorm>, boven: &mut Vec<Vorm>, doelen: &mut Vec<Doel>, c: &Commit, yy: f32, vaag: bool, key: Option<String>| {
        if yy > hh + 10.0 {
            return;
        }
        let dub = b.dubbel.contains(&c.h);
        if let Some(&bron) = c.van.as_deref().filter(|v| *v != b.naam).and_then(|v| ind.lane.get(v)) {
            pijl(onder, x(bron), xi, yy, lk(th, bron), false);
        }
        let a = if vaag { 0.6 } else if dub { 0.55 } else { 1.0 };
        if c.n > 0 {
            boven.push(Vorm::Hol { x: xi, y: yy, z: MERGE * 0.9, rand: KOP_LIJN, kleur: kleur(k, a), vul: bg });
        } else {
            boven.push(Vorm::Vlak { x: xi, y: yy, z: if dub { DUB } else { PUNT } * 0.9, kleur: kleur(k, a) });
        }
        let tip = vec![(datum(c.ts), Soort::Mute), (titel(c), Soort::Gewoon), (c.auteur.clone(), Soort::Mute)];
        doelen.push(doel(xi, yy, 6.0, tip, key.map(Klik::Groep)));
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
        let dik: f32 = it.iter().map(|o| 2.0 * straal_van(b, o)).sum();
        let tussen = (onderkant - bovenkant - dik) / (it.len() + 1) as f32;
        let mut yy = onderkant;
        for o in it {
            let r = straal_van(b, &o);
            yy -= tussen + r;
            match o {
                Item::Groep { run, key, vaag } if yy <= hh + 10.0 => {
                    let licht = vaag || run.iter().all(|(c, _)| b.dubbel.contains(&c.h));
                    boven.push(Vorm::Vlak { x: xi, y: yy, z: GROEP * 0.9, kleur: kleur(k, if licht { 0.6 } else { 1.0 }) });
                    let n = run.len();
                    boven.push(tekst(xi + GROEP + 5.0, yy - 9.0, n.to_string(), t::KLEIN, true, kleur(k, if licht { 0.7 } else { 1.0 }), false));
                    let mut tip = vec![(format!("{n} commits"), Soort::Kop)];
                    tip.extend(run.iter().take(8).map(|(c, _)| (format!("{}  {}", datum(c.ts), titel(c)), Soort::Gewoon)));
                    if n > 8 {
                        tip.push((format!("+{} {}", n - 8, taal::t("meer", "more")), Soort::Mute));
                    }
                    doelen.push(doel(xi, yy, 11.0, tip, Some(Klik::Groep(key))));
                }
                Item::Groep { .. } => {}
                Item::Los { c, vaag, key } => punt(onder, boven, doelen, c, yy, vaag, key),
            }
            yy -= r;
        }
    }
    // het eindpunt van de branch; vol als dit de map van de tab is
    let rk = if b.hier { HIER } else if b.alleen_origin { KOP - 1.5 } else { KOP };
    let rand = kleur(k, if leeg(b) { 0.4 } else { 1.0 });
    boven.push(Vorm::Hol { x: xi, y: yt, z: rk * 0.9, rand: KOP_LIJN, kleur: rand, vul: if b.hier { kleur(k, 1.0) } else { bg } });
    if b.hier {
        boven.push(Vorm::Vlak { x: xi, y: yt, z: HIER * 0.4 * 0.9, kleur: bg });
    }
    doelen.push(doel(xi, yt, 13.0, tip, None));
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
    // Onder de muis krijgt de knop het vlak `sel`. Zijn doel komt hierna op deze plek in de lijst.
    let vul = if staat.hover == Some(tek.doelen.len()) { kleur(th.sel, 1.0) } else { kleur(th.bg, 1.0) };
    tek.boven.push(Vorm::Knop { x: cx_ - w / 2.0, y: cy - 10.0, w, h: t::H_KNOP, rand: kleur(th.line_strong, 1.0), vul });
    tek.boven.push(tekst(cx_ - w / 2.0 + 8.0, cy - 9.0, label, t::KLEIN, false, kleur(th.ink, 1.0), false));
    tek.doelen.push(Doel { x: cx_, y: cy, rx: w / 2.0, ry: 10.0, tip: Vec::new(), klik: Some(Klik::Agents) });
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

fn verf(tek: &Tekening, hover: Option<usize>, bounds: Bounds<Pixels>, th: &Thema, window: &mut Window, cx: &mut App) {
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
            Vorm::Vlak { x, y, z, kleur } => {
                window.paint_quad(fill(Bounds::new(p(x - z, y - z), size(px(2.0 * z), px(2.0 * z))), *kleur));
            }
            Vorm::Hol { x, y, z, rand, kleur, vul } => {
                let zz = z + rand / 2.0;
                if zz > 0.0 {
                    let b = Bounds::new(p(x - zz, y - zz), size(px(2.0 * zz), px(2.0 * zz)));
                    window.paint_quad(quad(b, px(0.0), *vul, px(*rand), *kleur, BorderStyle::default()));
                }
            }
            Vorm::Knop { x, y, w, h, rand, vul } => {
                window.paint_quad(quad(Bounds::new(p(*x, *y), size(px(*w), px(*h))), px(0.0), *vul, px(1.0), *rand, BorderStyle::default()));
            }
            Vorm::Driehoek { punten, kleur } => {
                let mut pad = PathBuilder::fill();
                pad.move_to(p(punten[0].0, punten[0].1));
                pad.line_to(p(punten[1].0, punten[1].1));
                pad.line_to(p(punten[2].0, punten[2].1));
                pad.close();
                if let Ok(pad) = pad.build() {
                    window.paint_path(pad, *kleur);
                }
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
    // wat er onder de muis staat
    let Some(d) = hover.and_then(|i| tek.doelen.get(i)).filter(|d| !d.tip.is_empty()) else { return };
    let (wd, hh) = (f32::from(bounds.size.width), f32::from(bounds.size.height));
    let max_w = 340f32.min(wd - 16.0);
    let max_tekens = ((max_w - 20.0) / (12.0 * TEKEN)) as usize;
    let regels: Vec<(String, Soort)> = d
        .tip
        .iter()
        .map(|(s, soort)| {
            let s = if s.chars().count() > max_tekens { s.chars().take(max_tekens - 1).chain(['…']).collect() } else { s.clone() };
            (s, *soort)
        })
        .collect();
    let breed = regels.iter().map(|(s, _)| s.chars().count()).max().unwrap_or(0) as f32 * 12.0 * TEKEN + 20.0;
    let hoog = regels.len() as f32 * 17.0 + 14.0;
    let left = (d.x + 16.0).min(wd - breed - 8.0).max(8.0);
    let top = if d.y + 14.0 + hoog > hh { d.y - 14.0 - hoog } else { d.y + 14.0 };
    let b = Bounds::new(p(left, top), size(px(breed), px(hoog)));
    window.paint_quad(quad(b, px(0.0), rgb(th.bg), px(1.0), rgb(th.line_strong), BorderStyle::default()));
    for (n, (s, soort)) in regels.iter().enumerate() {
        let (k, vet) = match soort {
            Soort::Kop => (th.ink, true),
            Soort::Gewoon => (th.ink, false),
            Soort::Mute => (th.mute, false),
        };
        schrijf(s, 12.0, vet, kleur(k, 1.0), p(left + 10.0, top + 7.0 + n as f32 * 17.0), false, window, cx);
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
