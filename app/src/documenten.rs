//! Het venster Documenten: markdown, tabellen (csv en xlsx), json, afbeeldingen en pdf, elk in een
//! tab. Er is hoogstens één zo'n venster. Verandert een bestand op schijf, dan leest het venster
//! het opnieuw. Markdown en json kan ik ook bewerken.

mod bewerk;
mod json;
mod markdown;
mod pdf;
mod tabel;

use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, SystemTime},
};

use gpui_kit::{prelude::FluentBuilder, *};

use crate::bestandsicoon;
use crate::taal::t;
use crate::thema::{self as t, Thema};
use crate::venster::{lijn, naam, tab_vak};
use bewerk::{Bewerk, Vraag};

actions!(documenten, [SluitTab, VorigeTab, VolgendeTab, Bewaar]);

/// Zo vaak kijkt het venster of een open bestand op schijf veranderde.
const KIJK_ELKE: Duration = Duration::from_millis(500);

/// Wat het venster kan tonen.
#[derive(Clone, Copy, PartialEq)]
enum Soort {
    Markdown,
    Tabel,
    Json,
    Beeld,
    Pdf,
}

fn soort(pad: &Path) -> Option<Soort> {
    let ext = pad.extension()?.to_str()?.to_ascii_lowercase();
    Some(match ext.as_str() {
        "md" | "markdown" => Soort::Markdown,
        "csv" | "tsv" | "xlsx" | "xlsm" | "xls" | "ods" => Soort::Tabel,
        "json" => Soort::Json,
        "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg" => Soort::Beeld,
        "pdf" => Soort::Pdf,
        _ => return None,
    })
}

/// Of Jera dit bestand zelf toont.
pub fn kan(pad: &Path) -> bool {
    soort(pad).is_some() && !pad.is_dir()
}

/// Of Jera dit bestand ook bewerkt; dan hoeft het niet naar een andere app.
pub fn bewerkt(pad: &Path) -> bool {
    kan(pad) && soort(pad).is_some_and(bewerkbaar)
}

/// Het open venster Documenten.
struct Venster(AnyWindowHandle, WeakEntity<Documenten>);

impl Global for Venster {}

pub fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("cmd-w", SluitTab, Some("Documenten")),
        KeyBinding::new("cmd-shift-[", VorigeTab, Some("Documenten")),
        KeyBinding::new("cmd-shift-]", VolgendeTab, Some("Documenten")),
        KeyBinding::new("cmd-s", Bewaar, Some("Documenten")),
    ]);
}

/// Heeft een tab in Documenten wijzigingen die niet bewaard zijn, dan toont het venster die tab
/// met de vraag of ik wil bewaren, en is het antwoord waar: nog niet stoppen.
pub fn houd_tegen(cx: &mut App) -> bool {
    let Some(Venster(venster, docs)) = cx.try_global::<Venster>() else { return false };
    let (venster, docs) = (*venster, docs.clone());
    venster.update(cx, |_, window, cx| docs.update(cx, |docs, cx| docs.houd_tegen(window, cx)).unwrap_or(false)).unwrap_or(false)
}

/// Opent `pad` in het venster Documenten, in de tab waarin het al staat of in een nieuwe.
/// Onwaar als Jera dit bestand niet zelf toont.
pub fn open(pad: &Path, cx: &mut App) -> bool {
    if !kan(pad) {
        return false;
    }
    let pad = pad.to_path_buf();
    if let Some(Venster(venster, docs)) = cx.try_global::<Venster>() {
        let (venster, docs) = (*venster, docs.clone());
        let p = pad.clone();
        let gelukt = venster.update(cx, |_, window, cx| {
            docs.update(cx, |docs, cx| docs.toon(p, window, cx)).is_ok() && {
                window.activate_window();
                true
            }
        });
        if matches!(gelukt, Ok(true)) {
            cx.activate(true);
            return true;
        }
    }
    let bounds = Bounds::centered(None, size(px(960.0), px(1000.0)), cx);
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: Some(TitlebarOptions {
            title: Some(t("Documenten", "Documents").into()),
            appears_transparent: true,
            traffic_light_position: Some(point(px(14.0), px(11.0))),
        }),
        ..Default::default()
    };
    let geopend = gpui_kit::open_window(options, cx, |window, cx| {
        cx.new(|cx| {
            let mut docs = Documenten::new(window, cx);
            docs.toon(pad, window, cx);
            docs
        })
    });
    match geopend {
        Ok((venster, docs)) => {
            cx.set_global(Venster(venster, docs.downgrade()));
            cx.activate(true);
            true
        }
        Err(_) => false,
    }
}

pub struct Documenten {
    tabs: Vec<Doc>,
    actief: usize,
    volgende_id: usize,
    focus: FocusHandle,
    vraag: Option<Vraag>,
}

struct Doc {
    id: usize,
    pad: PathBuf,
    soort: Soort,
    /// Wanneer het bestand veranderde, bij het laatste lezen.
    gewijzigd: Option<SystemTime>,
    inhoud: Inhoud,
    /// De tekst op schijf, van markdown en json.
    tekst: SharedString,
    /// Bestaat zodra ik markdown of json voor het eerst bewerk.
    bewerk: Option<Bewerk>,
}

enum Inhoud {
    Laden,
    Fout(SharedString),
    Markdown(SharedString),
    Beeld(Arc<Image>),
    Tabel(Entity<tabel::Tabel>),
    Json(Entity<json::Json>),
    Pdf(Entity<pdf::Pdf>),
}

/// Wat een achtergrondtaak uit een bestand las.
enum Gelezen {
    Fout(String),
    Markdown(String),
    Beeld(Arc<Image>),
    Tabel(Vec<tabel::Blad>),
    Json(serde_json::Value),
    Pdf(Vec<(f32, f32)>),
}

/// Wat in het bestand staat, en van markdown en json ook de tekst.
fn lees(pad: &Path, soort: Soort) -> (Gelezen, Option<SharedString>) {
    let fout = |f: String| Gelezen::Fout(f);
    let gelezen = match soort {
        Soort::Markdown | Soort::Json => match std::fs::read(pad) {
            Ok(bytes) => {
                let tekst = String::from_utf8_lossy(&bytes).into_owned();
                return (uit_tekst(soort, &tekst), Some(tekst.into()));
            }
            Err(f) => fout(f.to_string()),
        },
        Soort::Beeld => {
            let ext = pad.extension().and_then(|e| e.to_str()).unwrap_or_default().to_ascii_lowercase();
            let formaat = match ext.as_str() {
                "jpg" | "jpeg" => ImageFormat::Jpeg,
                "gif" => ImageFormat::Gif,
                "webp" => ImageFormat::Webp,
                "svg" => ImageFormat::Svg,
                _ => ImageFormat::Png,
            };
            match std::fs::read(pad) {
                Ok(bytes) => Gelezen::Beeld(Arc::new(Image::from_bytes(formaat, bytes))),
                Err(f) => fout(f.to_string()),
            }
        }
        Soort::Tabel => tabel::lees(pad).map_or_else(fout, Gelezen::Tabel),
        Soort::Pdf => pdf::lees(pad).map_or_else(fout, Gelezen::Pdf),
    };
    (gelezen, None)
}

/// Markdown of json uit zijn tekst.
fn uit_tekst(soort: Soort, tekst: &str) -> Gelezen {
    match soort {
        Soort::Json => match serde_json::from_str(tekst) {
            Ok(waarde) => Gelezen::Json(waarde),
            Err(f) => Gelezen::Fout(json_fout(&f)),
        },
        _ => Gelezen::Markdown(tekst.to_string()),
    }
}

fn json_fout(f: &serde_json::Error) -> String {
    format!("{} {}, {} {}.", t("Geen geldige json: regel", "Not valid json: line"), f.line(), t("teken", "column"), f.column())
}

impl Documenten {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus = cx.focus_handle();
        window.focus(&focus, cx);
        cx.observe_window_appearance(window, |_, _, cx| cx.notify()).detach();
        cx.spawn_in(window, async move |this, cx| {
            loop {
                cx.background_executor().timer(KIJK_ELKE).await;
                if this.update_in(cx, |docs, window, cx| docs.kijk(window, cx)).is_err() {
                    break;
                }
            }
        })
        .detach();
        let docs = cx.weak_entity();
        window.on_window_should_close(cx, move |window, cx| {
            !docs.update(cx, |docs, cx| docs.houd_tegen(window, cx)).unwrap_or(false)
        });
        #[cfg(feature = "schermafdruk")]
        schermafdruk(window, cx);
        Self { tabs: Vec::new(), actief: 0, volgende_id: 0, focus, vraag: None }
    }

    /// Toont `pad`: de tab waarin het al staat, of een nieuwe achteraan.
    fn toon(&mut self, pad: PathBuf, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.focus, cx);
        if let Some(index) = self.tabs.iter().position(|doc| doc.pad == pad) {
            self.actief = index;
            cx.notify();
            return;
        }
        let Some(soort) = soort(&pad) else { return };
        let id = self.volgende_id;
        self.volgende_id += 1;
        let gewijzigd = gewijzigd(&pad);
        self.tabs.push(Doc { id, pad, soort, gewijzigd, inhoud: Inhoud::Laden, tekst: SharedString::default(), bewerk: None });
        self.actief = self.tabs.len() - 1;
        self.lees(id, window, cx);
        cx.notify();
    }

    /// Leest elk bestand opnieuw dat sinds de vorige keer veranderde.
    fn kijk(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let mut opnieuw = Vec::new();
        for doc in &mut self.tabs {
            let nu = gewijzigd(&doc.pad);
            if nu != doc.gewijzigd {
                doc.gewijzigd = nu;
                opnieuw.push(doc.id);
            }
        }
        for id in opnieuw {
            self.lees(id, window, cx);
        }
    }

    fn lees(&mut self, id: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(doc) = self.tabs.iter().find(|doc| doc.id == id) else { return };
        let (pad, soort) = (doc.pad.clone(), doc.soort);
        cx.spawn_in(window, async move |this, cx| {
            let (gelezen, tekst) = cx.background_executor().spawn(async move { lees(&pad, soort) }).await;
            let _ = this.update_in(cx, |docs, window, cx| docs.zet(id, gelezen, tekst, window, cx));
        })
        .detach();
    }

    /// Zet wat gelezen is in de tab. Heb ik wijzigingen die niet bewaard zijn, dan blijft de tab
    /// zoals hij is; anders krijgt ook de editor de nieuwe tekst.
    fn zet(&mut self, id: usize, gelezen: Gelezen, tekst: Option<SharedString>, window: &mut Window, cx: &mut Context<Self>) {
        let Some(doc) = self.tabs.iter_mut().find(|doc| doc.id == id) else { return };
        if let Some(tekst) = tekst {
            let mijn = doc.gewijzigd(cx);
            if let Some(bewerk) = &mut doc.bewerk {
                if mijn {
                    bewerk.buiten |= tekst != doc.tekst;
                    return;
                }
                bewerk.neem_over(tekst.clone(), window, cx);
            }
            doc.tekst = tekst;
        }
        self.vul(id, gelezen, cx);
    }

    /// Toont wat gelezen is. Een tabel, json of pdf houdt waar ik was.
    fn vul(&mut self, id: usize, gelezen: Gelezen, cx: &mut Context<Self>) {
        let Some(doc) = self.tabs.iter_mut().find(|doc| doc.id == id) else { return };
        let pad = doc.pad.clone();
        doc.inhoud = match (&doc.inhoud, gelezen) {
            (Inhoud::Tabel(tabel), Gelezen::Tabel(bladen)) => {
                tabel.update(cx, |tabel, cx| tabel.vervang(bladen, cx));
                return;
            }
            (Inhoud::Json(json), Gelezen::Json(waarde)) => {
                json.update(cx, |json, cx| json.vervang(waarde, cx));
                return;
            }
            (Inhoud::Pdf(pdf), Gelezen::Pdf(maten)) => {
                pdf.update(cx, |pdf, cx| pdf.vervang(maten, cx));
                return;
            }
            (_, Gelezen::Fout(fout)) => Inhoud::Fout(fout.into()),
            (_, Gelezen::Markdown(tekst)) => Inhoud::Markdown(tekst.into()),
            (_, Gelezen::Beeld(beeld)) => Inhoud::Beeld(beeld),
            (_, Gelezen::Tabel(bladen)) => Inhoud::Tabel(cx.new(|_| tabel::Tabel::new(bladen))),
            (_, Gelezen::Json(waarde)) => Inhoud::Json(cx.new(|_| json::Json::new(waarde))),
            (_, Gelezen::Pdf(maten)) => Inhoud::Pdf(cx.new(|_| pdf::Pdf::new(pad, maten))),
        };
        cx.notify();
    }

    /// Sluit de tab, of vraagt eerst of ik wil bewaren.
    fn sluit(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(doc) = self.tabs.get(index) else { return };
        if doc.gewijzigd(cx) {
            self.actief = index;
            self.vraag = Some(Vraag::Sluiten(doc.id));
            cx.notify();
            return;
        }
        self.haal_weg(index, window, cx);
    }

    fn haal_weg(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if index >= self.tabs.len() {
            return;
        }
        let doc = self.tabs.remove(index);
        if let Inhoud::Pdf(pdf) = doc.inhoud {
            pdf.update(cx, |pdf, cx| pdf.ruim_op(window, cx));
        }
        if self.tabs.is_empty() {
            window.remove_window();
            return;
        }
        if self.actief > index || self.actief >= self.tabs.len() {
            self.actief = self.actief.saturating_sub(1);
        }
        cx.notify();
    }

    fn titelbalk(&self, th: &Thema, cx: &mut Context<Self>) -> impl IntoElement {
        let tabs = self.tabs.iter().enumerate().map(|(index, doc)| {
            let naam = naam(&doc.pad);
            let gewijzigd = doc.gewijzigd(cx);
            // Twee tabs met dezelfde naam: de map erachter zegt welke het is.
            let dubbel = self.tabs.iter().filter(|ander| crate::venster::naam(&ander.pad) == naam).count() > 1;
            let map = doc.pad.parent().map(crate::venster::naam).filter(|_| dubbel);
            let groep = SharedString::from(format!("doc-{index}"));
            tab_vak(th, index == self.actief)
                .id(("doc", doc.id))
                .group(groep.clone())
                .flex_shrink(1.0)
                .min_w(px(96.0))
                .max_w(px(260.0))
                .child(bestandsicoon::bestand(&doc.pad, th))
                .child(div().min_w_0().truncate().child(naam))
                .children(map.map(|map| div().text_color(rgb(th.mute)).font_weight(FontWeight::NORMAL).child(map)))
                // Niet bewaard: een stip, die onder de muis het kruis wordt.
                .when(gewijzigd, |d| {
                    d.child(
                        div()
                            .flex()
                            .justify_center()
                            .min_w(px(t::ICOON))
                            .text_color(rgb(th.mute))
                            .group_hover(groep.clone(), |s| s.hidden())
                            .child("●"),
                    )
                })
                .child(
                    div()
                        .id(("sluit", doc.id))
                        .flex()
                        .justify_center()
                        .min_w(px(t::ICOON))
                        .text_color(rgb(th.mute))
                        .font_weight(FontWeight::NORMAL)
                        .when(!gewijzigd, |d| d.invisible().group_hover(groep.clone(), |s| s.visible()))
                        .when(gewijzigd, |d| d.hidden().group_hover(groep.clone(), |s| s.flex()))
                        .hover(|d| d.text_color(rgb(th.ink)))
                        .child("×")
                        .on_click(cx.listener(move |this, _, window, cx| {
                            cx.stop_propagation();
                            this.sluit(index, window, cx);
                        })),
                )
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.actief = index;
                    cx.notify();
                }))
        });
        let doc = self.tabs.get(self.actief);
        let pad = doc.map(|doc| doc.pad.clone());
        // Markdown en json bewerk ik hier; de rest kan naar de app die macOS ervoor heeft.
        let rechts = match doc {
            Some(doc) if bewerkbaar(doc.soort) => {
                let bewerkt = doc.bewerk.as_ref().is_some_and(|bewerk| bewerk.aan);
                tab_vak(th, false)
                    .id("bewerken")
                    .flex_none()
                    .child(if bewerkt { t("lezen", "read") } else { t("bewerken", "edit") })
                    .on_click(cx.listener(|this, _, window, cx| this.wissel(window, cx)))
            }
            _ => tab_vak(th, false)
                .id("in-app")
                .flex_none()
                .child(t("in app openen", "open in app"))
                .on_click(cx.listener(move |_, _, _, cx| {
                    if let Some(pad) = &pad {
                        cx.open_with_system(pad);
                    }
                })),
        };
        let fout = doc.and_then(|doc| doc.bewerk.as_ref()).and_then(|bewerk| bewerk.fout.clone());
        div()
            .flex()
            .flex_none()
            .h(px(t::H_TITELBALK))
            .pl(px(t::B_LICHTEN))
            .bg(rgb(th.bar))
            .border_b(px(t::LIJN))
            .border_color(rgb(th.line))
            .child(
                div()
                    .id("tabs")
                    .flex()
                    .flex_1()
                    .min_w_0()
                    .overflow_x_scroll()
                    .children(tabs.flat_map(|tab| [lijn(th).into_any_element(), tab.into_any_element()]))
                    .child(lijn(th)),
            )
            .children(fout.map(|fout| {
                div()
                    .flex()
                    .flex_shrink(1.0)
                    .min_w_0()
                    .items_center()
                    .px(px(t::TAB_PAD))
                    .text_color(rgb(th.mute))
                    .child(div().truncate().child(fout))
            }))
            .child(lijn(th))
            .child(rechts)
    }
}

fn bewerkbaar(soort: Soort) -> bool {
    matches!(soort, Soort::Markdown | Soort::Json)
}

/// JERA_DOC_SHOT=pad bewaart elke seconde dit venster als afbeelding. JERA_DOC_TAB=n kiest na
/// drie seconden tab n, JERA_DOC_SCROLL=y scrolt daarna zoveel punten omlaag. JERA_DOC_TYP=tekst
/// zet dan de tab op bewerken en typt de tekst, JERA_DOC_BEWAAR bewaart na zes seconden.
#[cfg(feature = "schermafdruk")]
fn schermafdruk(window: &mut Window, cx: &mut Context<Documenten>) {
    let Ok(pad) = std::env::var("JERA_DOC_SHOT") else { return };
    let tab: Option<usize> = std::env::var("JERA_DOC_TAB").ok().and_then(|n| n.parse().ok());
    let scroll: Option<f32> = std::env::var("JERA_DOC_SCROLL").ok().and_then(|n| n.parse().ok());
    let typ = std::env::var("JERA_DOC_TYP");
    let bewaar = std::env::var("JERA_DOC_BEWAAR").is_ok();
    cx.spawn_in(window, async move |this, cx| {
        for tik in 1.. {
            cx.background_executor().timer(Duration::from_secs(1)).await;
            let pad = pad.clone();
            let gedaan = this.update_in(cx, |docs, window, cx| {
                if tik == 3 {
                    if let Some(tab) = tab {
                        docs.actief = tab.min(docs.tabs.len().saturating_sub(1));
                        cx.notify();
                    }
                }
                if tik == 4 {
                    if let Some(y) = scroll {
                        let midden = window.viewport_size().map(|m| m / 2.0);
                        let event = ScrollEvent {
                            position: point(midden.width, midden.height),
                            delta: ScrollDelta::Pixels(point(px(0.0), px(-y))),
                            ..Default::default()
                        };
                        window.dispatch_event(PlatformInput::Scroll(event), cx);
                    }
                    if let Ok(tekst) = &typ {
                        docs.wissel(window, cx);
                        docs.typ(tekst, window, cx);
                    }
                }
                if tik == 6 && bewaar {
                    window.dispatch_action(Box::new(Bewaar), cx);
                }
                if let Ok(beeld) = window.render_to_image() {
                    let tijdelijk = format!("{pad}.tmp.png");
                    if beeld.save(&tijdelijk).is_ok() {
                        let _ = std::fs::rename(&tijdelijk, &pad);
                    }
                }
            });
            if gedaan.is_err() {
                break;
            }
        }
    })
    .detach();
}

fn gewijzigd(pad: &Path) -> Option<SystemTime> {
    std::fs::metadata(pad).and_then(|m| m.modified()).ok()
}

/// Eén korte regel midden in het venster, voor een fout of een leeg bestand.
fn regel(th: &Thema, tekst: SharedString) -> AnyElement {
    div()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .px(px(t::PAD))
        .text_color(rgb(th.mute))
        .whitespace_normal()
        .child(tekst)
        .into_any_element()
}

impl Render for Documenten {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let th = t::thema(window);
        let doc = self.tabs.get(self.actief);
        if let Some(doc) = doc {
            window.set_window_title(&naam(&doc.pad));
        }
        let inhoud = doc.map(|doc| match (&doc.bewerk, &doc.inhoud) {
            (Some(bewerk), _) if bewerk.aan => bewerk.toon(doc.soort, th, window, cx),
            (_, Inhoud::Laden) => div().into_any_element(),
            (_, Inhoud::Fout(fout)) => regel(th, fout.clone()),
            (_, Inhoud::Markdown(tekst)) => markdown::toon(doc.id, tekst.clone(), &doc.pad, th, window),
            (_, Inhoud::Beeld(beeld)) => div()
                .size_full()
                .p(px(24.0))
                .child(img(beeld.clone()).size_full().object_fit(ObjectFit::ScaleDown))
                .into_any_element(),
            (_, Inhoud::Tabel(tabel)) => tabel.clone().into_any_element(),
            (_, Inhoud::Json(json)) => json.clone().into_any_element(),
            (_, Inhoud::Pdf(pdf)) => pdf.clone().into_any_element(),
        });
        div()
            .key_context("Documenten")
            .track_focus(&self.focus)
            .on_action(cx.listener(|this, _: &SluitTab, window, cx| this.sluit(this.actief, window, cx)))
            .on_action(cx.listener(|this, _: &VorigeTab, _, cx| {
                this.actief = (this.actief + this.tabs.len().max(1) - 1) % this.tabs.len().max(1);
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &VolgendeTab, _, cx| {
                this.actief = (this.actief + 1) % this.tabs.len().max(1);
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &Bewaar, _, cx| {
                this.bewaar(this.actief, false, cx);
            }))
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(th.bg))
            .text_color(rgb(th.ink))
            .font_family(t::LETTER)
            .text_size(px(t::TEKST))
            .line_height(relative(t::REGEL))
            .whitespace_nowrap()
            .child(self.titelbalk(th, cx))
            .child(div().flex_1().min_h_0().children(inhoud))
            .children(self.dialoog(th, cx))
    }
}
