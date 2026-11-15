//! Bestanden in de linkerkolom: de boom met de open mappen en het menu van een twee-vingerklik.

use super::*;

/// Een map of bestand in de bestandsboom.
pub(super) struct Item {
    pub(super) pad: PathBuf,
    pub(super) map: bool,
}

/// Het menu van een twee-vingerklik op een regel in de bestandsboom.
pub(super) struct Menu {
    pad: PathBuf,
    plek: Point<Pixels>,
    /// Wat er op het klembord van macOS staat om te plakken.
    plakbaar: Vec<PathBuf>,
    /// De lijst met soorten naast nieuw staat open.
    nieuw_open: bool,
}

/// Wat ik onder nieuw kan maken, na de map: het woord en de extensie.
pub(super) const SOORTEN: [(&str, &str); 13] = [
    ("bestand", ""),
    ("markdown", ".md"),
    ("rust", ".rs"),
    ("python", ".py"),
    ("typescript", ".ts"),
    ("javascript", ".js"),
    ("html", ".html"),
    ("css", ".css"),
    ("json", ".json"),
    ("toml", ".toml"),
    ("yaml", ".yml"),
    ("shell", ".sh"),
    ("tekst", ".txt"),
];

#[derive(Clone, Copy)]
pub(super) enum Keuze {
    Nieuw(Option<&'static str>),
    Kopieer,
    Plak,
    Hernoem,
    Prullenmand,
}

/// Deze namen staan niet in de bestandsboom.
pub(super) const VERBORGEN: [&str; 2] = [".git", ".DS_Store"];

/// Wat er in `map` staat: eerst de mappen, dan de bestanden, op naam.
pub(super) fn lees_map(map: &Path) -> Vec<Item> {
    let mut items: Vec<Item> = std::fs::read_dir(map)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| !VERBORGEN.contains(&e.file_name().to_string_lossy().as_ref()))
        .map(|e| Item { map: e.path().is_dir(), pad: e.path() })
        .collect();
    items.sort_by_cached_key(|i| (!i.map, naam(&i.pad).to_lowercase()));
    items
}

/// De map waar `map` in staat, met ~ voor de thuismap.
pub(super) fn kort_pad(map: &Path) -> String {
    let ouder = map.parent().unwrap_or(map);
    let thuis = crate::thuis();
    match ouder.strip_prefix(&thuis) {
        Ok(rest) if rest.as_os_str().is_empty() => "~".to_string(),
        Ok(rest) => format!("~/{}", rest.display()),
        Err(_) => ouder.display().to_string(),
    }
}

impl Jera {
    /// Een map gaat open of dicht, een bestand opent in de app die macOS ervoor heeft.
    pub(super) fn klik_boom(&mut self, pad: &Path, cx: &mut Context<Self>) {
        let Some(tab) = self.tabs.get_mut(self.actief) else { return };
        if !pad.is_dir() {
            cx.open_with_system(pad);
            tab.bestand = Some(pad.to_path_buf());
        } else if tab.open.contains_key(pad) {
            // Dicht, met alles eronder.
            tab.open.retain(|map, _| !map.starts_with(pad));
        } else {
            tab.open.insert(pad.to_path_buf(), lees_map(pad));
        }
        cx.notify();
    }

    pub(super) fn open_menu(&mut self, pad: PathBuf, plek: Point<Pixels>, cx: &mut Context<Self>) {
        let plakbaar = cx
            .read_from_clipboard()
            .and_then(|item| {
                item.entries().iter().find_map(|entry| match entry {
                    ClipboardEntry::ExternalPaths(paden) => Some(paden.paths().to_vec()),
                    _ => None,
                })
            })
            .unwrap_or_default();
        self.menu = Some(Menu { pad, plek, plakbaar, nieuw_open: false });
        cx.notify();
    }

    pub(super) fn kies(&mut self, keuze: Keuze, window: &mut Window, cx: &mut Context<Self>) {
        let Some(Menu { pad, plakbaar, .. }) = self.menu.take() else { return };
        let Some(tab) = self.tabs.get_mut(self.actief) else { return };
        // Nieuw en plakken gaan in de map zelf, of in de map waar het bestand in staat.
        let map = if pad.is_dir() { pad.clone() } else { pad.parent().map(Path::to_path_buf).unwrap_or_default() };
        match keuze {
            Keuze::Nieuw(ext) => {
                tab.open.entry(map.clone()).or_insert_with(|| lees_map(&map));
                // De cursor staat voor de extensie.
                self.start_naam(Doel::Nieuw { map, ext }, ext.unwrap_or(""), 0..0, window, cx);
            }
            Keuze::Hernoem => {
                let naam = naam(&pad);
                let stam = match naam.rfind('.') {
                    Some(i) if i > 0 => i,
                    _ => naam.len(),
                };
                self.start_naam(Doel::Hernoem(pad), &naam, 0..stam, window, cx);
            }
            Keuze::Kopieer => cx.background_executor().spawn(async move { bewerk::kopieer(&pad) }).detach(),
            // Een grote map kopiëren duurt even; het venster wacht er niet op.
            Keuze::Plak => {
                tab.open.entry(map.clone()).or_insert_with(|| lees_map(&map));
                tab.plakken.push(map.clone());
                let root = tab.root.clone();
                cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
                    let m = map.clone();
                    cx.background_executor().spawn(async move { bewerk::plak(&plakbaar, &m) }).await;
                    let _ = this.update(cx, |this, cx| {
                        if let Some(tab) = this.tabs.iter_mut().find(|tab| tab.root == root) {
                            if let Some(i) = tab.plakken.iter().position(|p| *p == map) {
                                tab.plakken.remove(i);
                            }
                            tab.ververs();
                            cx.notify();
                        }
                    });
                })
                .detach();
            }
            Keuze::Prullenmand => {
                bewerk::prullenmand(&pad);
                if tab.bestand.as_ref().is_some_and(|b| b.starts_with(&pad)) {
                    tab.bestand = None;
                }
                tab.ververs();
            }
        }
        cx.notify();
    }

    pub(super) fn zet_nieuw_open(&mut self, open: bool, cx: &mut Context<Self>) {
        if let Some(menu) = self.menu.as_mut().filter(|m| m.nieuw_open != open) {
            menu.nieuw_open = open;
            cx.notify();
        }
    }

    /// Het menu op de plek van de klik. Onder de muis gaat naast nieuw de lijst met soorten open.
    pub(super) fn menu_vak(&self, menu: &Menu, th: &Thema, cx: &mut Context<Self>) -> impl IntoElement {
        let hoofdmap = self.tabs.get(self.actief).is_some_and(|tab| tab.root == menu.pad);
        // Een regel: icoon, woord en rechts eventueel iets in `mute`.
        let regel = |id: &'static str, icoon: AnyElement, label: &'static str, rechts: Option<AnyElement>| {
            div()
                .id(id)
                .flex()
                .items_center()
                .gap(px(t::GAP))
                .h(px(t::H_RIJ))
                .px(px(t::PAD))
                .child(icoon)
                .child(div().flex_1().child(label))
                .children(rechts)
        };
        let actie = |keuze: Keuze, icoon: &'static [u8], label: &'static str, kan: bool| {
            regel(label, actie_icoon(icoon, th), label, None)
                .on_hover(cx.listener(|this, hover: &bool, _, cx| {
                    if *hover {
                        this.zet_nieuw_open(false, cx);
                    }
                }))
                .when(kan, |d| {
                    d.hover(|d| d.bg(rgb(th.sel)))
                        .on_click(cx.listener(move |this, _, window, cx| this.kies(keuze, window, cx)))
                })
                .when(!kan, |d| d.text_color(rgb(th.mute)))
        };
        let nieuw = regel("nieuw", actie_icoon(NIEUW, th), t("nieuw", "new"), Some(pijl(PIJL_DICHT, th).into_any_element()))
            .when(menu.nieuw_open, |d| d.bg(rgb(th.sel)))
            .on_hover(cx.listener(|this, hover: &bool, _, cx| {
                if *hover {
                    this.zet_nieuw_open(true, cx);
                }
            }))
            .on_click(cx.listener(|this, _, _, cx| this.zet_nieuw_open(true, cx)));
        let scheiding = || scheiding(th);
        let vak = || menuvak(th).min_w(px(180.0));
        let soorten = menu.nieuw_open.then(|| {
            let soort = |id: &'static str, icoon: AnyElement, label: &'static str, ext: Option<&'static str>| {
                let rechts = ext.filter(|e| !e.is_empty()).map(|e| div().text_color(rgb(th.mute)).child(e).into_any_element());
                regel(id, icoon, label, rechts)
                    .hover(|d| d.bg(rgb(th.sel)))
                    .on_click(cx.listener(move |this, _, window, cx| this.kies(Keuze::Nieuw(ext), window, cx)))
            };
            vak()
                // De randen van beide vakken vallen samen.
                .ml(px(-t::LIJN))
                .child(soort("soort-map", bestandsicoon::map(false, th), t("map", "folder"), None))
                .child(scheiding())
                .children(SOORTEN.iter().map(|(label, ext)| {
                    let icoon = bestandsicoon::bestand(Path::new(&format!("naam{ext}")), th);
                    let woord = match *label {
                        "bestand" => t("bestand", "file"),
                        "tekst" => t("tekst", "text"),
                        _ => label,
                    };
                    soort(label, icoon, woord, Some(*ext))
                }))
        });
        deferred(
            anchored().position(menu.plek).snap_to_window_with_margin(px(8.0)).child(
                div()
                    .id("menu")
                    .flex()
                    .items_start()
                    .on_mouse_down_out(cx.listener(|this, _, _, cx| {
                        this.menu = None;
                        cx.notify();
                    }))
                    .child(
                        vak()
                            .child(nieuw)
                            .child(scheiding())
                            .child(actie(Keuze::Kopieer, KOPIEER, t("kopiëren", "copy"), true))
                            .child(actie(Keuze::Plak, PLAK, t("plakken", "paste"), !menu.plakbaar.is_empty()))
                            .when(!hoofdmap, |d| {
                                d.child(scheiding())
                                    .child(actie(Keuze::Hernoem, HERNOEM, t("naam wijzigen", "rename"), true))
                                    .child(actie(Keuze::Prullenmand, PRULLENMAND, t("naar prullenmand", "move to trash"), true))
                            }),
                    )
                    .children(soorten),
            ),
        )
        .with_priority(1)
    }

    /// De regels van de bestandsboom onder `map`, met de open mappen erin.
    pub(super) fn boom(
        &self,
        tab: &Tab,
        map: &Path,
        diepte: usize,
        th: &Thema,
        cx: &mut Context<Self>,
        rijen: &mut Vec<AnyElement>,
    ) {
        let Some(items) = tab.open.get(map) else { return };
        if let Some(veld) = &self.naam_veld {
            if let Doel::Nieuw { map: m, ext } = &veld.doel {
                if m == map {
                    let icoon = match ext {
                        None => bestandsicoon::map(false, th),
                        Some(ext) => bestandsicoon::bestand(Path::new(&format!("naam{ext}")), th),
                    };
                    rijen.push(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(GAP_KLEIN))
                            .h(px(t::H_RIJ))
                            .pl(px(t::PAD + t::INSPRING * diepte as f32 + t::ICOON + GAP_KLEIN))
                            .pr(px(t::PAD))
                            .child(icoon)
                            .child(self.naam_invoer(veld, th, cx))
                            .into_any_element(),
                    );
                }
            }
        }
        for item in items {
            rijen.push(self.boom_rij(tab, item, diepte, None, th, cx));
            if item.map && tab.open.contains_key(&item.pad) {
                self.boom(tab, &item.pad, diepte + 1, th, cx, rijen);
            }
        }
    }

    /// Eén regel van de bestandsboom. Bij de hoofdmap staat `waar` naast de naam.
    pub(super) fn boom_rij(
        &self,
        tab: &Tab,
        item: &Item,
        diepte: usize,
        waar: Option<String>,
        th: &Thema,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let open = item.map && tab.open.contains_key(&item.pad);
        // Met het menu open is de regel van het menu gekozen, anders het laatst geopende bestand.
        let gekozen = match &self.menu {
            Some(menu) => menu.pad == item.pad,
            None => tab.bestand.as_ref() == Some(&item.pad),
        };
        let pad = item.pad.clone();
        let mut naam = naam(&item.pad);
        if item.map {
            naam.push('/');
        }
        let pijl = match (item.map, open) {
            (false, _) => div().flex_none().size(px(t::ICOON)).into_any_element(),
            (true, false) => pijl(PIJL_DICHT, th).into_any_element(),
            (true, true) => pijl(PIJL_OPEN, th).into_any_element(),
        };
        let icoon = match item.map {
            true => bestandsicoon::map(open, th),
            false => bestandsicoon::bestand(&item.pad, th),
        };
        let hernoem = self.naam_veld.as_ref().filter(|v| matches!(&v.doel, Doel::Hernoem(p) if *p == item.pad));
        let tekst = match (hernoem, waar) {
            (Some(veld), _) => self.naam_invoer(veld, th, cx),
            (None, None) => div().flex_1().min_w_0().truncate().child(naam).into_any_element(),
            (None, Some(waar)) => div()
                .flex()
                .flex_1()
                .min_w_0()
                .gap(px(GAP_KLEIN))
                .child(div().flex_none().child(naam))
                .child(div().min_w_0().truncate().text_color(rgb(th.mute)).child(waar))
                .into_any_element(),
        };
        div()
            .id(SharedString::from(format!("boom-{}", item.pad.display())))
            .flex()
            .items_center()
            .gap(px(GAP_KLEIN))
            .h(px(t::H_RIJ))
            .pl(px(t::PAD - t::STREEP + t::INSPRING * diepte as f32))
            .when(!item.map, |d| d.pr(px(t::PAD)))
            .border_l(px(t::STREEP))
            .border_color(transparent_black())
            .hover(|d| d.bg(rgb(th.sel)))
            .when(gekozen, |d| {
                d.bg(rgb(th.sel)).font_weight(FontWeight::BOLD).border_color(rgb(th.accent))
            })
            .child(pijl)
            .child(icoon)
            .child(tekst)
            // Het boogje van bezig zolang er in deze map geplakt wordt.
            .when(tab.plakken.contains(&item.pad), |d| {
                d.child(
                    svg()
                        .data(BOOG)
                        .flex_none()
                        .size(px(t::ICOON))
                        .text_color(rgb(th.ok))
                        .with_animation(
                            SharedString::from(format!("plakt-{}", item.pad.display())),
                            Animation::new(Duration::from_millis(1100)).repeat(),
                            |s, d| s.with_transformation(Transformation::rotate(percentage(d))),
                        ),
                )
            })
            .when(item.map, |d| {
                let map = pad.clone();
                d.child(
                    div()
                        .id(SharedString::from(format!("boom-plus-{}", pad.display())))
                        .flex()
                        .flex_none()
                        .items_center()
                        .justify_center()
                        .w(px(B_PLUS))
                        .h_full()
                        .text_color(rgb(th.mute))
                        .hover(|d| d.text_color(rgb(th.ink)))
                        .child("+")
                        .on_click(cx.listener(move |this, _, window, cx| {
                            cx.stop_propagation();
                            this.start(map.clone(), window, cx);
                        })),
                )
            })
            .on_mouse_down(MouseButton::Right, {
                let pad = pad.clone();
                cx.listener(move |this, e: &MouseDownEvent, _, cx| this.open_menu(pad.clone(), e.position, cx))
            })
            .on_click(cx.listener(move |this, _, _, cx| this.klik_boom(&pad, cx)))
            .into_any_element()
    }
}
