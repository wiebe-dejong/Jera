//! Het veld waarin ik een naam typ: een nieuwe naam voor een bestand of sessie, of die van iets nieuws.

use super::*;

/// Een naam die ik in de bestandsboom typ: een nieuwe naam, of die van iets nieuws.
pub(super) struct NaamVeld {
    pub(super) doel: Doel,
    pub(super) invoer: Entity<InputState>,
    fout: Option<&'static str>,
    _volg: Subscription,
}

pub(super) enum Doel {
    Hernoem(PathBuf),
    /// De naam van de sessie met dit id.
    Sessie(usize),
    /// Een map, of een bestand met deze extensie.
    Nieuw { map: PathBuf, ext: Option<&'static str> },
}

impl Jera {
    /// Opent het veld voor een naam, met `naam` erin en `kies` daarvan gekozen.
    pub(super) fn start_naam(
        &mut self,
        doel: Doel,
        naam: &str,
        kies: std::ops::Range<usize>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let invoer = cx.new(|cx| {
            let mut invoer = InputState::new(window, cx);
            invoer.set_value(naam.to_string(), window, cx);
            invoer.set_selected_range(kies, cx);
            invoer
        });
        invoer.update(cx, |invoer, cx| invoer.focus(window, cx));
        let volg = cx.subscribe_in(&invoer, window, |this, _, event, window, cx| match event {
            InputEvent::PressEnter { .. } => this.bevestig_naam(false, window, cx),
            InputEvent::Blur => this.bevestig_naam(true, window, cx),
            _ => {}
        });
        self.naam_veld = Some(NaamVeld { doel, invoer, fout: None, _volg: volg });
        cx.notify();
    }

    /// Hernoemt of maakt wat in het veld staat. Bij een fout blijft het veld open, of gaat
    /// het dicht als ik ergens anders heb geklikt (`weg`).
    pub(super) fn bevestig_naam(&mut self, weg: bool, window: &mut Window, cx: &mut Context<Self>) {
        let Some(veld) = self.naam_veld.as_mut() else { return };
        let naam = veld.invoer.read(cx).value().trim().to_string();
        let uitkomst = match &veld.doel {
            // Leeg zet de titel van Claude terug.
            Doel::Sessie(id) => {
                let id = *id;
                if let Some(sessie) = self.tabs.iter_mut().flat_map(|tab| &mut tab.sessies).find(|s| s.id == id) {
                    sessie.naam = (!naam.is_empty()).then_some(naam);
                }
                self.stop_naam(window, cx);
                return;
            }
            Doel::Nieuw { ext, .. } if naam == ext.unwrap_or("") => Err(t("geen naam", "no name")),
            Doel::Hernoem(pad) => bewerk::hernoem(pad, &naam).map(|nieuw| (pad.clone(), nieuw)),
            Doel::Nieuw { map, ext } => bewerk::maak(map, &naam, ext.is_none()).map(|nieuw| (nieuw.clone(), nieuw)),
        };
        match uitkomst {
            Err(fout) if !weg => {
                veld.fout = Some(fout);
                cx.notify();
                return;
            }
            Err(_) => {}
            Ok((oud, nieuw)) => {
                if let Some(tab) = self.tabs.get_mut(self.actief) {
                    // Open mappen en het gekozen bestand verhuizen mee.
                    let verhuis = |p: PathBuf| match p.strip_prefix(&oud) {
                        Ok(rest) if rest.as_os_str().is_empty() => nieuw.clone(),
                        Ok(rest) => nieuw.join(rest),
                        Err(_) => p,
                    };
                    tab.open = tab.open.drain().map(|(map, items)| (verhuis(map), items)).collect();
                    tab.bestand = tab.bestand.take().map(verhuis);
                    tab.ververs();
                }
            }
        }
        self.stop_naam(window, cx);
    }

    pub(super) fn stop_naam(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.naam_veld.take().is_some() {
            self.focus(window, cx);
        }
        cx.notify();
    }

    /// Het veld waarin ik een naam typ, met rechts wat er mis is.
    pub(super) fn naam_invoer(&self, veld: &NaamVeld, th: &Thema, cx: &mut Context<Self>) -> AnyElement {
        div()
            .flex()
            .flex_1()
            .min_w_0()
            .items_center()
            .gap(px(GAP_KLEIN))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .px(px(4.0))
                    .border(px(t::LIJN))
                    .border_color(rgb(th.line_strong))
                    .child(Input::new(&veld.invoer)),
            )
            .children(veld.fout.map(|fout| div().flex_none().text_color(rgb(th.mute)).child(fout)))
            // Een klik in het veld is geen klik op de regel.
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .capture_key_down(cx.listener(|this, e: &KeyDownEvent, window, cx| {
                if e.keystroke.key == "escape" {
                    cx.stop_propagation();
                    this.stop_naam(window, cx);
                }
            }))
            .into_any_element()
    }
}
