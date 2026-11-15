//! Json als boom: een object of lijst klapt open en dicht, met ⌥ ook alles eronder.

use std::{collections::HashSet, ops::Range, sync::Arc};

use gpui_kit::{prelude::FluentBuilder, *};
use serde_json::Value;

use crate::thema::{self as t, Thema};
use crate::venster::{PIJL_DICHT, PIJL_OPEN, pijl};

/// Zo lang wordt een tekst in de boom hoogstens; de rest valt weg.
const MAX_TEKENS: usize = 400;
/// Heeft de buitenste laag hoogstens zoveel onderdelen, dan staan die bij het openen ook open.
const OPEN_BIJ_START: usize = 20;

/// Eén regel van de boom.
struct Regel {
    diepte: usize,
    sleutel: Option<SharedString>,
    /// De sleutel is een plek in een lijst.
    index: bool,
    waarde: SharedString,
    /// Een object of lijst: waar het staat, als json pointer, en of het open is.
    tak: Option<(String, bool)>,
}

pub struct Json {
    waarde: Value,
    open: HashSet<String>,
    regels: Arc<Vec<Regel>>,
    scroll: UniformListScrollHandle,
}

impl Json {
    pub fn new(waarde: Value) -> Self {
        let mut open = HashSet::from([String::new()]);
        let kinderen: Vec<(String, &Value)> = kinderen(&waarde, "");
        if kinderen.len() <= OPEN_BIJ_START {
            open.extend(kinderen.into_iter().filter(|(_, w)| w.is_object() || w.is_array()).map(|(p, _)| p));
        }
        let mut json = Self { waarde, open, regels: Arc::default(), scroll: UniformListScrollHandle::new() };
        json.bouw();
        json
    }

    /// Het bestand veranderde. Wat open stond, blijft open.
    pub fn vervang(&mut self, waarde: Value, cx: &mut Context<Self>) {
        self.waarde = waarde;
        self.bouw();
        cx.notify();
    }

    fn bouw(&mut self) {
        let mut regels = Vec::new();
        match &self.waarde {
            w @ (Value::Object(_) | Value::Array(_)) => voeg_toe(w, "", 0, &self.open, &mut regels),
            w => regels.push(Regel { diepte: 0, sleutel: None, index: false, waarde: blad(w), tak: None }),
        }
        self.regels = Arc::new(regels);
    }

    /// Klapt de tak open of dicht. Met ⌥ ook alles eronder.
    fn klap(&mut self, pad: String, open: bool, alles: bool, cx: &mut Context<Self>) {
        if alles {
            let mut takken = Vec::new();
            if let Some(waarde) = self.waarde.pointer(&pad) {
                verzamel(waarde, &pad, &mut takken);
            }
            for tak in takken {
                if open {
                    self.open.insert(tak);
                } else {
                    self.open.remove(&tak);
                }
            }
        } else if open {
            self.open.insert(pad);
        } else {
            self.open.remove(&pad);
        }
        self.bouw();
        cx.notify();
    }

    fn regels(&mut self, bereik: Range<usize>, window: &mut Window, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let th = t::thema(window);
        let regels = self.regels.clone();
        bereik.map(|i| self.regel(&regels[i], i, th, cx)).collect()
    }

    fn regel(&self, regel: &Regel, i: usize, th: &Thema, cx: &mut Context<Self>) -> AnyElement {
        let pijl_of_ruimte = match &regel.tak {
            Some((_, open)) => pijl(if *open { PIJL_OPEN } else { PIJL_DICHT }, th).into_any_element(),
            None => div().flex_none().w(px(t::ICOON)).into_any_element(),
        };
        div()
            .id(i)
            .w_full()
            .flex()
            .items_center()
            .gap(px(6.0))
            .h(px(t::H_RIJ))
            .pl(px(t::PAD + regel.diepte as f32 * t::INSPRING))
            .pr(px(t::PAD))
            .child(pijl_of_ruimte)
            .children(regel.sleutel.clone().map(|sleutel| {
                div()
                    .flex()
                    .flex_none()
                    .child(div().when(regel.index, |d| d.text_color(rgb(th.mute))).child(sleutel))
                    .child(div().text_color(rgb(th.mute)).child(":"))
            }))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .truncate()
                    .when(regel.tak.is_some(), |d| d.text_color(rgb(th.mute)))
                    .child(regel.waarde.clone()),
            )
            .when_some(regel.tak.clone(), |d, (pad, open)| {
                d.hover(|d| d.bg(rgb(th.sel))).on_click(cx.listener(move |this, klik: &ClickEvent, _, cx| {
                    this.klap(pad.clone(), !open, klik.modifiers().alt, cx)
                }))
            })
            .into_any_element()
    }
}

/// De onderdelen van een object of lijst, met hun pointer.
fn kinderen<'a>(waarde: &'a Value, pad: &str) -> Vec<(String, &'a Value)> {
    match waarde {
        Value::Object(map) => map.iter().map(|(k, w)| (format!("{pad}/{}", k.replace('~', "~0").replace('/', "~1")), w)).collect(),
        Value::Array(lijst) => lijst.iter().enumerate().map(|(i, w)| (format!("{pad}/{i}"), w)).collect(),
        _ => Vec::new(),
    }
}

fn voeg_toe(waarde: &Value, pad: &str, diepte: usize, open: &HashSet<String>, regels: &mut Vec<Regel>) {
    let sleutels: Vec<(SharedString, bool)> = match waarde {
        Value::Object(map) => map.keys().map(|k| (SharedString::from(k.clone()), false)).collect(),
        Value::Array(lijst) => (0..lijst.len()).map(|i| (SharedString::from(i.to_string()), true)).collect(),
        _ => return,
    };
    for ((pad, kind), (sleutel, index)) in kinderen(waarde, pad).into_iter().zip(sleutels) {
        let tak = kind.is_object() || kind.is_array();
        let is_open = tak && open.contains(&pad);
        regels.push(Regel {
            diepte,
            sleutel: Some(sleutel),
            index,
            waarde: if tak { telling(kind) } else { blad(kind) },
            tak: tak.then(|| (pad.clone(), is_open)),
        });
        if is_open {
            voeg_toe(kind, &pad, diepte + 1, open, regels);
        }
    }
}

/// Alle takken onder en met `waarde`.
fn verzamel(waarde: &Value, pad: &str, takken: &mut Vec<String>) {
    if waarde.is_object() || waarde.is_array() {
        takken.push(pad.to_string());
        for (pad, kind) in kinderen(waarde, pad) {
            verzamel(kind, &pad, takken);
        }
    }
}

/// Een object of lijst, kort: {3} of [12].
fn telling(waarde: &Value) -> SharedString {
    match waarde {
        Value::Object(map) => format!("{{{}}}", map.len()).into(),
        Value::Array(lijst) => format!("[{}]", lijst.len()).into(),
        _ => SharedString::default(),
    }
}

/// Een tekst, getal, waar of onwaar, of null, zoals in json.
fn blad(waarde: &Value) -> SharedString {
    let tekst = waarde.to_string();
    match tekst.char_indices().nth(MAX_TEKENS) {
        Some((i, _)) => format!("{}…", &tekst[..i]).into(),
        None => tekst.into(),
    }
}

impl Render for Json {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        uniform_list("json", self.regels.len(), cx.processor(Self::regels))
            .size_full()
            .py(px(8.0))
            .track_scroll(&self.scroll)
    }
}
