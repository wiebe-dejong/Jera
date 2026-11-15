//! De instellingen, in de menubalk onder Jera. Ze staan in
//! `~/Library/Application Support/Jera/instellingen.json`.

use std::{
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
};

use gpui_kit::*;

use crate::openen::{NieuwProject, OpenProject};
use crate::taal::{self, t};

actions!(jera, [IconenKleur, IconenZed, TaalNederlands, TaalEngels, Stop]);

/// De iconen in de bestandsboom: van Zed (de standaard) of in kleur (GitLab en Catppuccin).
static ZED: AtomicBool = AtomicBool::new(true);

pub fn zed() -> bool {
    ZED.load(Ordering::Relaxed)
}

fn bestand() -> PathBuf {
    crate::thuis().join("Library/Application Support/Jera/instellingen.json")
}

fn lees() -> serde_json::Value {
    std::fs::read_to_string(bestand()).ok().and_then(|tekst| serde_json::from_str(&tekst).ok()).unwrap_or_default()
}

/// Zet `sleutel` op `waarde`; de andere instellingen blijven staan.
fn bewaar(sleutel: &str, waarde: &str) {
    let mut json = lees();
    json[sleutel] = waarde.into();
    let pad = bestand();
    if let Some(map) = pad.parent() {
        let _ = std::fs::create_dir_all(map);
    }
    let _ = std::fs::write(pad, json.to_string());
}

/// Leest de instellingen en zet het menu.
pub fn init(cx: &mut App) {
    let json = lees();
    ZED.store(json["iconen"] != "kleur", Ordering::Relaxed);
    taal::zet_engels(match json["taal"].as_str() {
        Some("nl") => false,
        Some("en") => true,
        _ => taal::macos_engels(),
    });
    cx.on_action(|_: &IconenKleur, cx| kies_iconen(false, cx));
    cx.on_action(|_: &IconenZed, cx| kies_iconen(true, cx));
    cx.on_action(|_: &TaalNederlands, cx| kies_taal(false, cx));
    cx.on_action(|_: &TaalEngels, cx| kies_taal(true, cx));
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
    bewaar("iconen", if zed { "zed" } else { "kleur" });
    menu(cx);
    cx.refresh_windows();
}

fn kies_taal(engels: bool, cx: &mut App) {
    taal::zet_engels(engels);
    bewaar("taal", if engels { "en" } else { "nl" });
    menu(cx);
    cx.refresh_windows();
}

fn keuze(naam: &str, action: impl Action, checked: bool) -> MenuItem {
    MenuItem::Action { name: naam.to_string().into(), action: Box::new(action), os_action: None, checked, disabled: false }
}

fn menu(cx: &mut App) {
    cx.set_menus([Menu::new("Jera").items([
        MenuItem::action(t("Openen…", "Open…"), OpenProject),
        MenuItem::action(t("Nieuw project", "New project"), NieuwProject),
        MenuItem::separator(),
        MenuItem::submenu(Menu::new(t("Iconen", "Icons")).items([
            keuze(t("Kleur", "Color"), IconenKleur, !zed()),
            keuze(t("Zwartwit", "Monochrome"), IconenZed, zed()),
        ])),
        MenuItem::submenu(Menu::new(t("Taal", "Language")).items([
            keuze("Nederlands", TaalNederlands, !taal::engels()),
            keuze("English", TaalEngels, taal::engels()),
        ])),
        MenuItem::separator(),
        MenuItem::action(t("Stop Jera", "Quit Jera"), Stop),
    ])]);
}
