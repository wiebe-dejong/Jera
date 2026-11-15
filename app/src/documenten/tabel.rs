//! Een tabel uit csv, tsv of een werkboek (xlsx, xls, ods). Alleen de rijen in beeld worden
//! getekend, dus ook een groot bestand blijft vlot.

use std::{path::Path, sync::Arc};

use calamine::{Data, Reader};
use gpui_kit::{prelude::FluentBuilder, *};

use crate::taal::t;
use crate::thema::{self as t, Thema};
use crate::venster::{lijn, tab_vak};

/// De breedte van één teken in JetBrains Mono: 600 van de 1000 eenheden.
const TEKEN: f32 = t::TEKST * 0.6;
/// Ruimte links en rechts in een cel.
const CEL_PAD: f32 = 8.0;
/// Een kolom is hoogstens zoveel tekens breed; wat langer is, krijgt …
const MAX_TEKENS: usize = 40;
/// Zoveel rijen bepalen de breedte van een kolom.
const MEET_RIJEN: usize = 2000;

/// Eén blad: bij csv het hele bestand, bij een werkboek een tabblad.
pub struct Blad {
    naam: String,
    /// Bij csv de eerste regel. Een werkboek heeft letters, zoals in Excel.
    kop: Option<Vec<SharedString>>,
    rijen: Vec<Vec<SharedString>>,
    /// Waar het blad in een werkboek begint: rij en kolom, vanaf 0.
    begin: (usize, usize),
}

pub fn lees(pad: &Path) -> Result<Vec<Blad>, String> {
    let ext = pad.extension().and_then(|e| e.to_str()).unwrap_or_default().to_ascii_lowercase();
    match ext.as_str() {
        "csv" | "tsv" => lees_csv(pad, &ext),
        _ => lees_werkboek(pad),
    }
}

fn lees_csv(pad: &Path, ext: &str) -> Result<Vec<Blad>, String> {
    let bytes = std::fs::read(pad).map_err(|f| f.to_string())?;
    let tekst = String::from_utf8_lossy(&bytes);
    // Een Nederlandse Excel schrijft csv met puntkomma's.
    let eerste = tekst.lines().next().unwrap_or_default();
    let scheiding = match ext {
        "tsv" => b'\t',
        _ if eerste.matches(';').count() > eerste.matches(',').count() => b';',
        _ if eerste.matches('\t').count() > eerste.matches(',').count() => b'\t',
        _ => b',',
    };
    let mut lezer =
        csv::ReaderBuilder::new().has_headers(false).flexible(true).delimiter(scheiding).from_reader(tekst.as_bytes());
    let mut rijen: Vec<Vec<SharedString>> = Vec::new();
    for rij in lezer.records() {
        let rij = rij.map_err(|f| f.to_string())?;
        rijen.push(rij.iter().map(cel).collect());
    }
    let kop = (!rijen.is_empty()).then(|| rijen.remove(0));
    Ok(vec![Blad { naam: String::new(), kop, rijen, begin: (0, 0) }])
}

fn lees_werkboek(pad: &Path) -> Result<Vec<Blad>, String> {
    let mut boek = calamine::open_workbook_auto(pad).map_err(|f| f.to_string())?;
    let mut bladen = Vec::new();
    for naam in boek.sheet_names() {
        let Ok(bereik) = boek.worksheet_range(&naam) else { continue };
        let begin = bereik.start().map_or((0, 0), |(r, k)| (r as usize, k as usize));
        let rijen = bereik.rows().map(|rij| rij.iter().map(waarde).collect()).collect();
        bladen.push(Blad { naam, kop: None, rijen, begin });
    }
    Ok(bladen)
}

/// Een cel op één regel.
fn cel(tekst: &str) -> SharedString {
    if tekst.contains(['\n', '\r']) {
        tekst.replace("\r\n", " ").replace(['\n', '\r'], " ").into()
    } else {
        SharedString::from(tekst.to_string())
    }
}

fn waarde(data: &Data) -> SharedString {
    match data {
        Data::Empty => SharedString::default(),
        Data::Int(n) => n.to_string().into(),
        Data::Float(f) => getal(*f).into(),
        Data::String(s) | Data::DateTimeIso(s) | Data::DurationIso(s) => cel(s),
        Data::Bool(b) => (if *b { "TRUE" } else { "FALSE" }).into(),
        Data::DateTime(d) => match d.as_datetime() {
            Some(moment) if moment.time() == chrono::NaiveTime::MIN => moment.format("%Y-%m-%d").to_string().into(),
            Some(moment) => moment.format("%Y-%m-%d %H:%M").to_string().into(),
            None => getal(d.as_f64()).into(),
        },
        Data::Error(fout) => format!("{fout}").into(),
    }
}

/// Een getal met hoogstens vier decimalen, of zes cijfers als het kleiner is dan 1:
/// 55.7692307692 wordt 55.7692, 0.000123456 wordt 0.000123456.
fn getal(f: f64) -> String {
    if f.fract() == 0.0 && f.abs() < 1e15 {
        return format!("{}", f as i64);
    }
    let decimalen = match f.abs() {
        a if a >= 1.0 => 4,
        a => (5 - a.log10().floor() as i32).clamp(4, 12) as usize,
    };
    let tekst = format!("{f:.decimalen$}");
    tekst.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// De kolom zoals Excel hem noemt: A, B, …, Z, AA, AB.
fn letters(mut kolom: usize) -> String {
    let mut uit = Vec::new();
    loop {
        uit.push(b'A' + (kolom % 26) as u8);
        if kolom < 26 {
            break;
        }
        kolom = kolom / 26 - 1;
    }
    uit.reverse();
    String::from_utf8(uit).unwrap_or_default()
}

/// Een blad met wat nodig is om het te tekenen.
struct Gemeten {
    blad: Blad,
    kop: Vec<SharedString>,
    breedtes: Vec<f32>,
    /// Kolommen met alleen getallen staan rechts.
    rechts: Vec<bool>,
    /// De kolom met rijnummers.
    nummers: f32,
}

impl Gemeten {
    fn new(blad: Blad) -> Self {
        let kolommen =
            blad.rijen.iter().map(Vec::len).chain(blad.kop.as_ref().map(Vec::len)).max().unwrap_or(0);
        let kop: Vec<SharedString> = match &blad.kop {
            Some(kop) => (0..kolommen).map(|k| kop.get(k).cloned().unwrap_or_default()).collect(),
            None => (0..kolommen).map(|k| letters(blad.begin.1 + k).into()).collect(),
        };
        let mut tekens: Vec<usize> = kop.iter().map(|k| k.chars().count()).collect();
        let mut getallen = vec![true; kolommen];
        let mut gevuld = vec![false; kolommen];
        // Een werkboek heeft vaak zijn kop in de eerste rij; die telt niet mee voor getallen.
        let kop_rij = usize::from(blad.kop.is_none());
        for (r, rij) in blad.rijen.iter().take(MEET_RIJEN).enumerate() {
            for (k, waarde) in rij.iter().enumerate() {
                tekens[k] = tekens[k].max(waarde.chars().count());
                if !waarde.is_empty() && r >= kop_rij {
                    gevuld[k] = true;
                    getallen[k] &= is_getal(waarde);
                }
            }
        }
        let breedtes = tekens.iter().map(|&n| breedte(n.clamp(3, MAX_TEKENS))).collect();
        let rechts = getallen.iter().zip(&gevuld).map(|(g, v)| *g && *v).collect();
        let nummers = breedte((blad.begin.0 + blad.rijen.len()).to_string().len().max(2));
        Self { blad, kop, breedtes, rechts, nummers }
    }

    fn totaal(&self) -> f32 {
        self.nummers + self.breedtes.iter().sum::<f32>()
    }
}

fn breedte(tekens: usize) -> f32 {
    tekens as f32 * TEKEN + 2.0 * CEL_PAD + t::LIJN
}

fn is_getal(tekst: &str) -> bool {
    let tekst = tekst.trim().trim_end_matches('%').trim_start_matches(['€', '$']).trim();
    tekst.replace(',', ".").parse::<f64>().is_ok()
}

pub struct Tabel {
    bladen: Vec<Arc<Gemeten>>,
    gekozen: usize,
    scroll: UniformListScrollHandle,
}

impl Tabel {
    pub fn new(bladen: Vec<Blad>) -> Self {
        Self {
            bladen: bladen.into_iter().map(|b| Arc::new(Gemeten::new(b))).collect(),
            gekozen: 0,
            scroll: UniformListScrollHandle::new(),
        }
    }

    /// Het bestand veranderde. Het gekozen blad en de plek blijven.
    pub fn vervang(&mut self, bladen: Vec<Blad>, cx: &mut Context<Self>) {
        let naam = self.bladen.get(self.gekozen).map(|b| b.blad.naam.clone());
        self.bladen = bladen.into_iter().map(|b| Arc::new(Gemeten::new(b))).collect();
        self.gekozen = self.bladen.iter().position(|b| Some(&b.blad.naam) == naam.as_ref()).unwrap_or(0);
        cx.notify();
    }

    fn bladen_balk(&self, th: &Thema, cx: &mut Context<Self>) -> impl IntoElement {
        let bladen = self.bladen.iter().enumerate().map(|(index, gemeten)| {
            tab_vak(th, index == self.gekozen)
                .id(("blad", index))
                .h_full()
                .child(gemeten.blad.naam.clone())
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.gekozen = index;
                    this.scroll.scroll_to_item(0, ScrollStrategy::Top);
                    cx.notify();
                }))
        });
        div()
            .id("bladen")
            .flex()
            .flex_none()
            .h(px(t::H_RIJ + t::STREEP))
            .bg(rgb(th.bar))
            .border_t(px(t::LIJN))
            .border_color(rgb(th.line))
            .overflow_x_scroll()
            .children(bladen.flat_map(|blad| [blad.into_any_element(), lijn(th).into_any_element()]))
    }
}

/// Een rij: het nummer en de cellen.
fn rij(gemeten: &Gemeten, nummer: SharedString, cellen: &[SharedString], kop: bool, th: &Thema) -> Div {
    let cel = |breed: f32| {
        div()
            .flex()
            .flex_none()
            .items_center()
            .w(px(breed))
            .h_full()
            .px(px(CEL_PAD))
            .border_r(px(t::LIJN))
            .border_color(rgb(th.line))
    };
    div()
        .flex()
        .flex_none()
        .w(px(gemeten.totaal()))
        .h(px(t::H_RIJ))
        .border_b(px(t::LIJN))
        .border_color(rgb(if kop { th.line_strong } else { th.line }))
        .when(kop, |d| d.font_weight(FontWeight::BOLD))
        .child(cel(gemeten.nummers).justify_end().text_color(rgb(th.mute)).font_weight(FontWeight::NORMAL).child(nummer))
        .children(gemeten.breedtes.iter().enumerate().map(|(k, &breed)| {
            let rechts = gemeten.rechts[k] && !kop;
            let tekst = cellen.get(k).cloned().unwrap_or_default();
            cel(breed).child(div().w_full().truncate().when(rechts, |d| d.text_right()).child(tekst))
        }))
}

impl Render for Tabel {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let th = t::thema(window);
        let Some(gemeten) = self.bladen.get(self.gekozen).cloned() else {
            return super::regel(th, t("Leeg.", "Empty.").into());
        };
        let rijen = gemeten.blad.rijen.len();
        let lijst = {
            let gemeten = gemeten.clone();
            uniform_list("rijen", rijen, move |bereik, window, _| {
                let th = t::thema(window);
                bereik
                    .map(|i| {
                        let nummer = SharedString::from((gemeten.blad.begin.0 + i + 1).to_string());
                        rij(&gemeten, nummer, &gemeten.blad.rijen[i], false, th)
                    })
                    .collect()
            })
            .flex_1()
            .track_scroll(&self.scroll)
        };
        let leeg = rijen == 0 && gemeten.kop.is_empty();
        div()
            .size_full()
            .flex()
            .flex_col()
            .child(
                div().id("tabel").flex_1().min_h_0().overflow_x_scroll().child(
                    div()
                        .flex()
                        .flex_col()
                        .h_full()
                        .w(px(gemeten.totaal()))
                        .child(rij(&gemeten, SharedString::default(), &gemeten.kop, true, th))
                        .child(lijst),
                ),
            )
            .when(leeg, |d| d.child(super::regel(th, t("Leeg.", "Empty.").into())))
            .when(self.bladen.len() > 1, |d| d.child(self.bladen_balk(th, cx)))
            .into_any_element()
    }
}
