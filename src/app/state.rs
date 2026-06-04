use crate::data::{GameCatalog, base_discovery_state, discovered_count};
use crate::effects::ElementEffect;
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy)]
pub enum DragOrigin {
    Inventory,
    Slot,
}

#[derive(Debug, Clone, Copy)]
pub struct DragState {
    pub element_index: usize,
    pub origin: DragOrigin,
    pub column: u16,
    pub row: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct RecipePreview {
    pub left: usize,
    pub right: usize,
    pub result: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuView {
    Closed,
    Main,
    Controls,
    ResetConfirm,
}

impl MenuView {
    pub const fn is_open(self) -> bool {
        !matches!(self, Self::Closed)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuItem {
    Resume,
    Controls,
    ResetGame,
}

impl MenuItem {
    const ALL: [Self; 3] = [Self::Resume, Self::Controls, Self::ResetGame];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Resume => "resume",
            Self::Controls => "controls",
            Self::ResetGame => "reset game",
        }
    }

    pub fn move_by(self, delta: isize) -> Self {
        let current = Self::ALL
            .iter()
            .position(|item| *item == self)
            .unwrap_or_default() as isize;
        let max = Self::ALL.len().saturating_sub(1) as isize;
        let next = current.saturating_add(delta).clamp(0, max) as usize;
        Self::ALL[next]
    }
}

#[derive(Debug, Clone)]
pub struct Banner {
    pub text: String,
    pub ttl: u8,
    pub highlight: Option<usize>,
}

impl Banner {
    pub(super) fn new(text: impl Into<String>, ttl: u8, highlight: Option<usize>) -> Self {
        Self {
            text: text.into(),
            ttl,
            highlight,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CatalogState {
    pub discovered: Vec<bool>,
    pub discovery_order: Vec<usize>,
    pub selected: [Option<usize>; 2],
    pub palette_cursor: usize,
    pub palette_page: usize,
    pub drag: Option<DragState>,
    pub recent: VecDeque<usize>,
    pub banner: Option<Banner>,
    pub effects: Vec<ElementEffect>,
    pub recipe_preview: Option<RecipePreview>,
    pub slot_flash: [u8; 2],
}

impl CatalogState {
    pub(super) fn new(catalog: &GameCatalog) -> Self {
        let discovered = base_discovery_state(catalog);
        let discovery_order = catalog.base_indices.clone();

        Self {
            discovered,
            discovery_order,
            selected: [None, None],
            palette_cursor: 0,
            palette_page: 0,
            drag: None,
            recent: VecDeque::new(),
            banner: None,
            effects: Vec::new(),
            recipe_preview: None,
            slot_flash: [0, 0],
        }
    }

    pub(super) fn discovered_count(&self) -> usize {
        discovered_count(&self.discovered)
    }

    pub(super) const fn clear_selection(&mut self) {
        self.selected = [None, None];
    }
}
