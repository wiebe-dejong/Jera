//! Het venster: tabs in de titelbalk, de linkerkolom, de Render-regel, de terminal en het git-paneel.

use std::path::{Path, PathBuf};

use gpui_kit::{prelude::FluentBuilder, *};

use crate::terminal::{TerminalEvent, TerminalView};
use crate::thema::{self as t, Thema};

/// Het commando waarmee elke sessie start, via de eigen shell.
pub const STARTCOMMANDO: &str = "claude";

#[derive(Clone, Copy, PartialEq)]
enum Stand {
    Sessies,
    Bestanden,
    Eerder,
}

struct Sessie {
    id: usize,
    map: PathBuf,
    terminal: Entity<TerminalView>,
}

/// Eén tab per git-repo. Een map zonder git is zijn eigen tab.
struct Tab {
    root: PathBuf,
    sessies: Vec<Sessie>,
    gekozen: usize,
}

pub struct Cockpit {
    tabs: Vec<Tab>,
    actief: usize,
    stand: Stand,
    git_open: bool,
    volgende_id: usize,
}

impl Cockpit {
    pub fn new() -> Self {
        Self { tabs: Vec::new(), actief: 0, stand: Stand::Sessies, git_open: true, volgende_id: 0 }
    }

    /// Start een sessie in `map`, in de tab van zijn repo.
    pub fn start(&mut self, map: PathBuf, window: &mut Window, cx: &mut Context<Self>) {
        let root = repo_root(&map);
        let terminal = cx.new(|cx| TerminalView::new(map.clone(), STARTCOMMANDO, cx));
        let id = self.volgende_id;
        self.volgende_id += 1;
        cx.subscribe(&terminal, move |this, _, event, cx| match event {
            TerminalEvent::Titel => cx.notify(),
            TerminalEvent::Gestopt => this.verwijder(id, cx),
        })
        .detach();

        let index = match self.tabs.iter().position(|tab| tab.root == root) {
            Some(index) => index,
            None => {
                self.tabs.push(Tab { root, sessies: Vec::new(), gekozen: 0 });
                self.tabs.len() - 1
            }
        };
        let tab = &mut self.tabs[index];
        tab.sessies.push(Sessie { id, map, terminal });
        tab.gekozen = tab.sessies.len() - 1;
        self.actief = index;
        self.stand = Stand::Sessies;
        self.focus(window, cx);
        cx.notify();
    }

    pub fn gekozen_terminal(&self) -> Option<&Entity<TerminalView>> {
        let tab = self.tabs.get(self.actief)?;
        Some(&tab.sessies.get(tab.gekozen)?.terminal)
    }

    fn focus(&self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(terminal) = self.gekozen_terminal() {
            let focus = terminal.read(cx).focus.clone();
            window.focus(&focus, cx);
        }
    }

    fn verwijder(&mut self, id: usize, cx: &mut Context<Self>) {
        for index in 0..self.tabs.len() {
            let tab = &mut self.tabs[index];
            let Some(pos) = tab.sessies.iter().position(|s| s.id == id) else { continue };
            tab.sessies.remove(pos);
            if tab.gekozen > pos || tab.gekozen >= tab.sessies.len() {
                tab.gekozen = tab.gekozen.saturating_sub(1);
            }
            if tab.sessies.is_empty() {
                self.tabs.remove(index);
                if self.actief >= index {
                    self.actief = self.actief.saturating_sub(1);
                }
            }
            break;
        }
        cx.notify();
    }

    fn kies_map(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let paden = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: None,
        });
        cx.spawn_in(window, async move |this, cx| {
            let Ok(Ok(Some(paden))) = paden.await else { return };
            let Some(map) = paden.into_iter().next() else { return };
            let _ = this.update_in(cx, |this, window, cx| this.start(map, window, cx));
        })
        .detach();
    }

    fn titelbalk(&self, th: &Thema, cx: &mut Context<Self>) -> impl IntoElement {
        let tabs = self.tabs.iter().enumerate().map(|(index, tab)| {
            let actief = index == self.actief;
            tab_vak(th, actief)
                .id(("tab", index))
                .child(naam(&tab.root))
                .child(div().font_weight(FontWeight::NORMAL).text_color(rgb(th.mute)).child(tab.sessies.len().to_string()))
                .on_click(cx.listener(move |this, _, window, cx| {
                    this.actief = index;
                    this.focus(window, cx);
                    cx.notify();
                }))
        });
        div()
            .flex()
            .flex_none()
            .h(px(t::H_TITELBALK))
            .pl(px(t::B_LICHTEN))
            .bg(rgb(th.bar))
            .border_b(px(t::LIJN))
            .border_color(rgb(th.line))
            .children(tabs.flat_map(|tab| [lijn(th).into_any_element(), tab.into_any_element()]))
            .child(lijn(th))
            .child(
                tab_vak(th, false)
                    .id("map")
                    .child("+ map")
                    .on_click(cx.listener(|this, _, window, cx| this.kies_map(window, cx))),
            )
            .child(lijn(th))
    }

    fn kolom(&self, th: &Thema, cx: &mut Context<Self>) -> impl IntoElement {
        let knop = |stand: Stand, label: &'static str| {
            let actief = self.stand == stand;
            div()
                .id(label)
                .flex()
                .items_center()
                .pt(px(t::STREEP))
                .border_b(px(t::STREEP))
                .border_color(transparent_black())
                .text_color(rgb(th.mute))
                .when(actief, |d| {
                    d.text_color(rgb(th.ink)).font_weight(FontWeight::BOLD).border_color(rgb(th.accent))
                })
                .child(label)
                .on_click(cx.listener(move |this, _, _, cx| {
                    this.stand = stand;
                    cx.notify();
                }))
        };
        let schakelaar = div()
            .flex()
            .flex_none()
            .gap(px(18.0))
            .h(px(t::H_SCHAKELAAR))
            .px(px(t::PAD))
            .border_b(px(t::LIJN))
            .border_color(rgb(th.line))
            .child(knop(Stand::Sessies, "sessies"))
            .child(knop(Stand::Bestanden, "bestanden"))
            .child(knop(Stand::Eerder, "eerder"));

        let inhoud = match (self.stand, self.tabs.get(self.actief)) {
            (Stand::Sessies, Some(tab)) => self.sessielijst(tab, th, cx).into_any_element(),
            (Stand::Sessies, None) => lege_staat(th, cx).into_any_element(),
            _ => div().into_any_element(),
        };
        div()
            .flex()
            .flex_col()
            .flex_none()
            .w(px(t::B_KOLOM))
            .border_r(px(t::LIJN))
            .border_color(rgb(th.line))
            .child(schakelaar)
            .child(div().id("kolom").flex_1().overflow_y_scroll().child(inhoud))
    }

    /// De sessies van de tab, per map: de hoofdmap eerst, dan de submappen.
    fn sessielijst(&self, tab: &Tab, th: &Thema, cx: &mut Context<Self>) -> impl IntoElement {
        let mut mappen: Vec<&Path> = tab.sessies.iter().map(|s| s.map.as_path()).collect();
        mappen.sort_by_key(|map| (*map != tab.root, map.to_path_buf()));
        mappen.dedup();

        let groepen = mappen.into_iter().map(|map| {
            let label = match map.strip_prefix(&tab.root) {
                Ok(rel) if !rel.as_os_str().is_empty() => format!("{}/", rel.display()),
                _ => format!("{}/", naam(map)),
            };
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
                        .w(px(32.0))
                        .h_full()
                        .child("+")
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.start(map_buf.clone(), window, cx)
                        })),
                );
            let rijen = tab.sessies.iter().enumerate().filter(|(_, s)| s.map == map).map(
                |(index, sessie)| {
                    let gekozen = index == tab.gekozen;
                    div()
                        .id(("sessie", sessie.id))
                        .flex()
                        .items_center()
                        .gap(px(t::GAP))
                        .h(px(t::H_RIJ))
                        .pl(px(t::PAD - t::STREEP))
                        .pr(px(t::PAD))
                        .border_l(px(t::STREEP))
                        .border_color(transparent_black())
                        .when(gekozen, |d| {
                            d.bg(rgb(th.sel)).font_weight(FontWeight::BOLD).border_color(rgb(th.accent))
                        })
                        .child(div().flex_1().min_w_0().truncate().child(titel(sessie, cx)))
                        .on_click(cx.listener(move |this, _, window, cx| {
                            if let Some(tab) = this.tabs.get_mut(this.actief) {
                                tab.gekozen = index;
                            }
                            this.focus(window, cx);
                            cx.notify();
                        }))
                },
            );
            div().child(kop).children(rijen)
        });
        div().children(groepen)
    }

    fn render_regel(&self, th: &Thema, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_none()
            .h(px(t::H_RENDER))
            .border_b(px(t::LIJN))
            .border_color(rgb(th.line))
            .child(div().flex_1().min_w_0())
            .child(lijn(th))
            .child(
                div()
                    .id("git")
                    .flex()
                    .flex_none()
                    .items_center()
                    .px(px(t::TAB_PAD))
                    .pt(px(t::STREEP))
                    .border_b(px(t::STREEP))
                    .border_color(transparent_black())
                    .text_color(rgb(th.mute))
                    .when(self.git_open, |d| d.text_color(rgb(th.ink)).border_color(rgb(th.accent)))
                    .child("git")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.git_open = !this.git_open;
                        cx.notify();
                    })),
            )
    }
}

impl Render for Cockpit {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let th = t::thema(window);
        if let Some(tab) = self.tabs.get(self.actief) {
            window.set_window_title(&naam(&tab.root));
        }
        let terminal = self.gekozen_terminal().cloned();
        div()
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
            .child(
                div()
                    .flex()
                    .flex_1()
                    .min_h_0()
                    .child(self.kolom(th, cx))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .flex_1()
                            .min_w_0()
                            .child(self.render_regel(th, cx))
                            .child(
                                div()
                                    .flex()
                                    .flex_1()
                                    .min_h_0()
                                    .child(div().flex_1().min_w_0().children(terminal))
                                    .when(self.git_open, |d| {
                                        d.child(
                                            div()
                                                .flex_none()
                                                .w(px(t::B_GIT))
                                                .border_l(px(t::LIJN))
                                                .border_color(rgb(th.line)),
                                        )
                                    }),
                            ),
                    ),
            )
    }
}

/// Een vak in de titelbalk: vlak met een streep erboven als het actief is.
fn tab_vak(th: &Thema, actief: bool) -> Div {
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
}

/// Een staande lijn tussen twee vakken.
fn lijn(th: &Thema) -> Div {
    div().flex_none().w(px(t::LIJN)).h_full().bg(rgb(th.line))
}

fn lege_staat(th: &Thema, cx: &mut Context<Cockpit>) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .items_start()
        .gap(px(t::PAD))
        .p(px(t::PAD))
        .text_color(rgb(th.mute))
        .child("Nog geen sessie.")
        .child(
            div()
                .id("leeg-map")
                .flex()
                .items_center()
                .h(px(20.0))
                .px(px(8.0))
                .border(px(t::LIJN))
                .border_color(rgb(th.line_strong))
                .text_color(rgb(th.ink))
                .child("+ map")
                .on_click(cx.listener(|this, _, window, cx| this.kies_map(window, cx))),
        )
}

/// De titel die Claude Code zet, zonder het tekentje ervoor.
/// Zolang die "Claude Code" is, de naam van de map.
fn titel(sessie: &Sessie, cx: &App) -> String {
    let titel = sessie.terminal.read(cx).title.trim_start_matches(|c: char| !c.is_alphanumeric());
    if titel.is_empty() || titel == "Claude Code" { naam(&sessie.map) } else { titel.to_string() }
}

fn naam(map: &Path) -> String {
    map.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_else(|| map.display().to_string())
}

/// De hoofdmap van de repo van `map`. Een worktree valt onder zijn repo.
fn repo_root(map: &Path) -> PathBuf {
    let uit = std::process::Command::new("git")
        .args(["rev-parse", "--path-format=absolute", "--git-common-dir"])
        .current_dir(map)
        .output();
    match uit {
        Ok(uit) if uit.status.success() => {
            let git_dir = PathBuf::from(String::from_utf8_lossy(&uit.stdout).trim());
            match git_dir.file_name() {
                Some(n) if n == ".git" => git_dir.parent().map(Path::to_path_buf).unwrap_or(git_dir),
                _ => git_dir,
            }
        }
        _ => map.to_path_buf(),
    }
}
