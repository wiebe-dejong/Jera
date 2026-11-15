//! De commit view: zolang ik commit staat hij op de plek van de graaf. De bestanden die nog niet
//! gecommit zijn, elk met een vinkje, het bericht (Claude stelt het voor) en `commit` en `commit en push`.

use gpui_kit::base::input::{InputEvent, Textarea, TextareaState};

use super::*;
use crate::git::Wijziging;

/// Zo lang na het laatste vinkje schrijft Claude een nieuw voorstel.
const WACHT: Duration = Duration::from_millis(600);

pub(super) struct CommitView {
    pub(super) branch: String,
    pub(super) map: PathBuf,
    bestanden: Vec<Wijziging>,
    /// De paden zonder vinkje: die gaan niet mee.
    uit: HashSet<String>,
    pub(super) invoer: Entity<TextareaState>,
    /// Het voorstel van Claude dat in het veld staat. Staat er iets anders, dan heb ik getypt.
    voorstel: String,
    /// Telt op bij elk voorstel: een ouder voorstel dat later binnenkomt telt niet meer.
    ronde: u32,
    _volg: Subscription,
}

impl CommitView {
    /// De paden die meegaan; van een hernoemd bestand ook hoe het heette.
    pub(super) fn gekozen(&self) -> Vec<String> {
        self.bestanden.iter().filter(|w| !self.uit.contains(&w.pad)).flat_map(|w| [Some(w.pad.clone()), w.oud.clone()]).flatten().collect()
    }
}

impl GitPaneel {
    /// Opent de commit view van een branch. Claude schrijft een voorstel voor het bericht.
    pub(super) fn open_commit(&mut self, naam: String, window: &mut Window, cx: &mut Context<Self>) {
        let Some(map) = self.repo.as_ref().and_then(|r| r.branches.iter().find(|b| b.naam == naam)).and_then(|b| b.map.clone()) else { return };
        let invoer = cx.new(|cx| {
            // het veld groeit mee met het bericht; Enter commit, Shift+Enter is een nieuwe regel
            let mut veld = TextareaState::new(window, cx).auto_grow(1, 5).submit_on_enter(true);
            veld.set_placeholder("…", window, cx);
            veld
        });
        invoer.update(cx, |veld, cx| veld.focus(window, cx));
        let tak = naam.clone();
        let volg = cx.subscribe_in(&invoer, window, move |this, _, event, _, cx| {
            if let InputEvent::PressEnter { shift: false, .. } = event {
                this.doe(Actie::Commit { push: false }, tak.clone(), cx);
            }
        });
        self.menu = None;
        self.commit = Some(CommitView {
            branch: naam,
            map: map.clone(),
            bestanden: Vec::new(),
            uit: HashSet::new(),
            invoer: invoer.clone(),
            voorstel: String::new(),
            ronde: 0,
            _volg: volg,
        });
        cx.notify();
        cx.spawn_in(window, async move |this, cx| {
            let bestanden = cx.background_executor().spawn(async move { git::wijzigingen(&map) }).await;
            let _ = this.update_in(cx, |p, window, cx| {
                let Some(view) = p.commit.as_mut().filter(|v| v.invoer == invoer) else { return };
                view.bestanden = bestanden;
                p.stel_voor(Duration::ZERO, window, cx);
                cx.notify();
            });
        })
        .detach();
    }

    fn sluit_commit(&mut self, cx: &mut Context<Self>) {
        if self.bezig.is_none() {
            self.commit = None;
            cx.notify();
        }
    }

    /// Zet het vinkje van een bestand aan of uit. Het voorstel volgt de bestanden met een vinkje.
    fn wissel(&mut self, pad: &str, window: &mut Window, cx: &mut Context<Self>) {
        let Some(view) = self.commit.as_mut() else { return };
        if !view.uit.remove(pad) {
            view.uit.insert(pad.to_string());
        }
        self.stel_voor(WACHT, window, cx);
        cx.notify();
    }

    /// Laat Claude het bericht schrijven voor de bestanden met een vinkje. Wat ik zelf typte blijft staan.
    fn stel_voor(&mut self, wacht: Duration, window: &mut Window, cx: &mut Context<Self>) {
        let Some(view) = self.commit.as_mut() else { return };
        view.ronde += 1;
        let (ronde, invoer, map, paden) = (view.ronde, view.invoer.clone(), view.map.clone(), view.gekozen());
        if paden.is_empty() {
            return;
        }
        cx.spawn_in(window, async move |this, cx| {
            cx.background_executor().timer(wacht).await;
            let nog = this.read_with(cx, |p, _| p.commit.as_ref().is_some_and(|v| v.invoer == invoer && v.ronde == ronde));
            if !nog.unwrap_or(false) {
                return;
            }
            let bericht = cx.background_executor().spawn(async move { git::bericht(&map, &paden) }).await;
            let _ = this.update_in(cx, |p, window, cx| {
                let Some(view) = p.commit.as_mut().filter(|v| v.invoer == invoer && v.ronde == ronde) else { return };
                if invoer.read(cx).value() != view.voorstel {
                    return;
                }
                match bericht {
                    Some(bericht) => {
                        view.voorstel = bericht.clone();
                        invoer.update(cx, |veld, cx| veld.set_value(bericht, window, cx));
                    }
                    None => invoer.update(cx, |veld, cx| veld.set_placeholder(taal::t("bericht", "message"), window, cx)),
                }
            });
        })
        .detach();
    }

    /// De commit view, zo breed als de graaf.
    pub(super) fn commit_vak(&self, th: &Thema, cx: &mut Context<Self>) -> Option<AnyElement> {
        let view = self.commit.as_ref()?;
        let repo = self.repo.as_ref()?;
        let (aan, totaal) = (view.bestanden.iter().filter(|w| !view.uit.contains(&w.pad)).count(), view.bestanden.len());
        let woord = if totaal == 1 { taal::t("bestand", "file") } else { taal::t("bestanden", "files") };
        let teller = if aan == totaal { format!("{totaal} {woord}") } else { format!("{aan} {} {totaal} {woord}", taal::t("van", "of")) };
        // push pas als er niets te halen is: anders weigert GitHub hem
        let push = self.remote == Some(true)
            && repo.branches.iter().find(|b| b.naam == view.branch).is_some_and(|b| b.origin.as_ref().is_none_or(|o| o.behind == 0));
        let kan = aan > 0 && self.bezig.is_none();

        let kop = div()
            .flex()
            .flex_none()
            .items_center()
            .gap(px(t::GAP))
            .h(px(t::H_RIJ + 8.0))
            .px(px(t::PAD))
            .border_b(px(t::LIJN))
            .border_color(rgb(th.line))
            .child(div().flex_1().min_w_0().truncate().font_weight(FontWeight::BOLD).child(view.branch.clone()))
            .child(div().flex_none().text_color(rgb(th.mute)).child(teller))
            .child(
                div()
                    .id("commit-sluit")
                    .flex()
                    .flex_none()
                    .justify_center()
                    .w(px(t::ICOON))
                    .text_color(rgb(th.mute))
                    .hover(|d| d.text_color(rgb(th.ink)))
                    .child("×")
                    .on_click(cx.listener(|p, _, _, cx| p.sluit_commit(cx))),
            );

        let lijst = div().id("commit-lijst").flex_1().min_h_0().overflow_y_scroll().children(view.bestanden.iter().enumerate().map(|(i, w)| {
            let aan = !view.uit.contains(&w.pad);
            let (map, naam) = w.pad.rsplit_once('/').unwrap_or(("", &w.pad));
            let pad = w.pad.clone();
            div()
                .id(("bestand", i))
                .flex()
                .flex_none()
                .items_center()
                .gap(px(t::GAP))
                .h(px(t::H_RIJ))
                .px(px(t::PAD))
                .hover(|d| d.bg(rgb(th.sel)))
                .child(vinkje(aan, th))
                .child(div().flex_none().text_color(rgb(th.mute)).child(w.soort.to_string()))
                .child(div().min_w_0().truncate().when(!aan, |d| d.text_color(rgb(th.mute))).child(naam.to_string()))
                .child(div().flex_1().min_w_0().truncate().text_color(rgb(th.mute)).child(map.to_string()))
                .on_click(cx.listener(move |p, _, window, cx| p.wissel(&pad, window, cx)))
        }));

        let knop = |id: &'static str, woord: &'static str, push: bool| {
            let tak = view.branch.clone();
            let vak = if kan { crate::venster::knop_vak(th) } else { crate::venster::knop_uit(th) };
            vak.id(id).child(woord).when(kan, |d| d.on_click(cx.listener(move |p, _, _, cx| p.doe(Actie::Commit { push }, tak.clone(), cx))))
        };
        let onder = div()
            .flex()
            .flex_col()
            .flex_none()
            .gap(px(8.0))
            .p(px(t::PAD))
            .border_t(px(t::LIJN))
            .border_color(rgb(th.line))
            .child(div().px(px(4.0)).border(px(t::LIJN)).border_color(rgb(th.line_strong)).child(Textarea::new(&view.invoer)))
            .child(
                div()
                    .flex()
                    .justify_end()
                    .gap(px(t::GAP))
                    .child(knop("commit-ok", "commit", false))
                    .when(push, |d| d.child(knop("commit-push", taal::t("commit en push", "commit and push"), true))),
            );

        Some(
            div()
                .flex()
                .flex_col()
                .w(px(breedte(repo, self.agents)))
                .flex_1()
                .min_h_0()
                .capture_key_down(cx.listener(|p, e: &KeyDownEvent, _, cx| {
                    if e.keystroke.key == "escape" {
                        cx.stop_propagation();
                        p.sluit_commit(cx);
                    }
                }))
                .child(kop)
                .child(lijst)
                .child(onder)
                .into_any_element(),
        )
    }

    /// Voor de testhulp: opent de commit view van de branch van de map.
    #[cfg(feature = "schermafdruk")]
    pub fn test_commit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let naam = self.repo.as_ref().and_then(|r| r.branches.iter().find(|b| b.hier)).map(|b| b.naam.clone());
        if let Some(naam) = naam {
            self.open_commit(naam, window, cx);
        }
    }
}

/// Het vinkje van een bestand: een vakje met een rand, met het vinkje erin als het bestand meegaat.
fn vinkje(aan: bool, th: &Thema) -> Div {
    div()
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .size(px(t::ICOON))
        .border(px(t::LIJN))
        .border_color(rgb(th.line_strong))
        .when(aan, |d| d.child(gpui_kit::svg().data(crate::venster::VINK).size(px(10.0)).text_color(rgb(th.ink))))
}
