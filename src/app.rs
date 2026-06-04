mod input;
pub(crate) mod state;

use crate::data::{GameCatalog, active_palette_indices, normalize};
use crate::effects::ElementEffect;
use crate::layout::{atlas_page_size, atlas_page_start, scene_layout};
use crate::ui;
use ratatui::Frame;
use ratatui::layout::Rect;
pub(crate) use state::{
    Banner, CatalogState, DragOrigin, DragState, MenuItem, MenuView, RecipePreview,
};

#[derive(Debug, Clone, Copy)]
enum HitTarget {
    Inventory(usize),
    Slot(usize),
}

#[derive(Debug)]
pub struct App {
    pub(crate) catalogs: Vec<GameCatalog>,
    pub(crate) states: Vec<CatalogState>,
    pub(crate) active_catalog: usize,
    pub(crate) tick_counter: u64,
    pub(crate) menu_view: MenuView,
    pub(crate) menu_item: MenuItem,
    viewport: Rect,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        Self::with_catalogs(GameCatalog::load_playable_books())
    }

    pub fn with_catalogs(catalogs: Vec<GameCatalog>) -> Self {
        let states = catalogs.iter().map(CatalogState::new).collect();
        Self {
            catalogs,
            states,
            active_catalog: 0,
            tick_counter: 0,
            menu_view: MenuView::Closed,
            menu_item: MenuItem::Resume,
            viewport: Rect::new(0, 0, 0, 0),
        }
    }

    pub fn render(&mut self, frame: &mut Frame<'_>) {
        self.set_viewport(frame.area());
        ui::render_app(frame, self);
    }

    pub(crate) fn set_viewport(&mut self, viewport: Rect) {
        self.viewport = viewport;
        self.sync_active_palette_page_to_cursor();
    }

    #[doc(hidden)]
    pub fn reveal_elements_for_preview(&mut self, names: &[&str]) {
        let catalog_index = self.active_catalog;
        let indices = names
            .iter()
            .filter_map(|name| {
                self.catalogs[catalog_index]
                    .name_to_index
                    .get(&normalize(name))
                    .copied()
            })
            .collect::<Vec<_>>();

        for element_index in indices {
            if !self.states[catalog_index]
                .discovered
                .get(element_index)
                .copied()
                .unwrap_or(false)
            {
                self.discover_element(catalog_index, element_index, false);
            }
        }

        let state = &mut self.states[catalog_index];
        state.clear_selection();
        state.drag = None;
        state.banner = None;
        state.effects.clear();
        state.recent.clear();
        state.slot_flash = [0, 0];
        state.recipe_preview = None;
        state.palette_cursor = self.catalogs[catalog_index].base_indices.len();
        state.palette_page = 0;
    }

    #[doc(hidden)]
    pub fn preview_drag_element(&mut self, name: &str, column: u16, row: u16) {
        let catalog_index = self.active_catalog;
        let Some(&element_index) = self.catalogs[catalog_index]
            .name_to_index
            .get(&normalize(name))
        else {
            return;
        };

        self.states[catalog_index].drag = Some(DragState {
            element_index,
            origin: DragOrigin::Inventory,
            column,
            row,
        });
    }

    pub fn tick(&mut self) {
        self.tick_counter = self.tick_counter.wrapping_add(1);
        for index in 0..self.catalogs.len() {
            self.reconcile_unlocks(index);
            self.age_banner(index);
            self.age_effects(index);
            self.age_slot_flashes(index);
        }
    }

    pub(crate) fn active_catalog(&self) -> &GameCatalog {
        &self.catalogs[self.active_catalog]
    }

    pub(crate) fn active_state(&self) -> &CatalogState {
        &self.states[self.active_catalog]
    }

    pub(crate) fn active_state_mut(&mut self) -> &mut CatalogState {
        &mut self.states[self.active_catalog]
    }

    pub(crate) fn active_palette(&self) -> Vec<usize> {
        active_palette_indices(self.active_catalog(), &self.active_state().discovered)
    }

    pub(crate) fn active_discovered_count(&self) -> usize {
        self.active_state().discovered_count()
    }

    pub(crate) fn active_total(&self) -> usize {
        self.active_catalog().total
    }

    pub(crate) fn active_banner_text(&self) -> Option<&str> {
        self.active_state()
            .banner
            .as_ref()
            .map(|banner| banner.text.as_str())
    }

    pub(crate) fn active_banner_highlight(&self) -> Option<usize> {
        self.active_state()
            .banner
            .as_ref()
            .and_then(|banner| banner.highlight)
    }

    pub(crate) fn active_drag(&self) -> Option<DragState> {
        self.active_state().drag
    }

    pub(crate) const fn menu_view(&self) -> MenuView {
        self.menu_view
    }

    pub(crate) const fn menu_item(&self) -> MenuItem {
        self.menu_item
    }

    fn open_menu(&mut self) {
        self.menu_view = MenuView::Main;
        self.menu_item = MenuItem::Resume;
        self.active_state_mut().drag = None;
    }

    const fn close_menu(&mut self) {
        self.menu_view = MenuView::Closed;
        self.menu_item = MenuItem::Resume;
    }

    fn move_menu_item(&mut self, delta: isize) {
        if self.menu_view == MenuView::Main {
            self.menu_item = self.menu_item.move_by(delta);
        }
    }

    const fn open_menu_parent(&mut self) {
        match self.menu_view {
            MenuView::Closed => {}
            MenuView::Main => self.close_menu(),
            MenuView::Controls | MenuView::ResetConfirm => self.menu_view = MenuView::Main,
        }
    }

    fn activate_menu_item(&mut self) {
        match self.menu_view {
            MenuView::Closed => {}
            MenuView::Main => match self.menu_item {
                MenuItem::Resume => self.close_menu(),
                MenuItem::Controls => self.menu_view = MenuView::Controls,
                MenuItem::ResetGame => self.menu_view = MenuView::ResetConfirm,
            },
            MenuView::Controls => self.menu_view = MenuView::Main,
            MenuView::ResetConfirm => self.reset_active_game(),
        }
    }

    fn reset_active_game(&mut self) {
        let catalog_index = self.active_catalog;
        self.states[catalog_index] = CatalogState::new(&self.catalogs[catalog_index]);
        self.states[catalog_index].banner = Some(Banner::new("game reset", 6, None));
        self.close_menu();
        self.sync_active_palette_page_to_cursor();
    }

    fn drop_element_into_slot(&mut self, element_index: usize, slot: usize) {
        let catalog = self.active_catalog();
        let element_name = catalog.canonical_name(element_index).to_string();
        if slot > 1 {
            return;
        }

        let should_resolve = {
            let state = self.active_state_mut();
            state.selected[slot] = Some(element_index);
            state.slot_flash[slot] = 4;
            state.selected[0].is_some() && state.selected[1].is_some()
        };

        if should_resolve {
            self.resolve_active_selection();
            return;
        }

        self.active_state_mut().banner =
            Some(Banner::new(format!("selected {}", element_name), 4, None));
    }

    fn combine_two_elements(&mut self, left: usize, right: usize) {
        {
            let state = self.active_state_mut();
            state.selected = [Some(left), Some(right)];
        }
        self.resolve_active_selection();
    }

    fn select_visible_slot(&mut self, palette_slot: usize) {
        let palette_index = self
            .active_palette_page_start()
            .saturating_add(palette_slot);
        let palette = self.active_palette();
        let Some(&element_index) = palette.get(palette_index) else {
            return;
        };
        self.select_element_by_index(element_index);
    }

    fn select_cursor_element(&mut self) {
        let palette = self.active_palette();
        let cursor = self
            .active_state()
            .palette_cursor
            .min(palette.len().saturating_sub(1));
        let Some(&element_index) = palette.get(cursor) else {
            return;
        };
        self.select_element_by_index(element_index);
    }

    fn select_element_by_index(&mut self, element_index: usize) {
        let catalog = self.active_catalog();
        let element_name = catalog.canonical_name(element_index).to_string();
        let state = self.active_state_mut();

        match state.selected {
            [None, None] => {
                state.recipe_preview = None;
                state.selected[0] = Some(element_index);
                state.banner = Some(Banner::new(format!("selected {}", element_name), 4, None));
            }
            [Some(_), None] => {
                state.selected[1] = Some(element_index);
                self.resolve_active_selection();
            }
            [Some(_), Some(_)] => {
                state.recipe_preview = None;
                state.selected = [Some(element_index), None];
                state.banner = Some(Banner::new(format!("selected {}", element_name), 4, None));
            }
            [None, Some(_)] => {
                state.recipe_preview = None;
                state.selected = [Some(element_index), None];
                state.banner = Some(Banner::new(format!("selected {}", element_name), 4, None));
            }
        }
    }

    fn move_palette_cursor(&mut self, delta: isize) {
        let palette = self.active_palette();
        if palette.is_empty() {
            return;
        }

        let len = palette.len() as isize;
        let mut cursor = self.active_state().palette_cursor as isize + delta;
        if cursor < 0 {
            cursor = 0;
        }
        if cursor >= len {
            cursor = len - 1;
        }

        let page_size = self.inventory_visible_capacity().max(1);
        let state = self.active_state_mut();
        state.palette_cursor = cursor as usize;
        Self::sync_palette_page(state, palette.len(), page_size);
    }

    fn move_palette_cursor_to_start(&mut self) {
        let palette = self.active_palette();
        if palette.is_empty() {
            return;
        }
        let page_size = self.inventory_visible_capacity().max(1);
        let state = self.active_state_mut();
        state.palette_cursor = 0;
        Self::sync_palette_page(state, palette.len(), page_size);
    }

    fn move_palette_cursor_to_end(&mut self) {
        let palette = self.active_palette();
        if palette.is_empty() {
            return;
        }
        let page_size = self.inventory_visible_capacity().max(1);
        let state = self.active_state_mut();
        state.palette_cursor = palette.len() - 1;
        Self::sync_palette_page(state, palette.len(), page_size);
    }

    fn move_palette_page(&mut self, delta: isize) {
        let palette_len = self.active_palette().len();
        if palette_len == 0 {
            return;
        }
        let page_size = self.inventory_visible_capacity().max(1);
        let page_count = palette_len.div_ceil(page_size).max(1);
        let current = self
            .active_state()
            .palette_page
            .min(page_count.saturating_sub(1));
        let next = (current as isize)
            .saturating_add(delta)
            .clamp(0, page_count.saturating_sub(1) as isize) as usize;
        self.set_palette_page(next);
    }

    fn set_palette_page(&mut self, page: usize) {
        let palette_len = self.active_palette().len();
        if palette_len == 0 {
            return;
        }
        let page_size = self.inventory_visible_capacity().max(1);
        let page_count = palette_len.div_ceil(page_size).max(1);
        let page = page.min(page_count.saturating_sub(1));
        let cursor = page
            .saturating_mul(page_size)
            .min(palette_len.saturating_sub(1));
        let state = self.active_state_mut();
        state.palette_page = page;
        state.palette_cursor = cursor;
    }

    fn sync_active_palette_page_to_cursor(&mut self) {
        let palette_len = self.active_palette().len();
        let page_size = self.inventory_visible_capacity().max(1);
        let state = self.active_state_mut();
        Self::sync_palette_page(state, palette_len, page_size);
    }

    fn sync_palette_page(state: &mut CatalogState, palette_len: usize, page_size: usize) {
        if palette_len == 0 {
            state.palette_page = 0;
            state.palette_cursor = 0;
            return;
        }

        let page_size = page_size.max(1);
        state.palette_cursor = state.palette_cursor.min(palette_len.saturating_sub(1));
        let max_page = palette_len.saturating_sub(1) / page_size;
        state.palette_page = (state.palette_cursor / page_size).min(max_page);
    }

    pub(crate) fn active_palette_page_start(&self) -> usize {
        let palette_len = self.active_palette().len();
        if palette_len == 0 || self.viewport.width == 0 || self.viewport.height == 0 {
            return 0;
        }
        let scene = scene_layout(self.viewport);
        atlas_page_start(scene.board, palette_len, self.active_state().palette_page)
    }

    pub(crate) fn active_palette_page_size(&self) -> usize {
        let palette_len = self.active_palette().len();
        if palette_len == 0 || self.viewport.width == 0 || self.viewport.height == 0 {
            return 1;
        }
        let scene = scene_layout(self.viewport);
        atlas_page_size(scene.board, palette_len)
    }

    fn resolve_active_selection(&mut self) {
        let catalog_index = self.active_catalog;
        let [Some(left), Some(right)] = self.states[catalog_index].selected else {
            return;
        };

        let catalog = &self.catalogs[catalog_index];
        let outputs = catalog.recipe_outputs(left, right);

        if outputs.is_empty() {
            let state = &mut self.states[catalog_index];
            state.banner = Some(Banner::new("nothing happens", 6, None));
            state.recipe_preview = None;
            state.selected = [Some(left), Some(right)];
            return;
        }

        let maybe_new = outputs.iter().copied().find(|index| {
            !self.states[catalog_index]
                .discovered
                .get(*index)
                .copied()
                .unwrap_or(false)
        });

        let state = &mut self.states[catalog_index];
        state.selected = [Some(left), Some(right)];
        if let Some(element_index) = maybe_new {
            self.states[catalog_index].recipe_preview = Some(RecipePreview {
                left,
                right,
                result: element_index,
            });
            self.discover_element(catalog_index, element_index, true);
        } else {
            state.recipe_preview = Some(RecipePreview {
                left,
                right,
                result: outputs[0],
            });
            state.banner = Some(Banner::new("already known", 6, None));
        }
    }

    fn discover_element(&mut self, catalog_index: usize, element_index: usize, from_recipe: bool) {
        if self.states[catalog_index]
            .discovered
            .get(element_index)
            .copied()
            .unwrap_or(false)
        {
            return;
        }

        let element_name = self.catalogs[catalog_index]
            .canonical_name(element_index)
            .to_string();
        {
            let state = &mut self.states[catalog_index];
            state.discovered[element_index] = true;
            state.discovery_order.push(element_index);
        }

        let palette = self.active_palette();
        if let Some(position) = palette
            .iter()
            .position(|candidate| *candidate == element_index)
        {
            let page_size = self.inventory_visible_capacity().max(1);
            let state = &mut self.states[catalog_index];
            state.palette_cursor = position;
            Self::sync_palette_page(state, palette.len(), page_size);
        }

        {
            let state = &mut self.states[catalog_index];
            state.recent.push_front(element_index);
            while state.recent.len() > 6 {
                state.recent.pop_back();
            }
            state
                .effects
                .push(ElementEffect::birth(element_index, &element_name));
            state.banner = Some(Banner::new(
                if from_recipe {
                    "new element discovered".to_string()
                } else {
                    "new element unlocked".to_string()
                },
                10,
                Some(element_index),
            ));
        }
    }

    fn reconcile_unlocks(&mut self, catalog_index: usize) {
        loop {
            let discovered_count = self.states[catalog_index].discovered_count();
            let unlocks = self.catalogs[catalog_index].count_unlocks.clone();
            let mut changed = false;

            for (index, minimum) in unlocks {
                if discovered_count >= minimum
                    && !self.states[catalog_index]
                        .discovered
                        .get(index)
                        .copied()
                        .unwrap_or(false)
                {
                    self.discover_element(catalog_index, index, false);
                    changed = true;
                }
            }

            if !changed {
                break;
            }
        }
    }

    fn age_banner(&mut self, catalog_index: usize) {
        if let Some(banner) = self.states[catalog_index].banner.as_mut() {
            if banner.ttl > 0 {
                banner.ttl -= 1;
            }
            if banner.ttl == 0 {
                self.states[catalog_index].banner = None;
            }
        }
    }

    fn age_effects(&mut self, catalog_index: usize) {
        self.states[catalog_index]
            .effects
            .retain_mut(ElementEffect::age);
    }

    fn age_slot_flashes(&mut self, catalog_index: usize) {
        for flash in &mut self.states[catalog_index].slot_flash {
            *flash = flash.saturating_sub(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input_event::{
        Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
    };
    use crate::layout::{
        atlas_page_size, atlas_panel, board_inner, grimoire_layout, iso_board_cells, scene_layout,
    };

    // Indices in TEST_JSON:
    // 0 Air* 1 Earth* 2 Fire* 3 Water* (base) | 4 Steam 5 Mud 6 Lava 7 Dust
    // 8 Rain | 9 Pressure(min5) 10 Plasma(min6) | 11 Stone 12 Sand 13 Glass
    const TEST_JSON: &str = r#"{
        "source": "unit-test",
        "total": 14,
        "elements": [
            {"name": "Air", "base": true},
            {"name": "Earth", "base": true},
            {"name": "Fire", "base": true},
            {"name": "Water", "base": true},
            {"name": "Steam", "recipes": [["Water", "Fire"]]},
            {"name": "Mud", "recipes": [["Water", "Earth"]]},
            {"name": "Lava", "recipes": [["Fire", "Earth"]]},
            {"name": "Dust", "recipes": [["Air", "Earth"]]},
            {"name": "Rain", "recipes": [["Water", "Air"]]},
            {"name": "Pressure", "unlock": {"kind": "discovered_count", "minimum": 5}},
            {"name": "Plasma", "unlock": {"kind": "discovered_count", "minimum": 6}},
            {"name": "Stone"},
            {"name": "Sand"},
            {"name": "Glass"}
        ]
    }"#;

    const AIR: usize = 0;
    const EARTH: usize = 1;
    const FIRE: usize = 2;
    const WATER: usize = 3;
    const STEAM: usize = 4;
    const LAVA: usize = 6;
    const PRESSURE: usize = 9;
    const PLASMA: usize = 10;

    fn app() -> App {
        App::with_catalogs(vec![GameCatalog::from_raw_json(TEST_JSON)])
    }

    fn sized(width: u16, height: u16) -> App {
        let mut a = app();
        a.viewport = Rect::new(0, 0, width, height);
        a
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

    fn discovered(app: &App, index: usize) -> bool {
        app.active_state().discovered[index]
    }

    /// Centre of the board cell currently rendering `element_index`.
    fn board_cell_center(app: &App, element_index: usize) -> (u16, u16) {
        let scene = scene_layout(app.viewport);
        let panel = atlas_panel(
            scene.board,
            atlas_page_size(scene.board, app.active_palette().len()),
        );
        let inner = board_inner(panel);
        let palette = app.active_palette();
        let cells = iso_board_cells(inner, palette.len(), app.active_palette_page_start());
        let cell = cells
            .iter()
            .find(|cell| palette[cell.index] == element_index)
            .expect("element should have a visible board cell");
        (
            cell.top.x + cell.top.width / 2,
            cell.top.y + cell.top.height / 2,
        )
    }

    fn workbench_slot_center(app: &App, slot: usize) -> (u16, u16) {
        let scene = scene_layout(app.viewport);
        let grimoire = grimoire_layout(scene.grimoire);
        let rect = match slot {
            0 => grimoire.slot_left,
            1 => grimoire.slot_right,
            _ => panic!("unsupported workbench slot index: {slot}"),
        };
        (rect.x + rect.width / 2, rect.y + rect.height / 2)
    }

    #[test]
    fn starts_with_only_base_elements_discovered() {
        let app = app();
        assert_eq!(app.active_discovered_count(), 4);
        assert_eq!(app.active_total(), 14);
        assert!(discovered(&app, AIR) && discovered(&app, WATER));
        assert!(!discovered(&app, STEAM));
        assert_eq!(app.active_palette(), vec![0, 1, 2, 3]);
    }

    #[test]
    fn selecting_one_element_seats_it_and_announces_it() {
        let mut app = app();
        app.select_element_by_index(WATER);
        assert_eq!(app.active_state().selected, [Some(WATER), None]);
        assert_eq!(app.active_banner_text(), Some("selected Water"));
    }

    #[test]
    fn selecting_two_compatible_elements_discovers_the_result() {
        let mut app = app();
        app.select_element_by_index(WATER);
        app.select_element_by_index(FIRE);
        assert!(discovered(&app, STEAM));
        assert_eq!(app.active_discovered_count(), 5);
        assert_eq!(app.active_banner_text(), Some("new element discovered"));
        assert_eq!(app.active_banner_highlight(), Some(STEAM));
        let preview = app.active_state().recipe_preview.expect("preview set");
        assert_eq!(preview.result, STEAM);
        assert_eq!(app.active_state().selected, [Some(WATER), Some(FIRE)]);
    }

    #[test]
    fn menu_reset_restores_the_current_book_to_a_fresh_game() {
        let mut app = app();
        app.combine_two_elements(WATER, FIRE);
        assert!(discovered(&app, STEAM));

        app.handle_event(key(KeyCode::Char('m')));
        app.handle_event(key(KeyCode::Down));
        app.handle_event(key(KeyCode::Down));
        app.handle_event(key(KeyCode::Enter));
        app.handle_event(key(KeyCode::Enter));

        assert_eq!(app.active_discovered_count(), 4);
        assert!(!discovered(&app, STEAM));
        assert_eq!(app.active_state().selected, [None, None]);
        assert_eq!(app.active_banner_text(), Some("game reset"));
    }

    #[test]
    fn combining_an_unknown_pair_says_nothing_happens() {
        let mut app = app();
        app.combine_two_elements(AIR, FIRE); // no recipe defined
        assert_eq!(app.active_banner_text(), Some("nothing happens"));
        assert_eq!(app.active_state().selected, [Some(AIR), Some(FIRE)]);
        assert_eq!(app.active_discovered_count(), 4);
    }

    #[test]
    fn recombining_a_known_result_says_already_known() {
        let mut app = app();
        app.combine_two_elements(WATER, FIRE); // discovers Steam
        assert!(discovered(&app, STEAM));
        app.combine_two_elements(WATER, FIRE); // again
        assert_eq!(app.active_banner_text(), Some("already known"));
        assert_eq!(app.active_discovered_count(), 5); // no extra discovery
    }

    #[test]
    fn selecting_with_both_slots_full_restarts_the_pair() {
        let mut app = app();
        app.active_state_mut().selected = [Some(WATER), Some(FIRE)];
        app.select_element_by_index(EARTH);
        assert_eq!(app.active_state().selected, [Some(EARTH), None]);
    }

    #[test]
    fn selecting_from_a_dangling_second_slot_restarts_cleanly() {
        let mut app = app();
        app.active_state_mut().selected = [None, Some(FIRE)];
        app.select_element_by_index(WATER);
        assert_eq!(app.active_state().selected, [Some(WATER), None]);
    }

    #[test]
    fn discover_element_is_idempotent() {
        let mut app = app();
        app.discover_element(0, STEAM, true);
        let order_len = app.active_state().discovery_order.len();
        app.discover_element(0, STEAM, true);
        assert_eq!(app.active_state().discovery_order.len(), order_len);
    }

    #[test]
    fn recent_discoveries_are_capped_at_six() {
        let mut app = app();
        for index in [4usize, 5, 6, 7, 8, 11, 12] {
            app.discover_element(0, index, false);
        }
        let recent = &app.active_state().recent;
        assert_eq!(recent.len(), 6);
        // Most recent is at the front.
        assert_eq!(*recent.front().unwrap(), 12);
    }

    #[test]
    fn tick_unlocks_cascade_by_discovered_count() {
        let mut app = app();
        // Reach a count of 5 by crafting Steam; the tick should then cascade:
        // count 5 -> Pressure (count 6) -> Plasma (count 7).
        app.combine_two_elements(WATER, FIRE);
        assert_eq!(app.active_discovered_count(), 5);
        assert!(!discovered(&app, PRESSURE));
        app.tick();
        assert!(discovered(&app, PRESSURE), "min-5 unlock should fire");
        assert!(discovered(&app, PLASMA), "cascade min-6 unlock should fire");
        assert_eq!(app.active_discovered_count(), 7);
    }

    #[test]
    fn digit_key_selects_the_matching_visible_slot() {
        let mut app = sized(100, 40);
        app.handle_event(key(KeyCode::Char('1')));
        assert_eq!(app.active_state().selected, [Some(AIR), None]);
        assert_eq!(app.active_banner_text(), Some("selected Air"));
    }

    #[test]
    fn preview_drag_helper_sets_drag_for_named_element() {
        let mut app = app();

        app.preview_drag_element("Water", 10, 11);

        let drag = app.active_drag().expect("preview drag should be active");
        assert_eq!(drag.element_index, WATER);
        assert_eq!(drag.column, 10);
        assert_eq!(drag.row, 11);
    }

    #[test]
    fn resize_event_updates_the_cached_viewport_before_next_draw() {
        let mut app = sized(100, 40);

        app.handle_event(Event::Resize(64, 24));

        assert_eq!(app.viewport, Rect::new(0, 0, 64, 24));
    }

    #[test]
    fn esc_and_c_clear_the_selection() {
        let mut app = app();
        app.select_element_by_index(WATER);
        app.handle_event(key(KeyCode::Esc));
        assert_eq!(app.active_state().selected, [None, None]);

        app.select_element_by_index(WATER);
        app.handle_event(key(KeyCode::Char('c')));
        assert_eq!(app.active_state().selected, [None, None]);
    }

    #[test]
    fn cursor_navigation_clamps_and_jumps_to_ends() {
        let mut app = sized(100, 40);
        // 4 base elements -> indices 0..3.
        app.handle_event(key(KeyCode::Right));
        assert_eq!(app.active_state().palette_cursor, 1);
        app.handle_event(key(KeyCode::Left));
        app.handle_event(key(KeyCode::Left)); // clamps at 0
        assert_eq!(app.active_state().palette_cursor, 0);
        app.handle_event(key(KeyCode::End));
        assert_eq!(app.active_state().palette_cursor, 3);
        app.handle_event(key(KeyCode::Home));
        assert_eq!(app.active_state().palette_cursor, 0);
    }

    #[test]
    fn page_keys_jump_between_discrete_atlas_pages() {
        let mut app = sized(100, 12);
        app.reveal_elements_for_preview(&[
            "Steam", "Mud", "Lava", "Dust", "Rain", "Stone", "Sand", "Glass",
        ]);
        app.handle_event(key(KeyCode::Home));
        let page_size = app.inventory_visible_capacity().max(1);
        assert!(
            app.active_palette().len() > page_size,
            "fixture should need multiple atlas pages"
        );

        app.handle_event(key(KeyCode::PageDown));
        assert_eq!(app.active_state().palette_page, 1);
        assert_eq!(app.active_state().palette_cursor, page_size);

        app.handle_event(key(KeyCode::PageUp));
        assert_eq!(app.active_state().palette_page, 0);
        assert_eq!(app.active_state().palette_cursor, 0);
    }

    #[test]
    fn mouse_wheel_does_not_scroll_the_atlas_grid() {
        let mut app = sized(100, 12);
        app.reveal_elements_for_preview(&[
            "Steam", "Mud", "Lava", "Dust", "Rain", "Stone", "Sand", "Glass",
        ]);
        app.handle_event(key(KeyCode::Home));
        let (x, y) = board_cell_center(&app, AIR);

        app.handle_event(mouse(MouseEventKind::ScrollDown, x, y));

        assert_eq!(app.active_state().palette_page, 0);
        assert_eq!(app.active_state().palette_cursor, 0);
    }

    #[test]
    fn dragging_one_board_element_onto_another_combines_them() {
        let mut app = sized(100, 40);
        let (wx, wy) = board_cell_center(&app, WATER);
        let (fx, fy) = board_cell_center(&app, FIRE);
        app.handle_event(mouse(MouseEventKind::Down(MouseButton::Left), wx, wy));
        assert!(app.active_drag().is_some());
        app.handle_event(mouse(MouseEventKind::Drag(MouseButton::Left), fx, fy));
        app.handle_event(mouse(MouseEventKind::Up(MouseButton::Left), fx, fy));
        assert!(app.active_drag().is_none());
        assert!(discovered(&app, STEAM));
    }

    #[test]
    fn releasing_a_drag_on_empty_space_just_selects_the_element() {
        let mut app = sized(100, 40);
        let (wx, wy) = board_cell_center(&app, WATER);
        app.handle_event(mouse(MouseEventKind::Down(MouseButton::Left), wx, wy));
        // Release far outside any pane.
        app.handle_event(mouse(MouseEventKind::Up(MouseButton::Left), 99, 39));
        assert!(app.active_drag().is_none());
        assert_eq!(app.active_state().selected, [Some(WATER), None]);
    }

    #[test]
    fn filling_both_recipe_slots_resolves_the_craft() {
        let mut app = app();
        app.drop_element_into_slot(WATER, 0);
        assert_eq!(app.active_state().selected, [Some(WATER), None]);
        assert_eq!(app.active_banner_text(), Some("selected Water"));
        app.drop_element_into_slot(FIRE, 1);
        assert!(discovered(&app, STEAM));
    }

    #[test]
    fn successful_craft_keeps_slots_live_for_follow_up_replacement() {
        let mut app = app();
        app.drop_element_into_slot(WATER, 0);
        app.drop_element_into_slot(FIRE, 1);
        assert!(discovered(&app, STEAM));
        assert_eq!(app.active_state().selected, [Some(WATER), Some(FIRE)]);
        let preview = app.active_state().recipe_preview.expect("recipe preview");
        assert_eq!(
            (preview.left, preview.right, preview.result),
            (WATER, FIRE, STEAM)
        );
    }

    #[test]
    fn dragging_earth_onto_the_left_workbench_slot_replaces_water_and_discovers_lava() {
        let mut app = sized(100, 40);
        app.drop_element_into_slot(WATER, 0);
        app.drop_element_into_slot(FIRE, 1);
        let (earth_x, earth_y) = board_cell_center(&app, EARTH);
        let (left_x, left_y) = workbench_slot_center(&app, 0);

        app.handle_event(mouse(
            MouseEventKind::Down(MouseButton::Left),
            earth_x,
            earth_y,
        ));
        app.handle_event(mouse(
            MouseEventKind::Drag(MouseButton::Left),
            left_x,
            left_y,
        ));
        app.handle_event(mouse(MouseEventKind::Up(MouseButton::Left), left_x, left_y));

        assert_eq!(app.active_state().selected, [Some(EARTH), Some(FIRE)]);
        assert!(discovered(&app, LAVA));
        let preview = app.active_state().recipe_preview.expect("recipe preview");
        assert_eq!(
            (preview.left, preview.right, preview.result),
            (EARTH, FIRE, LAVA)
        );
    }

    #[test]
    fn dropping_into_an_out_of_range_slot_is_a_noop() {
        let mut app = app();
        app.drop_element_into_slot(WATER, 2);
        assert_eq!(app.active_state().selected, [None, None]);
        assert!(app.active_banner_text().is_none());
    }

    #[test]
    fn combined_book_is_playable_to_completion_through_app_state() {
        let mut app = App::new();

        loop {
            let before = app.active_discovered_count();
            app.tick();
            let discovered: Vec<usize> = app
                .active_state()
                .discovered
                .iter()
                .enumerate()
                .filter_map(|(index, seen)| (*seen).then_some(index))
                .collect();

            for (position, &left) in discovered.iter().enumerate() {
                for &right in &discovered[position..] {
                    app.combine_two_elements(left, right);
                }
            }

            if app.active_discovered_count() == before {
                break;
            }
        }

        let missing: Vec<_> = app
            .active_catalog()
            .elements
            .iter()
            .enumerate()
            .filter_map(|(index, element)| {
                (!app.active_state().discovered[index]).then_some(element.name.as_str())
            })
            .collect();
        assert!(
            missing.is_empty(),
            "combined recipe book should be discoverable through app resolution, missing: {missing:?}"
        );
    }

    #[test]
    fn reveal_for_preview_unlocks_and_resets_transient_state() {
        let mut app = sized(100, 40);
        app.drop_element_into_slot(WATER, 0); // dirties selection/banner/slot flash
        assert_ne!(app.active_state().slot_flash, [0, 0]);
        app.reveal_elements_for_preview(&["Steam", "Mud"]);
        assert!(discovered(&app, STEAM) && discovered(&app, 5));
        let state = app.active_state();
        assert_eq!(state.selected, [None, None]);
        assert!(state.banner.is_none());
        assert!(state.effects.is_empty());
        assert_eq!(state.slot_flash, [0, 0]);
        assert!(state.recipe_preview.is_none());
    }

    #[test]
    fn banner_ages_out_after_its_ttl() {
        let mut app = app();
        app.select_element_by_index(WATER); // banner ttl = 4
        assert!(app.active_banner_text().is_some());
        for _ in 0..4 {
            app.tick();
        }
        assert!(app.active_banner_text().is_none());
    }
}
