//! Render a ratatui [`Buffer`] to an RGBA raster so the terminal UI can be
//! captured as a PNG and inspected visually (by a human or an agent).
//!
//! The game draws sprites entirely with Unicode *block glyphs* (`▀▄▌▐▙▟…`), so
//! we reproduce them faithfully without any font: each block glyph maps to the
//! sub-cell quadrants it fills, painted in the cell's foreground colour over its
//! background. ASCII labels use a small embedded 5×7 bitmap font; box-drawing
//! glyphs become thin line fills. The colour mapping mirrors the HTML preview's
//! `css_color` so the PNG and HTML views never disagree.

use image::{Rgba, RgbaImage};
use ratatui::buffer::Buffer;
use ratatui::style::Color;
use std::path::Path;

/// Pixels per terminal cell, width. 8 halves cleanly into 4px quadrants.
pub const CELL_W: u32 = 8;
/// Pixels per terminal cell, height. 16 halves into 8px quadrants and fits the 7px font.
pub const CELL_H: u32 = 16;

/// Whether a colour is being used for the glyph (foreground) or the cell fill
/// (background). Only matters for [`Color::Reset`], which must stay visible.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorRole {
    Foreground,
    Background,
}

/// Render a buffer at the recommended [`CELL_W`]×[`CELL_H`] cell size.
pub fn buffer_to_png_default(buffer: &Buffer) -> RgbaImage {
    buffer_to_png(buffer, CELL_W, CELL_H)
}

/// Render a ratatui buffer to an RGBA image, `cell_w`×`cell_h` pixels per cell.
pub fn buffer_to_png(buffer: &Buffer, cell_w: u32, cell_h: u32) -> RgbaImage {
    let area = buffer.area;
    let width = (area.width as u32) * cell_w;
    let height = (area.height as u32) * cell_h;
    let mut img = RgbaImage::new(width.max(1), height.max(1));

    for y in 0..area.height {
        for x in 0..area.width {
            let cell = &buffer[(area.x + x, area.y + y)];
            paint_cell(
                &mut img,
                (x as u32) * cell_w,
                (y as u32) * cell_h,
                cell_w,
                cell_h,
                cell,
            );
        }
    }

    img
}

/// Save a rendered image to `path`, creating parent directories as needed.
pub fn save_png(img: &RgbaImage, path: impl AsRef<Path>) -> std::io::Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    img.save(path).map_err(std::io::Error::other)
}

/// Map a ratatui [`Color`] to RGBA. Mirrors the HTML preview's `css_color`,
/// except [`Color::Reset`] resolves to a visible tone per [`ColorRole`].
pub const fn color_to_rgba(color: Color, role: ColorRole) -> Rgba<u8> {
    let (r, g, b) = match color {
        // Reset: looks like the terminal — dark scene bg, light text fg.
        Color::Reset => match role {
            ColorRole::Background => (7, 16, 18),
            ColorRole::Foreground => (221, 221, 221),
        },
        Color::Black => (0, 0, 0),
        Color::Red => (255, 85, 85),
        Color::Green => (85, 255, 85),
        Color::Yellow => (255, 255, 85),
        Color::Blue => (85, 85, 255),
        Color::Magenta => (255, 85, 255),
        Color::Cyan => (85, 255, 255),
        Color::Gray => (170, 170, 170),
        Color::DarkGray => (85, 85, 85),
        Color::LightRed => (255, 119, 119),
        Color::LightGreen => (119, 255, 119),
        Color::LightYellow => (255, 255, 119),
        Color::LightBlue => (119, 119, 255),
        Color::LightMagenta => (255, 119, 255),
        Color::LightCyan => (119, 255, 255),
        Color::White => (255, 255, 255),
        Color::Rgb(r, g, b) => (r, g, b),
        Color::Indexed(index) => (index, index, index),
    };
    Rgba([r, g, b, 255])
}

fn paint_cell(
    img: &mut RgbaImage,
    cx: u32,
    cy: u32,
    cw: u32,
    ch: u32,
    cell: &ratatui::buffer::Cell,
) {
    let bg = color_to_rgba(cell.bg, ColorRole::Background);
    fill_rect(img, cx, cy, cw, ch, bg);

    let symbol = cell.symbol();
    if symbol.is_empty() || symbol == " " {
        return;
    }
    let fg = color_to_rgba(cell.fg, ColorRole::Foreground);

    if let Some(mask) = quadrant_mask(symbol) {
        paint_quadrants(img, cx, cy, cw, ch, mask, fg);
        return;
    }
    if let Some(shade) = shade_factor(symbol) {
        let blended = blend(fg, bg, shade);
        fill_rect(img, cx, cy, cw, ch, blended);
        return;
    }
    if paint_box_drawing(img, cx, cy, cw, ch, symbol, fg) {
        return;
    }

    let ch0 = symbol.chars().next().unwrap_or(' ');
    if let Some(glyph) = font_glyph(ch0) {
        paint_font_glyph(img, cx, cy, cw, ch, &glyph, fg);
        return;
    }
    // Unknown decorative glyph (e.g. ◆ ✦ ⇆ ▣): centred diamond so it stays visible.
    paint_diamond(img, cx, cy, cw, ch, fg);
}

// ---------------------------------------------------------------------------
// Block glyphs → quadrant masks (inverse of sprites::quadrant_symbol).
// Bit layout: TL=8, TR=4, BL=2, BR=1.
// ---------------------------------------------------------------------------

fn quadrant_mask(symbol: &str) -> Option<u8> {
    Some(match symbol {
        "█" => 0b1111,
        "▀" => 0b1100, // top half
        "▄" => 0b0011, // bottom half
        "▌" => 0b1010, // left half
        "▐" => 0b0101, // right half
        "▖" => 0b0010,
        "▗" => 0b0001,
        "▘" => 0b1000,
        "▝" => 0b0100,
        "▚" => 0b1001,
        "▞" => 0b0110,
        "▙" => 0b1011,
        "▟" => 0b0111,
        "▛" => 0b1110,
        "▜" => 0b1101,
        _ => return None,
    })
}

fn paint_quadrants(
    img: &mut RgbaImage,
    cx: u32,
    cy: u32,
    cw: u32,
    ch: u32,
    mask: u8,
    color: Rgba<u8>,
) {
    let lw = cw / 2;
    let rw = cw - lw;
    let th = ch / 2;
    let bh = ch - th;
    if mask & 0b1000 != 0 {
        fill_rect(img, cx, cy, lw, th, color);
    }
    if mask & 0b0100 != 0 {
        fill_rect(img, cx + lw, cy, rw, th, color);
    }
    if mask & 0b0010 != 0 {
        fill_rect(img, cx, cy + th, lw, bh, color);
    }
    if mask & 0b0001 != 0 {
        fill_rect(img, cx + lw, cy + th, rw, bh, color);
    }
}

fn shade_factor(symbol: &str) -> Option<f32> {
    Some(match symbol {
        "░" => 0.25,
        "▒" => 0.5,
        "▓" => 0.75,
        _ => return None,
    })
}

// ---------------------------------------------------------------------------
// Box-drawing glyphs → thin line fills.
// ---------------------------------------------------------------------------

fn paint_box_drawing(
    img: &mut RgbaImage,
    cx: u32,
    cy: u32,
    cw: u32,
    ch: u32,
    symbol: &str,
    color: Rgba<u8>,
) -> bool {
    let hv = (ch / 8).max(1); // horizontal bar thickness (px)
    let vv = (cw / 8).max(1); // vertical bar thickness (px)
    let mid_x = cx + cw / 2 - vv / 2;
    let mid_y = cy + ch / 2 - hv / 2;
    let h_left = |img: &mut RgbaImage| fill_rect(img, cx, mid_y, cw / 2 + vv, hv, color);
    let h_right = |img: &mut RgbaImage| fill_rect(img, mid_x, mid_y, cx + cw - mid_x, hv, color);
    let h_full = |img: &mut RgbaImage| fill_rect(img, cx, mid_y, cw, hv, color);
    let v_top = |img: &mut RgbaImage| fill_rect(img, mid_x, cy, vv, ch / 2 + hv, color);
    let v_bottom = |img: &mut RgbaImage| fill_rect(img, mid_x, mid_y, vv, cy + ch - mid_y, color);
    let v_full = |img: &mut RgbaImage| fill_rect(img, mid_x, cy, vv, ch, color);

    match symbol {
        "─" | "━" | "╌" | "┈" => h_full(img),
        "│" | "┃" | "╎" | "┊" => v_full(img),
        "┌" | "╭" => {
            h_right(img);
            v_bottom(img);
        }
        "┐" | "╮" => {
            h_left(img);
            v_bottom(img);
        }
        "└" | "╰" => {
            h_right(img);
            v_top(img);
        }
        "┘" | "╯" => {
            h_left(img);
            v_top(img);
        }
        "├" => {
            v_full(img);
            h_right(img);
        }
        "┤" => {
            v_full(img);
            h_left(img);
        }
        "┬" => {
            h_full(img);
            v_bottom(img);
        }
        "┴" => {
            h_full(img);
            v_top(img);
        }
        "┼" => {
            h_full(img);
            v_full(img);
        }
        _ => return false,
    }
    true
}

// ---------------------------------------------------------------------------
// Primitives.
// ---------------------------------------------------------------------------

fn fill_rect(img: &mut RgbaImage, x: u32, y: u32, w: u32, h: u32, color: Rgba<u8>) {
    let max_x = (x + w).min(img.width());
    let max_y = (y + h).min(img.height());
    for py in y..max_y {
        for px in x..max_x {
            img.put_pixel(px, py, color);
        }
    }
}

fn blend(fg: Rgba<u8>, bg: Rgba<u8>, factor: f32) -> Rgba<u8> {
    let mix = |a: u8, b: u8| {
        (a as f32)
            .mul_add(factor, (b as f32) * (1.0 - factor))
            .round() as u8
    };
    Rgba([mix(fg[0], bg[0]), mix(fg[1], bg[1]), mix(fg[2], bg[2]), 255])
}

fn paint_diamond(img: &mut RgbaImage, cx: u32, cy: u32, cw: u32, ch: u32, color: Rgba<u8>) {
    let center_x = (cx + cw / 2) as i32;
    let center_y = (cy + ch / 2) as i32;
    let rx = (cw / 3).max(1) as i32;
    let ry = (ch / 3).max(1) as i32;
    for py in (center_y - ry)..=(center_y + ry) {
        for px in (center_x - rx)..=(center_x + rx) {
            if px < 0 || py < 0 {
                continue;
            }
            let nx = (px - center_x).abs();
            let ny = (py - center_y).abs();
            if nx * ry + ny * rx <= rx * ry
                && (px as u32) < img.width()
                && (py as u32) < img.height()
            {
                img.put_pixel(px as u32, py as u32, color);
            }
        }
    }
}

fn paint_font_glyph(
    img: &mut RgbaImage,
    cx: u32,
    cy: u32,
    cw: u32,
    ch: u32,
    glyph: &[u8; 7],
    color: Rgba<u8>,
) {
    // 5×7 glyph centred in the cell (1px L/R padding at width 8, ~4px top/bottom at height 16).
    let off_x = cx + (cw.saturating_sub(5)) / 2;
    let off_y = cy + (ch.saturating_sub(7)) / 2;
    for (row, bits) in glyph.iter().enumerate() {
        for col in 0..5u32 {
            if bits & (1 << (4 - col)) != 0 {
                let px = off_x + col;
                let py = off_y + row as u32;
                if px < img.width() && py < img.height() {
                    img.put_pixel(px, py, color);
                }
            }
        }
    }
}

/// A minimal 5×7 bitmap font. Lowercase reuses uppercase forms (legible for QA).
/// Each row's low 5 bits are columns, MSB = leftmost.
const fn font_glyph(ch: char) -> Option<[u8; 7]> {
    let upper = ch.to_ascii_uppercase();
    Some(match upper {
        ' ' => [0; 7],
        'A' => [
            0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'B' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110,
        ],
        'C' => [
            0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110,
        ],
        'D' => [
            0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110,
        ],
        'E' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111,
        ],
        'F' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'G' => [
            0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01111,
        ],
        'H' => [
            0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'I' => [
            0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ],
        'J' => [
            0b00111, 0b00010, 0b00010, 0b00010, 0b00010, 0b10010, 0b01100,
        ],
        'K' => [
            0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001,
        ],
        'L' => [
            0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111,
        ],
        'M' => [
            0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001,
        ],
        'N' => [
            0b10001, 0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001,
        ],
        'O' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'P' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'Q' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101,
        ],
        'R' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001,
        ],
        'S' => [
            0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
        'T' => [
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        'U' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'V' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100,
        ],
        'W' => [
            0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b11011, 0b10001,
        ],
        'X' => [
            0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001,
        ],
        'Y' => [
            0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        'Z' => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111,
        ],
        '0' => [
            0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110,
        ],
        '1' => [
            0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ],
        '2' => [
            0b01110, 0b10001, 0b00001, 0b00110, 0b01000, 0b10000, 0b11111,
        ],
        '3' => [
            0b11111, 0b00010, 0b00100, 0b00010, 0b00001, 0b10001, 0b01110,
        ],
        '4' => [
            0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010,
        ],
        '5' => [
            0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110,
        ],
        '6' => [
            0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110,
        ],
        '7' => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000,
        ],
        '8' => [
            0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110,
        ],
        '9' => [
            0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100,
        ],
        '.' => [
            0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b01100, 0b01100,
        ],
        ',' => [
            0b00000, 0b00000, 0b00000, 0b00000, 0b01100, 0b00100, 0b01000,
        ],
        '-' => [
            0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000,
        ],
        '+' => [
            0b00000, 0b00100, 0b00100, 0b11111, 0b00100, 0b00100, 0b00000,
        ],
        ':' => [
            0b00000, 0b01100, 0b01100, 0b00000, 0b01100, 0b01100, 0b00000,
        ],
        ';' => [
            0b00000, 0b01100, 0b01100, 0b00000, 0b01100, 0b00100, 0b01000,
        ],
        '/' => [
            0b00001, 0b00010, 0b00100, 0b00100, 0b01000, 0b10000, 0b10000,
        ],
        '\\' => [
            0b10000, 0b01000, 0b00100, 0b00100, 0b00010, 0b00001, 0b00001,
        ],
        '\'' => [
            0b00100, 0b00100, 0b01000, 0b00000, 0b00000, 0b00000, 0b00000,
        ],
        '"' => [
            0b01010, 0b01010, 0b01010, 0b00000, 0b00000, 0b00000, 0b00000,
        ],
        '!' => [
            0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00000, 0b00100,
        ],
        '?' => [
            0b01110, 0b10001, 0b00001, 0b00110, 0b00100, 0b00000, 0b00100,
        ],
        '(' => [
            0b00010, 0b00100, 0b01000, 0b01000, 0b01000, 0b00100, 0b00010,
        ],
        ')' => [
            0b01000, 0b00100, 0b00010, 0b00010, 0b00010, 0b00100, 0b01000,
        ],
        '[' => [
            0b01110, 0b01000, 0b01000, 0b01000, 0b01000, 0b01000, 0b01110,
        ],
        ']' => [
            0b01110, 0b00010, 0b00010, 0b00010, 0b00010, 0b00010, 0b01110,
        ],
        '#' => [
            0b01010, 0b01010, 0b11111, 0b01010, 0b11111, 0b01010, 0b01010,
        ],
        '%' => [
            0b11001, 0b11010, 0b00100, 0b01000, 0b10000, 0b01011, 0b10011,
        ],
        '&' => [
            0b01100, 0b10010, 0b10100, 0b01000, 0b10101, 0b10010, 0b01101,
        ],
        '<' => [
            0b00010, 0b00100, 0b01000, 0b10000, 0b01000, 0b00100, 0b00010,
        ],
        '>' => [
            0b01000, 0b00100, 0b00010, 0b00001, 0b00010, 0b00100, 0b01000,
        ],
        '=' => [
            0b00000, 0b00000, 0b11111, 0b00000, 0b11111, 0b00000, 0b00000,
        ],
        '*' => [
            0b00000, 0b00100, 0b10101, 0b01110, 0b10101, 0b00100, 0b00000,
        ],
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;
    use ratatui::style::Style;

    fn cell_color(img: &RgbaImage, col: u32, row: u32, sub_x: u32, sub_y: u32) -> Rgba<u8> {
        *img.get_pixel(col * CELL_W + sub_x, row * CELL_H + sub_y)
    }

    #[test]
    fn block_glyph_paints_only_its_quadrant() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 1, 1));
        // "▘" fills only the top-left quadrant.
        buffer[(0, 0)].set_symbol("▘");
        buffer[(0, 0)].set_style(
            Style::default()
                .fg(Color::Rgb(200, 50, 50))
                .bg(Color::Rgb(0, 0, 0)),
        );
        let img = buffer_to_png_default(&buffer);

        // Top-left pixel is foreground; bottom-right pixel is background.
        assert_eq!(cell_color(&img, 0, 0, 1, 1), Rgba([200, 50, 50, 255]));
        assert_eq!(
            cell_color(&img, 0, 0, CELL_W - 1, CELL_H - 1),
            Rgba([0, 0, 0, 255])
        );
    }

    #[test]
    fn full_block_fills_whole_cell() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 1, 1));
        buffer[(0, 0)].set_symbol("█");
        buffer[(0, 0)].set_style(
            Style::default()
                .fg(Color::Rgb(10, 220, 30))
                .bg(Color::Rgb(0, 0, 0)),
        );
        let img = buffer_to_png_default(&buffer);
        assert_eq!(cell_color(&img, 0, 0, 0, 0), Rgba([10, 220, 30, 255]));
        assert_eq!(
            cell_color(&img, 0, 0, CELL_W - 1, CELL_H - 1),
            Rgba([10, 220, 30, 255])
        );
    }

    #[test]
    fn image_dimensions_match_buffer() {
        let buffer = Buffer::empty(Rect::new(0, 0, 10, 4));
        let img = buffer_to_png_default(&buffer);
        assert_eq!(img.width(), 10 * CELL_W);
        assert_eq!(img.height(), 4 * CELL_H);
    }

    #[test]
    fn reset_background_is_opaque_dark() {
        let buffer = Buffer::empty(Rect::new(0, 0, 1, 1));
        let img = buffer_to_png_default(&buffer);
        // Default cells have Reset bg → opaque dark scene colour, not transparent.
        assert_eq!(*img.get_pixel(0, 0), Rgba([7, 16, 18, 255]));
    }

    #[test]
    fn letter_paints_some_foreground_pixels() {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 1, 1));
        buffer[(0, 0)].set_symbol("A");
        buffer[(0, 0)].set_style(
            Style::default()
                .fg(Color::Rgb(255, 255, 255))
                .bg(Color::Rgb(0, 0, 0)),
        );
        let img = buffer_to_png_default(&buffer);
        let white = (0..CELL_W * CELL_H).filter(|i| {
            let px = i % CELL_W;
            let py = i / CELL_W;
            *img.get_pixel(px, py) == Rgba([255, 255, 255, 255])
        });
        assert!(
            white.count() > 4,
            "letter 'A' should paint several foreground pixels"
        );
    }

    fn count_fg(img: &RgbaImage, fg: Rgba<u8>) -> usize {
        img.pixels().filter(|px| **px == fg).count()
    }

    fn one_cell(symbol: &str, fg: Color, bg: Color) -> RgbaImage {
        let mut buffer = Buffer::empty(Rect::new(0, 0, 1, 1));
        buffer[(0, 0)].set_symbol(symbol);
        buffer[(0, 0)].set_style(Style::default().fg(fg).bg(bg));
        buffer_to_png_default(&buffer)
    }

    #[test]
    fn reset_foreground_resolves_to_light_ink() {
        // The two roles of Color::Reset must resolve differently so text stays
        // readable against the scene background.
        assert_eq!(
            color_to_rgba(Color::Reset, ColorRole::Foreground),
            Rgba([221, 221, 221, 255])
        );
        assert_eq!(
            color_to_rgba(Color::Reset, ColorRole::Background),
            Rgba([7, 16, 18, 255])
        );
    }

    #[test]
    fn color_to_rgba_maps_named_indexed_and_rgb() {
        assert_eq!(
            color_to_rgba(Color::Red, ColorRole::Foreground),
            Rgba([255, 85, 85, 255])
        );
        assert_eq!(
            color_to_rgba(Color::White, ColorRole::Foreground),
            Rgba([255, 255, 255, 255])
        );
        assert_eq!(
            color_to_rgba(Color::Black, ColorRole::Background),
            Rgba([0, 0, 0, 255])
        );
        assert_eq!(
            color_to_rgba(Color::Indexed(42), ColorRole::Foreground),
            Rgba([42, 42, 42, 255])
        );
        assert_eq!(
            color_to_rgba(Color::Rgb(1, 2, 3), ColorRole::Foreground),
            Rgba([1, 2, 3, 255])
        );
        // Every base colour maps to an opaque pixel.
        for color in [
            Color::Green,
            Color::Yellow,
            Color::Blue,
            Color::Magenta,
            Color::Cyan,
            Color::Gray,
            Color::DarkGray,
            Color::LightRed,
            Color::LightGreen,
            Color::LightYellow,
            Color::LightBlue,
            Color::LightMagenta,
            Color::LightCyan,
        ] {
            assert_eq!(color_to_rgba(color, ColorRole::Foreground)[3], 255);
        }
    }

    #[test]
    fn every_block_glyph_has_a_quadrant_mask() {
        for glyph in [
            "█", "▀", "▄", "▌", "▐", "▖", "▗", "▘", "▝", "▚", "▞", "▙", "▟", "▛", "▜",
        ] {
            assert!(
                quadrant_mask(glyph).is_some(),
                "{glyph} should map to a mask"
            );
        }
        assert!(quadrant_mask("A").is_none());
        // ▛ is everything but the bottom-right quadrant.
        assert_eq!(quadrant_mask("▛"), Some(0b1110));
    }

    #[test]
    fn shaded_blocks_blend_toward_the_foreground() {
        // ▓ (0.75) should sit closer to the fg than ▒ (0.5) than ░ (0.25).
        let fg = Color::Rgb(255, 255, 255);
        let bg = Color::Rgb(0, 0, 0);
        let light = one_cell("░", fg, bg).get_pixel(0, 0)[0];
        let medium = one_cell("▒", fg, bg).get_pixel(0, 0)[0];
        let heavy = one_cell("▓", fg, bg).get_pixel(0, 0)[0];
        assert!(
            light < medium && medium < heavy,
            "{light} < {medium} < {heavy}"
        );
        assert_eq!(shade_factor("░"), Some(0.25));
        assert_eq!(shade_factor("X"), None);
    }

    #[test]
    fn box_drawing_paints_a_thin_line_not_a_full_cell() {
        let fg = Rgba([240, 240, 240, 255]);
        let img = one_cell("─", Color::Rgb(240, 240, 240), Color::Rgb(0, 0, 0));
        let painted = count_fg(&img, fg);
        // A horizontal rule covers far less than the whole 8×16 cell.
        assert!(
            painted > 0 && painted < (CELL_W * CELL_H) as usize / 2,
            "got {painted}"
        );
    }

    #[test]
    fn decorative_glyph_falls_back_to_a_centred_diamond() {
        let fg = Rgba([200, 160, 60, 255]);
        // ◆ has no font entry, no quadrant mask → diamond fallback paints pixels.
        let img = one_cell("◆", Color::Rgb(200, 160, 60), Color::Rgb(0, 0, 0));
        assert!(
            count_fg(&img, fg) > 0,
            "diamond fallback should paint something"
        );
    }

    #[test]
    fn every_box_drawing_glyph_paints_within_its_cell() {
        let fg = Rgba([240, 240, 240, 255]);
        for symbol in [
            "─", "━", "│", "┃", "┌", "╭", "┐", "╮", "└", "╰", "┘", "╯", "├", "┤", "┬", "┴", "┼",
        ] {
            let img = one_cell(symbol, Color::Rgb(240, 240, 240), Color::Rgb(0, 0, 0));
            let painted = count_fg(&img, fg);
            assert!(painted > 0, "{symbol} should paint at least one line pixel");
            assert!(
                painted < (CELL_W * CELL_H) as usize,
                "{symbol} should be a line, not a full fill"
            );
        }
        // A non-box glyph reports "not handled" so the caller can fall through.
        let mut buffer = Buffer::empty(Rect::new(0, 0, 1, 1));
        buffer[(0, 0)].set_symbol("Z");
        let img = buffer_to_png_default(&buffer);
        // 'Z' routed to the font path, not the box path — still paints something.
        assert!(count_fg(&img, color_to_rgba(Color::Reset, ColorRole::Foreground)) > 0);
    }

    #[test]
    fn font_glyph_covers_alnum_and_punctuation_but_not_unknowns() {
        for ch in "ABCXYZ0159.+-/=".chars() {
            assert!(font_glyph(ch).is_some(), "{ch} should have a glyph");
        }
        assert!(font_glyph('a').is_some(), "lowercase folds to uppercase");
        assert!(font_glyph('◆').is_none());
        assert_eq!(font_glyph(' '), Some([0; 7]));
    }
}
