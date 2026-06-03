use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use ratatui::style::Color;
use std::cell::RefCell;
use tui_alchemy::App;
use tui_alchemy::input_event::{
    Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use tui_alchemy::render_png::{ColorRole, color_to_rgba};

const DEFAULT_COLS: u16 = 88;
const DEFAULT_ROWS: u16 = 28;
const MIN_COLS: u16 = 56;
const MIN_ROWS: u16 = 18;

/// The live demo opens mid-play, on a workbench already full of discovered
/// element sprites — far more inviting than four lonely base tiles, and it fills
/// the screen so the board never reads as empty.
const DEMO_SEED: &[&str] = &[
    "Steam", "Mud", "Lava", "Rain", "Dust", "Energy", "Sea", "Cloud", "Stone", "Sand", "Metal",
    "Plant", "Mist", "Smoke",
];

thread_local! {
    static DEMO: RefCell<DemoState> = RefCell::new(DemoState::new());
}

struct DemoState {
    app: App,
    cols: u16,
    rows: u16,
    plain: String,
    ansi: Vec<u8>,
}

impl DemoState {
    fn new() -> Self {
        let mut app = App::new();
        app.reveal_elements_for_preview(DEMO_SEED);
        let mut demo = Self {
            app,
            cols: DEFAULT_COLS,
            rows: DEFAULT_ROWS,
            plain: String::new(),
            ansi: Vec::new(),
        };
        demo.render();
        demo
    }

    fn reset(&mut self) {
        self.app = App::new();
        self.app.reveal_elements_for_preview(DEMO_SEED);
        self.render();
    }

    fn resize(&mut self, cols: u16, rows: u16) {
        self.cols = cols.max(MIN_COLS);
        self.rows = rows.max(MIN_ROWS);
        self.app.handle_event(Event::Resize(self.cols, self.rows));
        self.render();
    }

    fn tick(&mut self) {
        self.app.tick();
        self.render();
    }

    fn key(&mut self, code: KeyCode) {
        self.app
            .handle_event(Event::Key(KeyEvent::new(code, KeyModifiers::NONE)));
        self.render();
    }

    fn mouse(&mut self, kind: MouseEventKind, column: u16, row: u16) {
        self.app.handle_event(Event::Mouse(MouseEvent {
            kind,
            column,
            row,
            modifiers: KeyModifiers::NONE,
        }));
        self.render();
    }

    fn render(&mut self) {
        let backend = TestBackend::new(self.cols, self.rows);
        let mut terminal = Terminal::new(backend).expect("test backend");
        terminal
            .draw(|frame| self.app.render(frame))
            .expect("draw frame");
        let buffer = terminal.backend().buffer();
        self.plain = buffer_to_plain(buffer);
        self.ansi = buffer_to_ansi(buffer);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn screen_ptr() -> *const u8 {
    DEMO.with(|demo| demo.borrow().ansi.as_ptr())
}

#[unsafe(no_mangle)]
pub extern "C" fn screen_len() -> usize {
    DEMO.with(|demo| demo.borrow().ansi.len())
}

#[unsafe(no_mangle)]
pub extern "C" fn reset_game() {
    DEMO.with(|demo| demo.borrow_mut().reset());
}

#[unsafe(no_mangle)]
pub extern "C" fn resize(cols: u16, rows: u16) {
    DEMO.with(|demo| demo.borrow_mut().resize(cols, rows));
}

#[unsafe(no_mangle)]
pub extern "C" fn tick() {
    DEMO.with(|demo| demo.borrow_mut().tick());
}

#[unsafe(no_mangle)]
pub extern "C" fn key_up() {
    DEMO.with(|demo| demo.borrow_mut().key(KeyCode::Up));
}

#[unsafe(no_mangle)]
pub extern "C" fn key_down() {
    DEMO.with(|demo| demo.borrow_mut().key(KeyCode::Down));
}

#[unsafe(no_mangle)]
pub extern "C" fn key_left() {
    DEMO.with(|demo| demo.borrow_mut().key(KeyCode::Left));
}

#[unsafe(no_mangle)]
pub extern "C" fn key_right() {
    DEMO.with(|demo| demo.borrow_mut().key(KeyCode::Right));
}

#[unsafe(no_mangle)]
pub extern "C" fn key_page_up() {
    DEMO.with(|demo| demo.borrow_mut().key(KeyCode::PageUp));
}

#[unsafe(no_mangle)]
pub extern "C" fn key_page_down() {
    DEMO.with(|demo| demo.borrow_mut().key(KeyCode::PageDown));
}

#[unsafe(no_mangle)]
pub extern "C" fn key_home() {
    DEMO.with(|demo| demo.borrow_mut().key(KeyCode::Home));
}

#[unsafe(no_mangle)]
pub extern "C" fn key_end() {
    DEMO.with(|demo| demo.borrow_mut().key(KeyCode::End));
}

#[unsafe(no_mangle)]
pub extern "C" fn key_enter() {
    DEMO.with(|demo| demo.borrow_mut().key(KeyCode::Enter));
}

#[unsafe(no_mangle)]
pub extern "C" fn key_escape() {
    DEMO.with(|demo| demo.borrow_mut().key(KeyCode::Esc));
}

#[unsafe(no_mangle)]
pub extern "C" fn key_backspace() {
    DEMO.with(|demo| demo.borrow_mut().key(KeyCode::Backspace));
}

#[unsafe(no_mangle)]
pub extern "C" fn key_char(ch: u32) {
    let Some(ch) = char::from_u32(ch) else {
        return;
    };
    DEMO.with(|demo| demo.borrow_mut().key(KeyCode::Char(ch)));
}

#[unsafe(no_mangle)]
pub extern "C" fn mouse_down(column: u16, row: u16) {
    DEMO.with(|demo| {
        demo.borrow_mut()
            .mouse(MouseEventKind::Down(MouseButton::Left), column, row)
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn mouse_drag(column: u16, row: u16) {
    DEMO.with(|demo| {
        demo.borrow_mut()
            .mouse(MouseEventKind::Drag(MouseButton::Left), column, row)
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn mouse_up(column: u16, row: u16) {
    DEMO.with(|demo| {
        demo.borrow_mut()
            .mouse(MouseEventKind::Up(MouseButton::Left), column, row)
    });
}

fn buffer_to_plain(buffer: &Buffer) -> String {
    let area = buffer.area;
    let mut out = String::new();
    for y in 0..area.height {
        for x in 0..area.width {
            let symbol = buffer[(area.x + x, area.y + y)].symbol();
            if symbol.is_empty() {
                out.push(' ');
            } else {
                out.push_str(symbol);
            }
        }
        if y + 1 < area.height {
            out.push('\n');
        }
    }
    out
}

fn buffer_to_ansi(buffer: &Buffer) -> Vec<u8> {
    let area = buffer.area;
    let mut out = Vec::with_capacity((area.width as usize + 16) * area.height as usize);
    out.extend_from_slice(b"\x1b[H");

    let mut current_fg = None;
    let mut current_bg = None;
    for y in 0..area.height {
        for x in 0..area.width {
            let cell = &buffer[(area.x + x, area.y + y)];
            let fg = to_rgb(cell.fg, ColorRole::Foreground);
            let bg = to_rgb(cell.bg, ColorRole::Background);
            if current_fg != Some(fg) || current_bg != Some(bg) {
                push_style(&mut out, fg, bg);
                current_fg = Some(fg);
                current_bg = Some(bg);
            }
            let symbol = cell.symbol();
            if symbol.is_empty() {
                out.push(b' ');
            } else {
                out.extend_from_slice(symbol.as_bytes());
            }
        }
        out.extend_from_slice(b"\x1b[0m");
        current_fg = None;
        current_bg = None;
        if y + 1 < area.height {
            out.extend_from_slice(b"\r\n");
        }
    }
    out.extend_from_slice(b"\x1b[0m");
    out
}

fn to_rgb(color: Color, role: ColorRole) -> (u8, u8, u8) {
    let rgba = color_to_rgba(color, role);
    (rgba[0], rgba[1], rgba[2])
}

fn push_style(out: &mut Vec<u8>, fg: (u8, u8, u8), bg: (u8, u8, u8)) {
    out.extend_from_slice(b"\x1b[38;2;");
    push_decimal(out, fg.0);
    out.push(b';');
    push_decimal(out, fg.1);
    out.push(b';');
    push_decimal(out, fg.2);
    out.extend_from_slice(b"m\x1b[48;2;");
    push_decimal(out, bg.0);
    out.push(b';');
    push_decimal(out, bg.1);
    out.push(b';');
    push_decimal(out, bg.2);
    out.push(b'm');
}

fn push_decimal(out: &mut Vec<u8>, value: u8) {
    if value >= 100 {
        out.push(b'0' + (value / 100));
        out.push(b'0' + ((value / 10) % 10));
        out.push(b'0' + (value % 10));
    } else if value >= 10 {
        out.push(b'0' + (value / 10));
        out.push(b'0' + (value % 10));
    } else {
        out.push(b'0' + value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plain() -> String {
        DEMO.with(|demo| demo.borrow().plain.clone())
    }

    #[test]
    fn initial_screen_renders_real_tui_panels() {
        reset_game();
        let screen = plain().to_ascii_lowercase();
        assert!(screen.contains("little alchemy"));
        assert!(screen.contains("crafting table workbench"));
        assert!(screen.contains("recipe table"));
        assert!(screen.contains("atlas"));
        assert!(screen.contains("progress"));
    }

    #[test]
    fn keyboard_play_discovers_steam_in_real_app() {
        reset_game();
        key_char('4' as u32);
        key_char('3' as u32);
        tick();
        let screen = plain().to_ascii_lowercase();
        assert!(screen.contains("steam"), "{screen}");
        assert!(
            screen.contains("5/755") || screen.contains("5 / 755"),
            "{screen}"
        );
    }
}
