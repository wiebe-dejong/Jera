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
    /// De lijst naast nieuw of naast pad kopiëren staat open.
    sub: Option<Sub>,
}

#[derive(Clone, Copy, PartialEq)]
pub(super) enum Sub {
    Nieuw,
    Pad,
}

/// Een bestand of map die ik uit de bestandsboom sleep.
#[derive(Clone)]
pub(super) struct Gesleept(pub(super) PathBuf);

impl Render for Gesleept {
    fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let th = t::thema(window);
        let icoon = match self.0.is_dir() {
            true => bestandsicoon::map(false, th),
            false => bestandsicoon::bestand(&self.0, th),
        };
        div()
            .flex()
            .items_center()
            .gap(px(GAP_KLEIN))
            .h(px(t::H_RIJ))
            .px(px(t::PAD))
            .bg(rgb(th.bg))
            .border(px(t::LIJN))
            .border_color(rgb(th.line_strong))
            .text_color(rgb(th.ink))
            .font_family(t::LETTER)
            .text_size(px(t::TEKST))
            .child(icoon)
            .child(naam(&self.0))
    }
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
    /// Het pad als tekst: vanaf de hoofdmap van het project, of helemaal.
    Pad { vanaf_root: bool },
    Plak,
    Hernoem,
    Prullenmand,
    /// In de app die macOS ervoor heeft, in plaats van in Documenten.
    InApp,
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
    /// Een map gaat open of dicht. Een bestand opent in Documenten als Jera het zelf toont,
    /// anders in de app die macOS ervoor heeft.
    pub(super) fn klik_boom(&mut self, pad: &Path, cx: &mut Context<Self>) {
        let Some(tab) = self.tabs.get_mut(self.actief) else { return };
        if !pad.is_dir() {
            let p = pad.to_path_buf();
            // Na deze klik: Documenten opent een venster, en dat kan niet midden in het bijwerken van dit.
            cx.defer(move |cx| {
                if !crate::documenten::open(&p, cx) {
                    cx.open_with_system(&p);
                }
            });
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
        self.menu = Some(Menu { pad, plek, plakbaar, sub: None });
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
            Keuze::Pad { vanaf_root } => {
                let tekst = match vanaf_root {
                    true => bewerk::vanaf(&pad, &tab.root),
                    false => pad.display().to_string(),
                };
                cx.write_to_clipboard(ClipboardItem::new_string(tekst));
            }
            Keuze::Plak => self.plak_in(plakbaar, map, false, cx),
            Keuze::InApp => cx.open_with_system(&pad),
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

    /// Kopieert `bronnen` naar `map` in de actieve tab, en zet de mappen tot `map` open.
    /// Met `pad_kopieren` (uit Finder) is het eerste nieuwe bestand daarna gekozen en staan de
    /// nieuwe paden vanaf de hoofdmap op het klembord. Een grote map kopiëren duurt even; het
    /// venster wacht er niet op.
    pub(super) fn plak_in(&mut self, bronnen: Vec<PathBuf>, map: PathBuf, pad_kopieren: bool, cx: &mut Context<Self>) {
        let Some(tab) = self.tabs.get_mut(self.actief) else { return };
        for m in map.ancestors().take_while(|m| m.starts_with(&tab.root)) {
            tab.open.entry(m.to_path_buf()).or_insert_with(|| lees_map(m));
        }
        tab.plakken.push(map.clone());
        let root = tab.root.clone();
        cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let m = map.clone();
            let nieuw = cx.background_executor().spawn(async move { bewerk::plak(&bronnen, &m) }).await;
            let _ = this.update(cx, |this, cx| {
                if pad_kopieren && !nieuw.is_empty() {
                    let paden: Vec<String> = nieuw.iter().map(|p| bewerk::vanaf(p, &root)).collect();
                    cx.write_to_clipboard(ClipboardItem::new_string(paden.join(" ")));
                }
                if let Some(tab) = this.tabs.iter_mut().find(|tab| tab.root == root) {
                    if let Some(i) = tab.plakken.iter().position(|p| *p == map) {
                        tab.plakken.remove(i);
                    }
                    if pad_kopieren && !nieuw.is_empty() {
                        tab.bestand = nieuw.first().cloned();
                    }
                    tab.ververs();
                    cx.notify();
                }
            });
        })
        .detach();
    }

    /// Een regel uit de bestandsboom is neergezet op `map`. Op zijn eigen regel is het een klik.
    pub(super) fn zet_neer(&mut self, bron: &Path, doel: &Path, map: PathBuf, cx: &mut Context<Self>) {
        if bron == doel {
            return self.klik_boom(bron, cx);
        }
        let Some(tab) = self.tabs.get_mut(self.actief) else { return };
        let Some(nieuw) = bewerk::verplaats(bron, &map) else { return };
        if let Some(rest) = tab.bestand.as_ref().and_then(|b| b.strip_prefix(bron).ok()) {
            tab.bestand = Some(nieuw.join(rest));
        }
        tab.open.entry(map.clone()).or_insert_with(|| lees_map(&map));
        tab.ververs();
        cx.notify();
    }

    pub(super) fn zet_sub(&mut self, sub: Option<Sub>, cx: &mut Context<Self>) {
        if let Some(menu) = self.menu.as_mut().filter(|m| m.sub != sub) {
            menu.sub = sub;
            cx.notify();
        }
    }

    /// Het menu op de plek van de klik. Onder de muis gaat naast nieuw de lijst met soorten open,
    /// naast pad kopiëren het volledige pad en het pad vanaf de hoofdmap.
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
                        this.zet_sub(None, cx);
                    }
                }))
                .when(kan, |d| {
                    d.hover(|d| d.bg(rgb(th.sel)))
                        .on_click(cx.listener(move |this, _, window, cx| this.kies(keuze, window, cx)))
                })
                .when(!kan, |d| d.text_color(rgb(th.mute)))
        };
        // Een regel met een lijst ernaast, die onder de muis opengaat.
        let met_lijst = |id: &'static str, icoon: &'static [u8], label: &'static str, sub: Sub| {
            regel(id, actie_icoon(icoon, th), label, Some(pijl(PIJL_DICHT, th).into_any_element()))
                .when(menu.sub == Some(sub), |d| d.bg(rgb(th.sel)))
                .on_hover(cx.listener(move |this, hover: &bool, _, cx| {
                    if *hover {
                        this.zet_sub(Some(sub), cx);
                    }
                }))
                .on_click(cx.listener(move |this, _, _, cx| this.zet_sub(Some(sub), cx)))
        };
        let scheiding = || scheiding(th);
        let vak = || menuvak(th).min_w(px(180.0));
        let root = self.tabs.get(self.actief).map(|tab| tab.root.clone()).unwrap_or_default();
        let paden = (menu.sub == Some(Sub::Pad)).then(|| {
            // Afgekapt aan het begin: het eind van een pad zegt het meest.
            let waarde = |tekst: String| {
                div().max_w(px(240.0)).overflow_hidden().whitespace_nowrap().text_ellipsis_start().text_color(rgb(th.mute)).child(tekst).into_any_element()
            };
            let pad = |id: &'static str, label: &'static str, vanaf_root: bool, tekst: String| {
                regel(id, actie_icoon(KOPIEER, th), label, Some(waarde(tekst)))
                    .gap(px(16.0))
                    .hover(|d| d.bg(rgb(th.sel)))
                    .on_click(cx.listener(move |this, _, window, cx| this.kies(Keuze::Pad { vanaf_root }, window, cx)))
            };
            vak()
                .ml(px(-t::LIJN))
                // Naast de regel pad kopiëren, niet bovenaan het menu.
                .mt(px(t::H_RIJ * 3.0 + 9.0))
                .child(pad("pad-root", t("vanaf projectmap", "from project root"), true, bewerk::vanaf(&menu.pad, &root)))
                .child(pad("pad-volledig", t("volledig pad", "full path"), false, menu.pad.display().to_string()))
        });
        let soorten = (menu.sub == Some(Sub::Nieuw)).then(|| {
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
                            .when(crate::documenten::kan(&menu.pad), |d| {
                                let icoon = bestandsicoon::bestand(&menu.pad, th);
                                d.child(
                                    regel("in-app", icoon, t("in app openen", "open in app"), None)
                                        .on_hover(cx.listener(|this, hover: &bool, _, cx| {
                                            if *hover {
                                                this.zet_sub(None, cx);
                                            }
                                        }))
                                        .hover(|d| d.bg(rgb(th.sel)))
                                        .on_click(cx.listener(|this, _, window, cx| this.kies(Keuze::InApp, window, cx))),
                                )
                                .child(scheiding())
                            })
                            .child(met_lijst("nieuw", NIEUW, t("nieuw", "new"), Sub::Nieuw))
                            .child(scheiding())
                            .child(actie(Keuze::Kopieer, KOPIEER, t("kopiëren", "copy"), true))
                            .child(actie(Keuze::Plak, PLAK, t("plakken", "paste"), !menu.plakbaar.is_empty()))
                            .child(met_lijst("pad", KOPIEER, t("pad kopiëren", "copy path"), Sub::Pad))
                            .when(!hoofdmap, |d| {
                                d.child(scheiding())
                                    .child(actie(Keuze::Hernoem, HERNOEM, t("naam wijzigen", "rename"), true))
                                    .child(actie(Keuze::Prullenmand, PRULLENMAND, t("naar prullenmand", "move to trash"), true))
                            }),
                    )
                    .children(soorten)
                    .children(paden),
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
        let map = match item.map {
            true => pad.clone(),
            false => pad.parent().map(Path::to_path_buf).unwrap_or_default(),
        };
        let (sel, accent) = (th.sel, th.accent);
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
            // Slepen verplaatst binnen het project; buiten het venster is het een gewoon bestand,
            // voor Finder of een andere app. De hoofdmap blijft staan.
            .when(diepte > 0, |d| {
                d.on_drag(Gesleept(pad.clone()), |gesleept, _, _, cx| cx.new(|_| gesleept.clone()))
                    .external_drag_payload::<Gesleept>(|gesleept, _, _| {
                        Some(ExternalDragPayload::Files(FileDragPaths::new([(gesleept.0.clone(), gesleept.0.is_dir())])))
                    })
            })
            // Neerzetten gaat in de map zelf, of in de map waar het bestand in staat.
            .drag_over::<Gesleept>(move |d, _, _, _| d.bg(rgb(sel)).border_color(rgb(accent)))
            .drag_over::<ExternalPaths>(move |d, _, _, _| d.bg(rgb(sel)).border_color(rgb(accent)))
            .on_drop::<Gesleept>({
                let (pad, map) = (pad.clone(), map.clone());
                cx.listener(move |this, gesleept: &Gesleept, _, cx| this.zet_neer(&gesleept.0, &pad, map.clone(), cx))
            })
            .on_drop::<ExternalPaths>({
                let map = map.clone();
                cx.listener(move |this, paden: &ExternalPaths, _, cx| this.plak_in(paden.paths().to_vec(), map.clone(), true, cx))
            })
            .on_click(cx.listener(move |this, _, _, cx| this.klik_boom(&pad, cx)))
            .into_any_element()
    }
}
