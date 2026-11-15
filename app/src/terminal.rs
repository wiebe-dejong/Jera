//! Eén terminal: `alacritty_terminal` als kern, zelf getekend in GPUI.
//! Met de kleuren en maten van het design system.

use std::{
    borrow::Cow,
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
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
    vte::{
        self,
        ansi::{Color, CursorShape, NamedColor, Processor, Rgb},
    },
};
use gpui_kit::*;

use crate::thema::{self, Thema};

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
    /// De eerste maat is gezet.
    gemeten: bool,
    /// Een nieuwe maat (kolommen, regels) en sinds wanneer die gelijk blijft.
    wacht: Option<(usize, usize, Instant)>,
}

/// Zo lang moet een nieuwe maat gelijk blijven voordat het programma hem krijgt.
/// Bij elke nieuwe maat tekent Claude Code alles opnieuw; tijdens slepen zou dat
/// tientallen keren gebeuren, en de tussenversies blijven dan in beeld staan.
const RUST_NA_SLEPEN: Duration = Duration::from_millis(150);

/// Zo lang staat de cursor aan, en zo lang uit.
const KNIPPER: Duration = Duration::from_millis(500);

pub enum TerminalEvent {
    /// Het programma heeft een andere titel gezet.
    Titel,
    /// Het programma in de terminal is gestopt.
    Gestopt,
}

pub struct TerminalView {
    metrics: Arc<Mutex<Metrics>>,
    term: Arc<FairMutex<Term<Proxy>>>,
    /// Leeg als de terminal niet is gestart.
    pty: Option<EventLoopSender>,
    pub focus: FocusHandle,
    /// De titel die het programma zet, leeg tot het dat doet.
    pub title: String,
    /// Het pid van de shell die het programma start. Leeg als de terminal niet is gestart.
    pub pid: Option<u32>,
    scroll_rest: f32,
    /// Het thema van de laatste keer tekenen, voor antwoorden op kleurvragen.
    thema: &'static Thema,
    /// De terminal heeft de focus in een actief venster: de cursor knippert.
    knippert: bool,
    /// De cursor is nu te zien.
    cursor_aan: bool,
    _knipper: Task<()>,
}

impl EventEmitter<TerminalEvent> for TerminalView {}

impl TerminalView {
    pub fn new(dir: PathBuf, command: &str, cx: &mut Context<Self>) -> Self {
        let (events_tx, events_rx) = flume::unbounded();
        let proxy = Proxy(events_tx);
        let grid = Grid { cols: 100, rows: 30 };
        let term = Arc::new(FairMutex::new(Term::new(Config::default(), &grid, proxy.clone())));

        // Via de eigen shell, zodat `claude` hetzelfde doet als in de terminal.
        let options = tty::Options {
            shell: Some(tty::Shell::new("/bin/zsh".into(), vec!["-ilc".into(), command.into()])),
            working_directory: Some(dir),
            drain_on_exit: false,
            env: HashMap::from([
                ("TERM".into(), "xterm-256color".into()),
                ("COLORTERM".into(), "truecolor".into()),
            ]),
        };
        let size = WindowSize { num_lines: 30, num_cols: 100, cell_width: 8, cell_height: 16 };
        // Start de terminal niet, dan staat de fout in deze terminal; de andere sessies draaien door.
        let (pty, pid) = match start(&options, size, term.clone(), proxy) {
            Ok((pty, pid)) => (Some(pty), Some(pid)),
            Err(fout) => {
                let tekst = format!("{} {fout}\r\n", crate::taal::t("De terminal start niet:", "The terminal does not start:"));
                Processor::<vte::ansi::StdSyncHandler>::new().advance(&mut *term.lock(), tekst.as_bytes());
                (None, None)
            }
        };

        cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            while let Ok(event) = events_rx.recv_async().await {
                if this.update(cx, |view, cx| view.on_term_event(event, cx)).is_err() {
                    break;
                }
            }
        })
        .detach();

        Self {
            metrics: Arc::default(),
            term,
            pty,
            focus: cx.focus_handle(),
            title: String::new(),
            pid,
            scroll_rest: 0.0,
            thema: &thema::DONKER,
            knippert: false,
            cursor_aan: true,
            _knipper: Task::ready(()),
        }
    }

    /// Zet de cursor aan. Met de focus knippert hij vanaf nu opnieuw, zonder staat hij stil.
    fn knipper(&mut self, cx: &mut Context<Self>) {
        self.cursor_aan = true;
        if !self.knippert {
            self._knipper = Task::ready(());
            return;
        }
        self._knipper = cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            loop {
                cx.background_executor().timer(KNIPPER).await;
                let wissel = this.update(cx, |view, cx| {
                    view.cursor_aan = !view.cursor_aan;
                    cx.notify();
                });
                if wissel.is_err() {
                    break;
                }
            }
        });
    }

    /// Stopt het programma: de pty gaat dicht en de shell krijgt SIGHUP.
    pub fn stop(&self) {
        if let Some(pty) = &self.pty {
            let _ = pty.send(Msg::Shutdown);
        }
    }

    pub fn write(&self, bytes: Vec<u8>) {
        if let Some(pty) = self.pty.as_ref().filter(|_| !bytes.is_empty()) {
            let _ = pty.send(Msg::Input(Cow::Owned(bytes)));
        }
    }

    fn on_term_event(&mut self, event: TermEvent, cx: &mut Context<Self>) {
        match event {
            TermEvent::Wakeup => cx.notify(),
            // Antwoorden van de terminal op vragen van het programma.
            TermEvent::PtyWrite(text) => self.write(text.into_bytes()),
            TermEvent::ColorRequest(index, format) => {
                let color = self.term.lock().colors()[index]
                    .unwrap_or_else(|| default_rgb(index, self.thema));
                self.write(format(color).into_bytes());
            }
            TermEvent::Title(title) => {
                self.title = title;
                cx.emit(TerminalEvent::Titel);
            }
            TermEvent::ResetTitle => {
                self.title.clear();
                cx.emit(TerminalEvent::Titel);
            }
            TermEvent::Exit | TermEvent::ChildExit(_) => cx.emit(TerminalEvent::Gestopt),
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
                cx.stop_propagation();
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
        // Tijdens het typen blijft de cursor aan.
        self.knipper(cx);
        cx.notify();
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
        let thema = thema::thema(window);
        self.thema = thema;
        let knippert = self.focus.is_focused(window) && window.is_window_active();
        if knippert != self.knippert {
            self.knippert = knippert;
            self.knipper(cx);
        }
        let cursor_aan = self.cursor_aan;
        let term = self.term.clone();
        let pty = self.pty.clone();
        let metrics = self.metrics.clone();
        div()
            .size_full()
            .bg(rgb(thema.bg))
            .track_focus(&self.focus)
            .on_key_down(cx.listener(Self::on_key))
            .on_scroll(cx.listener(Self::on_scroll))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_move(cx.listener(Self::on_mouse_move))
            .child(
                canvas(
                    |_, _, _| (),
                    move |bounds, _, window, cx| {
                        paint(bounds, &term, pty.as_ref(), &metrics, thema, cursor_aan, window, cx)
                    },
                )
                .size_full(),
            )
    }
}

/// Opent de pty met de shell en start de lus die hem leest. Geeft de zender en het pid van de shell.
fn start(
    options: &tty::Options,
    size: WindowSize,
    term: Arc<FairMutex<Term<Proxy>>>,
    proxy: Proxy,
) -> std::io::Result<(EventLoopSender, u32)> {
    let pty = tty::new(options, size, 0)?;
    let pid = pty.child().id();
    let event_loop = EventLoop::new(term, proxy, pty, false, false)?;
    let sender = event_loop.channel();
    event_loop.spawn();
    Ok((sender, pid))
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
            ..font(thema::LETTER)
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
    pty: Option<&EventLoopSender>,
    metrics: &Mutex<Metrics>,
    thema: &'static Thema,
    cursor_aan: bool,
    window: &mut Window,
    cx: &mut App,
) {
    let font_size = px(thema::TEKST);
    let pad_x = px(thema::TAB_PAD);
    let pad_y = px(thema::PAD);
    let default_fg = to_rgba(default_rgb(NamedColor::Foreground as usize, thema));

    // De breedte van een cel volgt uit de letter, de hoogte uit de regelhoogte.
    let sample = window.text_system().shape_line(
        "M".into(),
        font_size,
        &[text_run(1, default_fg, false, false, false)],
        None,
    );
    let cell_w = sample.width;
    let cell_h = (font_size * thema::TERMINAL_REGEL).round();

    let cols = (f32::from(bounds.size.width - pad_x * 2.0) / f32::from(cell_w)) as usize;
    let rows = (f32::from(bounds.size.height - pad_y * 2.0) / f32::from(cell_h)) as usize;
    let (cols, rows) = (cols.max(2), rows.max(1));

    let origin = point(bounds.origin.x + pad_x, bounds.origin.y + pad_y);
    let mut term = term.lock();
    {
        let mut m = metrics.lock().unwrap();
        m.origin = origin;
        m.cell_w = cell_w;
        m.cell_h = cell_h;
        let nieuw = (cols, rows) != (term.columns(), term.screen_lines());
        let nu = Instant::now();
        let klaar = match m.wacht {
            _ if !m.gemeten => true,
            Some((c, r, sinds)) if (c, r) == (cols, rows) => nu - sinds >= RUST_NA_SLEPEN,
            _ => {
                m.wacht = Some((cols, rows, nu));
                false
            }
        };
        if !nieuw {
            m.wacht = None;
        } else if klaar {
            m.gemeten = true;
            m.wacht = None;
            term.resize(Grid { cols, rows });
            if let Some(pty) = pty {
                let _ = pty.send(Msg::Resize(WindowSize {
                    num_lines: rows as u16,
                    num_cols: cols as u16,
                    cell_width: f32::from(cell_w) as u16,
                    cell_height: f32::from(cell_h) as u16,
                }));
            }
        } else {
            // Opnieuw tekenen tot de maat lang genoeg gelijk is gebleven.
            window.request_animation_frame();
        }
    }

    // Tijdens het wachten kan het oude raster breder zijn dan het vlak.
    window.with_content_mask(Some(ContentMask { bounds }), |window| {
        let content = term.renderable_content();
        let offset = content.display_offset as i32;
        let cell_origin =
            |row: i32, col: usize| point(origin.x + cell_w * col as f32, origin.y + cell_h * row as f32);

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

            let mut fg = resolve(cell.fg, content.colors, thema);
            let mut bg = resolve(cell.bg, content.colors, thema);
            let inverse = flags.contains(Flags::INVERSE);
            if inverse {
                std::mem::swap(&mut fg, &mut bg);
            }
            if flags.contains(Flags::DIM) {
                fg.a = 0.6;
            }
            let selected = content.selection.is_some_and(|range| range.contains(indexed.point));
            if selected {
                bg = rgb(thema.line_strong);
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

        if cursor_aan && content.cursor.shape != CursorShape::Hidden {
            let origin = cell_origin(content.cursor.point.line.0 + offset, content.cursor.point.column.0);
            let color = Rgba { a: 0.5, ..default_fg };
            window.paint_quad(fill(Bounds::new(origin, size(cell_w, cell_h)), color));
        }
    });
}

fn resolve(color: Color, colors: &Colors, thema: &Thema) -> Rgba {
    to_rgba(match color {
        Color::Spec(rgb) => rgb,
        Color::Named(name) => colors[name].unwrap_or_else(|| default_rgb(name as usize, thema)),
        Color::Indexed(index) => {
            colors[index as usize].unwrap_or_else(|| default_rgb(index as usize, thema))
        }
    })
}

fn to_rgba(rgb: Rgb) -> Rgba {
    Rgba { r: rgb.r as f32 / 255.0, g: rgb.g as f32 / 255.0, b: rgb.b as f32 / 255.0, a: 1.0 }
}

/// De kleur bij een index als het programma die niet zelf heeft gezet.
/// 0-15 ANSI, 16-231 kleurkubus, 232-255 grijs, daarna de benoemde kleuren.
fn default_rgb(index: usize, thema: &Thema) -> Rgb {
    let hex = match index {
        0..=15 => thema.ansi[index],
        16..=231 => {
            let i = (index - 16) as u32;
            let level = |v: u32| if v == 0 { 0 } else { 55 + 40 * v };
            level(i / 36) << 16 | level(i / 6 % 6) << 8 | level(i % 6)
        }
        232..=255 => {
            let v = 8 + 10 * (index - 232) as u32;
            v << 16 | v << 8 | v
        }
        257 => thema.bg,
        259..=266 => thema.ansi[index - 259],
        _ => thema.ink,
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
