//! De sessies van een tab in de linkerkolom, met hun agents, het menu van een sessie en het blok oud.

use super::*;
use crate::taal;

/// De agents onder een sessieregel: het team onder een kopje dat dichtklapt, met elk teamlid,
/// en daaronder een kopje met alleen het aantal subagents. Een klik op een teamlid kiest de sessie,
/// die op plek `plek` van de tab staat.
pub(super) fn agent_rijen(sessie: &Sessie, plek: usize, th: &Thema, cx: &mut Context<Jera>) -> Vec<AnyElement> {
    let id = sessie.id;
    let inspring = px(t::PAD + t::ICOON + t::GAP);
    let kopje = |label: &'static str| {
        div()
            .flex()
            .items_center()
            .gap(px(GAP_KLEIN))
            .h(px(20.0))
            .pl(inspring)
            .pr(px(t::PAD))
            .text_size(px(t::KLEIN))
            .text_color(rgb(th.mute))
            .child(div().flex_1().child(label))
    };
    let mut rijen = Vec::new();
    let team: Vec<(usize, &Agent)> = sessie.agents.iter().enumerate().filter(|(_, a)| a.kleur.is_some()).collect();
    if !team.is_empty() {
        let dicht = sessie.team_dicht;
        rijen.push(
            kopje("team")
                .id(("team", id))
                .hover(|d| d.bg(rgb(th.sel)))
                .child(pijl(if dicht { PIJL_DICHT } else { PIJL_OPEN }, th))
                .child(team.len().to_string())
                .on_click(cx.listener(move |this, _, _, cx| this.wissel_team(id, cx)))
                .into_any_element(),
        );
        if !dicht {
            for (index, agent) in team {
                rijen.push(
                    div()
                        .id(SharedString::from(format!("teamlid-{id}-{index}")))
                        .flex()
                        .items_center()
                        .gap(px(t::GAP))
                        .h(px(t::H_RIJ))
                        .pl(inspring)
                        .pr(px(t::PAD))
                        .hover(|d| d.bg(rgb(th.sel)))
                        .child(agent_icoon(agent.stand, SharedString::from(format!("draai-{id}-{index}")), th))
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .truncate()
                                .text_color(rgb(th.team[agent.kleur.unwrap_or(0)]))
                                .child(agent.tekst.clone()),
                        )
                        .on_click(cx.listener(move |this, _, window, cx| this.kies_sessie(plek, window, cx)))
                        .into_any_element(),
                );
            }
        }
    }
    let subagents = sessie.agents.iter().filter(|a| a.kleur.is_none()).count();
    if subagents > 0 {
        rijen.push(kopje("subagents").child(subagents.to_string()).into_any_element());
    }
    rijen
}

/// Het statusicoon van een agent: bezig, klaar of gestopt.
/// Dezelfde vormen als bij een sessie: het boogje, het vinkje en het streepje.
pub(super) fn agent_icoon(stand: agents::Stand, id: SharedString, th: &Thema) -> AnyElement {
    let status = match stand {
        agents::Stand::Bezig => Status::Bezig,
        agents::Stand::Klaar => Status::NetKlaar,
        agents::Stand::Gestopt => Status::Idle,
    };
    icoon(status, id, th)
}

/// De naam van een groep: de map ten opzichte van de hoofdmap, of de naam van de hoofdmap.
pub(super) fn map_label(map: &Path, root: &Path) -> String {
    match map.strip_prefix(root) {
        Ok(rel) if !rel.as_os_str().is_empty() => format!("{}/", rel.display()),
        _ => format!("{}/", naam(map)),
    }
}

/// Zoveel regels van het blok oud zijn tegelijk te zien, onder de kop.
pub(super) const OUD_ZICHTBAAR: usize = 8;

/// De dag van `tijd`: vandaag, gisteren of bijvoorbeeld 9 nov.
pub(super) fn dag(tijd: std::time::SystemTime) -> String {
    use chrono::Datelike;
    let datum = chrono::DateTime::<chrono::Local>::from(tijd).date_naive();
    let vandaag = chrono::Local::now().date_naive();
    match (vandaag - datum).num_days() {
        0 => t("vandaag", "today").into(),
        1 => t("gisteren", "yesterday").into(),
        _ if datum.year() == vandaag.year() => format!("{} {}", datum.day(), taal::maand(datum.month0() as usize)),
        _ => format!("{} {} {}", datum.day(), taal::maand(datum.month0() as usize), datum.year()),
    }
}

/// De naam die ik gaf, of de titel die Claude Code zet, zonder het tekentje ervoor.
/// Zolang die "Claude Code" is, de naam van de map.
pub(super) fn titel(sessie: &Sessie, cx: &App) -> String {
    if let Some(naam) = &sessie.naam {
        return naam.clone();
    }
    let titel = sessie.terminal.read(cx).title.trim_start_matches(|c: char| !c.is_alphanumeric());
    if titel.is_empty() || titel == "Claude Code" { naam(&sessie.map) } else { titel.to_string() }
}

/// Hoe lang iets loopt: `40 s`, `12 min`, `1 u 12 min`.
pub(super) fn duur(tijd: Duration) -> String {
    let minuten = tijd.as_secs() / 60;
    match minuten {
        0 => format!("{} s", tijd.as_secs()),
        1..60 => format!("{minuten} min"),
        _ => format!("{} {} {} min", minuten / 60, t("u", "h"), minuten % 60),
    }
}

impl Jera {
    /// Leest de oude sessies van de tab met `root` opnieuw, op de achtergrond.
    pub(super) fn laad_oud(&mut self, root: PathBuf, cx: &mut Context<Self>) {
        cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let r = root.clone();
            let oud = cx.background_executor().spawn(async move { oud::lees(&r) }).await;
            let _ = this.update(cx, |this, cx| {
                if let Some(tab) = this.tabs.iter_mut().find(|tab| tab.root == root) {
                    tab.oud = oud;
                    cx.notify();
                }
            });
        })
        .detach();
    }

    /// Klapt het blok oud open of dicht. Open leest het de oude sessies opnieuw.
    pub(super) fn wissel_oud(&mut self, cx: &mut Context<Self>) {
        let Some(tab) = self.tabs.get_mut(self.actief) else { return };
        tab.oud_open = !tab.oud_open;
        if tab.oud_open {
            let root = tab.root.clone();
            self.laad_oud(root, cx);
        }
        cx.notify();
    }

    /// Klapt een dag in het blok oud open of dicht.
    pub(super) fn wissel_dag(&mut self, dag: String, cx: &mut Context<Self>) {
        let Some(tab) = self.tabs.get_mut(self.actief) else { return };
        if !tab.oud_dicht.remove(&dag) {
            tab.oud_dicht.insert(dag);
        }
        cx.notify();
    }

    /// Zet een oude sessie voort in een nieuwe terminal.
    pub(super) fn hervat(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(tab) = self.tabs.get_mut(self.actief) else { return };
        if index >= tab.oud.len() {
            return;
        }
        let oud = tab.oud.remove(index);
        self.start_met(oud.map, &format!("{STARTCOMMANDO} --resume {}", oud.id), window, cx);
    }

    /// De sessies van de tab, per map: de hoofdmap eerst, dan de submappen.
    pub(super) fn sessielijst(&self, tab: &Tab, th: &Thema, cx: &mut Context<Self>) -> impl IntoElement {
        let mut mappen: Vec<&Path> = tab.sessies.iter().map(|s| s.map.as_path()).collect();
        mappen.sort_by_key(|map| (*map != tab.root, map.to_path_buf()));
        mappen.dedup();

        let groepen = mappen.into_iter().map(|map| {
            let label = map_label(map, &tab.root);
            let map_buf = map.to_path_buf();
            let kop = div()
                .flex()
                .items_center()
                .h(px(t::H_RIJ + 2.0))
                .mt(px(8.0))
                .pl(px(t::PAD))
                .text_color(rgb(th.mute))
                .child(div().flex_1().truncate().child(label))
                .child(
                    div()
                        .id(SharedString::from(format!("plus-{}", map.display())))
                        .flex()
                        .items_center()
                        .justify_center()
                        .w(px(B_PLUS))
                        .h_full()
                        .hover(|d| d.text_color(rgb(th.ink)))
                        .child("+")
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.start(map_buf.clone(), window, cx)
                        })),
                );
            let rijen = tab.sessies.iter().enumerate().filter(|(_, s)| s.map == map).map(
                |(index, sessie)| {
                    let gekozen = index == tab.gekozen;
                    let open = sessie.agents_open.unwrap_or(gekozen);
                    let id = sessie.id;
                    let regel = div()
                        .id(("sessie", sessie.id))
                        .flex()
                        .items_center()
                        .gap(px(t::GAP))
                        .h(px(t::H_RIJ))
                        .pl(px(t::PAD - t::STREEP))
                        .pr(px(t::PAD))
                        .border_l(px(t::STREEP))
                        .border_color(transparent_black())
                        .when(sessie.status == Status::Idle, |d| d.text_color(rgb(th.mute)))
                        .when(gekozen, |d| {
                            d.bg(rgb(th.sel)).font_weight(FontWeight::BOLD).border_color(rgb(th.accent))
                        })
                        .when(!gekozen, |d| d.hover(|d| d.bg(rgb(th.sel))))
                        // Zolang het menu van deze sessie open is, heeft de regel het vlak `sel`.
                        .when(self.sessie_menu.is_some_and(|(m, _)| m == id), |d| d.bg(rgb(th.sel)))
                        .child({
                            let woord = sessie.status.woord();
                            div()
                                .id(("status", sessie.id))
                                .flex_none()
                                .child(icoon(sessie.status, ("draai", sessie.id), th))
                                .tooltip(move |_, cx| cx.new(|_| Tip(woord)).into())
                        })
                        .child(match self.naam_veld.as_ref().filter(|v| matches!(v.doel, Doel::Sessie(s) if s == id)) {
                            Some(veld) => self.naam_invoer(veld, th, cx),
                            None => div().flex_1().min_w_0().truncate().child(titel(sessie, cx)).into_any_element(),
                        })
                        .when(!sessie.agents.is_empty(), |d| {
                            let bezig = sessie.agents.iter().filter(|a| a.stand == agents::Stand::Bezig).count();
                            d.child(
                                div()
                                    .id(("agents", id))
                                    .flex()
                                    .flex_none()
                                    .child(
                                        pijl(if open { PIJL_OPEN } else { PIJL_DICHT }, th)
                                            .hover(|d| d.text_color(rgb(th.ink))),
                                    )
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        cx.stop_propagation();
                                        this.wissel_agents(id, open, cx);
                                    })),
                            )
                            .when(!open, |d| {
                                d.child(
                                    div()
                                        .flex_none()
                                        .font_weight(FontWeight::NORMAL)
                                        .text_color(rgb(th.mute))
                                        .child(format!("{bezig}/{}", sessie.agents.len())),
                                )
                            })
                        })
                        .on_mouse_down(
                            MouseButton::Right,
                            cx.listener(move |this, e: &MouseDownEvent, _, cx| {
                                this.sessie_menu = Some((id, e.position));
                                cx.notify();
                            }),
                        )
                        .on_click(cx.listener(move |this, _, window, cx| this.kies_sessie(index, window, cx)));
                    div().child(regel).when(open, |d| d.children(agent_rijen(sessie, index, th, cx)))
                },
            );
            div().child(kop).children(rijen)
        });
        div().children(groepen)
    }

    /// Kiest sessie `index` van de tab.
    pub(super) fn kies_sessie(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(tab) = self.tabs.get_mut(self.actief) {
            tab.gekozen = index;
        }
        self.focus(window, cx);
        cx.notify();
    }

    /// Klapt de agents van sessie `id` open of dicht.
    pub(super) fn wissel_agents(&mut self, id: usize, open: bool, cx: &mut Context<Self>) {
        if let Some(sessie) = self.tabs.iter_mut().flat_map(|tab| tab.sessies.iter_mut()).find(|s| s.id == id) {
            sessie.agents_open = Some(!open);
        }
        cx.notify();
    }

    /// Klapt het team van sessie `id` open of dicht.
    pub(super) fn wissel_team(&mut self, id: usize, cx: &mut Context<Self>) {
        if let Some(sessie) = self.tabs.iter_mut().flat_map(|tab| tab.sessies.iter_mut()).find(|s| s.id == id) {
            sessie.team_dicht = !sessie.team_dicht;
        }
        cx.notify();
    }

    /// Onderaan de kolom: het blok oud, standaard dicht. Open schuift de kop omhoog en
    /// toont eronder de oude sessies per dag van het laatste bericht, de nieuwste eerst.
    /// Een dag klapt dicht; een klik op een sessie zet die voort.
    /// Er zijn `OUD_ZICHTBAAR` regels te zien; de rest scrolt binnen het blok.
    pub(super) fn oud_blok(&self, tab: &Tab, th: &Thema, cx: &mut Context<Self>) -> impl IntoElement {
        let kop = div()
            .id("oud")
            .flex()
            .items_center()
            .gap(px(GAP_KLEIN))
            .flex_none()
            .h(px(t::H_RIJ))
            .pl(px(t::PAD))
            .text_color(rgb(th.mute))
            .hover(|d| d.bg(rgb(th.sel)))
            .child(pijl(if tab.oud_open { PIJL_OPEN } else { PIJL_DICHT }, th))
            .child(t("oud", "old"))
            .child(div().ml(px(2.0)).child(tab.oud.len().to_string()))
            .on_click(cx.listener(|this, _, _, cx| this.wissel_oud(cx)));
        let mut lijst = div().id("oud-lijst").overflow_y_scroll();
        let mut regels = 0;
        if tab.oud_open {
            let mut vorige = None;
            for (index, oud) in tab.oud.iter().enumerate() {
                let dag = dag(oud.tijd);
                let dicht = tab.oud_dicht.contains(&dag);
                if vorige.as_ref() != Some(&dag) {
                    regels += 1;
                    let d = dag.clone();
                    lijst = lijst.child(
                        div()
                            .id(SharedString::from(format!("dag-{dag}")))
                            .flex()
                            .items_center()
                            .h(px(t::H_RIJ))
                            .pl(px(t::PAD))
                            .text_size(px(t::KLEIN))
                            .text_color(rgb(th.mute))
                            .hover(|d| d.bg(rgb(th.sel)))
                            .child(div().flex_1().child(dag.clone()))
                            // Rechts, onder de + van de mappen in de sessielijst.
                            .child(
                                div()
                                    .flex()
                                    .justify_center()
                                    .w(px(B_PLUS))
                                    .child(pijl(if dicht { PIJL_DICHT } else { PIJL_OPEN }, th)),
                            )
                            .on_click(cx.listener(move |this, _, _, cx| this.wissel_dag(d.clone(), cx))),
                    );
                    vorige = Some(dag);
                }
                if dicht {
                    continue;
                }
                regels += 1;
                lijst = lijst.child(
                    div()
                        .id(("hervat", index))
                        .flex()
                        .items_center()
                        .h(px(t::H_RIJ))
                        .px(px(t::PAD))
                        .hover(|d| d.bg(rgb(th.sel)))
                        .child(div().flex_1().min_w_0().truncate().child(oud.titel.clone()))
                        .on_click(cx.listener(move |this, _, window, cx| this.hervat(index, window, cx))),
                );
            }
        }
        div()
            .flex_none()
            .border_t(px(t::LIJN))
            .border_color(rgb(th.line))
            .child(kop)
            .when(regels > 0, |d| d.child(lijst.h(px(t::H_RIJ * regels.min(OUD_ZICHTBAAR) as f32))))
    }

    /// Het menu van een twee-vingerklik op een sessie: hoe lang hij loopt, dan naam wijzigen en stoppen.
    pub(super) fn sessie_menu_vak(&self, id: usize, plek: Point<Pixels>, th: &Thema, cx: &mut Context<Self>) -> Option<impl IntoElement> {
        let sessie = self.tabs.iter().flat_map(|tab| &tab.sessies).find(|s| s.id == id)?;
        let regel = |label: &'static str| div().id(label).flex().items_center().gap(px(t::GAP)).h(px(t::H_RIJ)).px(px(t::PAD));
        let info = |label: &'static str, waarde: String| {
            regel(label).child(div().flex_1().text_color(rgb(th.mute)).child(label)).child(waarde)
        };
        let titel = titel(sessie, cx);
        Some(
            deferred(
                anchored().position(plek).snap_to_window_with_margin(px(8.0)).child(
                    menuvak(th)
                        .id("sessiemenu")
                        .min_w(px(200.0))
                        .on_mouse_down_out(cx.listener(|this, _, _, cx| {
                            this.sessie_menu = None;
                            cx.notify();
                        }))
                        .child(info(t("looptijd", "runtime"), duur(sessie.gestart.elapsed())))
                        .child(scheiding(th))
                        .child(
                            regel("naam wijzigen")
                                .hover(|d| d.bg(rgb(th.sel)))
                                .child(actie_icoon(HERNOEM, th))
                                .child(t("naam wijzigen", "rename"))
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    this.sessie_menu = None;
                                    let lengte = titel.len();
                                    this.start_naam(Doel::Sessie(id), &titel, 0..lengte, window, cx);
                                })),
                        )
                        .child(
                            regel("stoppen")
                                .hover(|d| d.bg(rgb(th.sel)))
                                .text_color(rgb(th.bad))
                                .child(actie_icoon(STOP, th))
                                .child(t("stoppen", "stop"))
                                .on_click(cx.listener(move |this, _, window, cx| this.stop_sessie(id, window, cx))),
                        ),
                ),
            )
            .with_priority(1),
        )
    }
}
