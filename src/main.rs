use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::error::Error;
use std::io;
use std::time::{Duration, Instant};
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
                        app.handle_event(event);
                        redraw = true;
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
