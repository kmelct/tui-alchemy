use ratatui::layout::{Constraint, Direction, Layout, Margin, Rect};

pub(crate) const HEADER_HEIGHT: u16 = 2;

// --- Scene composition (compact rail | iso board | grimoire) ---
pub(crate) const RAIL_WIDTH_PCT: u16 = 16;
pub(crate) const RAIL_MIN_WIDTH: u16 = 16;
pub(crate) const RAIL_MAX_WIDTH: u16 = 18;
pub(crate) const COLUMN_GAP: u16 = 2; // backdrop gutter between rail and board
pub(crate) const NARROW_BREAKPOINT: u16 = 70;
pub(crate) const BOARD_HERO_PCT: u16 = 55;
// Max content width — keeps columns cohesive on very wide terminals. Height is
// never capped (fully responsive vertically).
pub(crate) const STAGE_MAX_WIDTH: u16 = 128;

// --- Isometric shelf tile geometry ---
// Tile width 10 keeps ~10-char element labels on a single line; depth comes
// from a front riser + cast shadow rather than a side sliver, so columns stay
// dense enough for the discovery board.
pub(crate) const ISO_TILE_WIDTH: u16 = 10;
pub(crate) const ISO_TILE_HEIGHT: u16 = 6; // 5 sprite rows + 1 label row
pub(crate) const ISO_DEPTH: u16 = 1; // front riser height
pub(crate) const ISO_SHADOW: u16 = 1; // cast-shadow row (doubles as vertical gap)
pub(crate) const ISO_SIDE: u16 = 0; // (unused) right shaded face
pub(crate) const ISO_GAP_X: u16 = 1; // gap between columns
pub(crate) const ISO_STAGGER: u16 = 1; // odd-row horizontal recede

const ISO_COL_STRIDE: u16 = ISO_TILE_WIDTH + ISO_SIDE + ISO_GAP_X;
const ISO_ROW_STRIDE: u16 = ISO_TILE_HEIGHT + ISO_DEPTH + ISO_SHADOW;

pub(crate) fn contains(rect: Rect, column: u16, row: u16) -> bool {
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

/// The playfield. Height is always the full terminal height (top-anchored) so
/// the board grows and shrinks as the window resizes — the scene is fully
/// responsive vertically. Width is capped and centred only so the columns stay
/// cohesive on very wide terminals (the backdrop fills the side margins).
pub(crate) fn stage_rect(area: Rect) -> Rect {
    let w = area.width.min(STAGE_MAX_WIDTH);
    Rect::new(
        area.x.saturating_add((area.width.saturating_sub(w)) / 2),
        area.y,
        w,
        area.height,
    )
}

pub(crate) fn scene_layout(area: Rect) -> SceneLayout {
    let stage = stage_rect(area);
    let main = Rect::new(
        stage.x,
        stage.y.saturating_add(HEADER_HEIGHT),
        stage.width,
        stage.height.saturating_sub(HEADER_HEIGHT),
    );

    if main.width < NARROW_BREAKPOINT {
        // Vertical stack for narrow terminals: keep the recipe table close to
        // the atlas instead of letting a mostly empty board consume all spare
        // height. Extra space falls through to the chamber backdrop below.
        let rail_h = 4.min(main.height.saturating_sub(8)).max(3).min(main.height);
        let after_rail = main.height.saturating_sub(rail_h);
        let grimoire_h = 12.min(after_rail.saturating_sub(6)).max(8).min(after_rail);
        let after_grimoire = after_rail.saturating_sub(grimoire_h);
        let board_h = if after_grimoire >= 6 {
            after_grimoire.min(14)
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

#[derive(Debug, Clone, Copy)]
pub(crate) struct RailSections {
    pub panel: Rect,
    pub stats: Rect,
    pub progress: Rect,
    pub status: Rect,
    pub catalog_strip: Rect,
}

pub(crate) fn rail_sections(rail: Rect) -> RailSections {
    // Compact HUD panel docked at the top of its column; the board is the
    // dominant canvas, so the rail is sized to its content and the chamber
    // backdrop fills the column below it.
    let panel_h = rail.height.clamp(8, 13).min(rail.height.max(1));
    let panel = Rect::new(rail.x, rail.y, rail.width, panel_h);
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
pub(crate) fn board_inner(board: Rect) -> Rect {
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
    ((area
        .width
        .saturating_sub(ISO_STAGGER)
        .saturating_add(ISO_GAP_X))
        / ISO_COL_STRIDE)
        .max(1) as usize
}

pub(crate) fn iso_rows(area: Rect) -> usize {
    (area.height / ISO_ROW_STRIDE).max(1) as usize
}

pub(crate) fn iso_capacity(area: Rect) -> usize {
    iso_columns(area).saturating_mul(iso_rows(area)).max(1)
}

pub(crate) fn iso_board_cells(area: Rect, count: usize, scroll: usize) -> Vec<IsoCell> {
    if area.width == 0 || area.height == 0 {
        return Vec::new();
    }
    let columns = iso_columns(area);
    let capacity = iso_capacity(area);
    let start = scroll.min(count);
    let end = start.saturating_add(capacity).min(count);
    let mut cells = Vec::with_capacity(end.saturating_sub(start));

    for index in start..end {
        let local = index - start;
        let row = (local / columns) as u16;
        let col = (local % columns) as u16;
        let stagger = (row % 2) * ISO_STAGGER;
        let x = area.x.saturating_add(stagger + col * ISO_COL_STRIDE);
        let y = area.y.saturating_add(row * ISO_ROW_STRIDE);

        if y.saturating_add(ISO_TILE_HEIGHT) > area.y.saturating_add(area.height) {
            break;
        }

        let top = Rect::new(x, y, ISO_TILE_WIDTH, ISO_TILE_HEIGHT);
        let face = Rect::new(
            x,
            y.saturating_add(ISO_TILE_HEIGHT),
            ISO_TILE_WIDTH,
            ISO_DEPTH,
        );
        let side = Rect::new(
            x.saturating_add(ISO_TILE_WIDTH),
            y.saturating_add(1),
            ISO_SIDE,
            ISO_TILE_HEIGHT.saturating_add(ISO_DEPTH).saturating_sub(1),
        );
        let shadow = Rect::new(
            x.saturating_add(1),
            y.saturating_add(ISO_TILE_HEIGHT).saturating_add(ISO_DEPTH),
            ISO_TILE_WIDTH,
            ISO_SHADOW,
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
    // Compact recipe panel docked at the top-right; the board is the dominant
    // canvas and the chamber backdrop fills the column below it.
    let panel_h = area.height.clamp(9, 14).min(area.height.max(1));
    let panel = Rect::new(area.x, area.y, area.width, panel_h);
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

fn inset(rect: Rect, by: u16) -> Rect {
    if rect.width <= by * 2 || rect.height <= by * 2 {
        return rect;
    }
    rect.inner(Margin {
        horizontal: by,
        vertical: by,
    })
}
