use ratatui::layout::{Constraint, Direction, Layout, Margin, Rect};

pub(crate) const HEADER_HEIGHT: u16 = 2;
pub(crate) const TOP_SAFE_MARGIN: u16 = 1;
pub(crate) const HEADER_WORKSHOP_GAP: u16 = 2;

// --- Scene composition (compact rail | iso board | grimoire) ---
pub(crate) const RAIL_WIDTH_PCT: u16 = 16;
pub(crate) const RAIL_MIN_WIDTH: u16 = 16;
pub(crate) const RAIL_MAX_WIDTH: u16 = 18;
pub(crate) const COLUMN_GAP: u16 = 2; // backdrop gutter between rail and board
pub(crate) const NARROW_BREAKPOINT: u16 = 70;
pub(crate) const BOARD_HERO_PCT: u16 = 55;
// Max content width — keeps columns cohesive on very wide terminals. Height is
// never capped (fully responsive vertically).
pub(crate) const STAGE_MAX_WIDTH: u16 = 156;

// --- Isometric shelf tile geometry ---
// Default tile geometry keeps normal and short terminals dense. Larger cells are
// selected dynamically for wide workspaces with only a few visible elements.
pub(crate) const ISO_TILE_WIDTH: u16 = 10;
pub(crate) const ISO_TILE_HEIGHT: u16 = 6; // 5 sprite rows + 1 label row
pub(crate) const ISO_DEPTH: u16 = 1; // front riser height
pub(crate) const ISO_SHADOW: u16 = 1; // cast-shadow row (doubles as vertical gap)
pub(crate) const ISO_SIDE: u16 = 0; // (unused) right shaded face
pub(crate) const ISO_GAP_X: u16 = 1; // gap between columns
pub(crate) const ISO_STAGGER: u16 = 1; // odd-row horizontal recede

#[derive(Debug, Clone, Copy)]
struct IsoGeometry {
    tile_width: u16,
    tile_height: u16,
    depth: u16,
    shadow: u16,
    side: u16,
    gap_x: u16,
    stagger: u16,
}

impl IsoGeometry {
    const DEFAULT: Self = Self {
        tile_width: ISO_TILE_WIDTH,
        tile_height: ISO_TILE_HEIGHT,
        depth: ISO_DEPTH,
        shadow: ISO_SHADOW,
        side: ISO_SIDE,
        gap_x: ISO_GAP_X,
        stagger: ISO_STAGGER,
    };

    const LARGE: Self = Self {
        tile_width: 14,
        tile_height: 8,
        depth: 1,
        shadow: 1,
        side: 0,
        gap_x: 1,
        stagger: 1,
    };

    const fn col_stride(self) -> u16 {
        self.tile_width
            .saturating_add(self.side)
            .saturating_add(self.gap_x)
    }

    const fn row_stride(self) -> u16 {
        self.tile_height
            .saturating_add(self.depth)
            .saturating_add(self.shadow)
    }
}
pub(crate) const fn contains(rect: Rect, column: u16, row: u16) -> bool {
    column >= rect.x
        && column < rect.x.saturating_add(rect.width)
        && row >= rect.y
        && row < rect.y.saturating_add(rect.height)
}

// ===========================================================================
// Scene layout: a compact stats rail, the isometric discovery board, and the
// open grimoire combining area. Header stays full-width on top.
// ===========================================================================

#[derive(Debug, Clone, Copy)]
pub(crate) struct SceneLayout {
    pub rail: Rect,
    pub board: Rect,
    pub grimoire: Rect,
}

/// The playfield. It keeps a small top safe area so the two-row logo/title band
/// never touches terminal chrome, then stays responsive with width capped and
/// centred only so the columns remain cohesive on very wide terminals.
pub(crate) fn stage_rect(area: Rect) -> Rect {
    let w = area.width.min(STAGE_MAX_WIDTH);
    let top_margin = top_safe_margin(area);
    Rect::new(
        area.x.saturating_add((area.width.saturating_sub(w)) / 2),
        area.y.saturating_add(top_margin),
        w,
        area.height.saturating_sub(top_margin),
    )
}

pub(crate) fn scene_layout(area: Rect) -> SceneLayout {
    let stage = stage_rect(area);
    let header_offset = HEADER_HEIGHT.saturating_add(header_workshop_gap(stage));
    let main = Rect::new(
        stage.x,
        stage.y.saturating_add(header_offset),
        stage.width,
        stage.height.saturating_sub(header_offset),
    );

    if main.width < NARROW_BREAKPOINT {
        // Vertical stack for narrow terminals: keep the recipe table close to
        // the atlas instead of letting a mostly empty board consume all spare
        // height. The board itself remains compact inside its allocated band.
        let rail_h = 10
            .min(main.height.saturating_sub(16))
            .max(8)
            .min(main.height);
        let after_rail = main.height.saturating_sub(rail_h);
        let grimoire_h = 12.min(after_rail.saturating_sub(6)).max(8).min(after_rail);
        let after_grimoire = after_rail.saturating_sub(grimoire_h);
        let board_h = if after_grimoire >= 6 {
            after_grimoire.min(12)
        } else {
            after_grimoire
        };
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(rail_h),
                Constraint::Length(board_h),
                Constraint::Length(grimoire_h),
                Constraint::Min(0),
            ])
            .split(main);
        return SceneLayout {
            rail: chunks[0],
            board: chunks[1],
            grimoire: chunks[2],
        };
    }

    let rail_w =
        (main.width.saturating_mul(RAIL_WIDTH_PCT) / 100).clamp(RAIL_MIN_WIDTH, RAIL_MAX_WIDTH);
    // rail | gutter | hero. The gutter keeps the rail's border out of the first
    // board tile's neighbourhood and gives the panels breathing room.
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(rail_w),
            Constraint::Length(COLUMN_GAP),
            Constraint::Min(0),
        ])
        .split(main);
    let hero = columns[2];
    // The grimoire needs ~34 cols for the recipe nameplate; give it a fixed
    // band and let the iso board take the rest of the hero.
    let grimoire_w = (hero.width.saturating_mul(100 - BOARD_HERO_PCT) / 100).clamp(34, 40);
    let hero_split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(grimoire_w)])
        .split(hero);

    SceneLayout {
        rail: columns[0],
        board: hero_split[0],
        grimoire: hero_split[1],
    }
}

const fn top_safe_margin(area: Rect) -> u16 {
    if area.width >= 130 && area.height >= 24 {
        TOP_SAFE_MARGIN
    } else {
        0
    }
}

const fn header_workshop_gap(stage: Rect) -> u16 {
    if stage.width >= 130 && stage.height >= 24 {
        HEADER_WORKSHOP_GAP
    } else {
        0
    }
}

const fn iso_geometry(area: Rect) -> IsoGeometry {
    if area.width >= 60 && area.height >= 12 {
        IsoGeometry::LARGE
    } else {
        IsoGeometry::DEFAULT
    }
}

fn iso_columns_for(area: Rect, geometry: IsoGeometry) -> usize {
    ((area
        .width
        .saturating_sub(geometry.stagger)
        .saturating_add(geometry.gap_x))
        / geometry.col_stride())
    .max(1) as usize
}

fn iso_rows_for(area: Rect, geometry: IsoGeometry) -> usize {
    (area.height / geometry.row_stride()).max(1) as usize
}

fn iso_capacity_for(area: Rect, geometry: IsoGeometry) -> usize {
    iso_columns_for(area, geometry)
        .saturating_mul(iso_rows_for(area, geometry))
        .max(1)
}

const fn centered_offset(available: u16, content: u16) -> u16 {
    available.saturating_sub(content) / 2
}

pub(crate) fn atlas_visible_count(area: Rect, total: usize, scroll: usize) -> usize {
    let panel = atlas_panel(area, total.saturating_sub(scroll));
    let available = board_inner(panel);
    let remaining = total.saturating_sub(scroll);
    let geometry = iso_geometry(available);
    remaining.min(iso_capacity_for(available, geometry)).max(1)
}
pub(crate) fn atlas_panel(area: Rect, count: usize) -> Rect {
    let available = board_inner(area);
    let geometry = iso_geometry(available);
    if area.width >= 60 && area.height >= 18 {
        let max_columns = iso_columns_for(available, geometry).max(1);
        let max_rows = iso_rows_for(available, geometry).max(1);
        let visible = count.max(1);
        let rows = visible.div_ceil(max_columns).max(2).min(max_rows);
        let panel_h = ((rows as u16).saturating_mul(geometry.row_stride()))
            .saturating_add(2)
            .clamp(12, area.height);
        Rect::new(area.x, area.y, area.width, panel_h)
    } else {
        if available.width == 0 || available.height == 0 {
            return area;
        }
        let max_columns = iso_columns_for(available, geometry).max(1);
        let max_rows = iso_rows_for(available, geometry).max(1);
        let visible = count.max(1);
        let min_rows = visible.div_ceil(max_columns).min(max_rows).max(1);
        let columns = (1..=max_columns)
            .filter(|cols| visible.div_ceil(*cols) == min_rows)
            .min_by_key(|cols| {
                (
                    min_rows.saturating_mul(*cols).saturating_sub(visible),
                    *cols,
                )
            })
            .unwrap_or(max_columns);
        let rows = visible.div_ceil(columns).min(max_rows);
        let content_w = geometry
            .stagger
            .saturating_add((columns as u16).saturating_mul(geometry.col_stride()))
            .saturating_sub(geometry.gap_x);
        let content_h = (rows as u16).saturating_mul(geometry.row_stride());
        let panel_w = content_w.saturating_add(2).clamp(1, area.width.max(1));
        let panel_h = content_h.saturating_add(2).clamp(1, area.height.max(1));
        let x = area
            .x
            .saturating_add((area.width.saturating_sub(panel_w)) / 2);
        let y_offset = if area.height >= 20 {
            0
        } else if area.height <= 18 {
            centered_offset(area.height, panel_h).min(1)
        } else {
            centered_offset(area.height, panel_h)
        };
        let y = area.y.saturating_add(y_offset);
        Rect::new(x, y, panel_w, panel_h)
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RailSections {
    pub panel: Rect,
    pub stats: Rect,
    pub progress: Rect,
    pub status: Rect,
    pub catalog_strip: Rect,
}

pub(crate) fn rail_sections(rail: Rect) -> RailSections {
    // Compact HUD panel kept cohesive even on tall screens; extra height turns
    // into chamber backdrop around it instead of dead panel body.
    let panel_h = rail.height.clamp(8, 13).min(rail.height.max(1));
    let panel_w = if rail.width > 28 { 24 } else { rail.width };
    let panel_y = rail.y;
    let panel = Rect::new(
        rail.x.saturating_add(centered_offset(rail.width, panel_w)),
        panel_y,
        panel_w,
        panel_h,
    );
    let inner = inset(panel, 1);
    let line = |offset: u16| Rect::new(inner.x, inner.y.saturating_add(offset), inner.width, 1);
    // Catalog switch shelf fills the lower portion of the compact panel.
    let catalog_strip = Rect::new(
        inner.x,
        inner.y.saturating_add(3),
        inner.width,
        inner.height.saturating_sub(3).max(2),
    );
    // "progress" title sits on the panel rim (row 0); keep "ready" within two
    // rows of it so it reads as a compact chip.
    RailSections {
        panel,
        stats: line(0),
        status: line(1),
        progress: line(2),
        catalog_strip,
    }
}

/// Divide a strip into `count` evenly spaced catalog tiles.
pub(crate) fn catalog_strip_rects(strip: Rect, count: usize) -> Vec<(usize, Rect)> {
    if count == 0 || strip.width == 0 || strip.height <= 1 {
        return Vec::new();
    }
    let gap = 1u16;
    let total_gap = gap.saturating_mul(count.saturating_sub(1) as u16);
    let tile_w = (strip.width.saturating_sub(total_gap) / count as u16).max(1);
    // Reserve the top two rows for the "catalog shelf" / "switch" labels.
    let tile_y = strip.y.saturating_add(2);
    let tile_h = strip.height.saturating_sub(2).max(1);
    (0..count)
        .map(|index| {
            let x = strip.x.saturating_add((index as u16) * (tile_w + gap));
            (index, Rect::new(x, tile_y, tile_w, tile_h))
        })
        .collect()
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct IsoCell {
    pub index: usize,
    pub top: Rect,
    pub face: Rect,
    pub side: Rect,
    pub shadow: Rect,
}

/// Inner drawing area of the board (inset for frame + room for the panel label).
pub(crate) const fn board_inner(board: Rect) -> Rect {
    if board.width <= 2 || board.height <= 2 {
        return board;
    }
    Rect::new(
        board.x.saturating_add(1),
        board.y.saturating_add(1),
        board.width.saturating_sub(2),
        board.height.saturating_sub(2),
    )
}

pub(crate) fn iso_columns(area: Rect) -> usize {
    iso_columns_for(area, iso_geometry(area))
}

pub(crate) fn iso_capacity(area: Rect) -> usize {
    iso_capacity_for(area, iso_geometry(area))
}
pub(crate) fn iso_board_cells(area: Rect, count: usize, scroll: usize) -> Vec<IsoCell> {
    if area.width == 0 || area.height == 0 {
        return Vec::new();
    }

    let geometry = iso_geometry(area);
    let max_columns = iso_columns_for(area, geometry);
    let capacity = iso_capacity_for(area, geometry);
    let start = scroll.min(count);
    let end = start.saturating_add(capacity).min(count);
    let mut cells = Vec::with_capacity(end.saturating_sub(start));
    let visible = end.saturating_sub(start).max(1);

    let columns = if area.width >= 60 && area.height >= 18 {
        visible.min(max_columns).max(1)
    } else {
        let max_rows = iso_rows_for(area, geometry).max(1);
        let min_rows = visible.div_ceil(max_columns).min(max_rows).max(1);
        (1..=max_columns)
            .filter(|cols| visible.div_ceil(*cols) == min_rows)
            .min_by_key(|cols| {
                (
                    min_rows.saturating_mul(*cols).saturating_sub(visible),
                    *cols,
                )
            })
            .unwrap_or(max_columns)
    };

    let content_w = geometry
        .stagger
        .saturating_add((columns as u16).saturating_mul(geometry.col_stride()))
        .saturating_sub(geometry.gap_x);
    let origin_x = area
        .x
        .saturating_add(centered_offset(area.width, content_w));
    let origin_y = area.y;

    for index in start..end {
        let local = index - start;
        let row = (local / columns) as u16;
        let col = (local % columns) as u16;
        let stagger = (row % 2) * geometry.stagger;
        let x = origin_x.saturating_add(stagger + col * geometry.col_stride());
        let y = origin_y.saturating_add(row * geometry.row_stride());
        if y.saturating_add(geometry.tile_height) > area.y.saturating_add(area.height) {
            break;
        }

        let top = Rect::new(x, y, geometry.tile_width, geometry.tile_height);
        let face = Rect::new(
            x,
            y.saturating_add(geometry.tile_height),
            geometry.tile_width,
            geometry.depth,
        );
        let side = Rect::new(
            x.saturating_add(geometry.tile_width),
            y.saturating_add(1),
            geometry.side,
            geometry
                .tile_height
                .saturating_add(geometry.depth)
                .saturating_sub(1),
        );
        let shadow = Rect::new(
            x.saturating_add(1),
            y.saturating_add(geometry.tile_height)
                .saturating_add(geometry.depth),
            geometry.tile_width,
            geometry.shadow,
        );

        cells.push(IsoCell {
            index,
            top,
            face,
            side,
            shadow,
        });
    }

    cells
}

/// Hit-test the lit tile faces (`top`) of the iso cells.
pub(crate) fn iso_hit(cells: &[IsoCell], column: u16, row: u16) -> Option<usize> {
    cells
        .iter()
        .find(|cell| contains(cell.top, column, row))
        .map(|cell| cell.index)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tall_atlas_panel_uses_available_height_before_scrolling() {
        let board = Rect::new(0, 0, 80, 42);
        let two_rows = atlas_panel(board, 10);
        let three_rows = atlas_panel(board, 11);

        assert!(
            three_rows.height > two_rows.height,
            "a tall atlas should grow to another row before requiring scroll"
        );
        assert_eq!(
            atlas_visible_count(board, 11, 0),
            11,
            "eleven discoveries should fit in the available tall atlas before scrolling"
        );

        let full_page = atlas_visible_count(board, 21, 0);
        let full_panel = atlas_panel(board, full_page);
        assert_eq!(
            iso_capacity(board_inner(full_panel)),
            full_page,
            "scroll capacity should match the rendered wide-screen atlas geometry"
        );
    }

    #[test]
    fn workbench_panel_uses_the_right_column_instead_of_a_top_ribbon() {
        let grimoire = grimoire_layout(Rect::new(0, 0, 36, 26));
        assert!(
            grimoire.panel.height >= 22,
            "the workbench should occupy most of the right column so drag-and-drop does not feel stranded in a tiny ribbon"
        );
        assert_eq!(grimoire.panel.y, 0);
    }
}
/// The recipe-table panel: a compact, bronze-rimmed bar holding three sockets
/// laid out horizontally — `ingredient + ingredient = result`. It is centred
/// vertically in its column rather than stretched, so it never becomes a tall
/// thin ribbon.
#[derive(Debug, Clone, Copy)]
pub(crate) struct GrimoireLayout {
    pub panel: Rect,
    pub nameplate: Rect,
    pub slot_left: Rect,
    pub slot_right: Rect,
    pub result: Rect,
    pub plus: Rect,
    pub equals: Rect,
}

pub(crate) fn grimoire_layout(area: Rect) -> GrimoireLayout {
    // The workbench is the primary first-session interaction surface, so it
    // should own the full right column instead of collapsing into a small top
    // ribbon with dead space underneath.
    let panel = area;
    let inner = inset(panel, 1);
    let nameplate = Rect::new(inner.x, inner.y, inner.width, 1);
    let body = Rect::new(
        inner.x,
        inner.y.saturating_add(1),
        inner.width,
        inner.height.saturating_sub(1).max(1),
    );

    let [slot_left, slot_right, result] = craft_slot_rects(body);
    let plus = Rect::new(
        slot_left.x.saturating_add(slot_left.width).saturating_add(
            (slot_right
                .x
                .saturating_sub(slot_left.x.saturating_add(slot_left.width)))
                / 2,
        ),
        body.y.saturating_add(body.height / 2),
        1,
        1,
    );
    let equals = Rect::new(
        slot_right
            .x
            .saturating_add(slot_right.width)
            .saturating_add(
                (result
                    .x
                    .saturating_sub(slot_right.x.saturating_add(slot_right.width)))
                    / 2,
            ),
        body.y.saturating_add(body.height / 2),
        1,
        1,
    );

    GrimoireLayout {
        panel,
        nameplate,
        slot_left,
        slot_right,
        result,
        plus,
        equals,
    }
}

/// Three horizontal sockets [ingredient] + [ingredient] = [result] with gaps
/// left for the operator glyphs.
fn craft_slot_rects(area: Rect) -> [Rect; 3] {
    let gap: u16 = if area.width >= 40 { 2 } else { 1 };
    let slot_w = (area.width.saturating_sub(gap.saturating_mul(2)) / 3).max(4);
    let left = Rect::new(area.x, area.y, slot_w, area.height);
    let middle = Rect::new(
        left.x.saturating_add(slot_w).saturating_add(gap),
        area.y,
        slot_w,
        area.height,
    );
    let result_x = middle.x.saturating_add(slot_w).saturating_add(gap);
    let result = Rect::new(
        result_x,
        area.y,
        area.x
            .saturating_add(area.width)
            .saturating_sub(result_x)
            .max(4),
        area.height,
    );
    [left, middle, result]
}

const fn inset(rect: Rect, by: u16) -> Rect {
    if rect.width <= by * 2 || rect.height <= by * 2 {
        return rect;
    }
    rect.inner(Margin {
        horizontal: by,
        vertical: by,
    })
}
