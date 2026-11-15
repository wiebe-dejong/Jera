//! Het paneel met drie kopjes: poorten, browsers en artifacts. Een kopje zonder regels staat er niet.
//! Elke regel noemt de sessie die het startte; "wees" als die sessie niet meer draait.

use super::*;
use crate::artifacts::{self, Artifact};
use crate::poorten::{self, Browser, Poort};

/// De breedte van de kolom rechts als het paneel open is.
pub(super) const B_PANEEL: f32 = 360.0;
const H_PLAATJE: f32 = 180.0;

pub(super) const POORT: &[u8] = include_bytes!("../../iconen/poort.svg");

impl Jera {
    /// De sessie in Jera met dit sessie-id.
    fn sessie_van(&self, id: &str, cx: &App) -> Option<&Sessie> {
        let (ouder, _) = self.poorten.sessies.get(id)?;
        self.tabs.iter().flat_map(|tab| &tab.sessies).find(|s| s.terminal.read(cx).pid == Some(*ouder))
    }

    /// Wat de sessie heeft draaien, één regel per poort of browser, voor onder de muis.
    pub(super) fn draait(&self, sessie: &Sessie, cx: &App) -> Vec<SharedString> {
        let Some(shell) = sessie.terminal.read(cx).pid else { return Vec::new() };
        let van = |sessie: &Option<String>| {
            sessie.as_ref().and_then(|id| self.poorten.sessies.get(id)).is_some_and(|(ouder, _)| *ouder == shell)
        };
        let poorten = self.poorten.poorten.iter().filter(|p| van(&p.sessie)).map(|p| format!("{}  {}", p.nummer, p.commando));
        let browsers = self.poorten.browsers.iter().filter(|b| van(&b.sessie)).map(|_| "browser".to_string());
        poorten.chain(browsers).map(Into::into).collect()
    }

    /// De knop rechts in de tabbalk, met de aantallen. Open heeft hij een streep in `accent` eronder.
    pub(super) fn poortenknop(&self, th: &Thema, cx: &mut Context<Self>) -> impl IntoElement {
        let aantal = |n: usize| div().text_color(rgb(th.mute)).mr(px(8.0)).child(n.to_string());
        div()
            .id("poortenknop")
            .flex()
            .flex_none()
            .items_center()
            .gap(px(8.0))
            .px(px(t::TAB_PAD))
            .pt(px(t::STREEP))
            .border_b(px(t::STREEP))
            .border_color(transparent_black())
            .text_color(rgb(th.mute))
            .when(self.poorten_open, |d| d.text_color(rgb(th.ink)).border_color(rgb(th.accent)))
            .when(!self.poorten_open, |d| d.hover(|d| d.text_color(rgb(th.ink))))
            .child(t("poorten", "ports"))
            .child(aantal(self.poorten.standaard().count()))
            .child("browsers")
            .child(aantal(self.poorten.browsers.len()))
            .on_click(cx.listener(|this, _, _, cx| {
                this.poorten_open = !this.poorten_open;
                cx.notify();
            }))
    }

    /// Bij wie iets hoort: de titel van de sessie, haar naam als ze niet in Jera draait, of "wees".
    fn van_wie(&self, sessie: &Option<String>, cx: &App) -> Option<String> {
        let id = sessie.as_ref()?;
        Some(match (self.sessie_van(id, cx), self.poorten.sessies.get(id)) {
            (Some(sessie), _) => titel(sessie, cx),
            (None, Some((_, naam))) => naam.clone(),
            (None, None) => t("wees", "orphan").into(),
        })
    }

    pub(super) fn poortenpaneel(&self, th: &Thema, cx: &mut Context<Self>) -> impl IntoElement {
        let stand = &self.poorten;
        let mut rijen: Vec<AnyElement> = Vec::new();

        // Poorten: standaard alleen die van een sessie of uit de thuismap, met Alles de rest.
        let standaard: Vec<&Poort> = stand.standaard().collect();
        let verborgen = stand.poorten.len() - standaard.len();
        let poorten: Vec<&Poort> = match self.alles {
            true => stand.poorten.iter().collect(),
            false => standaard,
        };
        // Ook zonder regels, als er iets verborgen luistert: dan staat Alles erin.
        if !poorten.is_empty() || verborgen > 0 {
            let alles = self.alles;
            let wezen: Vec<u32> = poorten.iter().filter(|p| stand.wees(&p.sessie)).map(|p| p.pid).collect();
            rijen.push(
                groepkop(t("Poorten", "Ports").into(), th)
                    .when(verborgen > 0, |d| {
                        d.child(
                            div()
                                .id("alles")
                                .flex()
                                .items_center()
                                .h_full()
                                .px(px(t::PAD))
                                .when(alles, |d| d.text_color(rgb(th.ink)))
                                .hover(|d| d.text_color(rgb(th.ink)))
                                .child(t("alles", "all"))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.alles = !this.alles;
                                    cx.notify();
                                })),
                        )
                    })
                    .when(!wezen.is_empty(), |d| {
                        d.child(gevaar_knop("stop-wezen", t("Stop wezen", "Stop orphans"), th).mr(px(t::PAD)).on_click(
                            cx.listener(move |_, _, _, cx| {
                                let wezen = wezen.clone();
                                cx.background_executor().spawn(async move { wezen.into_iter().for_each(poorten::stop) }).detach();
                            }),
                        ))
                    })
                    .into_any_element(),
            );
            for poort in poorten {
                rijen.push(self.poort_rij(poort, th, cx));
            }
        }

        if !stand.browsers.is_empty() {
            let wezen: Vec<String> = stand.browsers.iter().filter(|b| stand.wees(&b.sessie)).map(|b| b.naam.clone()).collect();
            rijen.push(
                groepkop(t("Browsers", "Browsers").into(), th)
                    .when(!wezen.is_empty(), |d| {
                        d.child(gevaar_knop("sluit-wezen", t("Sluit wezen", "Close orphans"), th).mr(px(t::PAD)).on_click(
                            cx.listener(move |_, _, _, cx| {
                                let wezen = wezen.clone();
                                cx.background_executor().spawn(async move { wezen.iter().for_each(|naam| poorten::sluit(naam)) }).detach();
                            }),
                        ))
                    })
                    .into_any_element(),
            );
            for browser in &stand.browsers {
                rijen.extend(self.browser_rijen(browser, th, cx));
            }
        }

        // Artifacts van de sessies die draaien, in de volgorde van de tabs.
        let volgorde: Vec<u32> = self.tabs.iter().flat_map(|tab| &tab.sessies).filter_map(|s| s.terminal.read(cx).pid).collect();
        let mut sessies: Vec<&String> = self.artifacts.keys().filter(|id| stand.sessies.contains_key(*id)).collect();
        sessies.sort_by_key(|id| {
            let (ouder, naam) = &stand.sessies[*id];
            (volgorde.iter().position(|p| p == ouder).unwrap_or(usize::MAX), naam.clone())
        });
        let artifacts: Vec<(&String, &Artifact)> =
            sessies.into_iter().flat_map(|id| self.artifacts[id].iter().map(move |a| (id, a))).collect();
        if !artifacts.is_empty() {
            rijen.push(groepkop(t("Artifacts", "Artifacts").into(), th).into_any_element());
            for (id, artifact) in artifacts {
                rijen.push(artifact_rij(artifact, self.van_wie(&Some(id.clone()), cx), th, cx));
            }
        }

        if rijen.is_empty() {
            return div().p(px(t::PAD)).text_color(rgb(th.mute)).child(t("Er luistert niets.", "Nothing is listening."));
        }
        div().pb(px(8.0)).children(rijen)
    }

    /// Eén poort: het nummer in vet, het commando, de sessie (of zonder sessie de map) in `mute`, Openen en Stoppen.
    fn poort_rij(&self, poort: &Poort, th: &Thema, cx: &mut Context<Self>) -> AnyElement {
        let (nummer, pid) = (poort.nummer, poort.pid);
        let map = self.van_wie(&poort.sessie, cx).unwrap_or_else(|| naam(&poort.map));
        let tip: Vec<SharedString> = vec![poort.commando.clone().into(), poort.map.display().to_string().into()];
        div()
            .id(("poort", nummer as usize))
            .flex()
            .items_center()
            .gap(px(t::GAP))
            .h(px(t::H_RIJ))
            .px(px(t::PAD))
            .child(div().flex_none().w(px(42.0)).font_weight(FontWeight::BOLD).child(nummer.to_string()))
            .child(
                div()
                    .id(("poorttekst", nummer as usize))
                    .flex()
                    .flex_1()
                    .min_w_0()
                    .gap(px(8.0))
                    .child(div().min_w_0().truncate().child(poort.commando.clone()))
                    .child(div().flex_none().text_color(rgb(th.mute)).child(map))
                    .tooltip(move |_, cx| cx.new(|_| Tip(tip.clone())).into()),
            )
            .child(
                knop_vak(th)
                    .id(("openen", nummer as usize))
                    .child(t("Openen", "Open"))
                    .on_click(cx.listener(move |_, _, _, _| poorten::open(nummer))),
            )
            .child(
                gevaar_knop(("stoppen", nummer as usize), t("Stoppen", "Stop"), th)
                    .on_click(cx.listener(move |_, _, _, _| poorten::stop(pid))),
            )
            .into_any_element()
    }

    /// Een agent-browser: de sessie die hem startte, Sluiten, en eronder het plaatje van wat hij nu toont.
    fn browser_rijen(&self, browser: &Browser, th: &Thema, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let naam = browser.naam.clone();
        let wie = self.van_wie(&browser.sessie, cx).unwrap_or_else(|| t("zonder sessie", "no session").into());
        let rij = div()
            .flex()
            .items_center()
            .gap(px(t::GAP))
            .h(px(t::H_RIJ))
            .px(px(t::PAD))
            .child(
                div()
                    .flex()
                    .flex_1()
                    .min_w_0()
                    .gap(px(8.0))
                    .child(div().min_w_0().truncate().child(wie)),
            )
            .child(
                gevaar_knop(SharedString::from(format!("sluit-{naam}")), t("Sluiten", "Close"), th).on_click(cx.listener(
                    move |_, _, _, cx| {
                        let naam = naam.clone();
                        cx.background_executor().spawn(async move { poorten::sluit(&naam) }).detach();
                    },
                )),
            )
            .into_any_element();
        // Het plaatje past in de breedte van het paneel en in `H_PLAATJE`, met de verhouding van de pagina.
        let plaatje = self.plaatjes.get(&browser.naam).map(|beeld| {
            let maat = beeld.size(0);
            let (b, h) = (maat.width.0.max(1) as f32, maat.height.0.max(1) as f32);
            let schaal = ((B_PANEEL - 2.0 * t::PAD - 2.0 * t::LIJN) / b).min(H_PLAATJE / h);
            div()
                .mx(px(t::PAD))
                .mt(px(2.0))
                .mb(px(6.0))
                .flex()
                .child(
                    div()
                        .border(px(t::LIJN))
                        .border_color(rgb(th.line))
                        .child(img(beeld.clone()).w(px(b * schaal)).h(px(h * schaal))),
                )
                .into_any_element()
        });
        std::iter::once(rij).chain(plaatje).collect()
    }
}

/// Een artifact: de titel, de sessie in `mute` en Openen. De link staat onder de muis.
fn artifact_rij(artifact: &Artifact, wie: Option<String>, th: &Thema, cx: &mut Context<Jera>) -> AnyElement {
    let url = artifact.url.clone();
    let tip: Vec<SharedString> = vec![url.clone().into()];
    div()
        .flex()
        .items_center()
        .gap(px(t::GAP))
        .h(px(t::H_RIJ))
        .px(px(t::PAD))
        .child(
            div()
                .id(SharedString::from(format!("artifact-{url}")))
                .flex_1()
                .min_w_0()
                .truncate()
                .child(artifact.titel.clone())
                .tooltip(move |_, cx| cx.new(|_| Tip(tip.clone())).into()),
        )
        .children(wie.map(|wie| div().flex_none().max_w(px(120.0)).truncate().text_color(rgb(th.mute)).child(wie)))
        .child(
            knop_vak(th)
                .id(SharedString::from(format!("artifact-open-{url}")))
                .child(t("Openen", "Open"))
                .on_click(cx.listener(move |_, _, _, _| artifacts::open(&url))),
        )
        .into_any_element()
}

/// De kop van een groep, in `mute`.
fn groepkop(kop: String, th: &Thema) -> Div {
    div()
        .flex()
        .items_center()
        .h(px(t::H_RIJ + 2.0))
        .mt(px(8.0))
        .pl(px(t::PAD))
        .text_color(rgb(th.mute))
        .child(div().flex_1().min_w_0().truncate().child(kop))
}

/// Een knop van de soort gevaar: tekst in `bad`, voor wat iets stopt of sluit.
fn gevaar_knop(id: impl Into<ElementId>, woord: &'static str, th: &Thema) -> Stateful<Div> {
    knop_vak(th).id(id).text_color(rgb(th.bad)).child(woord)
}
