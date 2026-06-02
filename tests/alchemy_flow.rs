use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::{Terminal, backend::TestBackend};
use std::path::PathBuf;
use tui_alchemy::{
    App,
    data::{CatalogKind, GameCatalog},
    sprites::SpriteSource,
};

const ATLAS_BG: ratatui::style::Color = ratatui::style::Color::Rgb(24, 38, 43);
const STEAM_BIRTH_AURA_BG: ratatui::style::Color = ratatui::style::Color::Rgb(39, 51, 65);
const STEAM_BIRTH_HALO_BG: ratatui::style::Color = ratatui::style::Color::Rgb(50, 65, 82);
const WORKBENCH_SLOT_BG: ratatui::style::Color = ratatui::style::Color::Rgb(64, 50, 55);

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

#[test]
fn selecting_water_and_fire_discovers_steam() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let initial_lines = buffer_lines(terminal.backend().buffer());
    assert!(
        initial_lines
            .first()
            .map(|line| line.contains("Little Alchemy"))
            .unwrap_or(false),
        "expected title in header, got: {:?}",
        initial_lines.first()
    );
    assert!(
        initial_lines
            .first()
            .map(|line| line.contains("4 / 755"))
            .unwrap_or(false),
        "expected discovered count in header, got: {:?}",
        initial_lines.first()
    );
    assert!(
        initial_lines
            .get(1)
            .map(|line| line.contains("Little Alchemy"))
            .unwrap_or(false),
        "expected active catalog in status line, got: {:?}",
        initial_lines.get(1)
    );

    app.handle_event(key(KeyCode::Char('4')));
    app.handle_event(key(KeyCode::Char('3')));
    app.tick();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let buffer = terminal.backend().buffer();
    let text = buffer_to_text(buffer);

    assert!(
        text.contains("steam"),
        "expected steam in rendered output, got: {text}"
    );
    assert!(
        text.contains("new element"),
        "expected discovery banner in rendered output, got: {text}"
    );
    assert!(
        text.contains("5/755") || text.contains("5 / 755"),
        "expected progress to increase, got: {text}"
    );
}

#[test]
fn selecting_water_twice_discovers_sea() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    terminal.draw(|frame| app.render(frame)).unwrap();

    app.handle_event(key(KeyCode::Char('4')));
    app.handle_event(key(KeyCode::Char('4')));
    app.tick();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let text = buffer_to_text(terminal.backend().buffer());

    assert!(
        text.contains("sea"),
        "same-element recipes such as water + water should be playable, got: {text}"
    );
    assert!(
        text.contains("5/755") || text.contains("5 / 755"),
        "expected progress to increase after water + water, got: {text}"
    );
}

#[test]
fn preview_reveal_surfaces_seeded_chain_without_placeholder_output() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    app.reveal_elements_for_preview(&["Sea", "Metal", "Storm"]);

    terminal.draw(|frame| app.render(frame)).unwrap();
    let text = buffer_to_text(terminal.backend().buffer());

    assert!(
        text.contains("sea"),
        "expected seeded sea in preview: {text}"
    );
    assert!(
        text.contains("metal"),
        "expected seeded metal in preview: {text}"
    );
    assert!(
        text.contains("storm"),
        "expected seeded storm in preview: {text}"
    );
    assert!(
        !text.contains("   G     ") && !text.contains("   M     "),
        "preview should not leave a single-letter placeholder in the output slot: {text}"
    );
}

#[test]
fn seeded_chain_preview_shows_a_dense_atlas_page() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();
    let seeded_names = [
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
    ];

    app.reveal_elements_for_preview(&seeded_names);

    terminal.draw(|frame| app.render(frame)).unwrap();
    let text = buffer_to_text(terminal.backend().buffer());
    let visible_seeded_labels = seeded_names
        .iter()
        .filter(|name| text.contains(&name.to_ascii_lowercase()))
        .count();

    // The isometric discovery board trades raw per-page density for depth and
    // readable labels; it shows a compact scrollable page (~12 shelves) rather
    // than the old flat 5-column atlas, so the readable-at-once count is lower.
    assert!(
        visible_seeded_labels >= 10,
        "seeded preview should show a dense but readable local 16-bit seed sheet, not sparse cards or touching labels; visible={visible_seeded_labels}\n{text}"
    );
    assert!(
        !text.contains("█"),
        "early progress in preview should not render as a stray meter block:\n{text}"
    );
}

#[test]
fn dragging_water_onto_fire_combines_elements() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let initial_lines = buffer_lines(terminal.backend().buffer());
    let water = find_text_position(&initial_lines, "water").expect("expected water tile");
    let fire = find_text_position(&initial_lines, "fire").expect("expected fire tile");

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

    terminal.draw(|frame| app.render(frame)).unwrap();
    let drag_text = buffer_to_text(terminal.backend().buffer());
    assert!(
        drag_text.contains("water"),
        "expected dragged element to remain visible, got: {drag_text}"
    );
    assert!(
        !drag_text.contains("drag inventory")
            && !drag_text.contains("drag canvas")
            && !drag_text.contains("dragging water"),
        "dragging should render as a held element ghost, not a labeled debug card: {drag_text}"
    );

    app.handle_event(mouse(MouseEventKind::Up(MouseButton::Left), fire.0, fire.1));
    app.tick();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let text = buffer_to_text(terminal.backend().buffer());
    assert!(
        text.contains("steam"),
        "expected steam in rendered output after drag-and-drop, got: {text}"
    );
    assert!(
        text.contains("new element"),
        "expected discovery banner after drag-and-drop, got: {text}"
    );
}

#[test]
fn creating_an_element_uses_birth_effect_without_a_large_modal_card() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    app.handle_event(key(KeyCode::Char('4')));
    app.handle_event(key(KeyCode::Char('3')));
    app.tick();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let text = buffer_to_text(terminal.backend().buffer());

    assert!(
        text.contains("new element"),
        "expected discovery feedback, got: {text}"
    );
    let discovery_mentions = text.matches("new element").count();
    assert!(
        discovery_mentions <= 2,
        "discovery feedback should stay in the HUD/tile effect, not repeat in a large modal: {text}"
    );
    assert!(
        text.contains("steam"),
        "expected the new element itself to carry discovery feedback, got: {text}"
    );
}

#[test]
fn created_element_tile_gets_an_anchored_birth_glow() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    app.handle_event(key(KeyCode::Char('4')));
    app.handle_event(key(KeyCode::Char('3')));
    app.tick();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let lines = buffer_lines(terminal.backend().buffer());
    let steam = find_text_position(&lines[2..], "steam").expect("expected steam");
    let steam = (steam.0, steam.1 + 2);
    let has_sprite_aura = (steam.1.saturating_sub(8)..steam.1).any(|row| {
        (steam.0..steam.0.saturating_add(10))
            .any(|column| terminal.backend().buffer()[(column, row)].bg != ATLAS_BG)
    });

    assert!(
        has_sprite_aura,
        "newly created tile should carry an anchored sprite glow, not a tinted label/card boundary:\n{}",
        lines.join("\n")
    );
}

#[test]
fn crafted_result_uses_a_larger_living_output_chamber() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    app.handle_event(key(KeyCode::Char('4')));
    app.handle_event(key(KeyCode::Char('3')));
    app.tick();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let lines = buffer_lines(terminal.backend().buffer());
    let result = find_text_position_in_region(&lines, "steam", 60, 8, 40, 15)
        .expect("expected steam to appear in the workbench result chamber");
    let graphic_rows = count_graphic_rows_above_label(&lines, result, 8, 4);
    let result_bg = terminal.backend().buffer()[(result.0, result.1.saturating_sub(2))].bg;

    assert!(
        graphic_rows >= 5,
        "crafted result needs a larger readable object moment, not the same tiny tile glyph; rows={graphic_rows}\n{}",
        lines.join("\n")
    );
    assert_ne!(
        result_bg,
        WORKBENCH_SLOT_BG,
        "newly crafted output chamber should feel alive instead of using the flat empty socket background:\n{}",
        lines.join("\n")
    );
}

#[test]
fn crafted_result_sprite_sits_inside_socket_instead_of_filling_the_chamber() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    app.handle_event(key(KeyCode::Char('4')));
    app.handle_event(key(KeyCode::Char('3')));
    app.tick();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let lines = buffer_lines(terminal.backend().buffer());
    let result = find_text_position_in_region(&lines, "steam", 60, 8, 40, 15)
        .expect("expected steam to appear in the workbench result chamber");
    let graphic_rows = count_graphic_rows_above_label(&lines, result, 8, 4);

    assert!(
        (5..=7).contains(&graphic_rows),
        "crafted output should be larger than an ingredient but still sit inside its socket instead of filling/cropping the chamber; rows={graphic_rows}\n{}",
        lines.join("\n")
    );
}

#[test]
fn crafted_result_visible_pixels_are_centered_over_the_label() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    app.handle_event(key(KeyCode::Char('4')));
    app.handle_event(key(KeyCode::Char('3')));
    app.tick();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let lines = buffer_lines(terminal.backend().buffer());
    let result = find_text_position_in_region(&lines, "steam", 60, 8, 40, 15)
        .expect("expected steam to appear in the workbench result chamber");
    let bounds = graphic_bounds_above_label(&lines, result, 8, 4)
        .expect("expected visible result sprite pixels above the label");
    let sprite_center = (bounds.0 + bounds.1) / 2;
    let label_center = result.0 as usize + "steam".len() / 2;
    let drift = sprite_center.abs_diff(label_center);

    assert!(
        drift <= 2,
        "crafted output sprite should be visually centered over its label, not shoved into the socket edge; bounds={bounds:?}, label={result:?}, drift={drift}\n{}",
        lines.join("\n")
    );
}

#[test]
fn successful_recipe_keeps_ingredient_context_while_result_is_alive() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    app.handle_event(key(KeyCode::Char('4')));
    app.handle_event(key(KeyCode::Char('3')));
    app.tick();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let lines = buffer_lines(terminal.backend().buffer());

    assert!(
        find_text_position_in_region(&lines, "water", 60, 8, 35, 15).is_some(),
        "successful crafting should keep the left ingredient visible while the result is alive:\n{}",
        lines.join("\n")
    );
    assert!(
        find_text_position_in_region(&lines, "fire", 60, 8, 35, 15).is_some(),
        "successful crafting should keep the right ingredient visible while the result is alive:\n{}",
        lines.join("\n")
    );
    assert!(
        find_text_position_in_region(&lines, "steam", 60, 8, 35, 15).is_some(),
        "successful crafting should show the created result in the same device:\n{}",
        lines.join("\n")
    );
}

#[test]
fn filled_ingredient_socket_uses_material_tinted_bed() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    app.handle_event(key(KeyCode::Char('4')));

    terminal.draw(|frame| app.render(frame)).unwrap();
    let lines = buffer_lines(terminal.backend().buffer());
    let water = find_text_position_in_region(&lines, "water", 60, 8, 35, 15)
        .expect("expected selected water to be seated in the workbench");
    let socket_bg = terminal.backend().buffer()[(water.0, water.1.saturating_sub(2))].bg;

    assert_ne!(
        socket_bg,
        WORKBENCH_SLOT_BG,
        "filled ingredient sockets should have material-tinted seating, not a flat boundary block:\n{}",
        lines.join("\n")
    );
}

#[test]
fn tab_keeps_the_single_combined_recipe_book_active() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let before = buffer_lines(terminal.backend().buffer());
    let before_text = before.join("\n");
    assert!(
        before_text.contains("4 / 755"),
        "the default game should expose one combined book with the union catalog total:\n{before_text}"
    );
    assert!(
        before_text.contains("recipe book")
            && !before_text.contains("LA1")
            && !before_text.contains("LA2"),
        "the UI should present one combined recipe book, not separate LA1/LA2 books:\n{before_text}"
    );

    app.handle_event(key(KeyCode::Tab));

    terminal.draw(|frame| app.render(frame)).unwrap();
    let after = buffer_lines(terminal.backend().buffer());
    let after_text = after.join("\n");
    assert!(
        after_text.contains("4 / 755") && after_text.contains("recipe book"),
        "tab should not switch away from the single combined recipe book:\n{after_text}"
    );
    assert!(
        !after_text.contains("Little Alchemy 2") && !after_text.contains("switch"),
        "there should be no second catalog or switch control once books are combined:\n{after_text}"
    );
}

#[test]
fn initial_screen_shows_one_recipe_book_not_two_catalog_books() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let text = buffer_to_text(terminal.backend().buffer());

    assert!(
        text.contains("recipe book"),
        "the progress rail should name the single combined recipe book:\n{text}"
    );
    assert!(
        !text.contains("LA1") && !text.contains("LA2") && !text.contains("catalog shelf"),
        "separate catalog book controls should be gone:\n{text}"
    );
}

#[test]
fn initial_screen_reads_like_an_atlas_not_a_debug_dashboard() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let text = buffer_to_text(terminal.backend().buffer());

    assert!(
        text.contains("atlas"),
        "expected user-facing atlas label, got: {text}"
    );
    assert!(
        text.contains("crafting table"),
        "expected the right rail to read as a Minecraft-like crafting device, got: {text}"
    );
    assert!(
        text.contains("recipe"),
        "expected the workbench to define the recipe surface, got: {text}"
    );
    assert!(
        text.contains("recipe book") && !text.contains("catalog shelf") && !text.contains("switch"),
        "expected one combined recipe book rail instead of separate catalog switching controls, got: {text}"
    );
    assert!(
        text.contains("workbench"),
        "expected the right rail to describe the crafting device, got: {text}"
    );
    assert!(
        !text.contains("craft device"),
        "right rail should name the actual Minecraft-like surface, not a vague debug device label: {text}"
    );
    assert!(
        text.contains("ingredient") && text.contains("result"),
        "expected workbench slots to define ingredients and result, got: {text}"
    );
    assert!(
        text.contains("+") && text.contains("="),
        "expected workbench to visualize ingredient + ingredient = result, got: {text}"
    );
    assert!(
        text.contains("▓") && text.contains("▄"),
        "expected workbench to read as a pixel-textured crafting table, got: {text}"
    );
    assert!(
        !text.contains("╔") && !text.contains("╚"),
        "workbench should not rely on terminal box outlines: {text}"
    );
    assert!(
        text.matches('▓').count() < 80 && text.matches('▒').count() < 100,
        "workbench texture should be material detail, not a noisy wall: {text}"
    );
    assert!(
        !text.contains("░"),
        "workbench and progress surfaces should avoid noisy stipple fill: {text}"
    );
    assert!(
        !text.contains("█"),
        "tiny early progress should not render as a stray block artifact: {text}"
    );
    assert!(
        !text.contains("CONTROL BLOCKS / UI"),
        "debug-y control heading should not be visible: {text}"
    );
    assert!(
        !text.contains("SLOTS / FRAMES / PANELS"),
        "debug-y slot heading should not be visible: {text}"
    );
    assert!(
        !text.contains("ingred..."),
        "crafting table labels should fit cleanly without truncated debug-looking text: {text}"
    );
    assert!(
        !text.contains("ingr..."),
        "crafting table labels should not collapse into vague truncated text: {text}"
    );
    assert!(
        text.contains("ingredient + ingredient = result"),
        "empty workbench sockets should be labeled as a clear crafting recipe, got: {text}"
    );
}

#[test]
fn header_reads_as_a_sprite_hud_not_plain_status_text() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let lines = buffer_lines(terminal.backend().buffer());
    let header = lines.iter().take(2).cloned().collect::<Vec<_>>().join("\n");

    assert!(
        header.contains("▛")
            && header.contains("▜")
            && header.contains("◆")
            && header.contains("▣"),
        "top HUD should use a framed sprite plaque for title and stats, not plain prose:\n{header}"
    );
    assert!(
        header.contains("LITTLE ALCHEMY") && header.contains("4 / 755"),
        "sprite HUD should preserve an uppercase fantasy title and combined count stats:\n{header}"
    );
}

#[test]
fn header_status_sits_inside_a_framed_plaque_not_bare_text() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let lines = buffer_lines(terminal.backend().buffer());
    let status_row = &lines[1];

    assert!(
        status_row.contains("▌")
            && status_row.contains("▐")
            && status_row.contains("crafting table workbench"),
        "the subtitle row should sit inside a framed sprite plaque instead of floating on the backdrop:\n{status_row}"
    );
}

#[test]
fn header_carries_a_clear_fantasy_tui_badge() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let lines = buffer_lines(terminal.backend().buffer());
    let header = lines.iter().take(2).cloned().collect::<Vec<_>>().join("\n");
    let logo_top = find_text_position(&lines[..2], "▛▀TUI▀▜").expect("expected top of TUI logo");
    let logo_bottom =
        find_text_position(&lines[..2], "▙▄▄✦▄▄▟").expect("expected bottom of TUI logo");
    let title = find_text_position(&lines[..2], "LITTLE ALCHEMY").expect("expected title");

    assert_eq!(
        logo_top.0, logo_bottom.0,
        "header TUI logo should be a real two-row tile with aligned top and bottom, not separate centered text fragments:\n{header}"
    );
    assert!(
        title.0 > logo_top.0 + 8
            && !header.contains("▌ TUI ▐")
            && !header.contains("little alchemy"),
        "header should use a clear uppercase fantasy title after the TUI tile, not a text chip or lowercase prose:\n{header}"
    );
}

#[test]
fn header_logo_reads_like_a_centered_banner_on_wide_terminals() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let lines = buffer_lines(terminal.backend().buffer());
    let title_row = &lines[0];
    let left = first_non_space_column(title_row).expect("header title should render");
    let right = last_non_space_column(title_row).expect("header title should render");
    let right_pad = title_row.chars().count().saturating_sub(right + 1);

    assert!(
        left >= 4 && right_pad >= 4 && left.abs_diff(right_pad) <= 3,
        "wide terminals should center the fantasy title banner instead of pinning it to the left:\n{title_row}"
    );
}

#[test]
fn progress_is_a_compact_chip_not_a_tall_empty_panel() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let lines = buffer_lines(terminal.backend().buffer());
    let progress_row = lines
        .iter()
        .position(|line| line.contains("progress"))
        .expect("expected progress chip label");
    let ready_row = lines
        .iter()
        .position(|line| line.contains("ready"))
        .expect("expected compact progress status");

    assert!(
        ready_row.saturating_sub(progress_row) <= 2,
        "progress should be a compact stat chip, not a tall vertical panel:\n{}",
        lines.join("\n")
    );
}

#[test]
fn backdrop_uses_layered_pixel_scene_not_a_flat_fill() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let buffer = terminal.backend().buffer();
    let mut backgrounds = Vec::new();
    for row in 0..buffer.area.height {
        for column in 0..buffer.area.width {
            let cell = &buffer[(column, row)];
            if cell.symbol() == " " && !backgrounds.contains(&cell.bg) {
                backgrounds.push(cell.bg);
            }
        }
    }

    assert!(
        backgrounds.len() >= 5,
        "background should read as a layered pixel scene, not a single flat terminal fill: {backgrounds:?}"
    );
}

#[test]
fn backdrop_has_center_stage_glow_not_a_uniform_wall() {
    let backend = TestBackend::new(160, 50);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let buffer = terminal.backend().buffer();
    let sample_row = buffer.area.height.saturating_mul(4) / 5;
    let side = average_background_brightness(buffer, 8, sample_row, 30, 1);
    let center = average_background_brightness(buffer, 70, sample_row, 20, 1);

    assert!(
        center > side + 4,
        "background should have an intentional central stage glow instead of a uniform star wall; side={side}, center={center}"
    );
}

#[test]
fn backdrop_specks_are_sparse_enough_to_read_as_atmosphere() {
    let backend = TestBackend::new(160, 50);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let buffer = terminal.backend().buffer();
    let mut specks = 0usize;
    let mut open_background = 0usize;
    for row in 0..buffer.area.height {
        for column in 0..buffer.area.width {
            let cell = &buffer[(column, row)];
            if cell.symbol() != " " {
                continue;
            }
            open_background += 1;
            if matches!(
                cell.bg,
                ratatui::style::Color::Rgb(28, 31, 46) | ratatui::style::Color::Rgb(56, 62, 88)
            ) {
                specks += 1;
            }
        }
    }

    assert!(
        specks * 100 <= open_background * 4,
        "background motes should be sparse polish, not visual noise; specks={specks}, open_background={open_background}"
    );
}

#[test]
fn wide_layout_wraps_panels_in_one_workshop_shell() {
    let backend = TestBackend::new(160, 50);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let text = buffer_to_text(terminal.backend().buffer());

    assert!(
        text.contains("✦ workshop"),
        "wide layouts should read as one responsive application shell with internal panels, not floating cards:\n{text}"
    );
}

#[test]
fn same_app_redraws_cleanly_across_dynamic_resize_steps() {
    let mut app = App::new();
    app.reveal_elements_for_preview(&[
        "Dust", "Energy", "Lava", "Mud", "Rain", "Sea", "Steam", "Cloud", "Plant", "Stone",
        "Metal", "Sand", "Sky", "Storm", "Glass", "Life", "Human", "Tool",
    ]);

    for (width, height, expect_shell) in [(64, 40, false), (100, 24, false), (160, 50, true)] {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| app.render(frame)).unwrap();
        let text = buffer_to_text(terminal.backend().buffer());

        assert!(
            text.contains("recipe book")
                && text.contains("✦ atlas")
                && text.contains("✦ recipe table"),
            "dynamic resize to {width}x{height} should preserve all major panels:\n{text}"
        );
        assert_eq!(
            text.contains("✦ workshop"),
            expect_shell,
            "dynamic resize to {width}x{height} should toggle the responsive outer shell at the wide breakpoint:\n{text}"
        );
    }
}
#[test]
fn narrow_layout_keeps_recipe_book_and_table_close_to_the_atlas() {
    let backend = TestBackend::new(64, 40);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();
    app.reveal_elements_for_preview(&["Steam", "Mud", "Lava", "Rain"]);

    terminal.draw(|frame| app.render(frame)).unwrap();
    let lines = buffer_lines(terminal.backend().buffer());
    let recipe_row = lines
        .iter()
        .position(|line| line.contains("recipe table"))
        .expect("expected recipe table in narrow layout");
    let atlas_labels = ["lava", "mud", "rain", "steam"]
        .iter()
        .filter_map(|label| find_text_position(&lines, label))
        .map(|(_, row)| row as usize)
        .max()
        .expect("expected revealed atlas labels");

    assert!(
        recipe_row.saturating_sub(atlas_labels) <= 8,
        "narrow terminals should not leave a large dead band between the atlas and recipe table:\n{}",
        lines.join("\n")
    );
}

#[test]
fn narrow_layout_keeps_the_recipe_book_tile_visible() {
    let backend = TestBackend::new(64, 40);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();
    app.reveal_elements_for_preview(&["Steam", "Mud", "Lava", "Rain"]);

    terminal.draw(|frame| app.render(frame)).unwrap();
    let text = buffer_to_text(terminal.backend().buffer());
    assert!(
        text.contains("recipe book") && text.contains("combined") && text.contains("▄▗▖"),
        "narrow layouts should preserve the single recipe-book control instead of collapsing it away:\n{text}"
    );
}

#[test]
fn narrow_layout_centers_the_recipe_book_panel_instead_of_stretching_it_wall_to_wall() {
    let backend = TestBackend::new(64, 40);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();
    app.reveal_elements_for_preview(&["Steam", "Mud", "Lava", "Rain"]);

    terminal.draw(|frame| app.render(frame)).unwrap();
    let lines = buffer_lines(terminal.backend().buffer());
    let progress_row = lines
        .iter()
        .find(|line| line.contains("progress"))
        .expect("expected progress panel row");
    let left = first_non_space_column(progress_row).expect("expected visible panel");
    let right = last_non_space_column(progress_row).expect("expected visible panel");
    let right_pad = progress_row.chars().count().saturating_sub(right + 1);

    assert!(
        left >= 6 && right_pad >= 6,
        "narrow layouts should keep the recipe-book panel compact and centered instead of stretching it full width:\n{}",
        lines.join("\n")
    );
}
#[test]
fn tall_layout_balances_the_scene_in_the_middle_of_the_chamber() {
    let backend = TestBackend::new(100, 48);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();
    app.reveal_elements_for_preview(&["Steam", "Mud", "Lava", "Rain", "Sea", "Cloud"]);

    terminal.draw(|frame| app.render(frame)).unwrap();
    let lines = buffer_lines(terminal.backend().buffer());
    let progress = find_text_position(&lines, "progress").expect("expected progress title");
    let atlas = find_text_position(&lines, "atlas").expect("expected atlas title");
    let recipe = find_text_position(&lines, "recipe table").expect("expected recipe table title");

    assert!(
        progress.1.abs_diff(atlas.1) <= 1 && recipe.1.abs_diff(atlas.1) <= 1 && atlas.1 <= 10,
        "tall layouts should keep the three main panels pinned to a shared top band instead of letting one drift lower into the background:\n{}",
        lines.join("\n")
    );
}

#[test]
fn atlas_uses_the_same_framed_title_bar_as_the_other_fantasy_panels() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let text = buffer_to_text(terminal.backend().buffer());

    assert!(
        text.contains("✦ atlas"),
        "the atlas should be framed as a titled fantasy panel instead of a loose label:\n{text}"
    );
}

#[test]
fn fantasy_panel_titles_are_embedded_in_two_sided_borders() {
    let backend = TestBackend::new(160, 50);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();
    app.reveal_elements_for_preview(&[
        "Dust", "Energy", "Lava", "Mud", "Rain", "Sea", "Steam", "Cloud", "Plant", "Stone",
        "Metal", "Sand",
    ]);

    terminal.draw(|frame| app.render(frame)).unwrap();
    let lines = buffer_lines(terminal.backend().buffer());

    for title in ["workshop", "progress", "atlas", "recipe table"] {
        let row = lines
            .iter()
            .find(|line| line.contains(title))
            .unwrap_or_else(|| panic!("expected {title} title:\n{}", lines.join("\n")));
        assert_eq!(
            row.matches(title).count(),
            1,
            "a fantasy frame should own one clear title, not repeat a loose label:\n{row}"
        );
        assert!(
            row_has_two_sided_title_border(row, title),
            "title `{title}` should be embedded into a continuous two-sided border like the CLI reference, not float inside a filled bar:\n{row}\n\n{}",
            lines.join("\n")
        );
    }
}

#[test]
fn recipe_hint_lives_in_workbench_not_header_prose() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let lines = buffer_lines(terminal.backend().buffer());
    let header = lines.iter().take(2).cloned().collect::<Vec<_>>().join("\n");
    let body = lines.iter().skip(2).cloned().collect::<Vec<_>>().join("\n");

    assert!(
        !header.contains("drop 2 elements") && !header.contains("ingredient + ingredient"),
        "header should stay quiet; recipe explanation belongs inside the workbench device:\n{header}"
    );
    assert!(
        body.contains("ingredient + ingredient = result") && !body.contains("drop 2 elements"),
        "workbench body should define the recipe like a crafted table, not a prose hint:\n{body}"
    );
}

#[test]
fn right_rail_defines_a_functional_workbench_device() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let text = buffer_to_text(terminal.backend().buffer());

    assert!(
        text.contains("recipe book"),
        "progress rail should read as the one combined recipe book, got: {text}"
    );
    assert!(
        !text.contains("catalog shelf") && !text.contains("switch"),
        "catalog switch controls should be removed after combining the books, got: {text}"
    );
    assert!(
        text.contains("crafting table"),
        "the right rail should read as a Minecraft-like crafting table, got: {text}"
    );
    assert!(
        text.contains("ingredient + ingredient = result"),
        "workbench sockets should use clear recipe labels, got: {text}"
    );
    assert!(
        !text.contains("drop 2 elements")
            && !text.contains("element books / switch books")
            && !text.contains("craft table recipe workbench"),
        "the workbench should avoid long debug-like prose labels, got: {text}"
    );
}

#[test]
fn recipe_book_rail_does_not_render_a_catalog_switch_arrow() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let text = buffer_to_text(terminal.backend().buffer());

    assert!(
        text.contains("recipe book")
            && !text.contains('⇆')
            && !text.contains("LA1")
            && !text.contains("LA2"),
        "single-book rail should not visually advertise switching between two books:\n{text}"
    );
}

#[test]
fn workbench_reads_as_a_crafted_table_surface_not_a_plain_panel() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let text = buffer_to_text(terminal.backend().buffer());

    assert!(
        text.contains("recipe table")
            && text.contains("ingredient + ingredient = result")
            && text.contains("result"),
        "workbench should explain the recipe through the device labels, not debug text inside sockets:\n{text}"
    );
    assert!(
        !text.contains("▣ input") && !text.contains("▣ output"),
        "empty sockets should render as pixel socket plates, not text-heavy debug boxes:\n{text}"
    );
    assert!(
        text.matches('◆').count() >= 3 && text.matches('▛').count() >= 4,
        "workbench should show socket-like crafted glyphs instead of plain bordered boxes:\n{text}"
    );
}

#[test]
fn empty_workbench_sockets_are_beveled_pixel_slots_not_diamond_placeholders() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let text = buffer_to_text(terminal.backend().buffer());

    assert!(
        !text.contains("◇◇◇") && !text.contains("◇◆◇"),
        "empty crafting sockets should look like beveled pixel slots, not placeholder diamond glyphs:\n{text}"
    );
    assert!(
        text.matches('▛').count() >= 4
            && text.matches('▜').count() >= 4
            && text.matches('▙').count() >= 4
            && text.matches('▟').count() >= 4,
        "empty crafting sockets should use repeated beveled pixel corners like a crafted table device:\n{text}"
    );
}

#[test]
fn workbench_recipe_label_sits_in_an_inlaid_nameplate_not_on_raw_planks() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let text = buffer_to_text(terminal.backend().buffer());

    assert!(
        !text.contains("▓ingredient") && !text.contains("result▓"),
        "recipe label should have breathing room inside the crafted table, not collide with plank texture:\n{text}"
    );
}

#[test]
fn workbench_socket_labels_define_ingredients_not_placeholder_items() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let text = buffer_to_text(terminal.backend().buffer());

    assert!(
        !text.contains("item a") && !text.contains("item b"),
        "workbench sockets should be named by their crafting role, not placeholder item labels:\n{text}"
    );
    assert!(
        text.matches("ingredient").count() >= 3 && text.contains("result"),
        "workbench should define two ingredient sockets and one result socket:\n{text}"
    );
}

#[test]
fn workbench_surface_has_table_legs_not_debug_side_noise() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let text = buffer_to_text(terminal.backend().buffer());

    assert!(
        text.matches('▌').count() >= 2 && text.matches('▐').count() >= 2,
        "crafting table should have readable blocky side posts/legs, not only loose stippled edges:\n{text}"
    );
    assert!(
        text.matches('▓').count() < 42,
        "crafting table texture should be deliberate chunky wood detail, not noisy vertical stipple:\n{text}"
    );
}

#[test]
fn atlas_tiles_have_their_own_fantasy_picture_frames() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let lines = buffer_lines(terminal.backend().buffer());
    let air = find_text_position(&lines, "air").expect("expected air tile");
    let framed_glyphs = count_matching_chars_in_region(
        &lines,
        air.0.saturating_sub(3),
        air.1.saturating_sub(8),
        air.0.saturating_add(5),
        air.1,
        &['▌', '▐', '▘', '▝', '▖', '▗'],
    );

    assert!(
        framed_glyphs >= 10,
        "atlas cards should have their own fantasy picture frames instead of blending straight into the panel:\n{}",
        lines.join("\n")
    );
}

#[test]
fn atlas_surfaces_do_not_use_heavy_box_frames() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let text = buffer_to_text(terminal.backend().buffer());

    assert!(
        !text.contains("┌atlas") && !text.contains("┌actions"),
        "atlas and action regions should feel like open sprite sheets, not boxed cards:\n{text}"
    );
}

#[test]
fn discovery_feedback_avoids_noisy_free_floating_particles() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    app.handle_event(key(KeyCode::Char('4')));
    app.handle_event(key(KeyCode::Char('3')));
    app.tick();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let text = buffer_to_text(terminal.backend().buffer());

    assert!(
        !text.contains("~") && !text.contains("*"),
        "discovery feedback should be a clean living sprite highlight, not noisy particle scatter:\n{text}"
    );
}

#[test]
fn discovery_aura_is_localized_motion_not_free_particle_noise() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    app.handle_event(key(KeyCode::Char('4')));
    app.handle_event(key(KeyCode::Char('3')));
    app.tick();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let first_lines = buffer_lines(terminal.backend().buffer());
    let steam = find_last_text_position(&first_lines, "steam").expect("expected steam");
    let aura_probe = (steam.0, steam.1.saturating_sub(7));
    let first_bg = terminal.backend().buffer()[aura_probe].bg;
    let first_signature = sprite_cell_signature(terminal.backend().buffer(), steam, 8, 8);
    let first_text = buffer_to_text(terminal.backend().buffer());

    app.tick();
    terminal.draw(|frame| app.render(frame)).unwrap();
    let second_bg = terminal.backend().buffer()[aura_probe].bg;
    let second_signature = sprite_cell_signature(terminal.backend().buffer(), steam, 8, 8);
    let second_text = buffer_to_text(terminal.backend().buffer());

    assert_ne!(
        first_bg, ATLAS_BG,
        "new element should have a local aura around the sprite, not only a plain label:\n{first_text}"
    );
    assert_eq!(
        first_bg, second_bg,
        "new element aura should not blink the whole object bed between debug colors"
    );
    assert_ne!(
        first_signature, second_signature,
        "new element should feel alive through localized sprite pixels, not by flashing the chamber"
    );
    assert!(
        !first_text.contains("~")
            && !first_text.contains("*")
            && !second_text.contains("~")
            && !second_text.contains("*"),
        "birth feedback should not use noisy free-floating particle glyphs"
    );
}

#[test]
fn created_element_gets_a_soft_halo_around_the_sprite_pixels() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    app.handle_event(key(KeyCode::Char('4')));
    app.handle_event(key(KeyCode::Char('3')));
    app.tick();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let lines = buffer_lines(terminal.backend().buffer());
    let steam = find_last_text_position(&lines, "steam").expect("expected steam");
    let halo_cells = empty_glow_cells_around_label(
        terminal.backend().buffer(),
        steam,
        8,
        8,
        STEAM_BIRTH_HALO_BG,
    );

    assert!(
        halo_cells >= 12,
        "new element should have a soft local halo around the sprite body, not only recolored sprite pixels; halo_cells={halo_cells}\n{}",
        lines.join("\n")
    );
}

#[test]
fn created_element_glow_does_not_fill_the_sprite_with_a_solid_bed() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    app.handle_event(key(KeyCode::Char('4')));
    app.handle_event(key(KeyCode::Char('3')));
    app.tick();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let lines = buffer_lines(terminal.backend().buffer());
    let steam = find_text_position(&lines[2..], "steam").expect("expected atlas steam tile");
    let steam = (steam.0, steam.1 + 2);
    let painted_pixels = colored_sprite_pixels_above_label(
        terminal.backend().buffer(),
        steam,
        8,
        8,
        STEAM_BIRTH_AURA_BG,
    );

    assert!(
        painted_pixels <= 4,
        "new element glow should be a halo/glint around the sprite, not a solid rectangular color bed; painted_pixels={painted_pixels}\n{}",
        lines.join("\n")
    );
}

#[test]
fn discovery_highlight_color_is_stable_across_ticks() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    app.handle_event(key(KeyCode::Char('4')));
    app.handle_event(key(KeyCode::Char('3')));
    app.tick();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let first_lines = buffer_lines(terminal.backend().buffer());
    let first_steam = find_last_text_position(&first_lines, "steam").expect("expected steam");
    let first_color = terminal.backend().buffer()[(first_steam.0, first_steam.1)].fg;

    app.tick();
    terminal.draw(|frame| app.render(frame)).unwrap();
    let second_lines = buffer_lines(terminal.backend().buffer());
    let second_steam = find_last_text_position(&second_lines, "steam").expect("expected steam");
    let second_color = terminal.backend().buffer()[(second_steam.0, second_steam.1)].fg;

    assert_eq!(
        first_color, second_color,
        "newly created elements should glow steadily, not blink between debug colors"
    );
}

#[test]
fn discovery_glint_moves_across_the_sprite_not_top_to_bottom() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    app.handle_event(key(KeyCode::Char('4')));
    app.handle_event(key(KeyCode::Char('3')));
    app.tick();

    let mut highlighted_rows = Vec::new();
    for _ in 0..4 {
        terminal.draw(|frame| app.render(frame)).unwrap();
        let lines = buffer_lines(terminal.backend().buffer());
        let steam = find_last_text_position(&lines, "steam").expect("expected steam");
        highlighted_rows.extend(highlight_rows_above_label(
            terminal.backend().buffer(),
            steam,
            8,
            8,
            ratatui::style::Color::Rgb(195, 172, 89),
        ));
        app.tick();
    }

    let min = highlighted_rows.iter().min().copied().unwrap_or_default();
    let max = highlighted_rows.iter().max().copied().unwrap_or_default();
    assert!(
        max.saturating_sub(min) <= 2,
        "new element glint should shimmer around the object core, not blink from top to bottom: {highlighted_rows:?}"
    );
}

#[test]
fn drag_overlay_is_a_stable_element_ghost_not_a_debug_blink() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let initial_lines = buffer_lines(terminal.backend().buffer());
    let water = find_text_position(&initial_lines, "water").expect("expected water tile");
    let fire = find_text_position(&initial_lines, "fire").expect("expected fire tile");

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

    terminal.draw(|frame| app.render(frame)).unwrap();
    let first_text = buffer_to_text(terminal.backend().buffer());
    let first_lines = buffer_lines(terminal.backend().buffer());
    let first_water = find_last_text_position(&first_lines, "water").expect("expected water");
    let first_color = terminal.backend().buffer()[(first_water.0, first_water.1)].fg;

    app.tick();
    terminal.draw(|frame| app.render(frame)).unwrap();
    let second_text = buffer_to_text(terminal.backend().buffer());
    let second_lines = buffer_lines(terminal.backend().buffer());
    let second_water = find_last_text_position(&second_lines, "water").expect("expected water");
    let second_color = terminal.backend().buffer()[(second_water.0, second_water.1)].fg;

    assert!(
        !first_text.contains("dragging water") && !second_text.contains("dragging water"),
        "dragging should feel like a held element, not a debug status label:\n{first_text}\n{second_text}"
    );
    assert_eq!(
        first_color, second_color,
        "dragging should not blink between debug colors"
    );
}

#[test]
fn drag_overlay_has_stable_shadow_without_debug_particles() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let initial_lines = buffer_lines(terminal.backend().buffer());
    let water = find_text_position(&initial_lines, "water").expect("expected water tile");
    let fire = find_text_position(&initial_lines, "fire").expect("expected fire tile");

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

    terminal.draw(|frame| app.render(frame)).unwrap();
    let first_lines = buffer_lines(terminal.backend().buffer());
    let first_bg = terminal.backend().buffer()[(fire.0, fire.1.saturating_sub(2))].bg;
    let first_text = buffer_to_text(terminal.backend().buffer());

    app.tick();
    terminal.draw(|frame| app.render(frame)).unwrap();
    let second_bg = terminal.backend().buffer()[(fire.0, fire.1.saturating_sub(2))].bg;
    let second_text = buffer_to_text(terminal.backend().buffer());

    assert_ne!(
        first_bg,
        ATLAS_BG,
        "drag overlay should read as a held object with a stable shadow:\n{}",
        first_lines.join("\n")
    );
    assert_eq!(first_bg, second_bg, "drag shadow should not blink");
    assert!(
        !first_text.contains("~")
            && !first_text.contains("*")
            && !second_text.contains("~")
            && !second_text.contains("*"),
        "drag shadow should avoid noisy particle glyphs"
    );
}

#[test]
fn drag_overlay_does_not_paint_a_rectangular_card_boundary() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let initial_lines = buffer_lines(terminal.backend().buffer());
    let water = find_text_position(&initial_lines, "water").expect("expected water tile");
    let fire = find_text_position(&initial_lines, "fire").expect("expected fire tile");
    let overlay_corner = (fire.0.saturating_sub(5), fire.1.saturating_sub(8));
    let backdrop_bg = terminal.backend().buffer()[overlay_corner].bg;

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

    terminal.draw(|frame| app.render(frame)).unwrap();
    let corner_bg = terminal.backend().buffer()[overlay_corner].bg;
    let text = buffer_to_text(terminal.backend().buffer());

    assert_eq!(
        corner_bg, backdrop_bg,
        "empty corners around the held sprite should preserve the backdrop, not form a rectangular debug card:\n{text}"
    );
}

#[test]
fn drag_overlay_holds_a_stable_sprite_frame_across_animation_ticks() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let initial_lines = buffer_lines(terminal.backend().buffer());
    let water = find_text_position(&initial_lines, "water").expect("expected water tile");
    let fire = find_text_position(&initial_lines, "fire").expect("expected fire tile");

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

    terminal.draw(|frame| app.render(frame)).unwrap();
    let first_lines = buffer_lines(terminal.backend().buffer());
    let first_water = find_last_text_position(&first_lines, "water").expect("expected water");
    let first_signature = sprite_cell_signature(terminal.backend().buffer(), first_water, 8, 8);

    for _ in 0..7 {
        app.tick();
    }
    terminal.draw(|frame| app.render(frame)).unwrap();
    let second_lines = buffer_lines(terminal.backend().buffer());
    let second_water = find_last_text_position(&second_lines, "water").expect("expected water");
    let second_signature = sprite_cell_signature(terminal.backend().buffer(), second_water, 8, 8);

    assert_eq!(
        first_signature, second_signature,
        "dragging should hold the picked-up element frame instead of blinking while the pointer moves"
    );
}

#[test]
fn starting_elements_render_as_a_compact_atlas_row() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let lines = buffer_lines(terminal.backend().buffer());

    let air = find_text_position(&lines, "air").expect("expected air tile");
    let water = find_text_position(&lines, "water").expect("expected water tile");

    assert_eq!(
        air.1,
        water.1,
        "the four starting elements should read as a compact atlas row, not oversized cards:\n{}",
        lines.join("\n")
    );
}

#[test]
fn wide_initial_atlas_scales_starter_sprites_for_large_viewports() {
    let backend = TestBackend::new(160, 50);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let lines = buffer_lines(terminal.backend().buffer());
    let air = find_text_position(&lines, "air").expect("expected air tile");
    let rows = graphic_row_count_above_label(&lines, air, 10, 8);
    let width = graphic_span_width_above_label(&lines, air, 12, 8);

    assert!(
        rows >= 5 && width >= 11,
        "large viewports should scale starter sprites up instead of rendering tiny fixed icons; rows={rows}, width={width}\n{}",
        lines.join("\n")
    );
}

#[test]
fn wide_viewport_keeps_panel_positions_stable_as_the_palette_grows() {
    let backend = TestBackend::new(160, 50);
    let mut small_terminal = Terminal::new(backend).unwrap();
    let mut small = App::new();
    small_terminal.draw(|frame| small.render(frame)).unwrap();
    let small_text = buffer_to_text(small_terminal.backend().buffer());
    let small_lines = buffer_lines(small_terminal.backend().buffer());
    let small_workshop =
        find_text_position(&small_lines, "workshop").expect("expected workshop shell");
    let small_atlas = find_text_position(&small_lines, "atlas").expect("expected atlas title");
    let small_recipe =
        find_text_position(&small_lines, "recipe table").expect("expected recipe table title");

    let backend = TestBackend::new(160, 50);
    let mut grown_terminal = Terminal::new(backend).unwrap();
    let mut grown = App::new();
    grown.reveal_elements_for_preview(&["Lava", "Cloud", "Steam"]);
    grown_terminal.draw(|frame| grown.render(frame)).unwrap();
    let grown_text = buffer_to_text(grown_terminal.backend().buffer());
    let grown_lines = buffer_lines(grown_terminal.backend().buffer());
    let grown_workshop =
        find_text_position(&grown_lines, "workshop").expect("expected workshop shell");
    let grown_atlas = find_text_position(&grown_lines, "atlas").expect("expected atlas title");
    let grown_recipe =
        find_text_position(&grown_lines, "recipe table").expect("expected recipe table title");

    assert_eq!(
        (small_workshop.1, small_atlas.1, small_recipe.1),
        (grown_workshop.1, grown_atlas.1, grown_recipe.1),
        "growing the palette should not make the main panels jump to different rows:\nSMALL\n{small_text}\n\nGROWN\n{grown_text}"
    );
}

#[test]
fn wide_workshop_shell_aligns_panel_titles_on_one_top_band() {
    let backend = TestBackend::new(160, 50);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();
    app.reveal_elements_for_preview(&[
        "Dust", "Energy", "Lava", "Mud", "Rain", "Sea", "Steam", "Cloud", "Plant", "Stone",
        "Metal", "Sand",
    ]);

    terminal.draw(|frame| app.render(frame)).unwrap();
    let lines = buffer_lines(terminal.backend().buffer());
    let progress = find_text_position(&lines, "progress").expect("expected progress title");
    let atlas = find_text_position(&lines, "atlas").expect("expected atlas title");
    let recipe = find_text_position(&lines, "recipe table").expect("expected recipe table title");

    assert!(
        progress.1.abs_diff(atlas.1) <= 1 && recipe.1.abs_diff(atlas.1) <= 1,
        "wide layouts should align the three panel titles along one top band instead of floating side cards lower than the atlas:\n{}",
        lines.join("\n")
    );
}
#[test]
fn short_layout_uses_the_atlas_capacity_before_showing_a_large_empty_body() {
    let backend = TestBackend::new(100, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();
    app.reveal_elements_for_preview(&["Steam", "Mud", "Lava", "Rain", "Sea", "Cloud"]);

    terminal.draw(|frame| app.render(frame)).unwrap();
    let lines = buffer_lines(terminal.backend().buffer());
    let mut row_counts = Vec::<(u16, usize)>::new();
    for row in ["lava", "mud", "rain", "sea", "steam", "cloud"]
        .into_iter()
        .filter_map(|label| find_text_position(&lines, label).map(|(_, row)| row))
    {
        if let Some((_, count)) = row_counts.iter_mut().find(|(existing, _)| *existing == row) {
            *count += 1;
        } else {
            row_counts.push((row, 1));
        }
    }

    assert!(
        row_counts.len() == 2 && row_counts.iter().all(|(_, count)| *count >= 3),
        "short layouts should pack the atlas into dense rows instead of leaving an underfilled tail row:\n{}",
        lines.join("\n")
    );
}

#[test]
fn starter_sprites_render_with_enough_rows_for_pixel_detail() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let lines = buffer_lines(terminal.backend().buffer());
    let fire = find_text_position(&lines, "fire").expect("expected fire tile");
    let start_row = fire.1.saturating_sub(7) as usize;
    let end_row = fire.1.saturating_sub(1) as usize;
    let graphic_rows = lines[start_row..=end_row]
        .iter()
        .filter(|line| {
            let start = fire.0.saturating_sub(5) as usize;
            let end = (fire.0 as usize + 8).min(line.chars().count());
            line.chars()
                .skip(start)
                .take(end.saturating_sub(start))
                .any(|ch| ch != ' ')
        })
        .count();

    assert!(
        graphic_rows >= 5,
        "starter icons need enough terminal rows to carry 16-bit pixel detail:\n{}",
        lines.join("\n")
    );
}

#[test]
fn starter_sprites_stay_compact_inside_tiles() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let lines = buffer_lines(terminal.backend().buffer());

    for label in ["air", "earth", "fire", "water"] {
        let position = find_text_position(&lines, label).expect("expected starter tile");
        let rows = graphic_row_count_above_label(&lines, position, 10, 9);
        assert!(
            rows <= 9,
            "{label} icon should stay compact even when responsive tiles scale up; rows={rows}\n{}",
            lines.join("\n")
        );
    }
}

#[test]
fn starter_sprites_render_with_enough_columns_for_pixel_detail() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let lines = buffer_lines(terminal.backend().buffer());

    for label in ["air", "earth", "fire", "water"] {
        let position = find_text_position(&lines, label).expect("expected starter tile");
        let width = graphic_span_width_above_label(&lines, position, 7, 6);
        assert!(
            width >= 7,
            "{label} should retain enough horizontal sprite columns for 16-bit object definition, width={width}\n{}",
            lines.join("\n")
        );
    }
}

#[test]
fn little_alchemy_one_catalog_has_the_full_wiki_set() {
    let catalog = &GameCatalog::load_all()[0];

    assert_eq!(catalog.title(), "Little Alchemy 1");
    assert_eq!(catalog.total, 589);
    assert_eq!(catalog.base_indices.len(), 4);
    assert_eq!(
        catalog
            .elements
            .iter()
            .filter(|element| element.hidden)
            .count(),
        9
    );

    let base_names: Vec<_> = catalog
        .base_indices
        .iter()
        .map(|index| catalog.canonical_name(*index))
        .collect();
    assert_eq!(base_names, ["Air", "Earth", "Fire", "Water"]);

    let water = catalog.name_to_index.get("water").copied().unwrap();
    let fire = catalog.name_to_index.get("fire").copied().unwrap();
    let outputs = catalog.recipe_outputs(water, fire);
    assert!(
        outputs
            .iter()
            .any(|index| catalog.canonical_name(*index).eq_ignore_ascii_case("steam")),
        "expected water + fire to produce steam"
    );
}

#[test]
fn little_alchemy_two_catalog_has_the_full_local_set() {
    let catalog = &GameCatalog::load_all()[1];

    assert_eq!(catalog.title(), "Little Alchemy 2");
    assert_eq!(catalog.total, 720);
    assert_eq!(catalog.elements.len(), 720);
}

#[test]
fn app_catalog_combines_both_source_books_into_one_recipe_book() {
    let catalog = GameCatalog::load_combined_book();

    assert_eq!(catalog.title(), "Little Alchemy");
    assert_eq!(catalog.total, 755);
    assert_eq!(catalog.elements.len(), 755);
    assert!(catalog.name_to_index.contains_key("godzilla"));
    assert!(catalog.name_to_index.contains_key("alchemist"));

    let water = catalog.name_to_index.get("water").copied().unwrap();
    let fire = catalog.name_to_index.get("fire").copied().unwrap();
    let outputs = catalog.recipe_outputs(water, fire);
    assert!(
        outputs
            .iter()
            .any(|index| catalog.canonical_name(*index).eq_ignore_ascii_case("steam")),
        "combined recipe book should preserve recipes from the source books"
    );
}

#[test]
fn catalog_entries_include_generated_pixel_sprite_paths() {
    let catalog = &GameCatalog::load_all()[0];
    let water = catalog.name_to_index.get("water").copied().unwrap();
    let element = &catalog.elements[water];

    assert_eq!(
        element.pixel_sprite_path,
        PathBuf::from("assets/pixel-sprites/little-alchemy-1/water.png")
    );
    assert_eq!(
        element.icon_path,
        PathBuf::from("assets/icons/little-alchemy-1/water.png")
    );
}

#[test]
fn sprite_lookup_prefers_generated_pixel_sprites() {
    let catalog = &GameCatalog::load_all()[0];
    let water = catalog.name_to_index.get("water").copied().unwrap();
    let element = &catalog.elements[water];

    let source =
        tui_alchemy::sprites::sprite_source_for_element(CatalogKind::LittleAlchemy1, element);

    assert_eq!(
        source,
        SpriteSource::Generated(PathBuf::from(
            "assets/pixel-sprites/little-alchemy-1/water.png"
        ))
    );
}

#[test]
fn missing_generated_sprite_uses_named_placeholder_instead_of_icon_mosaic() {
    let catalog = GameCatalog::from_raw_json(
        CatalogKind::LittleAlchemy1,
        r#"{
            "source": "unit-test",
            "total": 1,
            "elements": [{"name": "Definitely Missing Sprite", "base": true}]
        }"#,
    );
    let element = &catalog.elements[0];

    let source =
        tui_alchemy::sprites::sprite_source_for_element(CatalogKind::LittleAlchemy1, element);

    assert_eq!(source, SpriteSource::NamedPlaceholder);
}

#[test]
fn expanded_object_sprites_are_used_instead_of_fallback_silhouettes() {
    let catalog = &GameCatalog::load_all()[0];
    for slug in [
        "glasses", "clock", "boat", "car", "scissors", "wheel", "blade",
    ] {
        let index = catalog.name_to_index.get(slug).copied().unwrap();
        let element = &catalog.elements[index];

        assert_eq!(
            tui_alchemy::sprites::sprite_source_for_element(CatalogKind::LittleAlchemy1, element),
            SpriteSource::Generated(PathBuf::from(format!(
                "assets/pixel-sprites/little-alchemy-1/{slug}.png"
            ))),
            "{slug} should use an authored generated sprite, not the semantic fallback"
        );
    }
}

#[test]
fn reference_extra_sprites_are_used_instead_of_fallback_silhouettes() {
    let catalog = &GameCatalog::load_all()[0];
    for (name, slug) in [
        ("lizard", "lizard"),
        ("bread", "bread"),
        ("fishing rod", "fishing-rod"),
        ("crystal ball", "crystal-ball"),
        ("butterfly", "butterfly"),
        ("flying fish", "flying-fish"),
    ] {
        let index = catalog.name_to_index.get(name).copied().unwrap();
        let element = &catalog.elements[index];

        assert_eq!(
            tui_alchemy::sprites::sprite_source_for_element(CatalogKind::LittleAlchemy1, element),
            SpriteSource::Generated(PathBuf::from(format!(
                "assets/pixel-sprites/little-alchemy-1/{slug}.png"
            ))),
            "{name} should use an authored generated sprite, not the semantic fallback"
        );
    }
}

#[test]
fn craft_object_sprites_are_used_instead_of_fallback_silhouettes() {
    let catalog = &GameCatalog::load_all()[0];
    for (name, slug) in [("axe", "axe"), ("clay", "clay"), ("pottery", "pottery")] {
        let index = catalog.name_to_index.get(name).copied().unwrap();
        let element = &catalog.elements[index];

        assert_eq!(
            tui_alchemy::sprites::sprite_source_for_element(CatalogKind::LittleAlchemy1, element),
            SpriteSource::Generated(PathBuf::from(format!(
                "assets/pixel-sprites/little-alchemy-1/{slug}.png"
            ))),
            "{name} should use an authored generated sprite, not the semantic fallback"
        );
    }
}

#[test]
fn little_alchemy_two_reference_sprites_are_used_instead_of_fallback_silhouettes() {
    let catalog = &GameCatalog::load_all()[1];
    for (name, slug) in [("shovel", "shovel"), ("knife", "knife")] {
        let index = catalog.name_to_index.get(name).copied().unwrap();
        let element = &catalog.elements[index];

        assert_eq!(
            tui_alchemy::sprites::sprite_source_for_element(CatalogKind::LittleAlchemy2, element),
            SpriteSource::Generated(PathBuf::from(format!(
                "assets/pixel-sprites/little-alchemy-2/{slug}.png"
            ))),
            "{name} should use an authored LA2 generated sprite, not the semantic fallback"
        );
    }
}

#[test]
fn nature_cosmos_sprites_are_used_instead_of_fallback_silhouettes() {
    let catalog = &GameCatalog::load_all()[0];
    for (name, slug) in [
        ("seaweed", "seaweed"),
        ("hay", "hay"),
        ("bacteria", "bacteria"),
        ("wool", "wool"),
        ("cow", "cow"),
        ("horse", "horse"),
        ("rainbow", "rainbow"),
        ("star", "star"),
    ] {
        let index = catalog.name_to_index.get(name).copied().unwrap();
        let element = &catalog.elements[index];

        assert_eq!(
            tui_alchemy::sprites::sprite_source_for_element(CatalogKind::LittleAlchemy1, element),
            SpriteSource::Generated(PathBuf::from(format!(
                "assets/pixel-sprites/little-alchemy-1/{slug}.png"
            ))),
            "{name} should use an authored generated sprite, not the semantic fallback"
        );
    }
}

#[test]
fn little_alchemy_two_lightning_uses_generated_sprite() {
    let catalog = &GameCatalog::load_all()[1];
    let index = catalog.name_to_index.get("lightning").copied().unwrap();
    let element = &catalog.elements[index];

    assert_eq!(
        tui_alchemy::sprites::sprite_source_for_element(CatalogKind::LittleAlchemy2, element),
        SpriteSource::Generated(PathBuf::from(
            "assets/pixel-sprites/little-alchemy-2/lightning.png"
        )),
        "LA2 lightning should use an authored generated sprite, not the semantic fallback"
    );
}

#[test]
fn cosmic_electric_sprites_are_used_instead_of_fallback_silhouettes() {
    let catalog = &GameCatalog::load_all()[0];
    for (name, slug) in [
        ("planet", "planet"),
        ("space", "space"),
        ("electricity", "electricity"),
        ("wire", "wire"),
        ("light bulb", "light-bulb"),
        ("solar system", "solar-system"),
        ("galaxy", "galaxy"),
        ("telescope", "telescope"),
        ("rocket", "rocket"),
        ("astronaut", "astronaut"),
    ] {
        let index = catalog.name_to_index.get(name).copied().unwrap();
        let element = &catalog.elements[index];

        assert_eq!(
            tui_alchemy::sprites::sprite_source_for_element(CatalogKind::LittleAlchemy1, element),
            SpriteSource::Generated(PathBuf::from(format!(
                "assets/pixel-sprites/little-alchemy-1/{slug}.png"
            ))),
            "{name} should use an authored generated sprite, not the semantic fallback"
        );
    }
}

#[test]
fn natural_force_sprites_are_used_instead_of_fallback_silhouettes() {
    let catalog = &GameCatalog::load_all()[0];
    for (name, slug) in [
        ("earthquake", "earthquake"),
        ("flood", "flood"),
        ("geyser", "geyser"),
        ("granite", "granite"),
        ("gunpowder", "gunpowder"),
        ("obsidian", "obsidian"),
        ("ocean", "ocean"),
        ("salt", "salt"),
        ("algae", "algae"),
        ("ash", "ash"),
        ("eruption", "eruption"),
        ("explosion", "explosion"),
        ("fog", "fog"),
        ("hurricane", "hurricane"),
        ("tsunami", "tsunami"),
        ("wave", "wave"),
    ] {
        let index = catalog.name_to_index.get(name).copied().unwrap();
        let element = &catalog.elements[index];

        assert_eq!(
            tui_alchemy::sprites::sprite_source_for_element(CatalogKind::LittleAlchemy1, element),
            SpriteSource::Generated(PathBuf::from(format!(
                "assets/pixel-sprites/little-alchemy-1/{slug}.png"
            ))),
            "{name} should use an authored generated sprite, not the semantic fallback"
        );
    }
}

#[test]
fn constructed_botanical_sprites_are_used_instead_of_fallback_silhouettes() {
    let catalog = &GameCatalog::load_all()[0];
    for (name, slug) in [
        ("wall", "wall"),
        ("archipelago", "archipelago"),
        ("atomic bomb", "atomic-bomb"),
        ("beach", "beach"),
        ("boiler", "boiler"),
        ("bullet", "bullet"),
        ("cactus", "cactus"),
        ("desert", "desert"),
        ("dew", "dew"),
        ("diamond", "diamond"),
        ("dune", "dune"),
        ("fireworks", "fireworks"),
        ("garden", "garden"),
        ("ivy", "ivy"),
        ("moss", "moss"),
        ("pond", "pond"),
    ] {
        let index = catalog.name_to_index.get(name).copied().unwrap();
        let element = &catalog.elements[index];

        assert_eq!(
            tui_alchemy::sprites::sprite_source_for_element(CatalogKind::LittleAlchemy1, element),
            SpriteSource::Generated(PathBuf::from(format!(
                "assets/pixel-sprites/little-alchemy-1/{slug}.png"
            ))),
            "{name} should use an authored generated sprite, not the semantic fallback"
        );
    }
}

#[test]
fn midgame_device_world_sprites_are_used_instead_of_fallback_silhouettes() {
    let catalog = &GameCatalog::load_all()[0];
    for (name, slug) in [
        ("aquarium", "aquarium"),
        ("blender", "blender"),
        ("bridge", "bridge"),
        ("dam", "dam"),
        ("day", "day"),
        ("eclipse", "eclipse"),
        ("gold", "gold"),
        ("golem", "golem"),
        ("greenhouse", "greenhouse"),
        ("gun", "gun"),
        ("hourglass", "hourglass"),
        ("mirror", "mirror"),
        ("night", "night"),
        ("oasis", "oasis"),
        ("oxygen", "oxygen"),
        ("plankton", "plankton"),
    ] {
        let index = catalog.name_to_index.get(name).copied().unwrap();
        let element = &catalog.elements[index];

        assert_eq!(
            tui_alchemy::sprites::sprite_source_for_element(CatalogKind::LittleAlchemy1, element),
            SpriteSource::Generated(PathBuf::from(format!(
                "assets/pixel-sprites/little-alchemy-1/{slug}.png"
            ))),
            "{name} should use an authored generated sprite, not the semantic fallback"
        );
    }
}

#[test]
fn civilization_transport_sprites_are_used_instead_of_fallback_silhouettes() {
    let catalog = &GameCatalog::load_all()[0];
    for (name, slug) in [
        ("airplane", "airplane"),
        ("bank", "bank"),
        ("castle", "castle"),
        ("city", "city"),
        ("farm", "farm"),
        ("farmer", "farmer"),
        ("field", "field"),
        ("forest", "forest"),
        ("helicopter", "helicopter"),
        ("hospital", "hospital"),
        ("lake", "lake"),
        ("river", "river"),
        ("sailboat", "sailboat"),
        ("swamp", "swamp"),
        ("train", "train"),
        ("village", "village"),
    ] {
        let index = catalog.name_to_index.get(name).copied().unwrap();
        let element = &catalog.elements[index];

        assert_eq!(
            tui_alchemy::sprites::sprite_source_for_element(CatalogKind::LittleAlchemy1, element),
            SpriteSource::Generated(PathBuf::from(format!(
                "assets/pixel-sprites/little-alchemy-1/{slug}.png"
            ))),
            "{name} should use an authored generated sprite, not the semantic fallback"
        );
    }
}

#[test]
fn material_iconic_sprites_are_used_instead_of_fallback_silhouettes() {
    let catalog = &GameCatalog::load_all()[0];
    for (name, slug) in [
        ("isle", "isle"),
        ("grenade", "grenade"),
        ("horizon", "horizon"),
        ("mountain range", "mountain-range"),
        ("quicksand", "quicksand"),
        ("rust", "rust"),
        ("sandstone", "sandstone"),
        ("sandstorm", "sandstorm"),
        ("sound", "sound"),
        ("steel", "steel"),
        ("perfume", "perfume"),
        ("pyramid", "pyramid"),
        ("ring", "ring"),
        ("robot", "robot"),
        ("scythe", "scythe"),
        ("sunflower", "sunflower"),
    ] {
        let index = catalog.name_to_index.get(name).copied().unwrap();
        let element = &catalog.elements[index];

        assert_eq!(
            tui_alchemy::sprites::sprite_source_for_element(CatalogKind::LittleAlchemy1, element),
            SpriteSource::Generated(PathBuf::from(format!(
                "assets/pixel-sprites/little-alchemy-1/{slug}.png"
            ))),
            "{name} should use an authored generated sprite, not the semantic fallback"
        );
    }
}

#[test]
fn common_object_scenery_sprites_are_used_instead_of_fallback_silhouettes() {
    let catalog = &GameCatalog::load_all()[0];
    for (name, slug) in [
        ("skyscraper", "skyscraper"),
        ("sword", "sword"),
        ("tide", "tide"),
        ("water lily", "water-lily"),
        ("waterfall", "waterfall"),
        ("windmill", "windmill"),
        ("window", "window"),
        ("barn", "barn"),
        ("birdhouse", "birdhouse"),
        ("dynamite", "dynamite"),
        ("eagle", "eagle"),
        ("lamp", "lamp"),
        ("lawn mower", "lawn-mower"),
        ("microscope", "microscope"),
        ("oil", "oil"),
        ("paint", "paint"),
    ] {
        let index = catalog.name_to_index.get(name).copied().unwrap();
        let element = &catalog.elements[index];

        assert_eq!(
            tui_alchemy::sprites::sprite_source_for_element(CatalogKind::LittleAlchemy1, element),
            SpriteSource::Generated(PathBuf::from(format!(
                "assets/pixel-sprites/little-alchemy-1/{slug}.png"
            ))),
            "{name} should use an authored generated sprite, not the semantic fallback"
        );
    }
}

#[test]
fn fantasy_character_sprites_are_used_instead_of_fallback_silhouettes() {
    let catalog = &GameCatalog::load_all()[0];
    for (name, slug) in [
        ("angel", "angel"),
        ("corpse", "corpse"),
        ("cyborg", "cyborg"),
        ("fireman", "fireman"),
        ("gardener", "gardener"),
        ("grim reaper", "grim-reaper"),
        ("nerd", "nerd"),
        ("phoenix", "phoenix"),
        ("scarecrow", "scarecrow"),
        ("surfer", "surfer"),
        ("unicorn", "unicorn"),
        ("warrior", "warrior"),
        ("wizard", "wizard"),
        ("alligator", "alligator"),
        ("armor", "armor"),
        ("dragon", "dragon"),
    ] {
        let index = catalog.name_to_index.get(name).copied().unwrap();
        let element = &catalog.elements[index];

        assert_eq!(
            tui_alchemy::sprites::sprite_source_for_element(CatalogKind::LittleAlchemy1, element),
            SpriteSource::Generated(PathBuf::from(format!(
                "assets/pixel-sprites/little-alchemy-1/{slug}.png"
            ))),
            "{name} should use an authored generated sprite, not the semantic fallback"
        );
    }
}

#[test]
fn early_missing_character_object_sprites_are_used_instead_of_fallback_silhouettes() {
    let catalog = &GameCatalog::load_all()[0];
    for (name, slug) in [
        ("tobacco", "tobacco"),
        ("allergy", "allergy"),
        ("bayonet", "bayonet"),
        ("blood", "blood"),
        ("carbon dioxide", "carbon-dioxide"),
        ("cold", "cold"),
        ("double rainbow", "double-rainbow"),
        ("duck", "duck"),
        ("electrician", "electrician"),
        ("excalibur", "excalibur"),
        ("family", "family"),
        ("flamethrower", "flamethrower"),
        ("hard roe", "hard-roe"),
        ("hay bale", "hay-bale"),
        ("hummingbird", "hummingbird"),
        ("idea", "idea"),
    ] {
        let index = catalog.name_to_index.get(name).copied().unwrap();
        let element = &catalog.elements[index];

        assert_eq!(
            tui_alchemy::sprites::sprite_source_for_element(CatalogKind::LittleAlchemy1, element),
            SpriteSource::Generated(PathBuf::from(format!(
                "assets/pixel-sprites/little-alchemy-1/{slug}.png"
            ))),
            "{name} should use an authored generated sprite, not the semantic fallback"
        );
    }
}

#[test]
fn light_bird_object_sprites_are_used_instead_of_fallback_silhouettes() {
    let catalog = &GameCatalog::load_all()[0];
    for (name, slug) in [
        ("light", "light"),
        ("lightsaber", "lightsaber"),
        ("love", "love"),
        ("music", "music"),
        ("nest", "nest"),
        ("omelette", "omelette"),
        ("ostrich", "ostrich"),
        ("owl", "owl"),
        ("ozone", "ozone"),
        ("peacock", "peacock"),
        ("prism", "prism"),
        ("ruins", "ruins"),
        ("safe", "safe"),
        ("safety glasses", "safety-glasses"),
        ("seagull", "seagull"),
        ("sickness", "sickness"),
    ] {
        let index = catalog.name_to_index.get(name).copied().unwrap();
        let element = &catalog.elements[index];

        assert_eq!(
            tui_alchemy::sprites::sprite_source_for_element(CatalogKind::LittleAlchemy1, element),
            SpriteSource::Generated(PathBuf::from(format!(
                "assets/pixel-sprites/little-alchemy-1/{slug}.png"
            ))),
            "{name} should use an authored generated sprite, not the semantic fallback"
        );
    }
}

#[test]
fn accessory_device_nature_sprites_are_used_instead_of_fallback_silhouettes() {
    let catalog = &GameCatalog::load_all()[0];
    for (name, slug) in [
        ("sunglasses", "sunglasses"),
        ("swim goggles", "swim-goggles"),
        ("taser", "taser"),
        ("the one ring", "the-one-ring"),
        ("toucan", "toucan"),
        ("turtle", "turtle"),
        ("twilight", "twilight"),
        ("water gun", "water-gun"),
        ("wind turbine", "wind-turbine"),
        ("alarm clock", "alarm-clock"),
        ("black hole", "black-hole"),
        ("bone", "bone"),
        ("bonsai tree", "bonsai-tree"),
        ("caviar", "caviar"),
        ("chameleon", "chameleon"),
        ("charcoal", "charcoal"),
    ] {
        let index = catalog.name_to_index.get(name).copied().unwrap();
        let element = &catalog.elements[index];

        assert_eq!(
            tui_alchemy::sprites::sprite_source_for_element(CatalogKind::LittleAlchemy1, element),
            SpriteSource::Generated(PathBuf::from(format!(
                "assets/pixel-sprites/little-alchemy-1/{slug}.png"
            ))),
            "{name} should use an authored generated sprite, not the semantic fallback"
        );
    }
}

#[test]
fn animal_tech_monster_sprites_are_used_instead_of_fallback_silhouettes() {
    let catalog = &GameCatalog::load_all()[0];
    for (name, slug) in [
        ("chicken", "chicken"),
        ("christmas tree", "christmas-tree"),
        ("computer", "computer"),
        ("constellation", "constellation"),
        ("crow", "crow"),
        ("cuckoo", "cuckoo"),
        ("dinosaur", "dinosaur"),
        ("drone", "drone"),
        ("dry ice", "dry-ice"),
        ("duckling", "duckling"),
        ("egg timer", "egg-timer"),
        ("engineer", "engineer"),
        ("family tree", "family-tree"),
        ("fire extinguisher", "fire-extinguisher"),
        ("flashlight", "flashlight"),
        ("frankenstein", "frankenstein"),
    ] {
        let index = catalog.name_to_index.get(name).copied().unwrap();
        let element = &catalog.elements[index];

        assert_eq!(
            tui_alchemy::sprites::sprite_source_for_element(CatalogKind::LittleAlchemy1, element),
            SpriteSource::Generated(PathBuf::from(format!(
                "assets/pixel-sprites/little-alchemy-1/{slug}.png"
            ))),
            "{name} should use an authored generated sprite, not the semantic fallback"
        );
    }
}

#[test]
fn food_grave_magic_sprites_are_used_instead_of_fallback_silhouettes() {
    let catalog = &GameCatalog::load_all()[0];
    for (name, slug) in [
        ("fridge", "fridge"),
        ("fruit", "fruit"),
        ("grave", "grave"),
        ("harp", "harp"),
        ("herb", "herb"),
        ("jedi", "jedi"),
        ("lava lamp", "lava-lamp"),
        ("leaf", "leaf"),
        ("lighthouse", "lighthouse"),
        ("livestock", "livestock"),
        ("mayonnaise", "mayonnaise"),
        ("monarch", "monarch"),
        ("mummy", "mummy"),
        ("narwhal", "narwhal"),
        ("oil lamp", "oil-lamp"),
        ("optical fiber", "optical-fiber"),
    ] {
        let index = catalog.name_to_index.get(name).copied().unwrap();
        let element = &catalog.elements[index];

        assert_eq!(
            tui_alchemy::sprites::sprite_source_for_element(CatalogKind::LittleAlchemy1, element),
            SpriteSource::Generated(PathBuf::from(format!(
                "assets/pixel-sprites/little-alchemy-1/{slug}.png"
            ))),
            "{name} should use an authored generated sprite, not the semantic fallback"
        );
    }
}

#[test]
fn shore_tool_sky_sprites_are_used_instead_of_fallback_silhouettes() {
    let catalog = &GameCatalog::load_all()[0];
    for (name, slug) in [
        ("palm", "palm"),
        ("pegasus", "pegasus"),
        ("pigeon", "pigeon"),
        ("pilot", "pilot"),
        ("pitchfork", "pitchfork"),
        ("rose", "rose"),
        ("seaplane", "seaplane"),
        ("seasickness", "seasickness"),
        ("sewing machine", "sewing-machine"),
        ("shark", "shark"),
        ("shuriken", "shuriken"),
        ("skeleton", "skeleton"),
        ("smog", "smog"),
        ("soap", "soap"),
        ("soda", "soda"),
        ("solar cell", "solar-cell"),
    ] {
        let index = catalog.name_to_index.get(name).copied().unwrap();
        let element = &catalog.elements[index];

        assert_eq!(
            tui_alchemy::sprites::sprite_source_for_element(CatalogKind::LittleAlchemy1, element),
            SpriteSource::Generated(PathBuf::from(format!(
                "assets/pixel-sprites/little-alchemy-1/{slug}.png"
            ))),
            "{name} should use an authored generated sprite, not the semantic fallback"
        );
    }
}

#[test]
fn thin_tool_sprites_survive_terminal_downsampling() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    app.reveal_elements_for_preview(&["Pitchfork", "Shuriken", "Solar cell"]);

    terminal.draw(|frame| app.render(frame)).unwrap();
    let lines = buffer_lines(terminal.backend().buffer());
    let pitchfork = find_text_position(&lines, "pitchfork").expect("expected pitchfork label");
    let graphic_cells = count_graphic_cells_above_label(&lines, pitchfork, 7, 5);

    assert!(
        graphic_cells >= 8,
        "thin tools should still read as authored sprites at terminal size, not collapse into a tiny mark; cells={graphic_cells}\n{}",
        lines.join("\n")
    );
}

#[test]
fn space_time_undead_weather_sprites_are_used_instead_of_fallback_silhouettes() {
    let catalog = &GameCatalog::load_all()[0];
    for (name, slug) in [
        ("spaceship", "spaceship"),
        ("starfish", "starfish"),
        ("statue", "statue"),
        ("steam engine", "steam-engine"),
        ("sundial", "sundial"),
        ("super nova", "super-nova"),
        ("swimmer", "swimmer"),
        ("thread", "thread"),
        ("treehouse", "treehouse"),
        ("umbrella", "umbrella"),
        ("vampire", "vampire"),
        ("vulture", "vulture"),
        ("watch", "watch"),
        ("zombie", "zombie"),
        ("acid rain", "acid-rain"),
        ("alcohol", "alcohol"),
    ] {
        let index = catalog.name_to_index.get(name).copied().unwrap();
        let element = &catalog.elements[index];

        assert_eq!(
            tui_alchemy::sprites::sprite_source_for_element(CatalogKind::LittleAlchemy1, element),
            SpriteSource::Generated(PathBuf::from(format!(
                "assets/pixel-sprites/little-alchemy-1/{slug}.png"
            ))),
            "{name} should use an authored generated sprite, not the semantic fallback"
        );
    }
}

#[test]
fn alien_winter_food_character_sprites_are_used_instead_of_fallback_silhouettes() {
    let catalog = &GameCatalog::load_all()[0];
    for (name, slug) in [
        ("alien", "alien"),
        ("antarctica", "antarctica"),
        ("avalanche", "avalanche"),
        ("blizzard", "blizzard"),
        ("broom", "broom"),
        ("bulletproof vest", "bulletproof-vest"),
        ("camel", "camel"),
        ("campfire", "campfire"),
        ("chicken soup", "chicken-soup"),
        ("chicken wing", "chicken-wing"),
        ("coconut", "coconut"),
        ("coffin", "coffin"),
        ("crown", "crown"),
        ("darth vader", "darth-vader"),
        ("doctor", "doctor"),
        ("electric eel", "electric-eel"),
    ] {
        let index = catalog.name_to_index.get(name).copied().unwrap();
        let element = &catalog.elements[index];

        assert_eq!(
            tui_alchemy::sprites::sprite_source_for_element(CatalogKind::LittleAlchemy1, element),
            SpriteSource::Generated(PathBuf::from(format!(
                "assets/pixel-sprites/little-alchemy-1/{slug}.png"
            ))),
            "{name} should use an authored generated sprite, not the semantic fallback"
        );
    }
}

#[test]
fn material_nature_creature_sprites_are_used_instead_of_fallback_silhouettes() {
    let catalog = &GameCatalog::load_all()[0];
    for (name, slug) in [
        ("fabric", "fabric"),
        ("fence", "fence"),
        ("flour", "flour"),
        ("flute", "flute"),
        ("fossil", "fossil"),
        ("fountain", "fountain"),
        ("fruit tree", "fruit-tree"),
        ("glacier", "glacier"),
        ("gnome", "gnome"),
        ("goat", "goat"),
        ("godzilla", "godzilla"),
        ("gravestone", "gravestone"),
        ("graveyard", "graveyard"),
        ("hail", "hail"),
        ("iceberg", "iceberg"),
        ("igloo", "igloo"),
    ] {
        let index = catalog.name_to_index.get(name).copied().unwrap();
        let element = &catalog.elements[index];

        assert_eq!(
            tui_alchemy::sprites::sprite_source_for_element(CatalogKind::LittleAlchemy1, element),
            SpriteSource::Generated(PathBuf::from(format!(
                "assets/pixel-sprites/little-alchemy-1/{slug}.png"
            ))),
            "{name} should use an authored generated sprite, not the semantic fallback"
        );
    }
}

#[test]
fn atlas_wraps_multi_word_labels_instead_of_hiding_the_meaning() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    app.reveal_elements_for_preview(&["Carbon dioxide", "Double rainbow!", "Hard roe", "Hay bale"]);

    terminal.draw(|frame| app.render(frame)).unwrap();
    let text = buffer_to_text(terminal.backend().buffer());

    assert!(
        text.contains("carbon") && text.contains("dioxide"),
        "carbon dioxide should wrap into readable words, not disappear behind ellipsis:\n{text}"
    );
    assert!(
        text.contains("double") && text.contains("rainbow"),
        "double rainbow should wrap into readable words, not disappear behind ellipsis:\n{text}"
    );
    assert!(
        !text.contains("carbon di...") && !text.contains("double ra..."),
        "atlas labels should avoid ellipsis for meaningful two-word element names:\n{text}"
    );
}

#[test]
fn atlas_keeps_adjacent_long_labels_separated() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    app.reveal_elements_for_preview(&[
        "Wind turbine",
        "Alarm clock",
        "Bonsai tree",
        "Caviar",
        "Chameleon",
    ]);

    terminal.draw(|frame| app.render(frame)).unwrap();
    let text = buffer_to_text(terminal.backend().buffer());

    assert!(
        !text.contains("turbinealarm"),
        "adjacent atlas labels should keep readable space between tiles:\n{text}"
    );
    assert!(
        !text.contains("treecaviar"),
        "adjacent atlas labels should not visually merge into one token:\n{text}"
    );
}

#[test]
fn atlas_splits_long_single_word_labels_instead_of_ellipsis() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    app.reveal_elements_for_preview(&["Constellation", "Frankenstein", "Fire extinguisher"]);

    terminal.draw(|frame| app.render(frame)).unwrap();
    let text = buffer_to_text(terminal.backend().buffer());

    assert!(
        !text.contains("constel...")
            && !text.contains("franken...")
            && !text.contains("extingu..."),
        "long labels should split into readable chunks instead of hiding the end of the name:\n{text}"
    );
    assert!(
        text.contains("constel") && text.contains("lation"),
        "constellation should keep both recognizable word chunks:\n{text}"
    );
    assert!(
        text.contains("franken") && text.contains("stein"),
        "frankenstein should keep both recognizable word chunks:\n{text}"
    );
}

#[test]
fn atlas_splits_long_first_words_without_hiding_the_remaining_label() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    app.reveal_elements_for_preview(&["Bulletproof vest"]);

    terminal.draw(|frame| app.render(frame)).unwrap();
    let text = buffer_to_text(terminal.backend().buffer());

    assert!(
        !text.contains("bulletp..."),
        "long first words in multi-word labels should split cleanly instead of ellipsizing:\n{text}"
    );
    assert!(
        text.contains("bullet") && text.contains("proof vest"),
        "bulletproof vest should preserve both the long word and the remaining label:\n{text}"
    );
}

#[test]
fn missing_generated_sprite_renders_semantic_silhouette_not_initials() {
    let catalog = &GameCatalog::load_all()[0];
    let internet = catalog.name_to_index.get("internet").copied().unwrap();
    let flower = catalog.name_to_index.get("flower").copied().unwrap();
    let internet_element = &catalog.elements[internet];
    let flower_element = &catalog.elements[flower];

    let internet_lines = tui_alchemy::sprites::sprite_lines_for_element_with_size(
        CatalogKind::LittleAlchemy1,
        internet_element,
        8,
        8,
    );
    let flower_lines = tui_alchemy::sprites::sprite_lines_for_element_with_size(
        CatalogKind::LittleAlchemy1,
        flower_element,
        8,
        8,
    );
    let internet_symbols = sprite_symbols(&internet_lines);
    let flower_symbols = sprite_symbols(&flower_lines);

    assert!(
        !internet_symbols
            .chars()
            .any(|ch| ch.is_ascii_alphanumeric()),
        "missing sprites should render styled silhouettes, not initials: {internet_symbols:?}"
    );
    assert!(
        internet_symbols.chars().filter(|ch| *ch != ' ').count() >= 8,
        "semantic placeholder should have enough silhouette pixels: {internet_symbols:?}"
    );
    assert_ne!(
        internet_symbols, flower_symbols,
        "different element styles should not collapse into the same placeholder"
    );
}

#[test]
fn semantic_placeholder_animates_without_shape_jitter() {
    let catalog = &GameCatalog::load_all()[0];
    let internet = catalog.name_to_index.get("internet").copied().unwrap();
    let internet_element = &catalog.elements[internet];

    let first = tui_alchemy::sprites::sprite_lines_for_element_frame(
        CatalogKind::LittleAlchemy1,
        internet_element,
        8,
        8,
        0,
    );
    let second = tui_alchemy::sprites::sprite_lines_for_element_frame(
        CatalogKind::LittleAlchemy1,
        internet_element,
        8,
        8,
        6,
    );

    assert_eq!(
        sprite_symbols(&first),
        sprite_symbols(&second),
        "fallback animation should not wobble the object silhouette"
    );
}

fn sprite_symbols(lines: &[ratatui::text::Line<'_>]) -> String {
    lines
        .iter()
        .flat_map(|line| line.spans.iter())
        .map(|span| span.content.as_ref())
        .collect::<Vec<_>>()
        .join("")
}

fn buffer_to_text(buffer: &ratatui::buffer::Buffer) -> String {
    let mut out = String::new();
    let area = buffer.area;
    for y in 0..area.height {
        for x in 0..area.width {
            out.push_str(buffer[(area.x + x, area.y + y)].symbol());
        }
        out.push('\n');
    }
    out
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

fn average_background_brightness(
    buffer: &ratatui::buffer::Buffer,
    x: u16,
    y: u16,
    width: u16,
    height: u16,
) -> u16 {
    let mut total = 0u32;
    let mut count = 0u32;
    for row in y..y.saturating_add(height).min(buffer.area.height) {
        for column in x..x.saturating_add(width).min(buffer.area.width) {
            let cell = &buffer[(column, row)];
            if cell.symbol() == " " {
                total += color_brightness(cell.bg) as u32;
                count += 1;
            }
        }
    }
    if count == 0 {
        0
    } else {
        (total / count) as u16
    }
}

fn color_brightness(color: ratatui::style::Color) -> u16 {
    match color {
        ratatui::style::Color::Rgb(red, green, blue) => {
            (red as u16 + green as u16 + blue as u16) / 3
        }
        _ => 0,
    }
}

fn first_non_space_column(line: &str) -> Option<usize> {
    line.chars().position(|ch| ch != ' ')
}

fn last_non_space_column(line: &str) -> Option<usize> {
    line.chars()
        .enumerate()
        .filter_map(|(index, ch)| (ch != ' ').then_some(index))
        .last()
}

fn row_has_two_sided_title_border(row: &str, title: &str) -> bool {
    let Some(byte_column) = row.find(title) else {
        return false;
    };
    let title_start = row[..byte_column].chars().count();
    let title_end = title_start + title.chars().count();
    let chars = row.chars().collect::<Vec<_>>();
    let left_rails = chars[..title_start]
        .iter()
        .filter(|&&ch| is_title_border_rail(ch))
        .count();
    let right_rails = chars[title_end..]
        .iter()
        .filter(|&&ch| is_title_border_rail(ch))
        .count();

    left_rails >= 2 && right_rails >= 2
}

fn is_title_border_rail(ch: char) -> bool {
    matches!(ch, '─' | '━' | '═' | '╌' | '╾' | '╼')
}

fn find_text_position(lines: &[String], needle: &str) -> Option<(u16, u16)> {
    lines.iter().enumerate().find_map(|(row, line)| {
        line.find(needle).map(|byte_column| {
            let column = line[..byte_column].chars().count();
            (column as u16, row as u16)
        })
    })
}

fn find_last_text_position(lines: &[String], needle: &str) -> Option<(u16, u16)> {
    lines.iter().enumerate().rev().find_map(|(row, line)| {
        line.rfind(needle).map(|byte_column| {
            let column = line[..byte_column].chars().count();
            (column as u16, row as u16)
        })
    })
}

fn find_text_position_in_region(
    lines: &[String],
    needle: &str,
    x: u16,
    y: u16,
    width: u16,
    height: u16,
) -> Option<(u16, u16)> {
    let y_end = y.saturating_add(height).min(lines.len() as u16);
    for row in y..y_end {
        let line = &lines[row as usize];
        let x_start = x as usize;
        let x_end = x.saturating_add(width).min(line.chars().count() as u16) as usize;
        if x_start >= x_end {
            continue;
        }
        let slice = line
            .chars()
            .skip(x_start)
            .take(x_end.saturating_sub(x_start))
            .collect::<String>();
        if let Some(byte) = slice.find(needle) {
            // `find` returns a byte offset; block glyphs are multi-byte, so
            // convert to a character column before adding to `x`.
            let col = slice[..byte].chars().count();
            return Some((x + col as u16, row));
        }
    }
    None
}

fn count_matching_chars_in_region(
    lines: &[String],
    x0: u16,
    y0: u16,
    x1: u16,
    y1: u16,
    matches: &[char],
) -> usize {
    let y_end = y1.min(lines.len() as u16);
    (y0..y_end)
        .map(|row| {
            let line = &lines[row as usize];
            line.chars()
                .enumerate()
                .filter(|(column, ch)| {
                    let column = *column as u16;
                    column >= x0 && column < x1 && matches.contains(ch)
                })
                .count()
        })
        .sum()
}

fn count_graphic_rows_above_label(
    lines: &[String],
    label: (u16, u16),
    rows_above: u16,
    half_width: u16,
) -> usize {
    let start_row = label.1.saturating_sub(rows_above) as usize;
    let end_row = label.1.saturating_sub(1) as usize;
    lines[start_row..=end_row]
        .iter()
        .filter(|line| {
            let start = label.0.saturating_sub(half_width) as usize;
            let end = (label.0 as usize + half_width as usize).min(line.chars().count());
            line.chars()
                .skip(start)
                .take(end.saturating_sub(start))
                .any(|ch| ch != ' ')
        })
        .count()
}

fn count_graphic_cells_above_label(
    lines: &[String],
    label: (u16, u16),
    rows_above: u16,
    half_width: u16,
) -> usize {
    let start_row = label.1.saturating_sub(rows_above) as usize;
    let end_row = label.1.saturating_sub(1) as usize;
    lines[start_row..=end_row]
        .iter()
        .map(|line| {
            let start = label.0.saturating_sub(half_width) as usize;
            let end = (label.0 as usize + half_width as usize).min(line.chars().count());
            line.chars()
                .skip(start)
                .take(end.saturating_sub(start))
                .filter(|ch| *ch != ' ')
                .count()
        })
        .sum()
}

fn graphic_span_width_above_label(
    lines: &[String],
    label: (u16, u16),
    rows_above: u16,
    half_width: u16,
) -> usize {
    let start_row = label.1.saturating_sub(rows_above) as usize;
    let end_row = label.1.saturating_sub(1) as usize;
    let mut min_col: Option<usize> = None;
    let mut max_col: Option<usize> = None;

    for line in &lines[start_row..=end_row] {
        let start = label.0.saturating_sub(half_width) as usize;
        let end = (label.0 as usize + half_width as usize).min(line.chars().count());
        for (offset, ch) in line
            .chars()
            .skip(start)
            .take(end.saturating_sub(start))
            .enumerate()
        {
            if ch != ' ' {
                let col = start + offset;
                min_col = Some(min_col.map_or(col, |value| value.min(col)));
                max_col = Some(max_col.map_or(col, |value| value.max(col)));
            }
        }
    }

    match (min_col, max_col) {
        (Some(min_col), Some(max_col)) => max_col.saturating_sub(min_col) + 1,
        _ => 0,
    }
}

fn graphic_bounds_above_label(
    lines: &[String],
    label: (u16, u16),
    rows_above: u16,
    half_width: u16,
) -> Option<(usize, usize)> {
    let start_row = label.1.saturating_sub(rows_above) as usize;
    let end_row = label.1.saturating_sub(1) as usize;
    let mut min_col: Option<usize> = None;
    let mut max_col: Option<usize> = None;

    for line in &lines[start_row..=end_row] {
        let start = label.0.saturating_sub(half_width) as usize;
        let end = (label.0 as usize + half_width as usize).min(line.chars().count());
        for (offset, ch) in line
            .chars()
            .skip(start)
            .take(end.saturating_sub(start))
            .enumerate()
        {
            if ch != ' ' {
                let col = start + offset;
                min_col = Some(min_col.map_or(col, |value| value.min(col)));
                max_col = Some(max_col.map_or(col, |value| value.max(col)));
            }
        }
    }

    min_col.zip(max_col)
}

fn graphic_row_count_above_label(
    lines: &[String],
    label: (u16, u16),
    rows_above: u16,
    half_width: u16,
) -> usize {
    let start_row = label.1.saturating_sub(rows_above) as usize;
    let end_row = label.1.saturating_sub(1) as usize;

    lines[start_row..=end_row]
        .iter()
        .filter(|line| {
            let start = label.0.saturating_sub(half_width) as usize;
            let end = (label.0 as usize + half_width as usize).min(line.chars().count());
            line.chars()
                .skip(start)
                .take(end.saturating_sub(start))
                .any(|ch| ch != ' ')
        })
        .count()
}

fn highlight_rows_above_label(
    buffer: &ratatui::buffer::Buffer,
    label: (u16, u16),
    rows_above: u16,
    half_width: u16,
    highlight: ratatui::style::Color,
) -> Vec<u16> {
    let start_row = label.1.saturating_sub(rows_above);
    let end_row = label.1.saturating_sub(1);
    let start_col = label.0.saturating_sub(half_width);
    let end_col = label.0.saturating_add(half_width).min(buffer.area.width);
    let mut rows = Vec::new();

    for row in start_row..=end_row {
        for col in start_col..end_col {
            let cell = &buffer[(col, row)];
            if cell.symbol() != " " && cell.fg == highlight {
                rows.push(row);
            }
        }
    }

    rows
}

fn empty_glow_cells_around_label(
    buffer: &ratatui::buffer::Buffer,
    label: (u16, u16),
    rows_above: u16,
    half_width: u16,
    glow: ratatui::style::Color,
) -> usize {
    let start_row = label.1.saturating_sub(rows_above);
    let end_row = label.1.saturating_sub(1);
    let start_col = label.0.saturating_sub(half_width);
    let end_col = label.0.saturating_add(half_width).min(buffer.area.width);
    let mut cells = 0;

    for row in start_row..=end_row {
        for col in start_col..end_col {
            let cell = &buffer[(col, row)];
            if cell.symbol() == " " && cell.bg == glow {
                cells += 1;
            }
        }
    }

    cells
}

fn colored_sprite_pixels_above_label(
    buffer: &ratatui::buffer::Buffer,
    label: (u16, u16),
    rows_above: u16,
    half_width: u16,
    color: ratatui::style::Color,
) -> usize {
    let start_row = label.1.saturating_sub(rows_above);
    let end_row = label.1.saturating_sub(1);
    let start_col = label.0.saturating_sub(half_width);
    let end_col = label.0.saturating_add(half_width).min(buffer.area.width);
    let mut cells = 0;

    for row in start_row..=end_row {
        for col in start_col..end_col {
            let cell = &buffer[(col, row)];
            if cell.symbol() != " " && cell.bg == color {
                cells += 1;
            }
        }
    }

    cells
}

fn sprite_cell_signature(
    buffer: &ratatui::buffer::Buffer,
    label: (u16, u16),
    rows_above: u16,
    half_width: u16,
) -> Vec<(String, ratatui::style::Color)> {
    let start_row = label.1.saturating_sub(rows_above);
    let end_row = label.1.saturating_sub(1);
    let start_col = label.0.saturating_sub(half_width);
    let end_col = label.0.saturating_add(half_width).min(buffer.area.width);
    let mut cells = Vec::new();

    for row in start_row..=end_row {
        for col in start_col..end_col {
            let cell = &buffer[(col, row)];
            if cell.symbol() != " " {
                cells.push((cell.symbol().to_string(), cell.fg));
            }
        }
    }

    cells
}
