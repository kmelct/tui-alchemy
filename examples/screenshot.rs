//! Render the live UI to PNGs under `output/screenshot/` so the terminal
//! interface can be inspected as images. Run with `cargo run --example screenshot`.

use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::{Terminal, backend::TestBackend};
use tui_alchemy::App;
use tui_alchemy::render_png::{buffer_to_png_default, save_png};

fn main() {
    // 1. Initial screen (default size).
    let mut app = App::new();
    let mut terminal = make_terminal(100, 32);
    shoot("01-initial", &mut terminal, &mut app);

    // 2. A fresh discovery (water + fire = steam) with the birth banner live.
    app.handle_event(key(KeyCode::Char('4')));
    app.handle_event(key(KeyCode::Char('3')));
    app.tick();
    shoot("02-created-steam", &mut terminal, &mut app);

    // 3. Mid-drag ghost: pick up water, drag toward fire.
    let lines = buffer_lines(terminal.backend().buffer());
    let water = find_text_position(&lines, "water").unwrap_or((8, 10));
    let fire = find_text_position(&lines, "fire").unwrap_or((28, 10));
    app.handle_event(mouse(
        MouseEventKind::Down(MouseButton::Left),
        water.0,
        water.1,
    ));
    app.handle_event(mouse(
        MouseEventKind::Drag(MouseButton::Left),
        fire.0,
        fire.1,
    ));
    app.tick();
    shoot("03-drag-ghost", &mut terminal, &mut app);

    // 4. A densely-populated board to judge the grid / shelves.
    let mut full = App::new();
    full.reveal_elements_for_preview(&[
        "Dust",
        "Energy",
        "Lava",
        "Mud",
        "Pressure",
        "Rain",
        "Sea",
        "Steam",
        "Atmosphere",
        "Brick",
        "Cloud",
        "Plant",
        "Stone",
        "Volcano",
        "Wind",
        "Grass",
        "Metal",
        "Mountain",
        "Sand",
        "Sky",
        "Storm",
        "Glass",
        "Time",
        "Life",
        "Human",
        "Tool",
        "Book",
        "Bird",
        "Fish",
        "House",
        "Tree",
        "Vase",
    ]);
    shoot("04-populated-board", &mut terminal, &mut full);

    // 5. Narrow terminal (exercises the vertical layout branch).
    let mut narrow = App::new();
    narrow.reveal_elements_for_preview(&["Steam", "Mud", "Lava", "Rain"]);
    let mut narrow_terminal = make_terminal(64, 40);
    shoot("05-narrow", &mut narrow_terminal, &mut narrow);

    // 6. Large terminal.
    let mut large = App::new();
    large.reveal_elements_for_preview(&[
        "Dust", "Energy", "Lava", "Mud", "Rain", "Sea", "Steam", "Cloud", "Plant", "Stone",
        "Metal", "Sand", "Sky", "Storm", "Glass", "Life", "Human", "Tool",
    ]);
    let mut large_terminal = make_terminal(200, 60);
    shoot("06-xlarge", &mut large_terminal, &mut large);

    // 7 & 8. Same content at two heights — proves the layout responds to height.
    let names: &[&str] = &["Steam", "Mud", "Lava", "Rain", "Sea", "Cloud"];
    let mut short = App::new();
    short.reveal_elements_for_preview(names);
    let mut short_terminal = make_terminal(100, 24);
    shoot("07-height-24", &mut short_terminal, &mut short);

    let mut tall = App::new();
    tall.reveal_elements_for_preview(names);
    let mut tall_terminal = make_terminal(100, 48);
    shoot("08-height-48", &mut tall_terminal, &mut tall);

    println!("Wrote PNGs to output/screenshot/");
}

fn make_terminal(width: u16, height: u16) -> Terminal<TestBackend> {
    Terminal::new(TestBackend::new(width, height)).expect("terminal")
}

fn shoot(name: &str, terminal: &mut Terminal<TestBackend>, app: &mut App) {
    terminal.draw(|frame| app.render(frame)).expect("draw");
    let img = buffer_to_png_default(terminal.backend().buffer());
    save_png(&img, format!("output/screenshot/{name}.png")).expect("save png");
    println!("  {name}");
}

fn key(code: KeyCode) -> Event {
    Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
}

fn mouse(kind: MouseEventKind, column: u16, row: u16) -> Event {
    Event::Mouse(MouseEvent {
        kind,
        column,
        row,
        modifiers: KeyModifiers::NONE,
    })
}

fn buffer_lines(buffer: &ratatui::buffer::Buffer) -> Vec<String> {
    let area = buffer.area;
    let mut lines = Vec::new();
    for y in 0..area.height {
        let mut line = String::new();
        for x in 0..area.width {
            line.push_str(buffer[(area.x + x, area.y + y)].symbol());
        }
        lines.push(line);
    }
    lines
}

fn find_text_position(lines: &[String], needle: &str) -> Option<(u16, u16)> {
    lines.iter().enumerate().find_map(|(row, line)| {
        line.find(needle).map(|byte_column| {
            let column = line[..byte_column].chars().count();
            (column as u16, row as u16)
        })
    })
}
