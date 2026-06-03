use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::error::Error;
use std::io;
use std::time::{Duration, Instant};
use tui_alchemy::input_event::{
    Event as AppEvent, KeyCode as AppKeyCode, KeyEvent as AppKeyEvent,
    KeyModifiers as AppKeyModifiers, MouseButton as AppMouseButton, MouseEvent as AppMouseEvent,
    MouseEventKind as AppMouseEventKind,
};
use tui_alchemy::{App, about};

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args_os();
    let binary_name = args
        .next()
        .and_then(|path| {
            std::path::Path::new(&path)
                .file_name()
                .and_then(std::ffi::OsStr::to_str)
                .map(str::to_owned)
        })
        .unwrap_or_else(|| about::NAME.to_owned());

    if let Some(arg) = args.next() {
        return match arg.to_str() {
            Some("-h" | "--help") => {
                about::write_help(io::stdout().lock(), &binary_name)?;
                Ok(())
            }
            Some("-V" | "--version") => {
                about::write_version(io::stdout().lock())?;
                Ok(())
            }
            _ => {
                eprintln!("unknown argument: {}", arg.to_string_lossy());
                about::write_help(io::stderr().lock(), &binary_name)?;
                std::process::exit(2);
            }
        };
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut app = App::new();

    let result = run(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

fn into_app_event(event: Event) -> Option<AppEvent> {
    match event {
        Event::Key(key) => Some(AppEvent::Key(AppKeyEvent::new(
            match key.code {
                KeyCode::Up => AppKeyCode::Up,
                KeyCode::Down => AppKeyCode::Down,
                KeyCode::Left => AppKeyCode::Left,
                KeyCode::Right => AppKeyCode::Right,
                KeyCode::PageUp => AppKeyCode::PageUp,
                KeyCode::PageDown => AppKeyCode::PageDown,
                KeyCode::Home => AppKeyCode::Home,
                KeyCode::End => AppKeyCode::End,
                KeyCode::Esc => AppKeyCode::Esc,
                KeyCode::Enter => AppKeyCode::Enter,
                KeyCode::Backspace => AppKeyCode::Backspace,
                KeyCode::Char(ch) => AppKeyCode::Char(ch),
                _ => return None,
            },
            into_app_modifiers(key.modifiers),
        ))),
        Event::Mouse(mouse) => Some(AppEvent::Mouse(AppMouseEvent {
            kind: match mouse.kind {
                crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                    AppMouseEventKind::Down(AppMouseButton::Left)
                }
                crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Right) => {
                    AppMouseEventKind::Down(AppMouseButton::Right)
                }
                crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Middle) => {
                    AppMouseEventKind::Down(AppMouseButton::Middle)
                }
                crossterm::event::MouseEventKind::Up(crossterm::event::MouseButton::Left) => {
                    AppMouseEventKind::Up(AppMouseButton::Left)
                }
                crossterm::event::MouseEventKind::Up(crossterm::event::MouseButton::Right) => {
                    AppMouseEventKind::Up(AppMouseButton::Right)
                }
                crossterm::event::MouseEventKind::Up(crossterm::event::MouseButton::Middle) => {
                    AppMouseEventKind::Up(AppMouseButton::Middle)
                }
                crossterm::event::MouseEventKind::Drag(crossterm::event::MouseButton::Left) => {
                    AppMouseEventKind::Drag(AppMouseButton::Left)
                }
                crossterm::event::MouseEventKind::Drag(crossterm::event::MouseButton::Right) => {
                    AppMouseEventKind::Drag(AppMouseButton::Right)
                }
                crossterm::event::MouseEventKind::Drag(crossterm::event::MouseButton::Middle) => {
                    AppMouseEventKind::Drag(AppMouseButton::Middle)
                }
                crossterm::event::MouseEventKind::ScrollDown => AppMouseEventKind::ScrollDown,
                crossterm::event::MouseEventKind::ScrollUp => AppMouseEventKind::ScrollUp,
                crossterm::event::MouseEventKind::Moved => AppMouseEventKind::Moved,
                _ => return None,
            },
            column: mouse.column,
            row: mouse.row,
            modifiers: into_app_modifiers(mouse.modifiers),
        })),
        Event::Resize(width, height) => Some(AppEvent::Resize(width, height)),
        Event::FocusGained => Some(AppEvent::FocusGained),
        Event::FocusLost => Some(AppEvent::FocusLost),
        Event::Paste(_) => Some(AppEvent::Paste(())),
    }
}

fn into_app_modifiers(modifiers: crossterm::event::KeyModifiers) -> AppKeyModifiers {
    let mut out = AppKeyModifiers::NONE;
    if modifiers.contains(crossterm::event::KeyModifiers::SHIFT) {
        out = out.union(AppKeyModifiers::SHIFT);
    }
    if modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
        out = out.union(AppKeyModifiers::CONTROL);
    }
    if modifiers.contains(crossterm::event::KeyModifiers::ALT) {
        out = out.union(AppKeyModifiers::ALT);
    }
    if modifiers.contains(crossterm::event::KeyModifiers::SUPER) {
        out = out.union(AppKeyModifiers::SUPER);
    }
    out
}

fn run(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<(), Box<dyn Error>> {
    let tick_rate = Duration::from_millis(120);
    let mut last_tick = Instant::now();

    let mut redraw = true;

    loop {
        if redraw {
            terminal.draw(|frame| app.render(frame))?;
            redraw = false;
        }

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or(Duration::from_millis(0));

        if event::poll(timeout)? {
            loop {
                match event::read()? {
                    Event::Key(key) if key.code == KeyCode::Char('q') => return Ok(()),
                    event => {
                        if let Some(event) = into_app_event(event) {
                            app.handle_event(event);
                            redraw = true;
                        }
                    }
                }

                if !event::poll(Duration::from_millis(0))? {
                    break;
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.tick();
            last_tick = Instant::now();
            redraw = true;
        }
    }
}
