//! Onderdelen uit het design system die op meer plekken in het venster staan.

use super::*;

/// Een vak in de titelbalk: vlak met een streep erboven als het actief is.
pub(crate) fn tab_vak(th: &Thema, actief: bool) -> Div {
    div()
        .flex()
        .items_center()
        .gap(px(8.0))
        .mb(px(-t::LIJN))
        .px(px(t::TAB_PAD))
        .border_t(px(t::STREEP))
        .border_color(transparent_black())
        .text_color(rgb(th.mute))
        .when(actief, |d| {
            d.bg(rgb(th.bg)).text_color(rgb(th.ink)).font_weight(FontWeight::BOLD).border_color(rgb(th.accent))
        })
        .when(!actief, |d| d.hover(|d| d.text_color(rgb(th.ink))))
}

pub(super) const LOGO: &[u8] = include_bytes!("../../iconen/jera.svg");

pub(super) const BOOG: &[u8] = include_bytes!("../../iconen/bezig.svg");

pub(super) const SLOT: &[u8] = include_bytes!("../../iconen/toestemming.svg");

pub(crate) const VINK: &[u8] = include_bytes!("../../iconen/klaar.svg");

pub(super) const STREEPJE: &[u8] = include_bytes!("../../iconen/idle.svg");

pub(crate) const PIJL_DICHT: &[u8] = include_bytes!("../../iconen/pijl-dicht.svg");

pub(crate) const PIJL_OPEN: &[u8] = include_bytes!("../../iconen/pijl-open.svg");

pub(super) const NIEUW: &[u8] = include_bytes!("../../iconen/nieuw.svg");

pub(super) const KOPIEER: &[u8] = include_bytes!("../../iconen/kopieer.svg");

pub(super) const PLAK: &[u8] = include_bytes!("../../iconen/plak.svg");

pub(super) const HERNOEM: &[u8] = include_bytes!("../../iconen/hernoem.svg");

pub(super) const PRULLENMAND: &[u8] = include_bytes!("../../iconen/prullenmand.svg");

pub(super) const STOP: &[u8] = include_bytes!("../../iconen/stop.svg");

/// Het icoon van een actie in het menu, in `mute`.
pub(super) fn actie_icoon(data: &'static [u8], th: &Thema) -> AnyElement {
    svg().data(data).flex_none().size(px(t::ICOON)).text_color(rgb(th.mute)).into_any_element()
}

pub(crate) fn pijl(data: &[u8], th: &Thema) -> Svg {
    svg().data(data).flex_none().size(px(t::ICOON)).text_color(rgb(th.mute))
}

pub(super) fn kleur(status: Status, th: &Thema) -> u32 {
    match status {
        Status::Bezig | Status::NetKlaar => th.ok,
        Status::Antwoord => th.warn,
        Status::Toestemming => th.bad,
        Status::Idle => th.mute,
    }
}

/// Het statusicoon uit het design system, zonder woord.
pub(super) fn icoon(status: Status, id: impl Into<ElementId>, th: &Thema) -> AnyElement {
    let maat = px(t::ICOON);
    let beeld = |data: &[u8]| svg().data(data).flex_none().size(maat).text_color(rgb(kleur(status, th)));
    match status {
        Status::Bezig => beeld(BOOG)
            .with_animation(id, Animation::new(Duration::from_millis(1100)).repeat(), |s, d| {
                s.with_transformation(Transformation::rotate(percentage(d)))
            })
            .into_any_element(),
        Status::Antwoord => div()
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .size(maat)
            .rounded_full()
            .bg(rgb(th.warn))
            .text_color(rgb(th.bg))
            .text_size(px(t::KLEIN))
            .font_weight(FontWeight::BOLD)
            .child("?")
            .into_any_element(),
        Status::Toestemming => beeld(SLOT).into_any_element(),
        Status::NetKlaar => beeld(VINK).into_any_element(),
        Status::Idle => beeld(STREEPJE).into_any_element(),
    }
}

/// Een staande lijn tussen twee vakken.
pub(crate) fn lijn(th: &Thema) -> Div {
    div().flex_none().w(px(t::LIJN)).h_full().bg(rgb(th.line))
}

/// Een gewone knop uit het design system: een woord met een rand.
pub(crate) fn knop_vak(th: &Thema) -> Div {
    knop_uit(th).text_color(rgb(th.ink)).hover(|d| d.bg(rgb(th.sel)))
}

/// Een knop die nu niets doet: het woord in `mute`, en niets onder de muis.
pub(crate) fn knop_uit(th: &Thema) -> Div {
    div().flex().flex_none().items_center().h(px(t::H_KNOP)).px(px(8.0)).border(px(t::LIJN)).border_color(rgb(th.line_strong)).text_color(rgb(th.mute))
}

/// Het vak van een menu, met een rand in `line_strong`. De muis gaat er niet doorheen naar wat eronder ligt.
pub(crate) fn menuvak(th: &Thema) -> Div {
    div()
        .flex()
        .flex_col()
        .py(px(4.0))
        .bg(rgb(th.bg))
        .border(px(t::LIJN))
        .border_color(rgb(th.line_strong))
        .occlude()
}

/// De lijn tussen twee groepen in een menu.
pub(crate) fn scheiding(th: &Thema) -> Div {
    div().h(px(t::LIJN)).my(px(4.0)).bg(rgb(th.line))
}

/// Zo breed wordt een kaartje hoogstens; een langere regel loopt door op de volgende.
const TIP_BREED: f32 = 360.0;

/// Het kaartje dat onder de muis verschijnt, zoals het kaartje in het git-paneel.
/// De eerste regel in `ink`, de regels daarna in `mute`.
pub(super) struct Tip(pub(super) Vec<SharedString>);

impl Render for Tip {
    fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let th = t::thema(window);
        div()
            .px(px(10.0))
            .py(px(7.0))
            .bg(rgb(th.bg))
            .border(px(t::LIJN))
            .border_color(rgb(th.line_strong))
            .text_color(rgb(th.ink))
            .font_family(t::LETTER)
            .text_size(px(t::TEKST))
            .line_height(relative(t::REGEL))
            .max_w(px(TIP_BREED))
            .children(
                self.0.iter().enumerate().map(|(i, regel)| div().when(i > 0, |d| d.text_color(rgb(th.mute))).child(regel.clone())),
            )
    }
}
