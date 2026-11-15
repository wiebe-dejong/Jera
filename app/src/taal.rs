//! De taal van Jera: Nederlands of Engels. Kiezen gaat in het menu, zie `instellingen`.

use std::sync::atomic::{AtomicBool, Ordering};

static ENGELS: AtomicBool = AtomicBool::new(false);

pub fn engels() -> bool {
    ENGELS.load(Ordering::Relaxed)
}

pub fn zet_engels(engels: bool) {
    ENGELS.store(engels, Ordering::Relaxed);
}

/// De tekst in de gekozen taal.
pub fn t(nl: &'static str, en: &'static str) -> &'static str {
    if engels() { en } else { nl }
}

/// Zonder keuze: de eerste taal van macOS, Nederlands als die met nl begint.
pub fn macos_engels() -> bool {
    let uit = std::process::Command::new("defaults").args(["read", "-g", "AppleLanguages"]).output();
    let talen = uit.map(|u| String::from_utf8_lossy(&u.stdout).into_owned()).unwrap_or_default();
    !talen.trim_start_matches(|c: char| c == '(' || c == '"' || c.is_whitespace()).starts_with("nl")
}

const MAANDEN: [&str; 12] = ["jan", "feb", "mrt", "apr", "mei", "jun", "jul", "aug", "sep", "okt", "nov", "dec"];
const MONTHS: [&str; 12] = ["jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec"];
const DAGEN: [&str; 7] = ["ma", "di", "wo", "do", "vr", "za", "zo"];
const DAYS: [&str; 7] = ["mon", "tue", "wed", "thu", "fri", "sat", "sun"];

/// De maand kort, 0 is januari.
pub fn maand(i: usize) -> &'static str {
    t(MAANDEN[i], MONTHS[i])
}

/// De dag van de week kort, 0 is maandag.
pub fn weekdag(i: usize) -> &'static str {
    t(DAGEN[i], DAYS[i])
}
