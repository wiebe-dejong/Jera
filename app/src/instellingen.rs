//! De instellingen, in de menubalk onder Jera. Ze staan in
//! `~/Library/Application Support/Jera/instellingen.json`.

use std::{
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
};

use gpui_kit::*;

use crate::openen::{NieuwProject, OpenProject};

actions!(jera, [IconenKleur, IconenZed, Stop]);

/// De iconen in de bestandsboom: van Zed (de standaard) of in kleur (GitLab en Catppuccin).
static ZED: AtomicBool = AtomicBool::new(true);

pub fn zed() -> bool {
    ZED.load(Ordering::Relaxed)
}

fn bestand() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_default())
        .join("Library/Application Support/Jera/instellingen.json")
}

fn bewaar() {
    let json = serde_json::json!({ "iconen": if zed() { "zed" } else { "kleur" } });
    let pad = bestand();
    if let Some(map) = pad.parent() {
        let _ = std::fs::create_dir_all(map);
    }
    let _ = std::fs::write(pad, json.to_string());
}

/// Leest de instellingen en zet het menu.
pub fn init(cx: &mut App) {
    if let Ok(tekst) = std::fs::read_to_string(bestand()) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&tekst) {
            ZED.store(json["iconen"] != "kleur", Ordering::Relaxed);
        }
    }
    cx.on_action(|_: &IconenKleur, cx| kies_iconen(false, cx));
    cx.on_action(|_: &IconenZed, cx| kies_iconen(true, cx));
    cx.on_action(|_: &Stop, cx| cx.quit());
    cx.bind_keys([
        KeyBinding::new("cmd-q", Stop, None),
        KeyBinding::new("cmd-o", OpenProject, None),
        KeyBinding::new("cmd-n", NieuwProject, None),
    ]);
    menu(cx);
}

fn kies_iconen(zed: bool, cx: &mut App) {
    ZED.store(zed, Ordering::Relaxed);
    bewaar();
    menu(cx);
    cx.refresh_windows();
}

fn keuze(naam: &str, action: impl Action, checked: bool) -> MenuItem {
    MenuItem::Action { name: naam.to_string().into(), action: Box::new(action), os_action: None, checked, disabled: false }
}

fn menu(cx: &mut App) {
    cx.set_menus([Menu::new("Jera").items([
        MenuItem::action("Openen…", OpenProject),
        MenuItem::action("Nieuw project", NieuwProject),
        MenuItem::separator(),
        MenuItem::submenu(Menu::new("Iconen").items([
            keuze("Kleur", IconenKleur, !zed()),
            keuze("Zed", IconenZed, zed()),
        ])),
        MenuItem::separator(),
        MenuItem::action("Stop Jera", Stop),
    ])]);
}
