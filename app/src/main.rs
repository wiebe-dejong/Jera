//! Jera: Claude Code-sessies draaien en overzien.
//!
//! Gebruik: cargo run -- [map ...]. Elke map krijgt een sessie; zonder map staat Openen open.

extern crate gpui_kit as gpui;

mod agents;
mod artifacts;
mod bestandsicoon;
mod claude;
mod bewerk;
mod documenten;
mod git;
mod gitpaneel;
mod instellingen;
mod openen;
mod oud;
mod poorten;
mod status;
mod taal;
mod terminal;
mod thema;
mod venster;

use std::{borrow::Cow, path::PathBuf};

use gpui_kit::*;

use venster::Jera;

/// De thuismap.
pub fn thuis() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_default())
}

/// Waar Jera zijn gegevens bewaart. Een debug-build is de test-Jera en heeft een eigen map.
pub fn gegevens() -> PathBuf {
    let map = if cfg!(debug_assertions) { "Jera-test" } else { "Jera" };
    thuis().join("Library/Application Support").join(map)
}

const LETTERS: [&[u8]; 5] = [
    include_bytes!("../fonts/JetBrainsMono-Regular.ttf"),
    include_bytes!("../fonts/JetBrainsMono-Italic.ttf"),
    include_bytes!("../fonts/JetBrainsMono-Bold.ttf"),
    include_bytes!("../fonts/JetBrainsMono-BoldItalic.ttf"),
    include_bytes!("../fonts/JetBrainsMono-SemiBold.ttf"),
];

fn main() {
    // Start Jera vanuit een Claude-sessie, dan erven de sessies de kenmerken van die sessie,
    // en bewaart Claude Code hun gesprek niet.
    for (naam, _) in std::env::vars() {
        if naam == "CLAUDECODE" || naam.starts_with("CLAUDE_CODE_") {
            // Veilig: er draait nog geen andere thread.
            unsafe { std::env::remove_var(naam) };
        }
    }

    let mappen: Vec<PathBuf> = std::env::args().skip(1).map(PathBuf::from).collect();

    // JERA_SVG=pad: schrijf het git-paneel van de eerste map als SVG, zonder venster
    #[cfg(feature = "schermafdruk")]
    if let Ok(uit) = std::env::var("JERA_SVG") {
        let licht = std::env::var("JERA_THEMA").as_deref() == Ok("licht");
        let s = gitpaneel::svg(&mappen[0], licht, 852.0).expect("geen git");
        std::fs::write(uit, s).expect("schrijven");
        return;
    }

    gpui_kit::application().run(move |cx| {
        gpui_kit::init(cx);
        instellingen::init(cx);
        documenten::init(cx);
        cx.text_system()
            .add_fonts(LETTERS.iter().map(|bytes| Cow::Borrowed(*bytes)).collect())
            .expect("letters laden");

        #[allow(unused_mut)]
        let mut maat = size(px(1280.0), px(800.0));
        // JERA_START=900x700 opent het venster op die maat.
        #[cfg(feature = "schermafdruk")]
        if let Some((w, h)) = std::env::var("JERA_START").ok().and_then(|m| {
            let (w, h) = m.split_once('x')?;
            Some((w.parse().ok()?, h.parse().ok()?))
        }) {
            maat = size(px(w), px(h));
        }
        let bounds = Bounds::centered(None, maat, cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: Some(TitlebarOptions {
                title: Some("Jera".into()),
                appears_transparent: true,
                traffic_light_position: Some(point(px(14.0), px(11.0))),
            }),
            ..Default::default()
        };
        let (hoofd, _) = gpui_kit::open_window(options, cx, |window, cx| {
            cx.new(|cx| {
                dock_icoon(window);
                thema::zet_kit(window, cx);
                cx.observe_window_appearance(window, |_, window, cx| {
                    dock_icoon(window);
                    thema::zet_kit(window, cx);
                    cx.notify();
                })
                .detach();
                let mut jera = Jera::new(window, cx);
                if mappen.is_empty() {
                    jera.toon_openen(false, window, cx);
                }
                for map in mappen {
                    jera.start(map, window, cx);
                }
                Jera::volg(window, cx);
                #[cfg(feature = "schermafdruk")]
                testhulp(window, cx);
                // JERA_DOC=pad|pad opent die bestanden bij de start in Documenten.
                #[cfg(feature = "schermafdruk")]
                if let Ok(paden) = std::env::var("JERA_DOC") {
                    cx.defer(move |cx| {
                        for pad in paden.split('|') {
                            documenten::open(std::path::Path::new(pad), cx);
                        }
                    });
                }
                jera
            })
        })
        .expect("venster openen");
        // Jera stopt als het hoofdvenster dichtgaat, niet als Documenten dichtgaat.
        cx.on_window_closed(move |cx, venster| {
            if venster == hoofd.window_id() {
                cx.quit();
            }
        })
        .detach();
        cx.activate(true);
    });
}

/// Het icoon in het Dock, donker of licht zoals macOS. De test-Jera heeft er `test` op.
fn dock_icoon(window: &Window) {
    use objc2::{AllocAnyThread, MainThreadMarker};
    use objc2_app_kit::{NSApplication, NSImage};
    use objc2_foundation::{NSData, NSString};
    let png: &[u8] = match window.appearance() {
        WindowAppearance::Light | WindowAppearance::VibrantLight => include_bytes!("../iconen/dock-licht.png"),
        WindowAppearance::Dark | WindowAppearance::VibrantDark => include_bytes!("../iconen/dock.png"),
    };
    let Some(mtm) = MainThreadMarker::new() else { return };
    let Some(beeld) = NSImage::initWithData(NSImage::alloc(), &NSData::with_bytes(png)) else { return };
    // Veilig: een geldig NSImage, op de main thread.
    unsafe { NSApplication::sharedApplication(mtm).setApplicationIconImage(Some(&beeld)) };
    if cfg!(debug_assertions) {
        NSApplication::sharedApplication(mtm).dockTile().setBadgeLabel(Some(&NSString::from_str("test")));
    }
}

/// JERA_SHOT=pad bewaart elke seconde het venster als afbeelding.
/// JERA_TYPE=stap|stap typt de stappen een voor een in de gekozen sessie, vanaf acht seconden.
/// In een stap staat \\r voor Enter en \\e voor Escape. Een stap met @ ervoor gaat naar
/// `Jera::test_stap`. "muis x y" zet de muis op die plek, "klik x y" klikt daar en "toets x"
/// drukt die toets in zoals het toetsenbord. "sleep x y x y" sleept van de ene plek naar de
/// andere, "drop x y pad" zet dat bestand daar neer zoals vanuit Finder, "boven x y pad" sleept het
/// daarheen zonder los te laten.
#[cfg(feature = "schermafdruk")]
fn testhulp(window: &mut Window, cx: &mut Context<Jera>) {
    use std::time::Duration;
    let shot = std::env::var("JERA_SHOT").ok();
    let typed = std::env::var("JERA_TYPE").unwrap_or_default();
    let mut steps: Vec<String> = typed
        .split('|')
        .filter(|step| !step.is_empty())
        .map(|step| step.replace("\\r", "\r").replace("\\e", "\x1b"))
        .rev()
        .collect();
    // JERA_GROOTTE=900x700|1300x800 zet het venster na het typen op die maten, 30 ms na elkaar,
    // zoals bij slepen aan de rand. JERA_GROOTTE_NA=seconden: wanneer, standaard 8.
    let mut maten: Vec<Size<Pixels>> = std::env::var("JERA_GROOTTE")
        .unwrap_or_default()
        .split('|')
        .filter_map(|maat| {
            let (w, h) = maat.split_once('x')?;
            Some(size(px(w.parse().ok()?), px(h.parse().ok()?)))
        })
        .rev()
        .collect();
    let maten_na: usize =
        std::env::var("JERA_GROOTTE_NA").ok().and_then(|na| na.parse().ok()).unwrap_or(8);
    cx.spawn_in(window, async move |this, cx| {
        for tick in 1.. {
            cx.background_executor().timer(Duration::from_millis(1000)).await;
            let step = if tick >= 8 { steps.pop() } else { None };
            if tick >= maten_na && steps.is_empty() {
                while let Some(maat) = maten.pop() {
                    let _ = this.update_in(cx, |_, window, _| window.resize(maat));
                    cx.background_executor().timer(Duration::from_millis(30)).await;
                }
            }
            // "muis x y" zet de muis op die plek in het venster, in punten.
            let step = match step.as_deref().and_then(|s| s.strip_prefix("muis ")) {
                Some(plek) => {
                    if let Some((x, y)) = plek.split_once(' ').and_then(|(x, y)| Some((x.parse().ok()?, y.parse().ok()?))) {
                        let _ = cx.update(|window, cx| {
                            let event = MouseMoveEvent { position: point(px(x), px(y)), pressed_button: None, modifiers: Modifiers::default() };
                            window.dispatch_event(PlatformInput::MouseMove(event), cx);
                        });
                    }
                    None
                }
                None => step,
            };
            let step = match step {
                Some(stap) if stap.starts_with("toets ") => {
                    if let Ok(toets) = Keystroke::parse(Box::leak(stap[6..].to_string().into_boxed_str())) {
                        let _ = cx.update(|window, cx| window.dispatch_keystroke(toets, cx));
                    }
                    None
                }
                Some(stap) if stap.starts_with("klik ") => {
                    if let Some((x, y)) = stap[5..].split_once(' ').and_then(|(x, y)| Some((x.parse().ok()?, y.parse().ok()?))) {
                        let _ = cx.update(|window, cx| {
                            let position = point(px(x), px(y));
                            let neer = MouseDownEvent { button: MouseButton::Left, position, modifiers: Modifiers::default(), click_count: 1, first_mouse: false };
                            window.dispatch_event(PlatformInput::MouseDown(neer), cx);
                            let op = MouseUpEvent { button: MouseButton::Left, position, modifiers: Modifiers::default(), click_count: 1 };
                            window.dispatch_event(PlatformInput::MouseUp(op), cx);
                        });
                    }
                    None
                }
                Some(stap) if stap.starts_with("sleep ") => {
                    let p: Vec<f32> = stap[6..].split(' ').filter_map(|n| n.parse().ok()).collect();
                    if let [x1, y1, x2, y2] = p[..] {
                        let _ = cx.update(|window, cx| {
                            let (van, naar) = (point(px(x1), px(y1)), point(px(x2), px(y2)));
                            let neer = MouseDownEvent { button: MouseButton::Left, position: van, modifiers: Modifiers::default(), click_count: 1, first_mouse: false };
                            window.dispatch_event(PlatformInput::MouseDown(neer), cx);
                            for position in [van + point(px(10.0), px(10.0)), naar] {
                                let event = MouseMoveEvent { position, pressed_button: Some(MouseButton::Left), modifiers: Modifiers::default() };
                                window.dispatch_event(PlatformInput::MouseMove(event), cx);
                            }
                            let op = MouseUpEvent { button: MouseButton::Left, position: naar, modifiers: Modifiers::default(), click_count: 1 };
                            window.dispatch_event(PlatformInput::MouseUp(op), cx);
                        });
                    }
                    None
                }
                Some(stap) if stap.starts_with("drop ") || stap.starts_with("boven ") => {
                    let los = stap.starts_with("drop ");
                    let mut delen = stap.splitn(4, ' ').skip(1);
                    if let (Some(Ok(x)), Some(Ok(y)), Some(pad)) =
                        (delen.next().map(str::parse), delen.next().map(str::parse), delen.next())
                    {
                        let _ = cx.update(|window, cx| {
                            let position = point(px(x), px(y));
                            let paths = ExternalPaths([PathBuf::from(pad)].into_iter().collect());
                            window.dispatch_event(PlatformInput::FileDrop(FileDropEvent::Entered { position, paths }), cx);
                            window.dispatch_event(PlatformInput::FileDrop(FileDropEvent::Pending { position }), cx);
                            if los {
                                window.dispatch_event(PlatformInput::FileDrop(FileDropEvent::Submit { position }), cx);
                            }
                        });
                    }
                    None
                }
                step => step,
            };
            let shot = shot.clone();
            let gedaan = this.update_in(cx, |jera, window, cx| {
                match step {
                    Some(step) if step.starts_with('@') => jera.test_stap(&step[1..], window, cx),
                    Some(step) => {
                        if let Some(terminal) = jera.gekozen_terminal() {
                            terminal.read(cx).write(step.into_bytes());
                        }
                    }
                    None => {}
                }
                if let Some(path) = shot {
                    match window.render_to_image() {
                        Ok(image) => {
                            // Eerst ernaast, zodat wie de afbeelding leest nooit een halve krijgt.
                            let tijdelijk = format!("{path}.tmp.png");
                            if image.save(&tijdelijk).is_ok() {
                                let _ = std::fs::rename(&tijdelijk, &path);
                            }
                        }
                        Err(fout) => eprintln!("schermafdruk: {fout}"),
                    }
                }
            });
            if gedaan.is_err() {
                break;
            }
        }
    })
    .detach();
}
