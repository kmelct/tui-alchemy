use crate::app::App;
use crate::effects::{EffectKind, ElementStyle};
use crate::layout::{
    HEADER_HEIGHT, IsoCell, atlas_panel, board_inner, catalog_strip_rects, grimoire_layout,
    iso_board_cells, rail_sections, scene_layout, stage_rect,
};
use crate::palette::{palette_color, palette_color_for_seed};
use crate::sprites::{sprite_lines_for_element_frame, sprite_lines_for_path_with_size};
use crate::theme::{Ink, Surfaces};
use ratatui::Frame;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Paragraph, Wrap};

// Header HUD surfaces (kept identical to the legacy values the header expects).
const HUD_BG: Color = Surfaces::RAIL_BG;
const HUD_RIM: Color = Surfaces::RAIL_RIM;
const HUD_SHADOW: Color = Surfaces::RAIL_SHADOW;

// Sprite-effect colours pinned by the birth/aura tests (do not fold into theme).
const BIRTH_GLOW_BG: Color = Color::Rgb(56, 52, 45);
const BIRTH_HALO_BG: Color = Color::Rgb(70, 64, 50);
const DRAG_SHADOW_BG: Color = Color::Rgb(14, 21, 22);
/// Living "alive" bed under a freshly crafted result (legacy workbench body tone).
const BIRTH_BED_BG: Color = Color::Rgb(54, 39, 38);

pub fn render_app(frame: &mut Frame<'_>, app: &App) {
    let area = frame.area();
    if area.width == 0 || area.height == 0 {
        return;
    }

    render_backdrop(frame, area);
    let stage = stage_rect(area);
    render_header(frame, stage, app);

    let scene = scene_layout(area);
    render_stats_rail(frame, scene.rail, app);
    render_iso_board(frame, scene.board, app);
    render_grimoire(frame, scene.grimoire, app);

    if let Some(drag) = app.active_drag() {
        let drag_area = match drag.origin {
            crate::app::DragOrigin::Inventory => {
                atlas_panel(scene.board, app.active_palette().len())
            }
            crate::app::DragOrigin::Canvas => grimoire_layout(scene.grimoire).panel,
        };
        render_drag_overlay(
            frame,
            drag_area,
            app,
            drag.column,
            drag.row,
            drag.element_index,
            drag.origin,
        );
    }
}

fn render_backdrop(frame: &mut Frame<'_>, area: Rect) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    // A lit alchemist's chamber, painted purely with cell *backgrounds* (spaces,
    // never glyphs) so it reads as a real atmospheric surface and is never
    // mistaken for sprite/graphic content. A smooth vertical gradient gives the
    // void depth: a deep starry vault up top fades through a faint mid-air
    // horizon glow down to a calm stone floor in shadow. Brighter motes scatter
    // like stars across the upper vault and grow sparse and dim near the floor.
    let h = area.height.max(1) as f32;
    for y in area.y..area.y.saturating_add(area.height) {
        let t = y.saturating_sub(area.y) as f32 / (h - 1.0).max(1.0);
        let base = chamber_gradient(t);
        let in_vault = t < 0.5;
        let mut spans = Vec::with_capacity(area.width as usize);
        for x in area.x..area.x.saturating_add(area.width) {
            let hash = speck_hash(x, y);
            let bg = if in_vault {
                match hash % 43 {
                    0 => Surfaces::SPECK_LIT,
                    5 | 17 | 29 => Surfaces::SPECK_DIM,
                    _ => base,
                }
            } else {
                // Floor: only an occasional dim dust mote, so it stays calm.
                match hash % 97 {
                    0 => Surfaces::SPECK_DIM,
                    _ => base,
                }
            };
            spans.push(Span::styled(" ", Style::default().bg(bg)));
        }
        render_line(
            frame,
            Rect::new(area.x, y, area.width, 1),
            Line::from(spans),
        );
    }
}

/// Vertical chamber tone at normalized depth `t` (0 = ceiling vault, 1 = floor).
/// A handful of control stops lerped into a smooth gradient so the open space
/// reads as a deliberate, lit room rather than a flat fill.
fn chamber_gradient(t: f32) -> Color {
    // (stop, r, g, b)
    const STOPS: [(f32, u8, u8, u8); 5] = [
        (0.00, 8, 10, 18),  // ceiling vault — deepest blue-black
        (0.35, 12, 14, 22), // upper air (matches Surfaces::VOID)
        (0.55, 17, 18, 29), // faint horizon glow
        (0.78, 15, 15, 22), // near floor
        (1.00, 11, 11, 16), // floor in shadow
    ];
    let t = t.clamp(0.0, 1.0);
    let mut i = 0;
    while i + 1 < STOPS.len() && t > STOPS[i + 1].0 {
        i += 1;
    }
    let (t0, r0, g0, b0) = STOPS[i];
    let (t1, r1, g1, b1) = STOPS[(i + 1).min(STOPS.len() - 1)];
    let span = (t1 - t0).max(f32::EPSILON);
    let k = ((t - t0) / span).clamp(0.0, 1.0);
    let lerp = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * k).round() as u8;
    Color::Rgb(lerp(r0, r1), lerp(g0, g1), lerp(b0, b1))
}

fn speck_hash(x: u16, y: u16) -> u32 {
    let mut h = (x as u32).wrapping_mul(73_856_093) ^ (y as u32).wrapping_mul(19_349_663);
    h ^= h >> 13;
    h = h.wrapping_mul(0x5bd1_e995);
    h ^ (h >> 15)
}

fn fill_rect_bg(frame: &mut Frame<'_>, area: Rect, bg: Color) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let line = Line::from(Span::styled(
        " ".repeat(area.width as usize),
        Style::default().bg(bg),
    ));
    frame.render_widget(
        Paragraph::new(Text::from(
            (0..area.height).map(|_| line.clone()).collect::<Vec<_>>(),
        )),
        area,
    );
}

/// A bronze-rimmed dark panel: a solid bronze title bar with beveled corners,
/// thin side posts, and a clean base with a single centred stud. Shared by the
/// stats rail and the recipe table so every panel reads identically.
fn render_panel_frame(frame: &mut Frame<'_>, area: Rect, title: &str, title_color: Color) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    fill_rect_bg(frame, area, Surfaces::PANEL_BG);
    if area.width < 3 || area.height < 2 {
        return;
    }

    let corner = Style::default()
        .fg(Surfaces::PANEL_RIM)
        .bg(Surfaces::PANEL_BG);
    let bar = Style::default()
        .fg(title_color)
        .bg(Surfaces::PANEL_RIM)
        .add_modifier(Modifier::BOLD);
    let inner_w = area.width.saturating_sub(2);

    // Top: ▛ + solid bronze title bar (emblem + title, centred) + ▜
    let titled = center_line(Line::from(Span::styled(format!("✦ {title}"), bar)), inner_w);
    let mut top = vec![Span::styled("▛", corner)];
    top.extend(
        titled
            .spans
            .into_iter()
            .map(|span| Span::styled(span.content, bar)),
    );
    top.push(Span::styled("▜", corner));
    render_line(
        frame,
        Rect::new(area.x, area.y, area.width, 1),
        Line::from(top),
    );

    // Side posts.
    for y in area.y.saturating_add(1)..area.y.saturating_add(area.height.saturating_sub(1)) {
        render_line(
            frame,
            Rect::new(area.x, y, 1, 1),
            Line::from(Span::styled("▌", corner)),
        );
        render_line(
            frame,
            Rect::new(area.x.saturating_add(area.width.saturating_sub(1)), y, 1, 1),
            Line::from(Span::styled("▐", corner)),
        );
    }

    // Base: ▙ + ▄ rail with one centred ▓ stud + ▟
    let span = inner_w.max(1);
    let mut base = String::with_capacity(area.width as usize);
    base.push('▙');
    for i in 0..span {
        base.push(if i == span / 2 { '▓' } else { '▄' });
    }
    base.push('▟');
    render_line(
        frame,
        Rect::new(
            area.x,
            area.y.saturating_add(area.height.saturating_sub(1)),
            area.width,
            1,
        ),
        Line::from(Span::styled(base, corner)),
    );
}

fn render_header(frame: &mut Frame<'_>, area: Rect, app: &App) {
    let catalog = app.active_catalog();
    let discovered = app.active_discovered_count();
    let active_banner = app.active_banner_text();

    fill_rect_bg(
        frame,
        Rect::new(area.x, area.y, area.width, HEADER_HEIGHT.min(area.height)),
        HUD_BG,
    );

    let title_line = center_line(
        Line::from(vec![
            Span::styled("▛▀▜ ", Style::default().fg(palette_color(5)).bg(HUD_BG)),
            Span::styled(
                "little alchemy",
                Style::default()
                    .fg(palette_color(10))
                    .bg(HUD_BG)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled("◆ ", Style::default().fg(palette_color(9)).bg(HUD_BG)),
            Span::styled(
                format!("{discovered} / {}", app.active_total()),
                Style::default()
                    .fg(palette_color(9))
                    .bg(HUD_BG)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled("▣ ", Style::default().fg(palette_color(11)).bg(HUD_BG)),
            Span::styled(
                catalog.title(),
                Style::default()
                    .fg(palette_color(11))
                    .bg(HUD_BG)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled("▟▄▙", Style::default().fg(palette_color(5)).bg(HUD_BG)),
        ]),
        area.width,
    );

    render_line(frame, Rect::new(area.x, area.y, area.width, 1), title_line);

    let mut status_spans = vec![];

    if let Some(text) = active_banner {
        status_spans.push(Span::styled(
            text.to_string(),
            Style::default()
                .fg(palette_color(1))
                .add_modifier(Modifier::BOLD),
        ));
    }

    let status_line = if status_spans.is_empty() {
        center_line(
            Line::from(vec![
                Span::styled("▚ ", Style::default().fg(HUD_RIM).bg(HUD_SHADOW)),
                Span::styled(
                    format!("{}  crafting table workbench", catalog.title()),
                    Style::default().fg(palette_color(14)).bg(HUD_SHADOW),
                ),
                Span::styled(" ▞", Style::default().fg(HUD_RIM).bg(HUD_SHADOW)),
            ]),
            area.width,
        )
    } else {
        center_line(
            Line::from({
                let mut spans = vec![Span::styled(
                    "✦ ",
                    Style::default().fg(palette_color(1)).bg(HUD_SHADOW),
                )];
                spans.extend(status_spans.into_iter().map(|span| {
                    Span::styled(
                        span.content,
                        span.style.bg(HUD_SHADOW).add_modifier(Modifier::BOLD),
                    )
                }));
                spans
            }),
            area.width,
        )
    };

    if area.height > 1 {
        render_line(
            frame,
            Rect::new(area.x, area.y + 1, area.width, 1),
            status_line,
        );
    }
}

// ===========================================================================
// Compact stats rail (left)
// ===========================================================================

fn render_stats_rail(frame: &mut Frame<'_>, area: Rect, app: &App) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    if area.height <= 1 {
        return;
    }

    let catalog = app.active_catalog();
    let sections = rail_sections(area);
    render_panel_frame(
        frame,
        sections.panel,
        "progress",
        palette_color(Ink::CATALOG),
    );
    let bottom = sections.panel.y.saturating_add(sections.panel.height);

    if sections.stats.y < bottom {
        render_line(
            frame,
            sections.stats,
            center_line(
                Line::from(vec![
                    Span::styled("◆ ", Style::default().fg(palette_color(Ink::STAT))),
                    Span::styled(
                        format!("{}/{}", app.active_discovered_count(), catalog.total),
                        Style::default()
                            .fg(palette_color(Ink::STAT))
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                sections.stats.width,
            ),
        );
    }

    if sections.status.y < bottom {
        let status_text = app.active_banner_text().unwrap_or("ready");
        let status_color = if app.active_banner_text().is_some() {
            palette_color(Ink::SELECTED)
        } else {
            palette_color(Ink::HINT)
        };
        render_line(
            frame,
            sections.status,
            center_line(
                Line::from(vec![
                    Span::styled("✦ ", Style::default().fg(palette_color(Ink::FRAME))),
                    Span::styled(status_text.to_string(), Style::default().fg(status_color)),
                ]),
                sections.status.width,
            ),
        );
    }

    if sections.progress.y < bottom {
        render_line(
            frame,
            sections.progress,
            center_line(
                Line::from(Span::styled(
                    progress_bar(
                        app.active_discovered_count(),
                        catalog.total,
                        sections.progress.width.min(12) as usize,
                    ),
                    Style::default().fg(palette_color(Ink::TITLE)),
                )),
                sections.progress.width,
            ),
        );
    }

    render_catalog_strip(frame, sections.catalog_strip, app);
}

fn render_catalog_strip(frame: &mut Frame<'_>, strip: Rect, app: &App) {
    if strip.width == 0 || strip.height <= 2 {
        return;
    }

    let single_book = app.catalogs.len() == 1;
    render_band(
        frame,
        Rect::new(strip.x, strip.y, strip.width, 1),
        Line::from(Span::styled(
            if single_book {
                "recipe book"
            } else {
                "catalog shelf"
            },
            Style::default()
                .fg(palette_color(Ink::CATALOG))
                .add_modifier(Modifier::BOLD),
        )),
        Surfaces::RAIL_BG,
    );
    render_band(
        frame,
        Rect::new(strip.x, strip.y.saturating_add(1), strip.width, 1),
        Line::from(Span::styled(
            if single_book { "combined" } else { "switch" },
            Style::default().fg(palette_color(Ink::HINT)),
        )),
        Surfaces::RAIL_BG,
    );

    let controls = control_tiles(app);
    let rects = catalog_strip_rects(strip, controls.len());
    for &(index, rect) in &rects {
        let control = &controls[index];
        let accent = if control.is_active {
            palette_color(Ink::STAT)
        } else {
            palette_color(Ink::FRAME)
        };
        let icon_path = control.icon_path();
        let mut lines = sprite_lines_for_path_with_size(icon_path.as_ref(), control.label, 6, 6);
        lines.push(Line::from(Span::styled(
            fit_label(control.label, rect.width as usize),
            Style::default().fg(accent),
        )));
        render_shelf_tile(frame, rect, accent, lines, control.is_active);
    }
    if rects.len() > 1 {
        render_catalog_switch_arrow(frame, &rects);
    }
}

fn render_shelf_tile(
    frame: &mut Frame<'_>,
    area: Rect,
    accent: Color,
    lines: Vec<Line<'static>>,
    active: bool,
) {
    let bg = if active {
        Surfaces::PEDESTAL_TOP_ACTIVE
    } else {
        Surfaces::PEDESTAL_FACE
    };
    fill_rect_bg(frame, area, bg);
    let paragraph = Paragraph::new(Text::from(center_block(lines, area.height, area.width)))
        .style(Style::default().fg(accent).bg(bg))
        .alignment(ratatui::layout::Alignment::Left)
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, area);
}

// ===========================================================================
// Isometric discovery board (centre)
// ===========================================================================

fn render_iso_board(frame: &mut Frame<'_>, area: Rect, app: &App) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let catalog = app.active_catalog();
    let state = app.active_state();
    let palette = app.active_palette();
    let panel = atlas_panel(area, palette.len());
    render_panel_frame(frame, panel, "atlas", palette_color(Ink::FRAME));

    let inner = board_inner(panel);
    let cells = iso_board_cells(inner, palette.len(), state.palette_scroll);

    for cell in &cells {
        let element_index = palette[cell.index];
        let element = &catalog.elements[element_index];
        let has_birth_effect = state.effects.iter().any(|effect| {
            effect.element_index == element_index && effect.kind == EffectKind::Birth
        });
        let accent = if app.active_banner_highlight() == Some(element_index) {
            palette_color(9)
        } else if state.selected.contains(&Some(element_index)) {
            palette_color(1)
        } else if state.palette_cursor == cell.index {
            palette_color(9)
        } else {
            palette_color(6)
        };
        let is_active = has_birth_effect
            || state.selected.contains(&Some(element_index))
            || state.palette_cursor == cell.index;
        let top_bg = if is_active {
            Surfaces::PEDESTAL_TOP_ACTIVE
        } else {
            Surfaces::PEDESTAL_TOP
        };
        let label_style = Style::default().fg(accent).bg(top_bg).add_modifier(
            if state.palette_cursor == cell.index {
                Modifier::BOLD
            } else {
                Modifier::empty()
            },
        );
        let label_width = cell.top.width.max(1) as usize;
        let label_lines = fit_label_lines(&element.name.to_lowercase(), label_width, label_style);
        let sprite_tick = if app
            .active_drag()
            .is_some_and(|drag| drag.element_index == element_index)
        {
            0
        } else {
            app.tick_counter
        };
        let mut sprite_lines =
            sprite_lines_for_element_frame(catalog.kind, element, 8, 10, sprite_tick);
        if has_birth_effect {
            sprite_lines = living_sprite_glint(sprite_lines, app.tick_counter, palette_color(9));
            sprite_lines = lines_with_empty_halo(sprite_lines, birth_halo_bg(&element.name));
        }
        let max_sprite_lines = (cell.top.height as usize)
            .saturating_sub(label_lines.len())
            .max(1);
        sprite_lines = crop_lines_to_height(sprite_lines, max_sprite_lines);

        let mut lines = sprite_lines;
        lines.extend(label_lines);
        render_iso_pedestal(frame, cell);
        render_iso_tile_face(frame, cell.top, accent, top_bg, lines);
    }
}

fn render_iso_pedestal(frame: &mut Frame<'_>, cell: &IsoCell) {
    // Cast shadow on the floor below/right of the tile.
    fill_rect_bg(frame, cell.shadow, Surfaces::DROP_SHADOW);

    // Right-edge shadow column down the tile (the tile occludes the floor),
    // sitting in the inter-column gap to read as depth + separation.
    let edge = Rect::new(
        cell.top.x.saturating_add(cell.top.width),
        cell.top.y.saturating_add(1),
        1,
        cell.top.height,
    );
    fill_rect_bg(frame, edge, Surfaces::DROP_SHADOW);

    // Front riser: lit top lip (continuing the shelf surface) over a dark face.
    fill_rect_bg(frame, cell.face, Surfaces::PEDESTAL_FACE);
    if cell.face.width > 0 && cell.face.height > 0 {
        render_line(
            frame,
            cell.face,
            Line::from(Span::styled(
                "▀".repeat(cell.face.width as usize),
                Style::default()
                    .fg(Surfaces::PEDESTAL_TOP)
                    .bg(Surfaces::PEDESTAL_SIDE),
            )),
        );
    }
    fill_rect_bg(frame, cell.side, Surfaces::PEDESTAL_SIDE);
}

fn render_iso_tile_face(
    frame: &mut Frame<'_>,
    top: Rect,
    accent: Color,
    bg: Color,
    lines: Vec<Line<'static>>,
) {
    let paragraph = Paragraph::new(Text::from(center_block(lines, top.height, top.width)))
        .style(Style::default().fg(accent).bg(bg))
        .alignment(ratatui::layout::Alignment::Left)
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, top);
}

// ===========================================================================
// Open grimoire (the combining hero, right)
// ===========================================================================

fn render_grimoire(frame: &mut Frame<'_>, area: Rect, app: &App) {
    if area.width < 8 || area.height < 6 {
        return;
    }

    let g = grimoire_layout(area);
    render_panel_frame(frame, g.panel, "recipe table", palette_color(Ink::FRAME));

    // Recipe formula on the panel's top inner row.
    render_band(
        frame,
        g.nameplate,
        Line::from(vec![
            Span::styled("ingredient", Style::default().fg(palette_color(Ink::TITLE))),
            Span::styled(" + ", Style::default().fg(palette_color(Ink::STAT))),
            Span::styled("ingredient", Style::default().fg(palette_color(Ink::TITLE))),
            Span::styled(" = ", Style::default().fg(palette_color(Ink::STAT))),
            Span::styled("result", Style::default().fg(palette_color(Ink::STAT))),
        ]),
        Surfaces::PANEL_BG,
    );

    let state = app.active_state();
    let result_index = app
        .active_banner_highlight()
        .or_else(|| state.recent.front().copied())
        .or_else(|| state.recipe_preview.map(|recipe| recipe.result))
        .or_else(|| state.selected.iter().flatten().copied().next());
    let left_input = state
        .selected
        .first()
        .copied()
        .flatten()
        .or_else(|| state.recipe_preview.map(|recipe| recipe.left));
    let right_input = state
        .selected
        .get(1)
        .copied()
        .flatten()
        .or_else(|| state.recipe_preview.map(|recipe| recipe.right));

    // The recipe formula on the nameplate already carries the + and = glyphs,
    // so the sockets stay clean (g.plus / g.equals reserved for future use).
    let _ = (g.plus, g.equals);
    render_grimoire_slot(frame, g.slot_left, app, left_input);
    render_grimoire_slot(frame, g.slot_right, app, right_input);
    render_grimoire_result(frame, g.result, app, result_index);
}

fn render_grimoire_slot(frame: &mut Frame<'_>, rect: Rect, app: &App, element: Option<usize>) {
    let catalog = app.active_catalog();
    if let Some(element_index) = element {
        let element = &catalog.elements[element_index];
        let is_birth = app.active_banner_highlight() == Some(element_index);
        let accent = if is_birth {
            palette_color(Ink::STAT)
        } else {
            palette_color_for_seed(element_index as u64)
        };
        let mut sprite_lines =
            sprite_lines_for_element_frame(catalog.kind, element, 8, 8, app.tick_counter);
        sprite_lines = trim_empty_sprite_padding(sprite_lines);
        if is_birth {
            sprite_lines = living_sprite_glint(sprite_lines, app.tick_counter, palette_color(9));
            sprite_lines = lines_with_empty_halo(sprite_lines, birth_halo_bg(&element.name));
        }
        let bed = slot_bed_for_element(&element.name, element_index, false, is_birth);
        render_grimoire_plate(
            frame,
            rect,
            accent,
            bed,
            sprite_lines,
            &element.name.to_lowercase(),
            false,
            is_birth,
        );
    } else {
        render_grimoire_plate(
            frame,
            rect,
            palette_color(Ink::MUTED),
            Surfaces::SOCKET_BED,
            empty_socket_lines(false),
            "ingredient",
            false,
            false,
        );
    }
}

fn render_grimoire_result(frame: &mut Frame<'_>, rect: Rect, app: &App, element: Option<usize>) {
    let catalog = app.active_catalog();
    if let Some(element_index) = element {
        let element = &catalog.elements[element_index];
        let is_birth = app.active_banner_highlight() == Some(element_index);
        let accent = palette_color(Ink::STAT);
        let mut sprite_lines =
            sprite_lines_for_element_frame(catalog.kind, element, 12, 12, app.tick_counter);
        sprite_lines = trim_empty_sprite_padding(sprite_lines);
        if is_birth {
            sprite_lines = living_sprite_glint(sprite_lines, app.tick_counter, palette_color(9));
            sprite_lines = lines_with_empty_halo(sprite_lines, birth_halo_bg(&element.name));
        }
        let bed = slot_bed_for_element(&element.name, element_index, true, is_birth);
        render_grimoire_plate(
            frame,
            rect,
            accent,
            bed,
            sprite_lines,
            &element.name.to_lowercase(),
            true,
            is_birth,
        );
    } else {
        render_grimoire_plate(
            frame,
            rect,
            palette_color(Ink::STAT),
            Surfaces::SOCKET_BED,
            empty_socket_lines(true),
            "result",
            true,
            false,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn render_grimoire_plate(
    frame: &mut Frame<'_>,
    area: Rect,
    accent: Color,
    slot_color: Color,
    lines: Vec<Line<'static>>,
    label: &str,
    is_output: bool,
    is_birth: bool,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }
    if area.height <= 1 {
        render_line(
            frame,
            area,
            Line::from(Span::styled(
                fit_label(label, area.width as usize),
                Style::default().fg(accent),
            )),
        );
        return;
    }

    let slot_area = Rect::new(area.x, area.y, area.width, area.height.saturating_sub(1));
    let label_area = Rect::new(
        area.x,
        area.y.saturating_add(area.height.saturating_sub(1)),
        area.width,
        1,
    );
    let slot_bg = if is_birth { BIRTH_BED_BG } else { slot_color };
    let slot = Paragraph::new(Text::from(center_block(
        lines,
        slot_area.height,
        slot_area.width,
    )))
    .style(Style::default().fg(accent).bg(slot_bg))
    .alignment(ratatui::layout::Alignment::Left)
    .wrap(Wrap { trim: false });
    frame.render_widget(slot, slot_area);
    render_band(
        frame,
        label_area,
        Line::from(Span::styled(
            fit_label(label, area.width as usize),
            Style::default().fg(accent).add_modifier(if is_output {
                Modifier::BOLD
            } else {
                Modifier::empty()
            }),
        )),
        slot_bg,
    );
}

// ===========================================================================
// Shared helpers (sprite effects, layout text, etc.)
// ===========================================================================

fn birth_aura_bg(name: &str) -> Color {
    match ElementStyle::for_name(name) {
        ElementStyle::Water => Color::Rgb(39, 57, 89),
        ElementStyle::Steam => Color::Rgb(39, 51, 65),
        ElementStyle::Fire => Color::Rgb(84, 48, 40),
        ElementStyle::Earth => Color::Rgb(67, 51, 44),
        ElementStyle::Plant => Color::Rgb(42, 62, 47),
        ElementStyle::Light => Color::Rgb(82, 65, 42),
        ElementStyle::Metal | ElementStyle::Stone => Color::Rgb(54, 58, 72),
        ElementStyle::Container => Color::Rgb(42, 63, 73),
        ElementStyle::Organic => Color::Rgb(73, 49, 49),
        ElementStyle::Air | ElementStyle::Neutral => BIRTH_GLOW_BG,
    }
}

fn birth_halo_bg(name: &str) -> Color {
    match ElementStyle::for_name(name) {
        ElementStyle::Water => Color::Rgb(48, 75, 112),
        ElementStyle::Steam => Color::Rgb(50, 65, 82),
        ElementStyle::Fire => Color::Rgb(112, 58, 38),
        ElementStyle::Earth => Color::Rgb(86, 63, 48),
        ElementStyle::Plant => Color::Rgb(52, 83, 54),
        ElementStyle::Light => Color::Rgb(112, 91, 44),
        ElementStyle::Metal | ElementStyle::Stone => Color::Rgb(70, 75, 92),
        ElementStyle::Container => Color::Rgb(54, 82, 92),
        ElementStyle::Organic => Color::Rgb(93, 61, 57),
        ElementStyle::Air | ElementStyle::Neutral => BIRTH_HALO_BG,
    }
}

fn crop_lines_to_height(lines: Vec<Line<'static>>, max_height: usize) -> Vec<Line<'static>> {
    if lines.len() <= max_height {
        return lines;
    }

    let start = lines.len().saturating_sub(max_height) / 2;
    lines.into_iter().skip(start).take(max_height).collect()
}

fn trim_empty_sprite_padding(lines: Vec<Line<'static>>) -> Vec<Line<'static>> {
    let cells = lines
        .into_iter()
        .map(|line| {
            line.spans
                .into_iter()
                .flat_map(|span| {
                    let style = span.style;
                    span.content
                        .into_owned()
                        .chars()
                        .map(move |ch| (ch, style))
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<(char, Style)>>()
        })
        .collect::<Vec<_>>();

    let mut min_column: Option<usize> = None;
    let mut max_column: Option<usize> = None;
    for row in &cells {
        for (column, (ch, _)) in row.iter().enumerate() {
            if *ch != ' ' {
                min_column = Some(min_column.map_or(column, |value| value.min(column)));
                max_column = Some(max_column.map_or(column, |value| value.max(column)));
            }
        }
    }

    let Some(min_column) = min_column else {
        return cells
            .into_iter()
            .map(|row| {
                Line::from(
                    row.into_iter()
                        .map(|(ch, style)| Span::styled(ch.to_string(), style))
                        .collect::<Vec<_>>(),
                )
            })
            .collect();
    };
    let max_column = max_column.unwrap_or(min_column);
    let min_column = min_column.saturating_sub(1);
    let max_column = cells
        .iter()
        .map(|row| row.len().saturating_sub(1))
        .max()
        .map_or(max_column, |last_column| (max_column + 1).min(last_column));

    cells
        .into_iter()
        .map(|row| {
            Line::from(
                row.into_iter()
                    .enumerate()
                    .filter(|(column, _)| *column >= min_column && *column <= max_column)
                    .map(|(_, (ch, style))| Span::styled(ch.to_string(), style))
                    .collect::<Vec<_>>(),
            )
        })
        .collect()
}

fn empty_socket_lines(is_output: bool) -> Vec<Line<'static>> {
    let rim = Style::default().fg(palette_color(14));
    let shadow = Style::default().fg(Color::Rgb(83, 55, 45));
    let core = Style::default()
        .fg(if is_output {
            palette_color(9)
        } else {
            palette_color(11)
        })
        .add_modifier(Modifier::BOLD);

    vec![
        Line::from(Span::raw("       ")),
        Line::from(vec![
            Span::raw(" "),
            Span::styled("▛", shadow),
            Span::styled("▀▀▀", rim),
            Span::styled("▜", shadow),
            Span::raw(" "),
        ]),
        Line::from(vec![
            Span::raw(" "),
            Span::styled("▌ ", shadow),
            Span::styled("◆", core),
            Span::styled(" ▐", shadow),
            Span::raw(" "),
        ]),
        Line::from(vec![
            Span::raw(" "),
            Span::styled("▙", shadow),
            Span::styled("▄▄▄", rim),
            Span::styled("▟", shadow),
            Span::raw(" "),
        ]),
        Line::from(Span::raw("       ")),
    ]
}

fn slot_bed_for_element(name: &str, seed: usize, is_output: bool, is_birth: bool) -> Color {
    if is_birth {
        return birth_aura_bg(name);
    }

    let lower = name.to_ascii_lowercase();
    if lower.contains("water")
        || lower.contains("sea")
        || lower.contains("rain")
        || lower.contains("ice")
    {
        Color::Rgb(38, 65, 82)
    } else if lower.contains("fire")
        || lower.contains("lava")
        || lower.contains("sun")
        || lower.contains("heat")
    {
        Color::Rgb(86, 48, 38)
    } else if lower.contains("earth")
        || lower.contains("soil")
        || lower.contains("mud")
        || lower.contains("stone")
    {
        Color::Rgb(69, 55, 43)
    } else if lower.contains("air")
        || lower.contains("wind")
        || lower.contains("steam")
        || lower.contains("cloud")
    {
        Color::Rgb(54, 57, 78)
    } else if is_output {
        Color::Rgb(62, 55, 46)
    } else {
        match seed % 4 {
            0 => Color::Rgb(64, 55, 45),
            1 => Color::Rgb(45, 63, 55),
            2 => Color::Rgb(49, 58, 60),
            _ => Color::Rgb(68, 52, 48),
        }
    }
}

fn render_catalog_switch_arrow(frame: &mut Frame<'_>, rects: &[(usize, Rect)]) {
    if rects.len() < 2 {
        return;
    }

    let left = rects[0].1;
    let right = rects[1].1;
    let x = left
        .x
        .saturating_add(left.width)
        .saturating_add(right.x.saturating_sub(left.x.saturating_add(left.width)) / 2);
    let y = left.y.saturating_add(left.height / 2);
    render_line(
        frame,
        Rect::new(x, y, 1, 1),
        Line::from(Span::styled(
            "⇆",
            Style::default()
                .fg(palette_color(9))
                .add_modifier(Modifier::BOLD),
        )),
    );
}

fn control_tiles(app: &App) -> Vec<ControlTile> {
    if app.catalogs.len() == 1 {
        return vec![ControlTile::new("Book", "catalog-la1", true)];
    }

    let active = app.active_catalog;
    vec![
        ControlTile::new("LA1 book", "catalog-la1", active == 0),
        ControlTile::new("LA2 book", "catalog-la2", active == 1),
    ]
}

struct ControlTile {
    label: &'static str,
    icon_slug: &'static str,
    is_active: bool,
}

impl ControlTile {
    fn new(label: &'static str, icon_slug: &'static str, is_active: bool) -> Self {
        Self {
            label,
            icon_slug,
            is_active,
        }
    }

    fn icon_path(&self) -> std::path::PathBuf {
        std::path::PathBuf::from("assets/pixel-sprites/ui").join(format!("{}.png", self.icon_slug))
    }
}

fn render_drag_overlay(
    frame: &mut Frame<'_>,
    area: Rect,
    app: &App,
    column: u16,
    row: u16,
    element_index: usize,
    _origin: crate::app::DragOrigin,
) {
    let catalog = app.active_catalog();
    let element = &catalog.elements[element_index];
    let width = 10.min(area.width).max(1);
    let height = 7.min(area.height).max(1);
    let x = column
        .saturating_sub(width / 2)
        .min(area.x + area.width.saturating_sub(width));
    let y = row
        .saturating_sub(height.saturating_add(1))
        .max(area.y)
        .min(area.y + area.height.saturating_sub(height));
    let overlay = Rect::new(x, y, width, height);
    let mut lines = Vec::new();
    lines.extend(sprite_lines_for_element_frame(
        catalog.kind,
        element,
        8,
        8,
        0,
    ));
    let lines = lines
        .into_iter()
        .map(|line| line_with_pixel_bg(line, DRAG_SHADOW_BG))
        .collect::<Vec<_>>();

    let paragraph = Paragraph::new(Text::from(center_block(
        lines,
        overlay.height,
        overlay.width,
    )))
    .style(Style::default())
    .alignment(ratatui::layout::Alignment::Left)
    .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, overlay);
}

fn line_with_pixel_bg(line: Line<'static>, bg: Color) -> Line<'static> {
    Line::from(
        line.spans
            .into_iter()
            .flat_map(|span| {
                let style = span.style;
                span.content
                    .into_owned()
                    .chars()
                    .map(move |ch| {
                        let pixel_style = if ch == ' ' { style } else { style.bg(bg) };
                        Span::styled(ch.to_string(), pixel_style)
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>(),
    )
}

/// Set a uniform background on every cell of a single-row line, padding the
/// content to the rect width so no cell resets to the terminal default.
fn render_band(frame: &mut Frame<'_>, row: Rect, content: Line<'static>, bg: Color) {
    if row.width == 0 || row.height == 0 {
        return;
    }
    let centered = center_line(content, row.width);
    let spans = centered
        .spans
        .into_iter()
        .map(|span| {
            let style = span.style.bg(bg);
            Span::styled(span.content, style)
        })
        .collect::<Vec<_>>();
    render_line(
        frame,
        Rect::new(row.x, row.y, row.width, 1),
        Line::from(spans),
    );
}

fn lines_with_empty_halo(lines: Vec<Line<'static>>, bg: Color) -> Vec<Line<'static>> {
    let mut cells = lines
        .into_iter()
        .map(|line| {
            line.spans
                .into_iter()
                .flat_map(|span| {
                    let style = span.style;
                    span.content
                        .into_owned()
                        .chars()
                        .map(move |ch| (ch, style))
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<(char, Style)>>()
        })
        .collect::<Vec<_>>();

    let occupied = cells
        .iter()
        .map(|row| row.iter().map(|(ch, _)| *ch != ' ').collect::<Vec<_>>())
        .collect::<Vec<_>>();

    for row in 0..cells.len() {
        for column in 0..cells[row].len() {
            if occupied[row][column] {
                continue;
            }

            let neighbor_rows =
                row.saturating_sub(1)..=(row + 1).min(cells.len().saturating_sub(1));
            let mut neighbor_count = 0usize;
            let mut left = false;
            let mut right = false;
            let mut up = false;
            let mut down = false;
            for neighbor_row in neighbor_rows {
                let row_width = occupied[neighbor_row].len();
                if row_width == 0 {
                    continue;
                }
                let min_column = column.saturating_sub(1);
                let max_column = (column + 1).min(row_width.saturating_sub(1));
                // The index doubles as a coordinate in the adjacency maths below,
                // so a range loop is clearer here than an enumerated iterator.
                #[allow(clippy::needless_range_loop)]
                for neighbor_column in min_column..=max_column {
                    if !occupied[neighbor_row][neighbor_column] {
                        continue;
                    }
                    neighbor_count += 1;
                    left |= neighbor_row == row && neighbor_column + 1 == column;
                    right |= neighbor_row == row && neighbor_column == column + 1;
                    up |= neighbor_column == column && neighbor_row + 1 == row;
                    down |= neighbor_column == column && neighbor_row == row + 1;
                }
            }

            if (1..=3).contains(&neighbor_count) && !(left && right) && !(up && down) {
                cells[row][column].1 = cells[row][column].1.bg(bg);
            }
        }
    }

    cells
        .into_iter()
        .map(|row| {
            Line::from(
                row.into_iter()
                    .map(|(ch, style)| Span::styled(ch.to_string(), style))
                    .collect::<Vec<_>>(),
            )
        })
        .collect()
}

fn living_sprite_glint(lines: Vec<Line<'static>>, tick: u64, color: Color) -> Vec<Line<'static>> {
    let mut cells = Vec::new();
    for (row, line) in lines.iter().enumerate() {
        let mut column = 0usize;
        for span in &line.spans {
            for ch in span.content.chars() {
                if ch != ' ' {
                    cells.push((column, row));
                }
                column += 1;
            }
        }
    }

    if cells.is_empty() {
        return lines;
    }

    let min_x = cells.iter().map(|(x, _)| *x).min().unwrap_or(0);
    let max_x = cells.iter().map(|(x, _)| *x).max().unwrap_or(min_x);
    let min_y = cells.iter().map(|(_, y)| *y).min().unwrap_or(0);
    let max_y = cells.iter().map(|(_, y)| *y).max().unwrap_or(min_y);
    let width = max_x.saturating_sub(min_x).max(1);
    let height = max_y.saturating_sub(min_y).max(1);
    let core_y = min_y + height / 2;
    let anchors = [
        (min_x + width / 4, core_y),
        (min_x + width / 2, core_y),
        (min_x + (width * 3) / 4, core_y),
        (min_x + width / 2, core_y),
    ];
    let anchor = anchors[(tick as usize) % anchors.len()];
    let mut lit_cells = cells.clone();
    lit_cells.sort_by_key(|(x, y)| {
        let dx = x.abs_diff(anchor.0);
        let dy = y.abs_diff(anchor.1);
        (dy * 4) + dx
    });
    let lit_cells = lit_cells
        .into_iter()
        .take(3)
        .collect::<Vec<(usize, usize)>>();

    lines
        .into_iter()
        .enumerate()
        .map(|(row, line)| {
            let mut column = 0usize;
            let spans = line
                .spans
                .into_iter()
                .flat_map(|span| {
                    let style = span.style;
                    span.content
                        .into_owned()
                        .chars()
                        .map(|ch| {
                            let next_style = if ch != ' ' && lit_cells.contains(&(column, row)) {
                                style.fg(color).add_modifier(Modifier::BOLD)
                            } else {
                                style
                            };
                            column += 1;
                            Span::styled(ch.to_string(), next_style)
                        })
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();
            Line::from(spans)
        })
        .collect()
}

fn center_block(mut lines: Vec<Line<'static>>, height: u16, width: u16) -> Vec<Line<'static>> {
    let mut out = Vec::new();
    let content_height = lines.len() as u16;
    let top_pad = height.saturating_sub(content_height) / 2;
    for _ in 0..top_pad {
        out.push(Line::from(""));
    }

    for line in lines.drain(..) {
        out.push(center_line(line, width));
    }

    let bottom_pad = height.saturating_sub(out.len() as u16);
    for _ in 0..bottom_pad {
        out.push(Line::from(""));
    }

    out
}

fn center_line(line: Line<'static>, width: u16) -> Line<'static> {
    let content_width = line
        .spans
        .iter()
        .map(|span| span.content.chars().count())
        .sum::<usize>() as u16;
    if content_width >= width {
        return line;
    }

    let left_pad = (width - content_width) / 2;
    let right_pad = width - content_width - left_pad;
    let mut spans = Vec::with_capacity(width as usize);
    spans.extend(std::iter::repeat_n(Span::raw(" "), left_pad as usize));
    spans.extend(line.spans);
    spans.extend(std::iter::repeat_n(Span::raw(" "), right_pad as usize));
    Line::from(spans)
}

fn progress_bar(discovered: usize, total: usize, width: usize) -> String {
    if width == 0 {
        return String::new();
    }

    if total == 0 {
        return " ".repeat(width);
    }

    let ratio = (discovered as f32 / total as f32).clamp(0.0, 1.0);
    let filled = (ratio * width as f32).round() as usize;
    if filled < 4 {
        return " ".repeat(width);
    }
    let filled = filled.min(width);
    let empty = width.saturating_sub(filled);

    let mut out = String::with_capacity(width);
    out.push_str(&"█".repeat(filled));
    out.push_str(&" ".repeat(empty));
    out
}

fn fit_label(value: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }

    let count = value.chars().count();
    if count <= max_chars {
        return value.to_string();
    }

    if max_chars <= 3 {
        return value.chars().take(max_chars).collect();
    }

    let mut out: String = value.chars().take(max_chars - 3).collect();
    out.push_str("...");
    out
}

fn fit_label_lines(value: &str, max_chars: usize, style: Style) -> Vec<Line<'static>> {
    if max_chars == 0 {
        return vec![Line::from("")];
    }

    if value.chars().count() <= max_chars {
        return vec![Line::from(Span::styled(value.to_string(), style))];
    }

    let words: Vec<&str> = value.split_whitespace().collect();
    if words.len() < 2 {
        let (first, second) = split_long_label_word(value, max_chars);
        return vec![
            Line::from(Span::styled(first, style)),
            Line::from(Span::styled(second, style)),
        ];
    }
    if words[0].chars().count() >= max_chars {
        let (first, second_prefix) =
            split_long_label_word(words[0], max_chars.saturating_sub(1).max(1));
        let remainder = std::iter::once(second_prefix)
            .chain(words[1..].iter().map(|word| (*word).to_string()))
            .collect::<Vec<_>>()
            .join(" ");
        return vec![
            Line::from(Span::styled(first, style)),
            Line::from(Span::styled(hard_fit_label(&remainder, max_chars), style)),
        ];
    }

    let mut first_words = Vec::new();
    let mut first_len = 0usize;
    let mut split_at = 0usize;
    for (index, word) in words.iter().enumerate() {
        let word_len = word.chars().count();
        let candidate_len = if first_words.is_empty() {
            word_len
        } else {
            first_len + 1 + word_len
        };
        if candidate_len <= max_chars || first_words.is_empty() {
            first_words.push(*word);
            first_len = candidate_len;
            split_at = index + 1;
        } else {
            break;
        }
    }

    if split_at >= words.len() {
        return vec![Line::from(Span::styled(fit_label(value, max_chars), style))];
    }

    let first = fit_label(&first_words.join(" "), max_chars);
    let remainder = words[split_at..].join(" ");
    let second = if remainder.contains(' ') {
        fit_label(&remainder, max_chars)
    } else {
        hard_fit_label(&remainder, max_chars)
    };
    vec![
        Line::from(Span::styled(first, style)),
        Line::from(Span::styled(second, style)),
    ]
}

fn split_long_label_word(value: &str, max_chars: usize) -> (String, String) {
    let chars: Vec<char> = value.chars().collect();
    if chars.len() <= max_chars {
        return (value.to_string(), String::new());
    }

    let split_at = ((chars.len() + 2) / 2).min(max_chars).max(1);
    let first: String = chars.iter().take(split_at).collect();
    let second: String = chars.iter().skip(split_at).take(max_chars).collect();
    (first, second)
}

fn hard_fit_label(value: &str, max_chars: usize) -> String {
    value.chars().take(max_chars).collect()
}

fn render_line(frame: &mut Frame<'_>, area: Rect, line: Line<'static>) {
    let paragraph = Paragraph::new(Text::from(line));
    frame.render_widget(paragraph, area);
}

#[cfg(test)]
mod tests {
    use super::living_sprite_glint;
    use ratatui::style::{Color, Style};
    use ratatui::text::{Line, Span};

    #[test]
    fn living_sprite_glint_sweeps_sideways_not_top_to_bottom() {
        let sprite = vec![
            Line::from(Span::styled("XXXX", Style::default().fg(Color::White))),
            Line::from(Span::styled("XXXX", Style::default().fg(Color::White))),
            Line::from(Span::styled("XXXX", Style::default().fg(Color::White))),
            Line::from(Span::styled("XXXX", Style::default().fg(Color::White))),
        ];
        let mut highlighted_rows = Vec::new();

        for tick in 0..4 {
            let lines = living_sprite_glint(sprite.clone(), tick, Color::Red);
            for (row, line) in lines.iter().enumerate() {
                for span in &line.spans {
                    if span.style.fg == Some(Color::Red) {
                        highlighted_rows.push(row);
                    }
                }
            }
        }

        let min = highlighted_rows.iter().min().copied().unwrap_or_default();
        let max = highlighted_rows.iter().max().copied().unwrap_or_default();
        assert!(
            max.saturating_sub(min) <= 1,
            "living glint should shimmer across the sprite body, not blink from top to bottom: {highlighted_rows:?}"
        );
    }
}
