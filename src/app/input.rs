use super::{App, HitTarget};
use crate::app::{DragOrigin, DragState};
use crate::layout::{
    atlas_page_count, atlas_page_size, atlas_panel, atlas_tab_rects, board_inner, contains,
    grimoire_layout, iso_board_cells, iso_columns, iso_hit, scene_layout,
};
use crossterm::event::{Event, KeyCode, KeyEvent, MouseButton, MouseEvent, MouseEventKind};

impl App {
    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::Key(key) => self.handle_key(key),
            Event::Mouse(mouse) => self.handle_mouse(mouse),
            Event::Paste(_) | Event::Resize(_, _) => {}
            _ => {}
        }
    }

    fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.move_palette_cursor(-(self.inventory_columns().max(1) as isize))
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.move_palette_cursor(self.inventory_columns().max(1) as isize)
            }
            KeyCode::Left | KeyCode::Char('h') => self.move_palette_cursor(-1),
            KeyCode::Right | KeyCode::Char('l') => self.move_palette_cursor(1),
            KeyCode::PageUp => self.move_palette_page(-1),
            KeyCode::PageDown => self.move_palette_page(1),
            KeyCode::Home => self.move_palette_cursor_to_start(),
            KeyCode::End => self.move_palette_cursor_to_end(),
            KeyCode::Esc => self.active_state_mut().clear_selection(),
            KeyCode::Enter => self.select_cursor_element(),
            KeyCode::Char(ch) if key.modifiers.is_empty() => {
                if let Some(digit) = ch.to_digit(10) {
                    if (1..=9).contains(&digit) {
                        self.select_visible_slot((digit - 1) as usize);
                    }
                } else if ch == '[' {
                    self.move_palette_page(-1);
                } else if ch == ']' {
                    self.move_palette_page(1);
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
                if !self.click_atlas_tab(mouse.column, mouse.row) {
                    self.begin_drag(mouse);
                }
            }
            MouseEventKind::Drag(MouseButton::Left) => self.update_drag(mouse),
            MouseEventKind::Up(MouseButton::Left) => self.finish_drag(mouse),
            _ => {}
        }
    }

    fn begin_drag(&mut self, mouse: MouseEvent) {
        let Some(hit) = self.hit_test(mouse.column, mouse.row) else {
            return;
        };

        let element_index = match hit {
            HitTarget::Inventory(index) => index,
            HitTarget::Slot(slot) => match self.active_state().selected[slot] {
                Some(index) => index,
                None => return,
            },
        };

        let origin = match hit {
            HitTarget::Inventory(_) => DragOrigin::Inventory,
            HitTarget::Slot(_) => DragOrigin::Slot,
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
            Some(HitTarget::Inventory(target_index)) if target_index != drag.element_index => {
                self.combine_two_elements(drag.element_index, target_index);
            }
            Some(HitTarget::Inventory(_)) | None => {
                self.select_element_by_index(drag.element_index)
            }
        }
    }

    fn click_atlas_tab(&mut self, column: u16, row: u16) -> bool {
        if self.viewport.width == 0 || self.viewport.height == 0 {
            return false;
        }

        let palette_len = self.active_palette().len();
        if palette_len == 0 {
            return false;
        }
        let scene = scene_layout(self.viewport);
        let page_count = atlas_page_count(scene.board, palette_len);
        let panel = atlas_panel(scene.board, atlas_page_size(scene.board, palette_len));
        for tab in atlas_tab_rects(panel, page_count, self.active_state().palette_page) {
            if contains(tab.rect, column, row) {
                self.set_palette_page(tab.page);
                return true;
            }
        }

        false
    }

    fn hit_test(&self, column: u16, row: u16) -> Option<HitTarget> {
        if self.viewport.width == 0 || self.viewport.height == 0 {
            return None;
        }

        let scene = scene_layout(self.viewport);
        let inventory = atlas_panel(
            scene.board,
            atlas_page_size(scene.board, self.active_palette().len()),
        );

        if contains(inventory, column, row) {
            return self.hit_inventory(column, row);
        }

        if contains(grimoire_layout(scene.grimoire).panel, column, row) {
            return self.hit_canvas(column, row);
        }

        None
    }

    fn hit_inventory(&self, column: u16, row: u16) -> Option<HitTarget> {
        let scene = scene_layout(self.viewport);
        let inner = board_inner(atlas_panel(
            scene.board,
            atlas_page_size(scene.board, self.active_palette().len()),
        ));

        if !contains(inner, column, row) {
            return None;
        }

        let palette = self.active_palette();
        let page_start = self.active_palette_page_start();
        let cells = iso_board_cells(inner, palette.len(), page_start);
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

    pub(super) fn inventory_columns(&self) -> usize {
        let scene = scene_layout(self.viewport);
        let panel = atlas_panel(
            scene.board,
            atlas_page_size(scene.board, self.active_palette().len()),
        );
        iso_columns(board_inner(panel))
    }

    pub(super) fn inventory_visible_capacity(&self) -> usize {
        self.active_palette_page_size()
    }
}
