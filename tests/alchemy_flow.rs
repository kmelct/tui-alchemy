use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::{Terminal, backend::TestBackend, buffer::Buffer, style::Color};
use tui_alchemy::App;

const fn key(code: KeyCode) -> Event {
    Event::Key(KeyEvent::new(code, KeyModifiers::NONE))
}

const fn mouse(kind: MouseEventKind, column: u16, row: u16) -> Event {
    Event::Mouse(MouseEvent {
        kind,
        column,
        row,
        modifiers: KeyModifiers::NONE,
    })
}

#[test]
fn keyboard_pairing_discovers_steam_in_the_workbench() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    terminal.draw(|frame| app.render(frame)).unwrap();
    app.handle_event(key(KeyCode::Char('4')));
    app.handle_event(key(KeyCode::Char('3')));
    app.tick();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let text = buffer_to_text(terminal.backend().buffer());
    assert!(
        text.contains("steam"),
        "expected steam in the workbench: {text}"
    );
    assert!(
        text.contains("water"),
        "expected left ingredient to stay visible: {text}"
    );
    assert!(
        text.contains("fire"),
        "expected right ingredient to stay visible: {text}"
    );
    assert!(
        has_progress(&text, 5),
        "expected progress 5/755 after steam: {text}"
    );
}

#[test]
fn dragging_base_elements_into_the_workbench_discovers_steam() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let lines = buffer_lines(terminal.backend().buffer());
    let [left_slot, right_slot] = ingredient_slot_targets(&lines);
    let water = find_text_position(&lines, "water").expect("water tile");
    let fire = find_text_position(&lines, "fire").expect("fire tile");

    drag_between(&mut app, water, left_slot);
    drag_between(&mut app, fire, right_slot);
    app.tick();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let text = buffer_to_text(terminal.backend().buffer());
    assert!(
        text.contains("steam"),
        "expected steam after drag-and-drop: {text}"
    );
    assert!(
        text.contains("water"),
        "expected water to remain in the workbench: {text}"
    );
    assert!(
        text.contains("fire"),
        "expected fire to remain in the workbench: {text}"
    );
    assert!(
        has_progress(&text, 5),
        "expected progress 5/755 after steam: {text}"
    );
}

#[test]
fn dragging_toward_the_workbench_renders_a_visible_ghost() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let initial_text = buffer_to_text(terminal.backend().buffer());
    let lines = buffer_lines(terminal.backend().buffer());
    let [left_slot, _right_slot] = ingredient_slot_targets(&lines);
    let water = find_text_position(&lines, "water").expect("water tile");

    app.handle_event(mouse(
        MouseEventKind::Down(MouseButton::Left),
        water.0,
        water.1,
    ));
    app.handle_event(mouse(
        MouseEventKind::Drag(MouseButton::Left),
        left_slot.0,
        left_slot.1,
    ));

    terminal.draw(|frame| app.render(frame)).unwrap();
    let text = buffer_to_text(terminal.backend().buffer());
    assert_ne!(
        text, initial_text,
        "expected the in-flight drag to change the rendered workbench state"
    );
    assert!(
        !text.contains("dragging water")
            && !text.contains("drag inventory")
            && !text.contains("drag canvas"),
        "dragging should render as a ghost, not debug prose: {text}"
    );
}

#[test]
fn initial_workbench_uses_two_input_slots_and_one_result_slot() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let lines = buffer_lines(terminal.backend().buffer());
    let ingredients = find_all_text_positions(&lines, "ingredient");
    let results = find_all_text_positions(&lines, "result");

    assert!(
        ingredients.len() >= 4,
        "expected formula row and two slot labels: {lines:?}"
    );
    assert!(
        !results.is_empty(),
        "expected at least one result label: {lines:?}"
    );
}

#[test]
fn dragging_from_a_live_workbench_slot_can_chain_into_the_next_recipe() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    let [left_slot, right_slot] = craft_steam_via_workbench(&mut app, &mut terminal);
    drag_between(&mut app, left_slot, right_slot);
    app.tick();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let text = buffer_to_text(terminal.backend().buffer());
    assert!(
        text.contains("sea"),
        "expected water + water to discover sea from the live workbench: {text}"
    );
    assert!(
        text.contains("water"),
        "expected water to remain visible while chaining recipes: {text}"
    );
    assert!(
        has_progress(&text, 6),
        "expected progress 6/755 after sea: {text}"
    );
}

#[test]
fn dragging_earth_onto_a_live_slot_replaces_water_and_discovers_lava() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    let [left_slot, _right_slot] = craft_steam_via_workbench(&mut app, &mut terminal);
    terminal.draw(|frame| app.render(frame)).unwrap();
    let lines = buffer_lines(terminal.backend().buffer());
    let earth = find_text_position(&lines, "earth").expect("earth tile");

    drag_between(&mut app, earth, left_slot);
    app.tick();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let text = buffer_to_text(terminal.backend().buffer());
    assert!(
        text.contains("lava"),
        "expected earth + fire to discover lava: {text}"
    );
    assert!(
        text.contains("earth"),
        "expected replacement ingredient to stay visible: {text}"
    );
    assert!(
        text.contains("fire"),
        "expected untouched slot to stay visible: {text}"
    );
    assert!(
        has_progress(&text, 6),
        "expected progress 6/755 after lava: {text}"
    );
}

#[test]
fn replacing_a_live_slot_animates_for_more_than_one_frame() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    let [left_slot, _right_slot] = craft_steam_via_workbench(&mut app, &mut terminal);
    terminal.draw(|frame| app.render(frame)).unwrap();
    let lines = buffer_lines(terminal.backend().buffer());
    let earth = find_text_position(&lines, "earth").expect("earth tile");

    drag_between(&mut app, earth, left_slot);
    terminal.draw(|frame| app.render(frame)).unwrap();
    let before = slot_signature(terminal.backend().buffer(), left_slot);

    app.tick();
    terminal.draw(|frame| app.render(frame)).unwrap();
    let after = slot_signature(terminal.backend().buffer(), left_slot);

    assert_ne!(
        before, after,
        "expected replacing a live slot to animate across ticks"
    );
}

#[test]
fn seeded_preview_keeps_the_atlas_dense_while_the_workbench_stays_available() {
    let backend = TestBackend::new(100, 28);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    app.reveal_elements_for_preview(&[
        "Steam", "Mud", "Lava", "Dust", "Rain", "Sea", "Cloud", "Plant", "Storm", "Metal",
    ]);

    terminal.draw(|frame| app.render(frame)).unwrap();
    let text = buffer_to_text(terminal.backend().buffer());
    let visible = [
        "steam", "mud", "lava", "dust", "rain", "sea", "cloud", "plant", "storm", "metal",
    ]
    .into_iter()
    .filter(|name| text.contains(name))
    .count();

    assert!(visible >= 6, "expected a dense seeded atlas page: {text}");
    assert!(
        text.contains("ingredient"),
        "expected workbench inputs to remain visible: {text}"
    );
    assert!(
        text.contains("result"),
        "expected workbench result slot to remain visible: {text}"
    );
}

#[test]
fn wide_workbench_does_not_become_a_full_height_sidebar() {
    let backend = TestBackend::new(160, 50);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let lines = buffer_lines(terminal.backend().buffer());
    let title = find_text_position(&lines, "recipe table").expect("expected recipe table");
    let result = find_all_text_positions(&lines, "result")
        .into_iter()
        .max_by_key(|(_, row)| *row)
        .expect("expected result label");

    assert!(
        result.1.saturating_sub(title.1) <= 14,
        "the right recipe table should remain a compact workbench panel instead of a floor-to-ceiling sidebar:\n{}",
        lines.join("\n")
    );
}

#[test]
fn wide_layout_uses_panel_borders_without_a_second_page_shell() {
    let backend = TestBackend::new(160, 50);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();

    terminal.draw(|frame| app.render(frame)).unwrap();
    let text = buffer_to_text(terminal.backend().buffer());

    assert!(
        text.contains("✦ progress") && text.contains("✦ atlas") && text.contains("✦ recipe table"),
        "wide layouts should keep the three primary fantasy panel borders visible:\n{text}"
    );
    assert!(
        !text.contains("✦ workshop"),
        "wide layouts should not wrap compact panels in a second page-scale workshop shell that creates empty left/right side compartments:\n{text}"
    );
}

#[test]
fn atlas_uses_page_tabs_instead_of_scroll_position() {
    let backend = TestBackend::new(160, 50);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();
    reveal_many_elements(&mut app);

    terminal.draw(|frame| app.render(frame)).unwrap();
    let text = buffer_to_text(terminal.backend().buffer());
    assert!(
        text.contains("page 1/"),
        "atlas should expose a discrete page tab strip instead of hidden scroll state:\n{text}"
    );

    app.handle_event(key(KeyCode::PageDown));
    terminal.draw(|frame| app.render(frame)).unwrap();
    let text = buffer_to_text(terminal.backend().buffer());
    assert!(
        text.contains("page 2/"),
        "PageDown should switch to the next atlas page tab:\n{text}"
    );
}

#[test]
fn mouse_clicking_an_atlas_tab_switches_pages() {
    let backend = TestBackend::new(160, 50);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut app = App::new();
    reveal_many_elements(&mut app);

    terminal.draw(|frame| app.render(frame)).unwrap();
    let lines = buffer_lines(terminal.backend().buffer());
    let tab = find_text_position(&lines, "[2]").expect("expected second atlas page tab");
    app.handle_event(mouse(
        MouseEventKind::Down(MouseButton::Left),
        tab.0 + 1,
        tab.1,
    ));
    terminal.draw(|frame| app.render(frame)).unwrap();
    let text = buffer_to_text(terminal.backend().buffer());
    assert!(
        text.contains("page 2/"),
        "clicking a page tab should switch the atlas page:\n{text}"
    );
}

#[test]
fn live_workbench_state_survives_dynamic_resize() {
    let mut app = App::new();
    let mut terminal = Terminal::new(TestBackend::new(100, 28)).unwrap();
    let _ = craft_steam_via_workbench(&mut app, &mut terminal);

    for (width, height) in [(100, 28), (76, 24), (48, 22)] {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal.draw(|frame| app.render(frame)).unwrap();
        let text = buffer_to_text(terminal.backend().buffer());
        assert!(
            text.contains("water"),
            "expected water to survive resize {width}x{height}: {text}"
        );
        assert!(
            text.contains("fire"),
            "expected fire to survive resize {width}x{height}: {text}"
        );
        assert!(
            text.contains("steam"),
            "expected result to survive resize {width}x{height}: {text}"
        );
    }
}

fn reveal_many_elements(app: &mut App) {
    app.reveal_elements_for_preview(&[
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
    ]);
}

fn craft_steam_via_workbench(
    app: &mut App,
    terminal: &mut Terminal<TestBackend>,
) -> [(u16, u16); 2] {
    terminal.draw(|frame| app.render(frame)).unwrap();
    let lines = buffer_lines(terminal.backend().buffer());
    let [left_slot, right_slot] = ingredient_slot_targets(&lines);
    let water = find_text_position(&lines, "water").expect("water tile");
    let fire = find_text_position(&lines, "fire").expect("fire tile");

    drag_between(app, water, left_slot);
    drag_between(app, fire, right_slot);
    app.tick();
    [left_slot, right_slot]
}

fn drag_between(app: &mut App, from: (u16, u16), to: (u16, u16)) {
    app.handle_event(mouse(
        MouseEventKind::Down(MouseButton::Left),
        from.0,
        from.1,
    ));
    app.handle_event(mouse(MouseEventKind::Drag(MouseButton::Left), to.0, to.1));
    app.handle_event(mouse(MouseEventKind::Up(MouseButton::Left), to.0, to.1));
}

fn has_progress(text: &str, count: usize) -> bool {
    text.contains(&format!("{count} / 755")) || text.contains(&format!("{count}/755"))
}

fn ingredient_slot_targets(lines: &[String]) -> [(u16, u16); 2] {
    let mut hits = find_all_text_positions(lines, "ingredient");
    hits.sort_by_key(|(column, row)| (*row, *column));
    let mut slots: Vec<_> = hits.into_iter().rev().take(2).collect();
    slots.sort_by_key(|(column, _)| *column);
    [slots[0], slots[1]]
}

fn find_all_text_positions(lines: &[String], needle: &str) -> Vec<(u16, u16)> {
    let mut positions = Vec::new();
    for (row, line) in lines.iter().enumerate() {
        let mut start = 0;
        while let Some(byte_column) = line[start..].find(needle) {
            let byte_column = start + byte_column;
            let column = line[..byte_column].chars().count() as u16;
            positions.push((column, row as u16));
            start = byte_column + needle.len();
        }
    }
    positions
}

fn find_text_position(lines: &[String], needle: &str) -> Option<(u16, u16)> {
    lines.iter().enumerate().find_map(|(row, line)| {
        line.find(needle).map(|byte_column| {
            let column = line[..byte_column].chars().count();
            (column as u16, row as u16)
        })
    })
}

fn buffer_to_text(buffer: &Buffer) -> String {
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

fn buffer_lines(buffer: &Buffer) -> Vec<String> {
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

fn slot_signature(buffer: &Buffer, label: (u16, u16)) -> Vec<(String, Color, Color)> {
    let focus_col = label.0.saturating_add(4);
    let start_row = label.1.saturating_sub(10);
    let end_row = label.1.saturating_sub(1);
    let start_col = focus_col.saturating_sub(4);
    let end_col = focus_col
        .saturating_add(4)
        .min(buffer.area.width.saturating_sub(1));
    let mut cells = Vec::new();

    for row in start_row..=end_row {
        for col in start_col..=end_col {
            let cell = &buffer[(col, row)];
            cells.push((cell.symbol().to_string(), cell.fg, cell.bg));
        }
    }

    cells
}
