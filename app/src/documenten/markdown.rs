//! Markdown, getekend door de TextView van GPUI Kit in de letter en kleuren van Jera.

use std::path::{Path, PathBuf};

use gpui_kit::{
    base::text::{TextView, TextViewStyle},
    *,
};

use crate::thema::{self as t, Thema};

/// Zo breed wordt de tekst hoogstens; daarbuiten blijft het venster leeg.
pub(super) const BREED: f32 = 760.0;

pub(super) fn toon(id: usize, tekst: SharedString, pad: &Path, th: &Thema, window: &Window) -> AnyElement {
    let map = pad.parent().map(Path::to_path_buf).unwrap_or_default();
    let beeld_map = map.clone();
    let marge = ((window.viewport_size().width - px(BREED)) / 2.0).max(px(32.0));
    TextView::markdown(("md", id), tekst)
        .style(stijl(th))
        .selectable(true)
        .scrollable(true)
        .image_source(move |uri| bron(uri, &beeld_map))
        .on_link_click(move |url, _, _, cx| link(url, &map, cx))
        .size_full()
        .px(marge)
        .py(px(28.0))
        .whitespace_normal()
        .into_any_element()
}

/// Koppen zijn vet en even groot als de tekst, zoals alles in Jera.
fn stijl(th: &Thema) -> TextViewStyle {
    let kleur = |c: u32| -> Hsla { rgb(c).into() };
    let lijn = kleur(th.line);
    TextViewStyle::default()
        .with_foreground(kleur(th.ink))
        .with_muted_foreground(kleur(th.mute))
        .with_link(kleur(th.ink))
        .with_selection(kleur(th.line_strong))
        .with_code_background(kleur(th.bar))
        .with_inline_code(HighlightStyle { color: Some(kleur(th.ink)), background_color: Some(kleur(th.bar)), ..Default::default() })
        .with_border(lijn)
        .with_table_head(StyleRefinement::default().bg(kleur(th.bar)).font_weight(FontWeight::BOLD))
        .with_heading(|_| StyleRefinement::default().text_size(px(t::TEKST)).font_weight(FontWeight::BOLD))
        .with_code_block(
            StyleRefinement::default()
                .font_family(t::LETTER)
                .text_size(px(t::TEKST))
                .bg(kleur(th.bg))
                .border_1()
                .border_color(lijn)
                .p(px(t::PAD)),
        )
        .with_dark(!th.licht)
}

/// Een afbeelding in de tekst: een pad telt vanaf de map van het bestand.
fn bron(uri: &SharedUri, map: &Path) -> ImageSource {
    let tekst = uri.as_ref();
    if tekst.contains("://") && !tekst.starts_with("file://") {
        return ImageSource::Resource(Resource::Uri(uri.clone()));
    }
    ImageSource::Resource(Resource::Path(pad(tekst.trim_start_matches("file://"), map).into()))
}

/// Een link naar een website gaat naar Safari, een link naar een bestand dat Jera kan tonen naar
/// een tab, een ander bestand naar zijn app.
fn link(url: &str, map: &Path, cx: &mut App) {
    if url.starts_with('#') {
        return;
    }
    if url.contains("://") && !url.starts_with("file://") || url.starts_with("mailto:") {
        cx.open_url(url);
        return;
    }
    let deel = url.trim_start_matches("file://").split('#').next().unwrap_or_default();
    let pad = pad(deel, map);
    // Na deze klik: het venster is nu bezig en kan zichzelf niet bijwerken.
    cx.defer(move |cx| {
        if !super::open(&pad, cx) {
            cx.open_with_system(&pad);
        }
    });
}

fn pad(tekst: &str, map: &Path) -> PathBuf {
    let tekst = ontcijfer(tekst);
    match tekst.strip_prefix("~/") {
        Some(rest) => crate::thuis().join(rest),
        None => map.join(tekst),
    }
}

/// `%20` en dergelijke terug naar hun teken.
fn ontcijfer(tekst: &str) -> String {
    let bytes = tekst.as_bytes();
    let mut uit = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let hex = (bytes[i] == b'%')
            .then(|| tekst.get(i + 1..i + 3).and_then(|h| u8::from_str_radix(h, 16).ok()))
            .flatten();
        match hex {
            Some(b) => {
                uit.push(b);
                i += 3;
            }
            None => {
                uit.push(bytes[i]);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&uit).into_owned()
}
