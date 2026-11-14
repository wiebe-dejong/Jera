//! Cockpit: Claude Code-sessies draaien en overzien.
//!
//! Gebruik: cargo run -- [map ...]. Elke map krijgt een sessie; zonder map de huidige.

mod terminal;
mod thema;
mod venster;

use std::{borrow::Cow, path::PathBuf};

use gpui_kit::*;

use venster::Cockpit;

const LETTERS: [&[u8]; 5] = [
    include_bytes!("../fonts/JetBrainsMono-Regular.ttf"),
    include_bytes!("../fonts/JetBrainsMono-Italic.ttf"),
    include_bytes!("../fonts/JetBrainsMono-Bold.ttf"),
    include_bytes!("../fonts/JetBrainsMono-BoldItalic.ttf"),
    include_bytes!("../fonts/JetBrainsMono-SemiBold.ttf"),
];

fn main() {
    // Start Cockpit vanuit een Claude-sessie, dan erven de sessies de kenmerken van die sessie,
    // en bewaart Claude Code hun gesprek niet.
    for (naam, _) in std::env::vars() {
        if naam == "CLAUDECODE" || naam.starts_with("CLAUDE_CODE_") {
            // Veilig: er draait nog geen andere thread.
            unsafe { std::env::remove_var(naam) };
        }
    }

    let mut mappen: Vec<PathBuf> = std::env::args().skip(1).map(PathBuf::from).collect();
    if mappen.is_empty() {
        mappen.push(std::env::current_dir().expect("huidige map"));
    }

    gpui_kit::application().run(move |cx| {
        gpui_kit::init(cx);
        cx.text_system()
            .add_fonts(LETTERS.iter().map(|bytes| Cow::Borrowed(*bytes)).collect())
            .expect("letters laden");

        #[allow(unused_mut)]
        let mut maat = size(px(1280.0), px(800.0));
        // COCKPIT_START=900x700 opent het venster op die maat.
        #[cfg(feature = "schermafdruk")]
        if let Some((w, h)) = std::env::var("COCKPIT_START").ok().and_then(|m| {
            let (w, h) = m.split_once('x')?;
            Some((w.parse().ok()?, h.parse().ok()?))
        }) {
            maat = size(px(w), px(h));
        }
        let bounds = Bounds::centered(None, maat, cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            titlebar: Some(TitlebarOptions {
                title: Some("Cockpit".into()),
                appears_transparent: true,
                traffic_light_position: Some(point(px(14.0), px(11.0))),
            }),
            ..Default::default()
        };
        gpui_kit::open_window(options, cx, |window, cx| {
            cx.new(|cx| {
                cx.observe_window_appearance(window, |_, _, cx| cx.notify()).detach();
                let mut cockpit = Cockpit::new();
                for map in mappen {
                    cockpit.start(map, window, cx);
                }
                #[cfg(feature = "schermafdruk")]
                testhulp(window, cx);
                cockpit
            })
        })
        .expect("venster openen");
        cx.on_window_closed(|cx, _| cx.quit()).detach();
        cx.activate(true);
    });
}

/// COCKPIT_SHOT=pad bewaart elke seconde het venster als afbeelding.
/// COCKPIT_TYPE=stap|stap typt de stappen een voor een in de gekozen sessie, vanaf acht seconden.
/// In een stap staat \\r voor Enter en \\e voor Escape.
#[cfg(feature = "schermafdruk")]
fn testhulp(window: &mut Window, cx: &mut Context<Cockpit>) {
    use std::time::Duration;
    let shot = std::env::var("COCKPIT_SHOT").ok();
    let typed = std::env::var("COCKPIT_TYPE").unwrap_or_default();
    let mut steps: Vec<String> = typed
        .split('|')
        .filter(|step| !step.is_empty())
        .map(|step| step.replace("\\r", "\r").replace("\\e", "\x1b"))
        .rev()
        .collect();
    // COCKPIT_GROOTTE=900x700|1300x800 zet het venster na het typen op die maten, 30 ms na elkaar,
    // zoals bij slepen aan de rand. COCKPIT_GROOTTE_NA=seconden: wanneer, standaard 8.
    let mut maten: Vec<Size<Pixels>> = std::env::var("COCKPIT_GROOTTE")
        .unwrap_or_default()
        .split('|')
        .filter_map(|maat| {
            let (w, h) = maat.split_once('x')?;
            Some(size(px(w.parse().ok()?), px(h.parse().ok()?)))
        })
        .rev()
        .collect();
    let maten_na: usize =
        std::env::var("COCKPIT_GROOTTE_NA").ok().and_then(|na| na.parse().ok()).unwrap_or(8);
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
            let shot = shot.clone();
            let gedaan = this.update_in(cx, |cockpit, window, cx| {
                if let (Some(step), Some(terminal)) = (step, cockpit.gekozen_terminal()) {
                    terminal.read(cx).write(step.into_bytes());
                }
                if let Some(path) = shot {
                    if let Ok(image) = window.render_to_image() {
                        let _ = image.save(path);
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
