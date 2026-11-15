//! Markdown en json bewerken: de editor van GPUI Kit, ⌘S bewaart. Schrijft een sessie in het
//! bestand terwijl ik het open heb, dan neemt de editor dat over, tenzij ik zelf iets wijzigde
//! dat nog niet bewaard is; dan vraagt bewaren of ik wil overschrijven.

use gpui_kit::{
    base::input::{Editor, EditorState, InputEditorStyle, InputEvent},
    *,
};

use super::{Doc, Documenten, Gelezen, Soort, gewijzigd, uit_tekst};
use crate::thema::{self as t, Thema};
use crate::venster::{knop_vak, naam};

pub(super) struct Bewerk {
    editor: Entity<EditorState>,
    /// De tab toont de editor, niet de opgemaakte tekst.
    pub(super) aan: bool,
    /// Het bestand veranderde op schijf terwijl ik wijzigingen had.
    pub(super) buiten: bool,
    /// Waarom bewaren niet lukte.
    pub(super) fout: Option<SharedString>,
    _volg: Subscription,
}

/// Een vraag over de tab met dit id.
pub(super) enum Vraag {
    /// Sluiten met wijzigingen die niet bewaard zijn.
    Sluiten(usize),
    /// Bewaren terwijl het bestand op schijf veranderde.
    Buiten(usize),
}

impl Doc {
    /// Of de editor iets anders zegt dan het bestand.
    pub(super) fn gewijzigd(&self, cx: &App) -> bool {
        self.bewerk.as_ref().is_some_and(|bewerk| bewerk.editor.read(cx).value() != self.tekst)
    }
}

impl Bewerk {
    fn nieuw(id: usize, soort: Soort, tekst: SharedString, window: &mut Window, cx: &mut Context<Documenten>) -> Self {
        let editor = cx.new(|cx| {
            let mut editor = EditorState::new(window, cx).folding(false).line_number(soort == Soort::Json).soft_wrap(soort == Soort::Markdown);
            editor.set_value(tekst, window, cx);
            editor
        });
        let volg = cx.subscribe(&editor, move |docs, _, event: &InputEvent, cx| {
            if let InputEvent::Change = event {
                if let Some(bewerk) = docs.tabs.iter_mut().find(|doc| doc.id == id).and_then(|doc| doc.bewerk.as_mut()) {
                    bewerk.fout = None;
                }
                cx.notify();
            }
        });
        Self { editor, aan: true, buiten: false, fout: None, _volg: volg }
    }

    /// Zet `tekst` in de editor. De cursor en de plek blijven.
    pub(super) fn neem_over(&mut self, tekst: SharedString, window: &mut Window, cx: &mut App) {
        self.buiten = false;
        self.editor.update(cx, |editor, cx| {
            if editor.value() == tekst {
                return;
            }
            let (kies, scroll) = (editor.selected_range(), editor.scroll_offset());
            let rand = |i: usize| tekst.floor_char_boundary(i);
            editor.set_value(tekst.clone(), window, cx);
            editor.set_selected_range(rand(kies.start)..rand(kies.end), cx);
            editor.set_scroll_offset(scroll, cx);
        });
    }

    pub(super) fn toon(&self, soort: Soort, th: &Thema, window: &Window, cx: &mut App) -> AnyElement {
        // Markdown staat even breed en in het midden als bij lezen.
        let opvulling = match soort {
            Soort::Markdown => {
                let marge = ((window.viewport_size().width - px(super::markdown::BREED)) / 2.0).max(px(32.0));
                // De editor zet zelf 6 px voor elke regel; zo staat de tekst waar hij bij lezen staat.
                Edges { top: px(28.0), bottom: px(28.0), left: marge - px(6.0), right: marge }
            }
            _ => Edges { top: px(8.0), bottom: px(8.0), left: px(0.0), right: px(t::PAD) },
        };
        self.editor.update(cx, |editor, _| {
            editor.set_editor_style(stijl(th));
            editor.set_editor_paddings(opvulling);
        });
        div().size_full().child(Editor::new(&self.editor)).into_any_element()
    }
}

fn stijl(th: &Thema) -> InputEditorStyle {
    let kleur = |c: u32| -> Hsla { rgb(c).into() };
    InputEditorStyle {
        foreground: kleur(th.ink),
        muted_foreground: kleur(th.mute),
        background: kleur(th.bg),
        border: kleur(th.line),
        selection: kleur(th.line_strong),
        caret: kleur(th.ink),
        editor_gutter_background: Some(kleur(th.bg)),
        ..Default::default()
    }
}

impl Documenten {
    /// Wisselt de actieve tab tussen lezen en bewerken. Lezen toont wat in de editor staat, ook
    /// als het nog niet bewaard is.
    pub(super) fn wissel(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(doc) = self.tabs.get_mut(self.actief) else { return };
        let (id, soort) = (doc.id, doc.soort);
        match &mut doc.bewerk {
            Some(bewerk) if bewerk.aan => {
                bewerk.aan = false;
                let gelezen = uit_tekst(soort, &bewerk.editor.read(cx).value());
                window.focus(&self.focus, cx);
                self.vul(id, gelezen, cx);
            }
            Some(bewerk) => {
                bewerk.aan = true;
                bewerk.editor.update(cx, |editor, cx| editor.focus(window, cx));
            }
            None => {
                let bewerk = Bewerk::nieuw(id, soort, doc.tekst.clone(), window, cx);
                bewerk.editor.update(cx, |editor, cx| editor.focus(window, cx));
                doc.bewerk = Some(bewerk);
            }
        }
        cx.notify();
    }

    /// Bewaart de tab. Onwaar als dat niet lukte of eerst een vraag nodig is.
    pub(super) fn bewaar(&mut self, index: usize, overschrijf: bool, cx: &mut Context<Self>) -> bool {
        let Some(doc) = self.tabs.get_mut(index) else { return true };
        let Some(bewerk) = &mut doc.bewerk else { return true };
        let tekst = bewerk.editor.read(cx).value();
        if tekst == doc.tekst {
            return true;
        }
        let gelezen = uit_tekst(doc.soort, &tekst);
        if let Gelezen::Fout(fout) = &gelezen {
            bewerk.fout = Some(fout.clone().into());
            cx.notify();
            return false;
        }
        if bewerk.buiten && !overschrijf {
            self.vraag = Some(Vraag::Buiten(doc.id));
            cx.notify();
            return false;
        }
        if let Err(fout) = std::fs::write(&doc.pad, tekst.as_bytes()) {
            bewerk.fout = Some(fout.to_string().into());
            cx.notify();
            return false;
        }
        bewerk.buiten = false;
        bewerk.fout = None;
        doc.tekst = tekst;
        doc.gewijzigd = gewijzigd(&doc.pad);
        let id = doc.id;
        self.vul(id, gelezen, cx);
        true
    }

    /// Typt `tekst` in de editor van de actieve tab, voor `schermafdruk`.
    #[cfg(feature = "schermafdruk")]
    pub(super) fn typ(&mut self, tekst: &str, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(bewerk) = self.tabs.get(self.actief).and_then(|doc| doc.bewerk.as_ref()) {
            bewerk.editor.update(cx, |editor, cx| editor.insert(tekst.to_string(), window, cx));
        }
    }

    /// Mijn wijzigingen weg; de tab leest het bestand opnieuw.
    fn gooi_weg(&mut self, id: usize, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(doc) = self.tabs.iter_mut().find(|doc| doc.id == id) {
            if let Some(bewerk) = &mut doc.bewerk {
                bewerk.neem_over(doc.tekst.clone(), window, cx);
            }
        }
        self.lees(id, window, cx);
    }

    /// Heeft een tab wijzigingen die niet bewaard zijn: toon die met de vraag of ik wil bewaren.
    pub(super) fn houd_tegen(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let Some(index) = self.tabs.iter().position(|doc| doc.gewijzigd(cx)) else { return false };
        self.actief = index;
        self.vraag = Some(Vraag::Sluiten(self.tabs[index].id));
        window.activate_window();
        cx.activate(true);
        cx.notify();
        true
    }

    /// De vraag, als Dialoog uit het design system. Een klik ernaast annuleert.
    pub(super) fn dialoog(&self, th: &Thema, cx: &mut Context<Self>) -> Option<impl IntoElement> {
        let (id, buiten) = match self.vraag {
            Some(Vraag::Sluiten(id)) => (id, false),
            Some(Vraag::Buiten(id)) => (id, true),
            None => return None,
        };
        let naam = naam(&self.tabs.iter().find(|doc| doc.id == id)?.pad);
        let engels = crate::taal::engels();
        let (kop, gewoon, hoofd) = match (buiten, engels) {
            (false, false) => (format!("Wijzigingen in {naam} bewaren?"), "Niet bewaren", "Bewaren"),
            (false, true) => (format!("Save changes to {naam}?"), "Don't save", "Save"),
            (true, false) => (format!("{naam} is op schijf veranderd. Overschrijven?"), "Wijziging weggooien", "Overschrijven"),
            (true, true) => (format!("{naam} changed on disk. Overwrite?"), "Discard my changes", "Overwrite"),
        };
        let index = move |this: &Self| this.tabs.iter().position(|doc| doc.id == id);
        Some(
            deferred(
                div()
                    .id("vraag-achter")
                    .absolute()
                    .inset_0()
                    .flex()
                    .items_center()
                    .justify_center()
                    .occlude()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, _, cx| {
                            this.vraag = None;
                            cx.notify();
                        }),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .w(px(420.0))
                            .bg(rgb(th.bg))
                            .border(px(t::LIJN))
                            .border_color(rgb(th.line_strong))
                            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .h(px(t::H_TITELBALK))
                                    .px(px(t::PAD))
                                    .bg(rgb(th.bar))
                                    .font_weight(FontWeight::BOLD)
                                    .child(div().truncate().child(kop)),
                            )
                            .child(
                                div()
                                    .flex()
                                    .justify_end()
                                    .gap(px(8.0))
                                    .p(px(t::PAD))
                                    .border_t(px(t::LIJN))
                                    .border_color(rgb(th.line))
                                    .child(knop_vak(th).id("vraag-gewoon").child(gewoon).on_click(cx.listener(
                                        move |this, _, window, cx| {
                                            this.vraag = None;
                                            match (buiten, index(this)) {
                                                (true, _) => this.gooi_weg(id, window, cx),
                                                (false, Some(index)) => this.haal_weg(index, window, cx),
                                                (false, None) => {}
                                            }
                                            cx.notify();
                                        },
                                    )))
                                    .child(
                                        // De hoofdknop: vlak in `ink`, tekst in `bg`.
                                        div()
                                            .id("vraag-hoofd")
                                            .flex()
                                            .items_center()
                                            .h(px(t::H_KNOP))
                                            .px(px(8.0))
                                            .border(px(t::LIJN))
                                            .bg(rgb(th.ink))
                                            .border_color(rgb(th.ink))
                                            .text_color(rgb(th.bg))
                                            .font_weight(FontWeight::BOLD)
                                            .child(hoofd)
                                            .on_click(cx.listener(move |this, _, window, cx| {
                                                this.vraag = None;
                                                if let Some(index) = index(this) {
                                                    if this.bewaar(index, buiten, cx) && !buiten {
                                                        this.haal_weg(index, window, cx);
                                                    }
                                                }
                                                cx.notify();
                                            })),
                                    ),
                            ),
                    ),
            )
            .with_priority(3),
        )
    }
}
