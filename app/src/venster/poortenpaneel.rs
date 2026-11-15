//! Het paneel met poorten en agent-browsers, gegroepeerd onder de sessie die ze startte.
//! Daaronder wat geen sessie startte, en de wezen: wat bij een sessie hoort die niet meer draait.

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
        let browsers = self.poorten.browsers.iter().filter(|b| van(&b.sessie)).map(|b| format!("agent-browser  {}", b.naam));
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
            .child("agent-browser")
            .child(aantal(self.poorten.browsers.len()))
            .on_click(cx.listener(|this, _, _, cx| {
                this.poorten_open = !this.poorten_open;
                cx.notify();
            }))
    }

    pub(super) fn poortenpaneel(&self, th: &Thema, cx: &mut Context<Self>) -> impl IntoElement {
        let stand = &self.poorten;
        let standaard: Vec<&Poort> = stand.standaard().collect();

        // De sessies: die van Jera in de volgorde van de tabs, daarna die van elders.
        let volgorde: Vec<u32> = self.tabs.iter().flat_map(|tab| &tab.sessies).filter_map(|s| s.terminal.read(cx).pid).collect();
        let mut sessies: Vec<&String> = stand
            .poorten
            .iter()
            .filter_map(|p| p.sessie.as_ref())
            .chain(stand.browsers.iter().filter_map(|b| b.sessie.as_ref()))
            .chain(self.artifacts.keys())
            .filter(|id| stand.sessies.contains_key(*id))
            .collect();
        // Eerst alle dubbele weg: een sessie kan een poort, een browser en artifacts hebben.
        sessies.sort();
        sessies.dedup();
        sessies.sort_by_key(|id| {
            let (ouder, naam) = &stand.sessies[*id];
            (volgorde.iter().position(|p| p == ouder).unwrap_or(usize::MAX), naam.clone())
        });

        let mut rijen: Vec<AnyElement> = Vec::new();
        for id in sessies {
            let kop = match self.sessie_van(id, cx) {
                Some(sessie) => titel(sessie, cx),
                None => stand.sessies[id].1.clone(),
            };
            rijen.push(groepkop(kop, th).into_any_element());
            for poort in stand.poorten.iter().filter(|p| p.sessie.as_ref() == Some(id)) {
                rijen.push(self.poort_rij(poort, th, cx));
            }
            for browser in stand.browsers.iter().filter(|b| b.sessie.as_ref() == Some(id)) {
                rijen.extend(self.browser_rijen(browser, th, cx));
            }
            for artifact in self.artifacts.get(id).into_iter().flatten() {
                rijen.push(artifact_rij(artifact, th, cx));
            }
        }

        // Zonder sessie: standaard alleen wat uit de thuismap is gestart, met Alles de rest.
        let zonder: Vec<&Poort> = match self.alles {
            true => stand.poorten.iter().filter(|p| p.sessie.is_none()).collect(),
            false => standaard.iter().copied().filter(|p| p.sessie.is_none()).collect(),
        };
        let browsers_zonder: Vec<&Browser> = stand.browsers.iter().filter(|b| b.sessie.is_none()).collect();
        let verborgen = stand.poorten.len() - standaard.len();
        if !zonder.is_empty() || !browsers_zonder.is_empty() || verborgen > 0 {
            let alles = self.alles;
            rijen.push(
                groepkop(t("zonder sessie", "no session").into(), th)
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
                    .into_any_element(),
            );
            for poort in zonder {
                rijen.push(self.poort_rij(poort, th, cx));
            }
            for browser in browsers_zonder {
                rijen.extend(self.browser_rijen(browser, th, cx));
            }
        }

        // De wezen, met één knop die ze allemaal stopt.
        let wees_poorten: Vec<&Poort> = stand.poorten.iter().filter(|p| stand.wees(&p.sessie)).collect();
        let wees_browsers: Vec<&Browser> = stand.browsers.iter().filter(|b| stand.wees(&b.sessie)).collect();
        if !wees_poorten.is_empty() || !wees_browsers.is_empty() {
            let pids: Vec<u32> = wees_poorten.iter().map(|p| p.pid).collect();
            let namen: Vec<String> = wees_browsers.iter().map(|b| b.naam.clone()).collect();
            rijen.push(
                groepkop(t("wezen", "orphans").into(), th)
                    .child(
                        gevaar_knop("stop-alle", t("Stop alle", "Stop all"), th)
                            .mr(px(t::PAD))
                            .on_click(cx.listener(move |_, _, _, cx| {
                                let (pids, namen) = (pids.clone(), namen.clone());
                                cx.background_executor()
                                    .spawn(async move {
                                        pids.into_iter().for_each(poorten::stop);
                                        namen.iter().for_each(|naam| poorten::sluit(naam));
                                    })
                                    .detach();
                            })),
                    )
                    .into_any_element(),
            );
            for poort in wees_poorten {
                rijen.push(self.poort_rij(poort, th, cx));
            }
            for browser in wees_browsers {
                rijen.extend(self.browser_rijen(browser, th, cx));
            }
        }

        if rijen.is_empty() {
            return div().p(px(t::PAD)).text_color(rgb(th.mute)).child(t("Er luistert niets.", "Nothing is listening."));
        }
        div().pb(px(8.0)).children(rijen)
    }

    /// Eén poort: het nummer in vet, het commando, de map in `mute`, Openen en Stoppen.
    fn poort_rij(&self, poort: &Poort, th: &Thema, cx: &mut Context<Self>) -> AnyElement {
        let (nummer, pid) = (poort.nummer, poort.pid);
        let map = naam(&poort.map);
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

    /// Een agent-browser: de naam, Sluiten, en eronder het plaatje van wat hij nu toont.
    fn browser_rijen(&self, browser: &Browser, th: &Thema, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let naam = browser.naam.clone();
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
                    .child("agent-browser")
                    .child(div().min_w_0().truncate().text_color(rgb(th.mute)).child(naam.clone())),
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

/// Een artifact: de titel en Openen. De link staat onder de muis.
fn artifact_rij(artifact: &Artifact, th: &Thema, cx: &mut Context<Jera>) -> AnyElement {
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
