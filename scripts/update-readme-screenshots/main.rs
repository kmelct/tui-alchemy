use std::{error::Error, fs, path::Path};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use image::{Rgba, RgbaImage};
use ratatui::{Terminal, backend::TestBackend};
use tui_alchemy::{
    App,
    render_png::{buffer_to_png_default, save_png},
};

const README_PATH: &str = "README.md";
const CARGO_TOML_PATH: &str = "Cargo.toml";
const WEBSITE_INDEX_PATH: &str = "website/index.html";
const UNIX_INSTALLER_PATH: &str = "scripts/install-tui-alchemy.sh";
const WINDOWS_INSTALLER_PATH: &str = "scripts/install-tui-alchemy.ps1";
const SCREENSHOTS_DIR: &str = "docs/screenshots";
const BG: Rgba<u8> = Rgba([7, 16, 18, 255]);
const OUTER_RIM: Rgba<u8> = Rgba([170, 124, 54, 255]);
const INNER_RIM: Rgba<u8> = Rgba([84, 61, 39, 255]);
const SHADOW: Rgba<u8> = Rgba([18, 20, 30, 255]);
const SCREENSHOT_FILES: &[&str] = &[
    "hero.png",
    "01-select-element.png",
    "02-select-second.png",
    "03-get-result.png",
    "04-populated-board.png",
    "05-narrow.png",
    "06-xlarge.png",
    "07-height-24.png",
    "08-height-48.png",
];
const GLOW: Rgba<u8> = Rgba([37, 35, 48, 255]);

fn main() -> Result<(), Box<dyn Error>> {
    update_readme_screenshots()?;
    println!("Updated README screenshot assets in docs/screenshots/.");
    Ok(())
}

fn update_readme_screenshots() -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(SCREENSHOTS_DIR)?;
    remove_stale_screenshots()?;

    let hero = render_scene(120, 22, |app| {
        select_key(app, '4');
        select_key(app, '3');
        tick_ready(app);
    })?;
    save_png(
        &frame_hero_image(&hero),
        Path::new(SCREENSHOTS_DIR).join("hero.png"),
    )?;

    save_scene("01-select-element", 100, 24, |app| {
        select_key(app, '4');
        tick_ready(app);
    })?;
    save_scene("02-select-second", 100, 24, |app| {
        select_key(app, '4');
        app.preview_drag_element("Fire", 64, 12);
        tick_ready(app);
    })?;
    save_scene("03-get-result", 100, 24, |app| {
        select_key(app, '4');
        select_key(app, '3');
        tick_ready(app);
    })?;
    save_scene("04-populated-board", 100, 32, |app| {
        app.reveal_elements_for_preview(MANY_DISCOVERIES);
        tick_ready(app);
    })?;
    save_scene("05-narrow", 64, 40, |app| {
        app.reveal_elements_for_preview(&["Steam", "Mud", "Lava", "Rain"]);
        tick_ready(app);
    })?;
    save_scene("06-xlarge", 200, 60, |app| {
        app.reveal_elements_for_preview(MANY_DISCOVERIES);
        tick_ready(app);
    })?;
    save_scene("07-height-24", 100, 24, |app| {
        app.reveal_elements_for_preview(&["Steam", "Mud", "Lava", "Rain", "Sea", "Cloud"]);
        tick_ready(app);
    })?;
    save_scene("08-height-48", 100, 48, |app| {
        app.reveal_elements_for_preview(&["Steam", "Mud", "Lava", "Rain", "Sea", "Cloud"]);
        tick_ready(app);
    })?;

    update_readme_markdown()?;
    Ok(())
}

fn update_readme_markdown() -> Result<(), Box<dyn Error>> {
    let version = crate_version()?;
    let readme = fs::read_to_string(README_PATH)?;
    let readme = replace_marked_section(&readme, "readme-hero", &hero_markdown())?;
    let readme =
        replace_marked_section(&readme, "readme-screenshots", &interaction_flow_markdown())?;
    fs::write(README_PATH, update_readme_version_text(&readme, &version))?;
    update_versioned_project_files(&version)?;
    Ok(())
}

fn crate_version() -> Result<String, Box<dyn Error>> {
    let cargo_toml = fs::read_to_string(CARGO_TOML_PATH)?;
    for line in cargo_toml.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("version = \"") {
            if let Some(version) = rest.strip_suffix('"') {
                return Ok(version.to_owned());
            }
        }
    }
    Err("missing package version in Cargo.toml".into())
}

fn update_versioned_project_files(version: &str) -> Result<(), Box<dyn Error>> {
    update_file_text(WEBSITE_INDEX_PATH, |content| {
        update_website_version_text(content, version)
    })?;
    update_file_text(UNIX_INSTALLER_PATH, |content| {
        update_unix_installer_version_text(content, version)
    })?;
    update_file_text(WINDOWS_INSTALLER_PATH, |content| {
        update_windows_installer_version_text(content, version)
    })?;
    Ok(())
}

fn update_file_text(
    path: &str,
    updater: impl FnOnce(&str) -> String,
) -> Result<(), Box<dyn Error>> {
    let content = fs::read_to_string(path)?;
    let next = updater(&content);
    if next != content {
        fs::write(path, next)?;
    }
    Ok(())
}

fn update_readme_version_text(readme: &str, version: &str) -> String {
    let readme = replace_between_first(
        readme,
        "https://img.shields.io/badge/release-v",
        "-blue",
        version,
    );
    let readme = replace_between_first(&readme, "`docs/release-v", ".md`", version);
    replace_section_between(
        &readme,
        "## Quick start\n",
        "\n## How the screen works",
        &quick_start_markdown(),
    )
}

fn update_website_version_text(html: &str, version: &str) -> String {
    replace_between_first(html, "class=\"ver\">v", "</span>", version)
}

fn update_unix_installer_version_text(script: &str, version: &str) -> String {
    replace_between_first(
        script,
        "APP_VERSION=\"${TUI_ALCHEMY_VERSION:-",
        "}\"",
        version,
    )
}

fn update_windows_installer_version_text(script: &str, version: &str) -> String {
    replace_between_first(
        script,
        "if ([string]::IsNullOrWhiteSpace($Version)) { $Version = \"",
        "\" }",
        version,
    )
}

fn quick_start_markdown() -> String {
    "Install the latest release:\n\n```sh\ncurl -fsSL https://i.tui-alchemy.sh | sh\n```\n\nThen run:\n\n```sh\ntui-alchemy\n```\n\nCommand-line help and package metadata:\n\n```sh\ntui-alchemy --help\ntui-alchemy --version\n```\n\nFor local development from this repository:\n\n```sh\ncargo run\n```\n\nPress `q` to quit.\n".to_owned()
}

fn replace_between_first(source: &str, prefix: &str, suffix: &str, replacement: &str) -> String {
    let Some(start) = source.find(prefix) else {
        return source.to_owned();
    };
    let value_start = start + prefix.len();
    let Some(end_offset) = source[value_start..].find(suffix) else {
        return source.to_owned();
    };
    let value_end = value_start + end_offset;

    let mut next = String::with_capacity(source.len() + replacement.len());
    next.push_str(&source[..value_start]);
    next.push_str(replacement);
    next.push_str(&source[value_end..]);
    next
}

fn replace_section_between(
    source: &str,
    start_marker: &str,
    end_marker: &str,
    replacement: &str,
) -> String {
    let Some(start) = source.find(start_marker) else {
        return source.to_owned();
    };
    let content_start = start + start_marker.len();
    let Some(end_offset) = source[content_start..].find(end_marker) else {
        return source.to_owned();
    };
    let content_end = content_start + end_offset;

    let mut next = String::with_capacity(source.len() + replacement.len());
    next.push_str(&source[..content_start]);
    next.push('\n');
    next.push_str(replacement);
    if !replacement.ends_with('\n') {
        next.push('\n');
    }
    next.push_str(&source[content_end..]);
    next
}

fn save_scene(
    name: &str,
    width: u16,
    height: u16,
    setup: impl FnOnce(&mut App),
) -> Result<(), Box<dyn Error>> {
    let img = render_scene(width, height, setup)?;
    save_png(&img, Path::new(SCREENSHOTS_DIR).join(format!("{name}.png")))?;
    Ok(())
}

fn remove_stale_screenshots() -> Result<(), Box<dyn Error>> {
    for entry in fs::read_dir(SCREENSHOTS_DIR)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("png") {
            continue;
        }
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !SCREENSHOT_FILES.contains(&name) {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}

fn render_scene(
    width: u16,
    height: u16,
    setup: impl FnOnce(&mut App),
) -> Result<RgbaImage, Box<dyn Error>> {
    let mut app = App::new();
    setup(&mut app);

    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend)?;
    terminal.draw(|frame| app.render(frame))?;
    Ok(buffer_to_png_default(terminal.backend().buffer()))
}

fn select_key(app: &mut App, key: char) {
    app.handle_event(Event::Key(KeyEvent::new(
        KeyCode::Char(key),
        KeyModifiers::NONE,
    )));
}

fn tick_ready(app: &mut App) {
    for _ in 0..11 {
        app.tick();
    }
}

fn hero_markdown() -> String {
    "<p align=\"center\">\n  <img alt=\"Alchemy TUI workbench with Water and Fire resolving into Steam\" src=\"docs/screenshots/hero.png\">\n</p>\n".to_owned()
}

fn interaction_flow_markdown() -> String {
    "Use it as a three-step loop: **select an element → select a second element → get a result**.\n\n<p align=\"center\">\n  <img alt=\"Water selected in the workbench\" src=\"docs/screenshots/01-select-element.png\" width=\"31%\">\n  <img alt=\"Fire being dragged as the second ingredient\" src=\"docs/screenshots/02-select-second.png\" width=\"31%\">\n  <img alt=\"Water and Fire resolving into Steam\" src=\"docs/screenshots/03-get-result.png\" width=\"31%\">\n</p>\n\nThe left rail tracks progress, the center atlas holds discovered ingredients, and the right workbench resolves `ingredient + ingredient = result`.\n".to_owned()
}

fn replace_marked_section(readme: &str, marker: &str, replacement: &str) -> Result<String, String> {
    let start = format!("<!-- {marker}:start -->");
    let end = format!("<!-- {marker}:end -->");
    let Some(start_index) = readme.find(&start) else {
        return Err(format!("missing marker {start}"));
    };
    let after_start = start_index + start.len();
    let Some(end_offset) = readme[after_start..].find(&end) else {
        return Err(format!("missing marker {end}"));
    };
    let end_index = after_start + end_offset;

    let mut next = String::with_capacity(readme.len() + replacement.len());
    next.push_str(&readme[..after_start]);
    next.push('\n');
    next.push_str(replacement);
    if !replacement.ends_with('\n') {
        next.push('\n');
    }
    next.push_str(&readme[end_index..]);
    Ok(next)
}

fn frame_hero_image(source: &RgbaImage) -> RgbaImage {
    let source = crop_hero_source(source);
    let pad = 18;
    let border = 8;
    let width = source.width() + (pad + border) * 2;
    let height = source.height() + (pad + border) * 2;
    let mut framed = RgbaImage::from_pixel(width, height, BG);

    fill_rect(&mut framed, 0, 0, width, height, SHADOW);
    fill_rect(&mut framed, 4, 4, width - 8, height - 8, GLOW);
    stroke_rect(&mut framed, 2, 2, width - 5, height - 5, OUTER_RIM, 3);
    stroke_rect(&mut framed, 10, 10, width - 21, height - 21, INNER_RIM, 2);
    draw_corner_marks(&mut framed, width, height);

    let offset = pad + border;
    overlay(&mut framed, &source, offset, offset);
    framed
}

fn crop_hero_source(source: &RgbaImage) -> RgbaImage {
    let mut bounds: Option<(u32, u32, u32, u32)> = None;
    for row in 0..source.height() {
        for col in 0..source.width() {
            if is_salient_hero_pixel(*source.get_pixel(col, row)) {
                bounds = Some(match bounds {
                    Some((min_x, min_y, max_x, max_y)) => (
                        min_x.min(col),
                        min_y.min(row),
                        max_x.max(col),
                        max_y.max(row),
                    ),
                    None => (col, row, col, row),
                });
            }
        }
    }

    let Some((min_x, min_y, max_x, max_y)) = bounds else {
        return source.clone();
    };
    let pad = 16;
    let x = min_x.saturating_sub(pad);
    let y = min_y.saturating_sub(pad);
    let right = max_x
        .saturating_add(pad)
        .min(source.width().saturating_sub(1));
    let bottom = max_y
        .saturating_add(pad)
        .min(source.height().saturating_sub(1));
    let width = right.saturating_sub(x).saturating_add(1);
    let height = bottom.saturating_sub(y).saturating_add(1);
    let mut cropped = RgbaImage::from_pixel(width, height, BG);
    for row in 0..height {
        for col in 0..width {
            cropped.put_pixel(col, row, *source.get_pixel(x + col, y + row));
        }
    }
    cropped
}

fn is_salient_hero_pixel(pixel: Rgba<u8>) -> bool {
    let [r, g, b, a] = pixel.0;
    a > 0 && (r > 95 || g > 95 || b > 95)
}

fn overlay(target: &mut RgbaImage, source: &RgbaImage, x: u32, y: u32) {
    for row in 0..source.height() {
        for col in 0..source.width() {
            target.put_pixel(x + col, y + row, *source.get_pixel(col, row));
        }
    }
}

fn fill_rect(img: &mut RgbaImage, x: u32, y: u32, w: u32, h: u32, color: Rgba<u8>) {
    for row in y..y + h {
        for col in x..x + w {
            img.put_pixel(col, row, color);
        }
    }
}

fn stroke_rect(
    img: &mut RgbaImage,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    color: Rgba<u8>,
    thickness: u32,
) {
    for i in 0..thickness {
        fill_rect(img, x + i, y + i, w.saturating_sub(i * 2), 1, color);
        fill_rect(
            img,
            x + i,
            y + h.saturating_sub(1 + i),
            w.saturating_sub(i * 2),
            1,
            color,
        );
        fill_rect(img, x + i, y + i, 1, h.saturating_sub(i * 2), color);
        fill_rect(
            img,
            x + w.saturating_sub(1 + i),
            y + i,
            1,
            h.saturating_sub(i * 2),
            color,
        );
    }
}

fn draw_corner_marks(img: &mut RgbaImage, width: u32, height: u32) {
    let marks = ((width.min(height).saturating_sub(48)) / 3).min(18);
    for i in 0..marks {
        let color = if i % 2 == 0 { OUTER_RIM } else { INNER_RIM };
        fill_rect(img, 15 + i * 3, 15, 2, 9, color);
        fill_rect(img, 15, 15 + i * 3, 9, 2, color);
        fill_rect(img, width - 17 - i * 3, 15, 2, 9, color);
        fill_rect(img, width - 24, 15 + i * 3, 9, 2, color);
        fill_rect(img, 15 + i * 3, height - 24, 2, 9, color);
        fill_rect(img, 15, height - 17 - i * 3, 9, 2, color);
        fill_rect(img, width - 17 - i * 3, height - 24, 2, 9, color);
        fill_rect(img, width - 24, height - 17 - i * 3, 9, 2, color);
    }
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

#[cfg(test)]
mod tests {
    #[test]
    fn flow_markdown_uses_selected_result_screenshots() {
        let markdown = super::interaction_flow_markdown();

        assert!(markdown.contains("select an element → select a second element → get a result"));
        assert!(markdown.contains("docs/screenshots/01-select-element.png"));
        assert!(markdown.contains("docs/screenshots/02-select-second.png"));
        assert!(markdown.contains("docs/screenshots/03-get-result.png"));
    }

    #[test]
    fn marked_section_replacement_is_exact() {
        let readme = "before\n<!-- readme-screenshots:start -->\nstale\n<!-- readme-screenshots:end -->\nafter\n";
        let next = super::replace_marked_section(readme, "readme-screenshots", "fresh\n")
            .expect("marker exists");

        assert_eq!(
            next,
            "before\n<!-- readme-screenshots:start -->\nfresh\n<!-- readme-screenshots:end -->\nafter\n"
        );
    }

    #[test]
    fn readme_version_update_rewrites_release_badge_and_quick_install() {
        let readme = "<a href=\"https://github.com/kmelct/tui-alchemy/releases\"><img alt=\"Release\" src=\"https://img.shields.io/badge/release-v0.1.0-blue\"></a>\n## Quick start\n\n```sh\ncargo run\n```\n\n## How the screen works\nRelease notes: `docs/release-v0.1.0.md`\n";
        let next = super::update_readme_version_text(readme, "0.2.0");

        assert!(next.contains("release-v0.2.0-blue"));
        assert!(next.contains("curl -fsSL https://i.tui-alchemy.sh | sh"));
        assert!(next.contains("docs/release-v0.2.0.md"));
        assert!(!next.contains("release-v0.1.0-blue"));
    }

    #[test]
    fn hero_crop_removes_empty_dark_margins() {
        let mut source = image::RgbaImage::from_pixel(80, 70, image::Rgba([7, 16, 18, 255]));
        for row in 30..34 {
            for col in 34..42 {
                source.put_pixel(col, row, image::Rgba([170, 124, 54, 255]));
            }
        }

        let cropped = super::crop_hero_source(&source);

        assert!(cropped.width() < source.width());
        assert!(cropped.height() < source.height());
        assert!(*cropped.get_pixel(0, 0) != image::Rgba([170, 124, 54, 255]));
    }

    #[test]
    fn hero_frame_adds_space_for_fantasy_border() {
        let source = image::RgbaImage::from_pixel(8, 8, image::Rgba([7, 16, 18, 255]));
        let framed = super::frame_hero_image(&source);

        assert!(framed.width() > source.width());
        assert!(framed.height() > source.height());
        assert_ne!(*framed.get_pixel(0, 0), *source.get_pixel(0, 0));
    }
}
