//! De kleuren en maten uit `stijl/design-system/tokens.json`.

use gpui_kit::*;

pub const LETTER: &str = "JetBrains Mono";
pub const TEKST: f32 = 12.5;
pub const REGEL: f32 = 1.3;
pub const TERMINAL_REGEL: f32 = 1.55;

pub const PAD: f32 = 12.0;
pub const TAB_PAD: f32 = 14.0;
pub const GAP: f32 = 9.0;

pub const H_RIJ: f32 = 26.0;
pub const H_TITELBALK: f32 = 34.0;
pub const H_SCHAKELAAR: f32 = 30.0;
pub const H_RENDER: f32 = 26.0;
pub const B_KOLOM: f32 = 290.0;
pub const B_GIT: f32 = 346.0;

pub const LIJN: f32 = 1.0;
pub const STREEP: f32 = 2.0;

/// Ruimte links in de titelbalk voor de drie knoppen van macOS.
pub const B_LICHTEN: f32 = 80.0;

pub struct Thema {
    pub bg: u32,
    pub bar: u32,
    pub line: u32,
    pub line_strong: u32,
    pub ink: u32,
    pub mute: u32,
    pub sel: u32,
    pub accent: u32,
    /// De 16 ANSI-kleuren van de terminal. Geen token: Claude Code zet zelf zijn kleuren.
    pub ansi: [u32; 16],
}

pub const DONKER: Thema = Thema {
    bg: 0x0e1013,
    bar: 0x14171b,
    line: 0x23272e,
    line_strong: 0x333944,
    ink: 0xd5d9e0,
    mute: 0x8a93a2,
    sel: 0x1c2129,
    accent: 0x5fd7a7,
    ansi: [
        0x1d1f21, 0xff6b6b, 0x4fcb7c, 0xf2cc60, 0x6e9eff, 0xc08be0, 0x5fd7a7, 0xd5d9e0, 0x5a616b,
        0xff8f8f, 0x6fdb95, 0xf5d77f, 0x8db4ff, 0xd0a6ea, 0x86e3bd, 0xf2f4f7,
    ],
};

pub const LICHT: Thema = Thema {
    bg: 0xf6f6f2,
    bar: 0xecece6,
    line: 0xd9d9d0,
    line_strong: 0xc2c2b8,
    ink: 0x1e2227,
    mute: 0x5a616b,
    sel: 0xe2e2da,
    accent: 0x0a6b4b,
    ansi: [
        0x1e2227, 0xc9302c, 0x1a7f3c, 0x8a5a00, 0x2f6feb, 0x8e44ad, 0x0a6b4b, 0xd9d9d0, 0x5a616b,
        0xe0443f, 0x228a47, 0xad8200, 0x4a84f0, 0xa35bc0, 0x0f805a, 0xf6f6f2,
    ],
};

/// Het thema volgt macOS.
pub fn thema(window: &Window) -> &'static Thema {
    #[cfg(feature = "schermafdruk")]
    match std::env::var("COCKPIT_THEMA").as_deref() {
        Ok("licht") => return &LICHT,
        Ok("donker") => return &DONKER,
        _ => {}
    }
    match window.appearance() {
        WindowAppearance::Light | WindowAppearance::VibrantLight => &LICHT,
        WindowAppearance::Dark | WindowAppearance::VibrantDark => &DONKER,
    }
}
