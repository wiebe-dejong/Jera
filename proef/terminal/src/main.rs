//! Proef: draait Claude Code in een zelfgetekende terminal.
//! Kern: alacritty_terminal. Tekenen: GPUI via GPUI Kit.
//!
//! Gebruik: cargo run -- [map] [commando]
//! Met --features schermafdruk bewaart het venster zichzelf als afbeelding, zie `proef_hulp`.

use std::{
    borrow::Cow,
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use alacritty_terminal::{
    event::{Event as TermEvent, EventListener, WindowSize},
    event_loop::{EventLoop, EventLoopSender, Msg},
    grid::{Dimensions, Scroll},
    index::{Column, Line, Point as CellPoint, Side},
    selection::{Selection, SelectionType},
    sync::FairMutex,
    term::{Config, Term, TermMode, cell::Flags, color::Colors},
    tty,
    vte::ansi::{Color, CursorShape, NamedColor, Rgb},
};
use gpui_kit::*;

const FONT: &str = "Menlo";
const FONT_SIZE: f32 = 13.0;
const PADDING: f32 = 8.0;
const FOREGROUND: u32 = 0xe6e6e6;
const BACKGROUND: u32 = 0x16181d;
const SELECTION: u32 = 0x33507a;
const ANSI: [u32; 16] = [
    0x1d1f21, 0xcc6666, 0xb5bd68, 0xf0c674, 0x81a2be, 0xb294bb, 0x8abeb7, 0xc5c8c6, 0x666666,
    0xd54e53, 0xb9ca4a, 0xe7c547, 0x7aa6da, 0xc397d8, 0x70c0b1, 0xeaeaea,
];

#[derive(Clone)]
struct Proxy(flume::Sender<TermEvent>);

impl EventListener for Proxy {
    fn send_event(&self, event: TermEvent) {
        let _ = self.0.send(event);
    }
}

#[derive(Clone, Copy)]
struct Grid {
    cols: usize,
    rows: usize,
}

impl Dimensions for Grid {
    fn total_lines(&self) -> usize {
        self.rows
    }
    fn screen_lines(&self) -> usize {
        self.rows
    }
    fn columns(&self) -> usize {
        self.cols
    }
}

/// Waar het raster staat en hoe groot een cel is. Gezet bij het tekenen.
#[derive(Clone, Copy, Default)]
struct Metrics {
    origin: Point<Pixels>,
    cell_w: Pixels,
    cell_h: Pixels,
}

struct TerminalView {
    metrics: Arc<Mutex<Metrics>>,
    term: Arc<FairMutex<Term<Proxy>>>,
    pty: EventLoopSender,
    focus: FocusHandle,
    title: String,
    scroll_rest: f32,
}

impl TerminalView {
    fn new(dir: PathBuf, command: String, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let (events_tx, events_rx) = flume::unbounded();
        let proxy = Proxy(events_tx);
        let grid = Grid { cols: 100, rows: 30 };
        let term = Arc::new(FairMutex::new(Term::new(Config::default(), &grid, proxy.clone())));

        // Via de eigen shell, zodat `claude` hetzelfde doet als in de terminal.
        let options = tty::Options {
            shell: Some(tty::Shell::new("/bin/zsh".into(), vec!["-ilc".into(), command])),
            working_directory: Some(dir),
            drain_on_exit: false,
            env: HashMap::from([
                ("TERM".into(), "xterm-256color".into()),
                ("COLORTERM".into(), "truecolor".into()),
            ]),
        };
        let size = WindowSize { num_lines: 30, num_cols: 100, cell_width: 8, cell_height: 16 };
        let pty = tty::new(&options, size, 0).expect("pty openen");
        let event_loop =
            EventLoop::new(term.clone(), proxy, pty, false, false).expect("event loop starten");
        let pty = event_loop.channel();
        event_loop.spawn();

        cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            while let Ok(event) = events_rx.recv_async().await {
                if this.update(cx, |view, cx| view.on_term_event(event, cx)).is_err() {
                    break;
                }
            }
        })
        .detach();

        #[cfg(feature = "schermafdruk")]
        proef_hulp(pty.clone(), window, cx);

        let focus = cx.focus_handle();
        window.focus(&focus, cx);
        Self {
            metrics: Arc::default(),
            term,
            pty,
            focus,
            title: "Cockpit: terminalproef".into(),
            scroll_rest: 0.0,
        }
    }

    fn write(&self, bytes: Vec<u8>) {
        if !bytes.is_empty() {
            let _ = self.pty.send(Msg::Input(Cow::Owned(bytes)));
        }
    }

    fn on_term_event(&mut self, event: TermEvent, cx: &mut Context<Self>) {
        match event {
            TermEvent::Wakeup => cx.notify(),
            // Antwoorden van de terminal op vragen van het programma.
            TermEvent::PtyWrite(text) => self.write(text.into_bytes()),
            TermEvent::ColorRequest(index, format) => {
                let color = self.term.lock().colors()[index].unwrap_or_else(|| default_rgb(index));
                self.write(format(color).into_bytes());
            }
            TermEvent::Title(title) => {
                self.title = title;
                cx.notify();
            }
            TermEvent::Exit | TermEvent::ChildExit(_) => cx.quit(),
            _ => {}
        }
    }

    fn on_key(&mut self, event: &KeyDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        let key = &event.keystroke;
        let mode = *self.term.lock().mode();
        let bytes = if key.modifiers.platform {
            if key.key == "c" {
                if let Some(text) = self.term.lock().selection_to_string() {
                    cx.write_to_clipboard(ClipboardItem::new_string(text));
                }
                return;
            }
            if key.key != "v" {
                return;
            }
            let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) else {
                return;
            };
            if mode.contains(TermMode::BRACKETED_PASTE) {
                format!("\x1b[200~{text}\x1b[201~").into_bytes()
            } else {
                text.replace('\n', "\r").into_bytes()
            }
        } else {
            let Some(bytes) = key_bytes(key, mode) else {
                return;
            };
            bytes
        };
        {
            let mut term = self.term.lock();
            term.scroll_display(Scroll::Bottom);
            term.selection = None;
        }
        self.write(bytes);
        cx.stop_propagation();
    }

    /// De cel onder de muis, en of de muis links of rechts in die cel staat.
    fn cell_at(&self, position: Point<Pixels>) -> Option<(CellPoint, Side)> {
        let metrics = *self.metrics.lock().unwrap();
        if metrics.cell_w <= px(0.0) {
            return None;
        }
        let term = self.term.lock();
        let x = f32::from(position.x - metrics.origin.x) / f32::from(metrics.cell_w);
        let y = f32::from(position.y - metrics.origin.y) / f32::from(metrics.cell_h);
        let col = (x.max(0.0) as usize).min(term.columns() - 1);
        let row = (y.max(0.0) as usize).min(term.screen_lines() - 1);
        let line = Line(row as i32 - term.grid().display_offset() as i32);
        let side = if x.fract() < 0.5 { Side::Left } else { Side::Right };
        Some((CellPoint::new(line, Column(col)), side))
    }

    fn on_mouse_down(&mut self, event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        window.focus(&self.focus, cx);
        let Some((point, side)) = self.cell_at(event.position) else {
            return;
        };
        // Eén klik: per teken. Twee: per woord. Drie: per regel.
        let kind = match event.click_count {
            2 => SelectionType::Semantic,
            3 => SelectionType::Lines,
            _ => SelectionType::Simple,
        };
        self.term.lock().selection = Some(Selection::new(kind, point, side));
        cx.notify();
    }

    fn on_mouse_move(&mut self, event: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        if event.pressed_button != Some(MouseButton::Left) {
            return;
        }
        let Some((point, side)) = self.cell_at(event.position) else {
            return;
        };
        if let Some(selection) = self.term.lock().selection.as_mut() {
            selection.update(point, side);
        }
        cx.notify();
    }

    fn on_scroll(&mut self, event: &ScrollEvent, _: &mut Window, cx: &mut Context<Self>) {
        let line = px(16.0);
        self.scroll_rest += f32::from(event.delta.pixel_delta(line).y) / f32::from(line);
        let lines = self.scroll_rest.trunc();
        self.scroll_rest -= lines;
        if lines != 0.0 {
            self.term.lock().scroll_display(Scroll::Delta(lines as i32));
            cx.notify();
        }
    }
}

impl Render for TerminalView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        window.set_window_title(&self.title);
        let term = self.term.clone();
        let pty = self.pty.clone();
        let metrics = self.metrics.clone();
        div()
            .size_full()
            .bg(rgb(BACKGROUND))
            .track_focus(&self.focus)
            .on_key_down(cx.listener(Self::on_key))
            .on_scroll(cx.listener(Self::on_scroll))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_move(cx.listener(Self::on_mouse_move))
            .child(
                canvas(
                    |_, _, _| (),
                    move |bounds, _, window, cx| paint(bounds, &term, &pty, &metrics, window, cx),
                )
                .size_full(),
            )
    }
}

/// COCKPIT_SHOT=pad bewaart elke twee seconden het venster als afbeelding.
/// COCKPIT_TYPE=stap|stap typt de stappen een voor een in, vanaf acht seconden.
/// In een stap staat \\r voor Enter en \\e voor Escape.
#[cfg(feature = "schermafdruk")]
fn proef_hulp(pty: EventLoopSender, window: &mut Window, cx: &mut Context<TerminalView>) {
    use std::time::Duration;
    let shot = std::env::var("COCKPIT_SHOT").ok();
    let typed = std::env::var("COCKPIT_TYPE").unwrap_or_default();
    let mut steps: Vec<String> = typed
        .split('|')
        .filter(|step| !step.is_empty())
        .map(|step| step.replace("\\r", "\r").replace("\\e", "\x1b"))
        .rev()
        .collect();
    cx.spawn_in(window, async move |this, cx| {
        for tick in 1.. {
            cx.background_executor().timer(Duration::from_secs(2)).await;
            if tick >= 4 {
                if let Some(step) = steps.pop() {
                    let _ = pty.send(Msg::Input(Cow::Owned(step.into_bytes())));
                }
            }
            let Some(path) = shot.clone() else { continue };
            let saved = this.update_in(cx, |_, window, _| {
                if let Ok(image) = window.render_to_image() {
                    let _ = image.save(path);
                }
            });
            if saved.is_err() {
                break;
            }
        }
    })
    .detach();
}

/// Een rij tekens met dezelfde opmaak, die in één keer getekend wordt.
struct Batch {
    row: i32,
    col: usize,
    cells: usize,
    text: String,
    color: Rgba,
    bold: bool,
    italic: bool,
    underline: bool,
    wide: bool,
}

fn text_run(len: usize, color: Rgba, bold: bool, italic: bool, underline: bool) -> TextRun {
    TextRun {
        len,
        font: Font {
            weight: if bold { FontWeight::BOLD } else { FontWeight::NORMAL },
            style: if italic { FontStyle::Italic } else { FontStyle::Normal },
            ..font(FONT)
        },
        color: color.into(),
        background_color: None,
        underline: underline.then(|| UnderlineStyle {
            thickness: px(1.0),
            color: Some(color.into()),
            wavy: false,
        }),
        strikethrough: None,
    }
}

fn paint(
    bounds: Bounds<Pixels>,
    term: &FairMutex<Term<Proxy>>,
    pty: &EventLoopSender,
    metrics: &Mutex<Metrics>,
    window: &mut Window,
    cx: &mut App,
) {
    let font_size = px(FONT_SIZE);
    let padding = px(PADDING);
    let default_fg = to_rgba(default_rgb(NamedColor::Foreground as usize));

    // De maat van één cel volgt uit het lettertype.
    let sample = window.text_system().shape_line(
        "M".into(),
        font_size,
        &[text_run(1, default_fg, false, false, false)],
        None,
    );
    let cell_w = sample.width;
    let cell_h = (sample.ascent + sample.descent).ceil();

    let cols = (f32::from(bounds.size.width - padding * 2.0) / f32::from(cell_w)) as usize;
    let rows = (f32::from(bounds.size.height - padding * 2.0) / f32::from(cell_h)) as usize;
    let (cols, rows) = (cols.max(2), rows.max(1));

    *metrics.lock().unwrap() = Metrics {
        origin: point(bounds.origin.x + padding, bounds.origin.y + padding),
        cell_w,
        cell_h,
    };

    let mut term = term.lock();
    if cols != term.columns() || rows != term.screen_lines() {
        term.resize(Grid { cols, rows });
        let _ = pty.send(Msg::Resize(WindowSize {
            num_lines: rows as u16,
            num_cols: cols as u16,
            cell_width: f32::from(cell_w) as u16,
            cell_height: f32::from(cell_h) as u16,
        }));
    }

    let content = term.renderable_content();
    let offset = content.display_offset as i32;
    let cell_origin = |row: i32, col: usize| {
        point(
            bounds.origin.x + padding + cell_w * col as f32,
            bounds.origin.y + padding + cell_h * row as f32,
        )
    };

    let mut batches: Vec<Batch> = Vec::new();
    for indexed in content.display_iter {
        let cell = indexed.cell;
        let flags = cell.flags;
        if flags.intersects(Flags::WIDE_CHAR_SPACER | Flags::LEADING_WIDE_CHAR_SPACER) {
            continue;
        }
        let row = indexed.point.line.0 + offset;
        let col = indexed.point.column.0;
        let wide = flags.contains(Flags::WIDE_CHAR);
        let width = if wide { 2 } else { 1 };

        let mut fg = resolve(cell.fg, content.colors);
        let mut bg = resolve(cell.bg, content.colors);
        let inverse = flags.contains(Flags::INVERSE);
        if inverse {
            std::mem::swap(&mut fg, &mut bg);
        }
        if flags.contains(Flags::DIM) {
            fg.a = 0.6;
        }
        let selected = content.selection.is_some_and(|range| range.contains(indexed.point));
        if selected {
            bg = rgb(SELECTION);
        }
        if selected || inverse || cell.bg != Color::Named(NamedColor::Background) {
            let size = size(cell_w * width as f32, cell_h);
            window.paint_quad(fill(Bounds::new(cell_origin(row, col), size), bg));
        }
        if cell.c == ' ' || flags.contains(Flags::HIDDEN) {
            continue;
        }

        let bold = flags.contains(Flags::BOLD);
        let italic = flags.contains(Flags::ITALIC);
        let underline = flags.intersects(Flags::ALL_UNDERLINES);
        match batches.last_mut() {
            Some(last)
                if !wide
                    && !last.wide
                    && last.row == row
                    && last.col + last.cells == col
                    && last.color == fg
                    && (last.bold, last.italic, last.underline) == (bold, italic, underline) =>
            {
                last.text.push(cell.c);
                last.cells += 1;
            }
            _ => batches.push(Batch {
                row,
                col,
                cells: width,
                text: cell.c.to_string(),
                color: fg,
                bold,
                italic,
                underline,
                wide,
            }),
        }
    }

    for batch in batches {
        let run = text_run(batch.text.len(), batch.color, batch.bold, batch.italic, batch.underline);
        // Elk teken krijgt precies één cel, ook als het uit een ander lettertype komt.
        let force_width = (!batch.wide).then_some(cell_w);
        let line = window.text_system().shape_line(batch.text.into(), font_size, &[run], force_width);
        let _ = line.paint(cell_origin(batch.row, batch.col), cell_h, TextAlign::Left, None, window, cx);
    }

    if content.cursor.shape != CursorShape::Hidden {
        let origin = cell_origin(content.cursor.point.line.0 + offset, content.cursor.point.column.0);
        let color = Rgba { a: 0.5, ..default_fg };
        window.paint_quad(fill(Bounds::new(origin, size(cell_w, cell_h)), color));
    }
}

fn resolve(color: Color, colors: &Colors) -> Rgba {
    to_rgba(match color {
        Color::Spec(rgb) => rgb,
        Color::Named(name) => colors[name].unwrap_or_else(|| default_rgb(name as usize)),
        Color::Indexed(index) => {
            colors[index as usize].unwrap_or_else(|| default_rgb(index as usize))
        }
    })
}

fn to_rgba(rgb: Rgb) -> Rgba {
    Rgba { r: rgb.r as f32 / 255.0, g: rgb.g as f32 / 255.0, b: rgb.b as f32 / 255.0, a: 1.0 }
}

/// De kleur bij een index als het programma die niet zelf heeft gezet.
/// 0-15 ANSI, 16-231 kleurkubus, 232-255 grijs, daarna de benoemde kleuren.
fn default_rgb(index: usize) -> Rgb {
    let hex = match index {
        0..=15 => ANSI[index],
        16..=231 => {
            let i = (index - 16) as u32;
            let level = |v: u32| if v == 0 { 0 } else { 55 + 40 * v };
            level(i / 36) << 16 | level(i / 6 % 6) << 8 | level(i % 6)
        }
        232..=255 => {
            let v = 8 + 10 * (index - 232) as u32;
            v << 16 | v << 8 | v
        }
        257 => BACKGROUND,
        259..=266 => ANSI[index - 259],
        _ => FOREGROUND,
    };
    Rgb { r: (hex >> 16) as u8, g: (hex >> 8) as u8, b: hex as u8 }
}

/// Zet een toets om naar de bytes die een terminal ervoor stuurt.
fn key_bytes(key: &Keystroke, mode: TermMode) -> Option<Vec<u8>> {
    let m = key.modifiers;
    let cursor = |letter: char| {
        let prefix = if mode.contains(TermMode::APP_CURSOR) { "\x1bO" } else { "\x1b[" };
        format!("{prefix}{letter}")
    };
    let text: String = match key.key.as_str() {
        "enter" if m.shift || m.alt => "\x1b\r".into(),
        "enter" => "\r".into(),
        "tab" if m.shift => "\x1b[Z".into(),
        "tab" => "\t".into(),
        "backspace" if m.alt => "\x1b\x7f".into(),
        "backspace" => "\x7f".into(),
        "escape" => "\x1b".into(),
        "left" if m.alt => "\x1bb".into(),
        "right" if m.alt => "\x1bf".into(),
        "up" => cursor('A'),
        "down" => cursor('B'),
        "right" => cursor('C'),
        "left" => cursor('D'),
        "home" => cursor('H'),
        "end" => cursor('F'),
        "pageup" => "\x1b[5~".into(),
        "pagedown" => "\x1b[6~".into(),
        "delete" => "\x1b[3~".into(),
        name if m.control && name.len() == 1 => {
            let byte = name.as_bytes()[0].to_ascii_lowercase();
            let code = match byte {
                b'a'..=b'z' => byte & 0x1f,
                b'[' => 0x1b,
                b'\\' => 0x1c,
                b']' => 0x1d,
                _ => return None,
            };
            (code as char).to_string()
        }
        "space" if m.control => "\0".into(),
        name if m.alt && name.len() == 1 => format!("\x1b{name}"),
        _ => key.key_char.clone()?,
    };
    Some(text.into_bytes())
}

fn main() {
    let mut args = std::env::args().skip(1);
    let dir = args.next().map(PathBuf::from).unwrap_or_else(|| std::env::current_dir().unwrap());
    let command = args.next().unwrap_or_else(|| "claude".into());

    gpui_kit::application().run(move |cx| {
        gpui_kit::init(cx);
        let bounds = Bounds::centered(None, size(px(1000.0), px(680.0)), cx);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            ..Default::default()
        };
        gpui_kit::open_window(options, cx, |window, cx| {
            cx.new(|cx| TerminalView::new(dir, command, window, cx))
        })
        .expect("venster openen");
        cx.on_window_closed(|cx, _| cx.quit()).detach();
        cx.activate(true);
    });
}
