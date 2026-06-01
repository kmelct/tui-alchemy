use crate::data::{
    GameCatalog, active_palette_indices, base_discovery_state, discovered_count, normalize,
};
use crate::effects::ElementEffect;
use crate::layout::{
    board_inner, catalog_strip_rects, contains, grimoire_layout, iso_board_cells, iso_capacity,
    iso_columns, iso_hit, rail_sections, scene_layout,
};
use crate::ui;
use crossterm::event::{Event, KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};
use ratatui::Frame;
use ratatui::layout::Rect;
use std::collections::VecDeque;

#[allow(dead_code)]
pub(crate) const PALETTE_PAGE_SIZE: usize = 12;

#[derive(Debug, Clone, Copy)]
pub(crate) enum DragOrigin {
    Inventory,
    Canvas,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct DragState {
    pub element_index: usize,
    pub origin: DragOrigin,
    pub column: u16,
    pub row: u16,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RecipePreview {
    pub left: usize,
    pub right: usize,
    pub result: usize,
}

#[derive(Debug, Clone, Copy)]
enum Pane {
    Inventory,
    Canvas,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
enum HitTarget {
    Inventory(usize),
    Canvas(usize),
    Slot(usize),
}

#[derive(Debug, Clone)]
pub(crate) struct Banner {
    pub text: String,
    pub ttl: u8,
    pub highlight: Option<usize>,
}

impl Banner {
    fn new(text: impl Into<String>, ttl: u8, highlight: Option<usize>) -> Self {
        Self {
            text: text.into(),
            ttl,
            highlight,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct CatalogState {
    pub discovered: Vec<bool>,
    pub discovery_order: Vec<usize>,
    pub selected: [Option<usize>; 2],
    pub palette_cursor: usize,
    pub palette_scroll: usize,
    pub canvas_scroll: usize,
    pub drag: Option<DragState>,
    pub recent: VecDeque<usize>,
    pub banner: Option<Banner>,
    pub effects: Vec<ElementEffect>,
    pub recipe_preview: Option<RecipePreview>,
}

impl CatalogState {
    fn new(catalog: &GameCatalog) -> Self {
        let discovered = base_discovery_state(catalog);
        let discovery_order = catalog.base_indices.clone();

        Self {
            discovered,
            discovery_order,
            selected: [None, None],
            palette_cursor: 0,
            palette_scroll: 0,
            canvas_scroll: 0,
            drag: None,
            recent: VecDeque::new(),
            banner: None,
            effects: Vec::new(),
            recipe_preview: None,
        }
    }

    fn discovered_count(&self) -> usize {
        discovered_count(&self.discovered)
    }

    fn clear_selection(&mut self) {
        self.selected = [None, None];
    }
}

#[derive(Debug)]
pub struct App {
    pub(crate) catalogs: Vec<GameCatalog>,
    pub(crate) states: Vec<CatalogState>,
    pub(crate) active_catalog: usize,
    pub(crate) tick_counter: u64,
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
            viewport: Rect::new(0, 0, 0, 0),
        }
    }

    pub fn render(&mut self, frame: &mut Frame<'_>) {
        self.viewport = frame.area();
        ui::render_app(frame, self);
    }

    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::Key(key) => self.handle_key(key),
            Event::Mouse(mouse) => self.handle_mouse(mouse),
            Event::Paste(_) | Event::Resize(_, _) => {}
            _ => {}
        }
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
        state.recipe_preview = None;
        state.palette_cursor = self.catalogs[catalog_index].base_indices.len();
        state.palette_scroll = self.catalogs[catalog_index].base_indices.len();
        state.canvas_scroll = 0;
    }

    pub fn tick(&mut self) {
        self.tick_counter = self.tick_counter.wrapping_add(1);
        for index in 0..self.catalogs.len() {
            self.reconcile_unlocks(index);
            self.age_banner(index);
            self.age_effects(index);
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

    fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Tab => self.switch_catalog(1),
            KeyCode::BackTab => self.switch_catalog(-1),
            KeyCode::Up | KeyCode::Char('k') => {
                self.move_palette_cursor(-(self.inventory_columns().max(1) as isize))
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.move_palette_cursor(self.inventory_columns().max(1) as isize)
            }
            KeyCode::Left | KeyCode::Char('h') => self.move_palette_cursor(-1),
            KeyCode::Right | KeyCode::Char('l') => self.move_palette_cursor(1),
            KeyCode::PageUp => {
                self.move_palette_cursor(-(self.inventory_visible_capacity().max(1) as isize))
            }
            KeyCode::PageDown => {
                self.move_palette_cursor(self.inventory_visible_capacity().max(1) as isize)
            }
            KeyCode::Home => self.move_palette_cursor_to_start(),
            KeyCode::End => self.move_palette_cursor_to_end(),
            KeyCode::Esc => self.active_state_mut().clear_selection(),
            KeyCode::Enter => self.select_cursor_element(),
            KeyCode::Char(ch) if key.modifiers.is_empty() => {
                if let Some(digit) = ch.to_digit(10) {
                    if (1..=9).contains(&digit) {
                        self.select_visible_slot((digit - 1) as usize);
                    }
                } else if ch == 'c' || ch == 'C' {
                    self.active_state_mut().clear_selection();
                }
            }
            _ => {}
        }
    }

    fn handle_mouse(&mut self, mouse: MouseEvent) {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if let Some(catalog_index) = self.hit_catalog_control(mouse.column, mouse.row) {
                    self.switch_to_catalog(catalog_index);
                } else {
                    self.begin_drag(mouse);
                }
            }
            MouseEventKind::Drag(MouseButton::Left) => self.update_drag(mouse),
            MouseEventKind::Up(MouseButton::Left) => self.finish_drag(mouse),
            MouseEventKind::ScrollUp => self.scroll_at(mouse.column, mouse.row, -1),
            MouseEventKind::ScrollDown => self.scroll_at(mouse.column, mouse.row, 1),
            _ => {}
        }
    }

    fn begin_drag(&mut self, mouse: MouseEvent) {
        let Some(hit) = self.hit_test(mouse.column, mouse.row) else {
            return;
        };

        let element_index = match hit {
            HitTarget::Inventory(index) | HitTarget::Canvas(index) => index,
            HitTarget::Slot(slot) => match self.active_state().selected[slot] {
                Some(index) => index,
                None => return,
            },
        };

        let origin = match hit {
            HitTarget::Inventory(_) => DragOrigin::Inventory,
            HitTarget::Canvas(_) | HitTarget::Slot(_) => DragOrigin::Canvas,
        };
        let state = self.active_state_mut();
        state.drag = Some(DragState {
            element_index,
            origin,
            column: mouse.column,
            row: mouse.row,
        });
        state.banner = None;
        state.recipe_preview = None;
    }

    fn update_drag(&mut self, mouse: MouseEvent) {
        let state = self.active_state_mut();
        if let Some(drag) = state.drag.as_mut() {
            drag.column = mouse.column;
            drag.row = mouse.row;
        }
    }

    fn finish_drag(&mut self, mouse: MouseEvent) {
        let Some(drag) = self.active_state().drag else {
            return;
        };

        let target = self.hit_test(mouse.column, mouse.row);
        self.active_state_mut().drag = None;

        match target {
            Some(HitTarget::Slot(slot)) => self.drop_element_into_slot(drag.element_index, slot),
            Some(HitTarget::Inventory(target_index)) | Some(HitTarget::Canvas(target_index))
                if target_index != drag.element_index =>
            {
                self.combine_two_elements(drag.element_index, target_index);
            }
            Some(HitTarget::Inventory(_)) | Some(HitTarget::Canvas(_)) => {
                self.select_element_by_index(drag.element_index);
            }
            None => self.select_element_by_index(drag.element_index),
        }
    }

    fn scroll_at(&mut self, column: u16, row: u16, delta: isize) {
        match self.hit_panel(column, row) {
            Some(Pane::Inventory) => self.scroll_inventory(delta),
            Some(Pane::Canvas) => self.scroll_canvas(delta),
            None => {}
        }
    }

    fn scroll_inventory(&mut self, delta: isize) {
        if delta == 0 {
            return;
        }

        let palette = self.active_palette();
        if palette.is_empty() {
            return;
        }

        let step = self.inventory_columns().max(1) as isize;
        let max_scroll = palette
            .len()
            .saturating_sub(self.inventory_visible_capacity().max(1));
        let current_scroll = self.active_state().palette_scroll as isize;
        let next = current_scroll + delta.saturating_mul(step);
        let state = self.active_state_mut();
        state.palette_scroll = next.clamp(0, max_scroll as isize) as usize;
        state.palette_cursor = state.palette_scroll.min(palette.len().saturating_sub(1));
    }

    fn scroll_canvas(&mut self, delta: isize) {
        if delta == 0 {
            return;
        }

        let visible = self.visible_canvas_count();
        let state = self.active_state_mut();
        let total = state.discovery_order.len();
        let max_scroll = total.saturating_sub(visible);
        let next = state.canvas_scroll as isize + delta;
        state.canvas_scroll = next.clamp(0, max_scroll as isize) as usize;
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

    fn hit_panel(&self, column: u16, row: u16) -> Option<Pane> {
        if self.viewport.width == 0 || self.viewport.height == 0 {
            return None;
        }

        let scene = scene_layout(self.viewport);
        if contains(scene.board, column, row) {
            return Some(Pane::Inventory);
        }
        if contains(scene.grimoire, column, row) {
            return Some(Pane::Canvas);
        }
        None
    }

    fn hit_test(&self, column: u16, row: u16) -> Option<HitTarget> {
        if self.viewport.width == 0 || self.viewport.height == 0 {
            return None;
        }

        let scene = scene_layout(self.viewport);

        if contains(scene.board, column, row) {
            return self.hit_inventory(column, row);
        }

        if contains(scene.grimoire, column, row) {
            return self.hit_canvas(column, row);
        }

        None
    }

    fn hit_inventory(&self, column: u16, row: u16) -> Option<HitTarget> {
        let scene = scene_layout(self.viewport);
        let inner = board_inner(scene.board);

        if !contains(inner, column, row) {
            return None;
        }

        let palette = self.active_palette();
        let state = self.active_state();
        let cells = iso_board_cells(inner, palette.len(), state.palette_scroll);
        iso_hit(&cells, column, row)
            .and_then(|visible_index| palette.get(visible_index).copied())
            .map(HitTarget::Inventory)
    }

    fn hit_canvas(&self, column: u16, row: u16) -> Option<HitTarget> {
        let scene = scene_layout(self.viewport);
        let grimoire = grimoire_layout(scene.grimoire);

        if contains(grimoire.slot_left, column, row) {
            return Some(HitTarget::Slot(0));
        }
        if contains(grimoire.slot_right, column, row) {
            return Some(HitTarget::Slot(1));
        }

        None
    }

    fn hit_catalog_control(&self, column: u16, row: u16) -> Option<usize> {
        if self.catalogs.len() <= 1 {
            return None;
        }

        if self.viewport.width == 0 || self.viewport.height == 0 {
            return None;
        }

        let scene = scene_layout(self.viewport);
        if !contains(scene.rail, column, row) {
            return None;
        }

        let strip = rail_sections(scene.rail).catalog_strip;
        let rects = catalog_strip_rects(strip, self.catalogs.len());
        rects
            .into_iter()
            .find(|(_, rect)| contains(*rect, column, row))
            .map(|(index, _)| index)
    }

    #[allow(dead_code)]
    pub(crate) fn visible_canvas_indices(&self) -> Vec<usize> {
        let state = self.active_state();
        let reversed = state.discovery_order.iter().rev().copied();
        let skipped = reversed.skip(state.canvas_scroll);
        let visible = self.visible_canvas_count();
        skipped.take(visible).collect()
    }

    pub(crate) fn visible_canvas_count(&self) -> usize {
        let scene = scene_layout(self.viewport);
        iso_capacity(board_inner(scene.board))
    }

    fn inventory_columns(&self) -> usize {
        let scene = scene_layout(self.viewport);
        iso_columns(board_inner(scene.board))
    }

    fn inventory_visible_capacity(&self) -> usize {
        let scene = scene_layout(self.viewport);
        iso_capacity(board_inner(scene.board))
    }

    fn switch_catalog(&mut self, delta: isize) {
        if self.catalogs.len() <= 1 {
            return;
        }

        let len = self.catalogs.len() as isize;
        let mut next = self.active_catalog as isize + delta;
        if next < 0 {
            next = len - 1;
        }
        if next >= len {
            next = 0;
        }
        self.active_catalog = next as usize;
        self.active_state_mut().drag = None;
        self.ensure_palette_cursor_visible();
    }

    fn switch_to_catalog(&mut self, catalog_index: usize) {
        if catalog_index >= self.catalogs.len() {
            return;
        }

        self.active_catalog = catalog_index;
        self.active_state_mut().drag = None;
        self.ensure_palette_cursor_visible();
    }

    fn select_visible_slot(&mut self, palette_slot: usize) {
        let state = self.active_state();
        let palette_index = state.palette_scroll.saturating_add(palette_slot);
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
        Self::sync_palette_scroll(state, palette.len(), page_size);
    }

    fn move_palette_cursor_to_start(&mut self) {
        let palette = self.active_palette();
        if palette.is_empty() {
            return;
        }
        let page_size = self.inventory_visible_capacity().max(1);
        let state = self.active_state_mut();
        state.palette_cursor = 0;
        Self::sync_palette_scroll(state, palette.len(), page_size);
    }

    fn move_palette_cursor_to_end(&mut self) {
        let palette = self.active_palette();
        if palette.is_empty() {
            return;
        }
        let page_size = self.inventory_visible_capacity().max(1);
        let state = self.active_state_mut();
        state.palette_cursor = palette.len() - 1;
        Self::sync_palette_scroll(state, palette.len(), page_size);
    }

    fn ensure_palette_cursor_visible(&mut self) {
        let palette = self.active_palette();
        if palette.is_empty() {
            return;
        }
        let page_size = self.inventory_visible_capacity().max(1);
        let state = self.active_state_mut();
        state.palette_cursor = state.palette_cursor.min(palette.len().saturating_sub(1));
        Self::sync_palette_scroll(state, palette.len(), page_size);
    }

    fn sync_palette_scroll(state: &mut CatalogState, palette_len: usize, page_size: usize) {
        if palette_len <= page_size {
            state.palette_scroll = 0;
            return;
        }

        if state.palette_cursor < state.palette_scroll {
            state.palette_scroll = state.palette_cursor;
        } else if state.palette_cursor >= state.palette_scroll + page_size {
            state.palette_scroll = state
                .palette_cursor
                .saturating_add(1)
                .saturating_sub(page_size);
        }

        let max_scroll = palette_len.saturating_sub(page_size);
        state.palette_scroll = state.palette_scroll.min(max_scroll);
    }

    fn resolve_active_selection(&mut self) {
        let catalog_index = self.active_catalog;
        let [left, right] = self.states[catalog_index].selected;
        let (Some(left), Some(right)) = (left, right) else {
            return;
        };

        let catalog = &self.catalogs[catalog_index];
        let outputs = catalog.recipe_outputs(left, right);

        if outputs.is_empty() {
            let state = &mut self.states[catalog_index];
            state.banner = Some(Banner::new("nothing happens", 6, None));
            state.clear_selection();
            state.recipe_preview = None;
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
        state.clear_selection();

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
            state.canvas_scroll = 0;
        }

        let palette = self.active_palette();
        if let Some(position) = palette
            .iter()
            .position(|candidate| *candidate == element_index)
        {
            let page_size = self.inventory_visible_capacity().max(1);
            let state = &mut self.states[catalog_index];
            state.palette_cursor = position;
            Self::sync_palette_scroll(state, palette.len(), page_size);
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::CatalogKind;
    use crossterm::event::KeyModifiers;

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
    const PRESSURE: usize = 9;
    const PLASMA: usize = 10;

    fn app() -> App {
        App::with_catalogs(vec![GameCatalog::from_raw_json(
            CatalogKind::LittleAlchemy1,
            TEST_JSON,
        )])
    }

    fn app_two() -> App {
        App::with_catalogs(vec![
            GameCatalog::from_raw_json(CatalogKind::LittleAlchemy1, TEST_JSON),
            GameCatalog::from_raw_json(CatalogKind::LittleAlchemy2, TEST_JSON),
        ])
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
        let inner = board_inner(scene.board);
        let palette = app.active_palette();
        let cells = iso_board_cells(inner, palette.len(), app.active_state().palette_scroll);
        let cell = cells
            .iter()
            .find(|cell| palette[cell.index] == element_index)
            .expect("element should have a visible board cell");
        (
            cell.top.x + cell.top.width / 2,
            cell.top.y + cell.top.height / 2,
        )
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
        // Selection is cleared after a resolution.
        assert_eq!(app.active_state().selected, [None, None]);
    }

    #[test]
    fn combining_an_unknown_pair_says_nothing_happens() {
        let mut app = app();
        app.combine_two_elements(AIR, FIRE); // no recipe defined
        assert_eq!(app.active_banner_text(), Some("nothing happens"));
        assert_eq!(app.active_state().selected, [None, None]);
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
    fn tab_and_backtab_wrap_around_catalogs() {
        let mut app = app_two();
        assert_eq!(app.active_catalog, 0);
        app.handle_event(key(KeyCode::Tab));
        assert_eq!(app.active_catalog, 1);
        app.handle_event(key(KeyCode::Tab)); // wraps back to 0
        assert_eq!(app.active_catalog, 0);
        app.handle_event(key(KeyCode::BackTab)); // wraps to last
        assert_eq!(app.active_catalog, 1);
    }

    #[test]
    fn switch_to_catalog_ignores_out_of_range_index() {
        let mut app = app_two();
        app.switch_to_catalog(99);
        assert_eq!(app.active_catalog, 0);
        app.switch_to_catalog(1);
        assert_eq!(app.active_catalog, 1);
    }

    #[test]
    fn digit_key_selects_the_matching_visible_slot() {
        let mut app = sized(100, 40);
        app.handle_event(key(KeyCode::Char('1')));
        assert_eq!(app.active_state().selected, [Some(AIR), None]);
        assert_eq!(app.active_banner_text(), Some("selected Air"));
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
    fn inventory_scroll_clamps_within_bounds() {
        let mut app = sized(100, 12);
        app.reveal_elements_for_preview(&[
            "Steam", "Mud", "Lava", "Dust", "Rain", "Stone", "Sand", "Glass",
        ]);
        let palette_len = app.active_palette().len();
        let capacity = app.inventory_visible_capacity();
        let max_scroll = palette_len.saturating_sub(capacity);
        // Scroll far down, then assert it never exceeds the maximum.
        for _ in 0..20 {
            app.scroll_inventory(1);
        }
        assert!(app.active_state().palette_scroll <= max_scroll);
        // Scroll far up returns to the top.
        for _ in 0..20 {
            app.scroll_inventory(-1);
        }
        assert_eq!(app.active_state().palette_scroll, 0);
        // A zero delta is a no-op.
        app.scroll_inventory(0);
        assert_eq!(app.active_state().palette_scroll, 0);
    }

    #[test]
    fn canvas_scroll_clamps_within_bounds() {
        let mut app = sized(100, 12);
        app.reveal_elements_for_preview(&[
            "Steam", "Mud", "Lava", "Dust", "Rain", "Stone", "Sand", "Glass",
        ]);
        for _ in 0..50 {
            app.scroll_canvas(1);
        }
        let visible = app.visible_canvas_count();
        let total = app.active_state().discovery_order.len();
        assert!(app.active_state().canvas_scroll <= total.saturating_sub(visible));
        for _ in 0..50 {
            app.scroll_canvas(-1);
        }
        assert_eq!(app.active_state().canvas_scroll, 0);
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
    fn dropping_into_an_out_of_range_slot_is_a_noop() {
        let mut app = app();
        app.drop_element_into_slot(WATER, 2);
        assert_eq!(app.active_state().selected, [None, None]);
        assert!(app.active_banner_text().is_none());
    }

    #[test]
    fn reveal_for_preview_unlocks_and_resets_transient_state() {
        let mut app = sized(100, 40);
        app.select_element_by_index(WATER); // dirties banner + selection
        app.reveal_elements_for_preview(&["Steam", "Mud"]);
        assert!(discovered(&app, STEAM) && discovered(&app, 5));
        let state = app.active_state();
        assert_eq!(state.selected, [None, None]);
        assert!(state.banner.is_none());
        assert!(state.effects.is_empty());
        assert!(state.recipe_preview.is_none());
    }

    #[test]
    fn clicking_a_catalog_tile_switches_the_active_catalog() {
        let mut app = app_two();
        app.viewport = Rect::new(0, 0, 100, 40);
        let scene = scene_layout(app.viewport);
        let strip = rail_sections(scene.rail).catalog_strip;
        let rects = catalog_strip_rects(strip, app.catalogs.len());
        let (_, tile) = rects
            .iter()
            .find(|(index, _)| *index == 1)
            .expect("la2 tile");
        let (x, y) = (tile.x + tile.width / 2, tile.y + tile.height / 2);
        app.handle_event(mouse(MouseEventKind::Down(MouseButton::Left), x, y));
        assert_eq!(app.active_catalog, 1);
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
