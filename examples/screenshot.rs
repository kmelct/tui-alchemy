use ratatui::{Terminal, backend::TestBackend};
use tui_alchemy::{
    App,
    render_png::{buffer_to_png_default, save_png},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    save_view("01-initial", 100, 32, &[])?;
    save_view("02-created-steam", 100, 32, &["Steam"])?;
    save_view("03-drag-ghost", 100, 32, &["Steam", "Mud"])?;
    save_view("04-populated-board", 100, 32, MANY_DISCOVERIES)?;
    save_view("05-narrow", 64, 40, &["Steam", "Mud", "Lava", "Rain"])?;
    save_view("06-xlarge", 200, 60, MANY_DISCOVERIES)?;
    save_view(
        "07-height-24",
        100,
        24,
        &["Steam", "Mud", "Lava", "Rain", "Sea", "Cloud"],
    )?;
    save_view(
        "08-height-48",
        100,
        48,
        &["Steam", "Mud", "Lava", "Rain", "Sea", "Cloud"],
    )?;
    println!("Wrote PNGs to output/screenshot/");
    Ok(())
}

const MANY_DISCOVERIES: &[&str] = &[
    "Dust",
    "Energy",
    "Lava",
    "Mud",
    "Rain",
    "Sea",
    "Steam",
    "Cloud",
    "Plant",
    "Stone",
    "Metal",
    "Sand",
    "Sky",
    "Storm",
    "Glass",
    "Life",
    "Human",
    "Tool",
    "Wind",
    "Eruption",
    "Smoke",
    "Land",
    "Mist",
    "Lightning",
];

fn save_view(
    name: &str,
    width: u16,
    height: u16,
    revealed: &[&str],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut app = App::new();
    if !revealed.is_empty() {
        app.reveal_elements_for_preview(revealed);
    }
    if name == "03-drag-ghost" {
        app.preview_drag_element("Water", width / 2, height / 2);
    }
    for _ in 0..4 {
        app.tick();
    }

    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend)?;
    terminal.draw(|frame| app.render(frame))?;
    let image = buffer_to_png_default(terminal.backend().buffer());
    save_png(&image, format!("output/screenshot/{name}.png"))?;
    Ok(())
}
