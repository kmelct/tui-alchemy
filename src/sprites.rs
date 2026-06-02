use crate::data::{ElementEntry, slugify};
use crate::effects::ElementStyle;
use image::{RgbaImage, imageops::FilterType};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

const SPRITE_WIDTH: u32 = 12;
const SPRITE_HEIGHT: u32 = 12;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct SpriteCacheKey {
    path: PathBuf,
    width: u32,
    height: u32,
}

static SPRITE_CACHE: OnceLock<Mutex<HashMap<SpriteCacheKey, Vec<Line<'static>>>>> = OnceLock::new();

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpriteSource {
    Generated(PathBuf),
    NamedPlaceholder,
}

pub fn sprite_lines_for_element(element: &ElementEntry) -> Vec<Line<'static>> {
    sprite_lines_for_element_with_size(element, SPRITE_WIDTH, SPRITE_HEIGHT)
}

pub fn sprite_lines_for_path(path: &Path, fallback_name: &str) -> Vec<Line<'static>> {
    sprite_lines_for_path_with_size(path, fallback_name, SPRITE_WIDTH, SPRITE_HEIGHT)
}

pub fn sprite_lines_for_element_with_size(
    element: &ElementEntry,
    width: u32,
    height: u32,
) -> Vec<Line<'static>> {
    sprite_lines_for_element_frame(element, width, height, 0)
}

pub fn sprite_lines_for_element_frame(
    element: &ElementEntry,
    width: u32,
    height: u32,
    tick: u64,
) -> Vec<Line<'static>> {
    match sprite_source_for_element(element) {
        SpriteSource::Generated(path) => {
            let frame_path = animated_frame_path(&path, tick).unwrap_or(path);
            load_sprite(&frame_path, width, height)
                .unwrap_or_else(|| named_placeholder(&element.name, width, height, tick))
        }
        SpriteSource::NamedPlaceholder => named_placeholder(&element.name, width, height, tick),
    }
}

pub fn sprite_source_for_element(element: &ElementEntry) -> SpriteSource {
    if element.pixel_sprite_path.exists() {
        SpriteSource::Generated(element.pixel_sprite_path.clone())
    } else {
        SpriteSource::NamedPlaceholder
    }
}

pub fn sprite_lines_for_path_with_size(
    path: &Path,
    fallback_name: &str,
    width: u32,
    height: u32,
) -> Vec<Line<'static>> {
    let cache = SPRITE_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let key = SpriteCacheKey {
        path: path.to_path_buf(),
        width,
        height,
    };
    if let Some(cached) = cache.lock().ok().and_then(|guard| guard.get(&key).cloned()) {
        return cached;
    }

    let lines = load_sprite(path, width, height)
        .unwrap_or_else(|| named_placeholder(fallback_name, width, height, 0));

    if let Ok(mut guard) = cache.lock() {
        guard.insert(key, lines.clone());
    }

    lines
}

fn load_sprite(path: &Path, width: u32, height: u32) -> Option<Vec<Line<'static>>> {
    let image = load_raster_image(path)?;
    Some(raster_to_sprite(&image, width, height))
}

fn animated_frame_path(path: &Path, tick: u64) -> Option<PathBuf> {
    let stem = path.file_stem()?.to_str()?;
    let extension = path.extension()?.to_str()?;
    let frame = (tick % 4) as usize;
    if frame == 0 {
        return None;
    }

    let candidate = path.with_file_name(format!("{stem}_idle_{frame}.{extension}"));
    candidate.exists().then_some(candidate)
}

fn load_raster_image(path: &Path) -> Option<RgbaImage> {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    match extension.as_str() {
        "png" | "jpg" | "jpeg" | "webp" => image::open(path).ok().map(|image| image.to_rgba8()),
        _ => None,
    }
}

fn raster_to_sprite(image: &RgbaImage, width: u32, height: u32) -> Vec<Line<'static>> {
    let resized = image::imageops::resize(
        image,
        width.saturating_mul(2).max(2),
        height,
        FilterType::Nearest,
    );
    quadrant_block_lines(&resized)
}

fn named_placeholder(name: &str, width: u32, height: u32, tick: u64) -> Vec<Line<'static>> {
    let style = ElementStyle::for_name(name);
    let image = semantic_placeholder_image(style, fxhash(&slugify(name)), (tick / 6) % 4);
    raster_to_sprite(&image, width.max(2), height.max(2))
}

fn semantic_placeholder_image(style: ElementStyle, seed: u64, frame: u64) -> RgbaImage {
    let mut image = RgbaImage::new(16, 16);
    let [dark, mid, light] = style_palette(style, seed.wrapping_add(frame * 31));

    match style {
        ElementStyle::Water => {
            diamond(&mut image, 8, 3, 5, 10, dark);
            diamond(&mut image, 8, 4, 4, 8, mid);
            rect_px(&mut image, 6, 6, 8, 8, light);
            rect_px(&mut image, 9, 12, 12, 14, light);
        }
        ElementStyle::Fire => {
            triangle(&mut image, 3, 13, 8, 2, 13, 13, dark);
            triangle(&mut image, 5, 13, 8, 4, 12, 13, mid);
            triangle(&mut image, 7, 12, 9, 6, 11, 12, light);
        }
        ElementStyle::Air => {
            line_px(&mut image, 2, 5, 11, 5, light);
            line_px(&mut image, 5, 8, 14, 8, mid);
            line_px(&mut image, 1, 11, 9, 11, dark);
            put_px(&mut image, 13, 6, light);
        }
        ElementStyle::Steam => {
            line_px(&mut image, 5, 14, 5, 6, mid);
            line_px(&mut image, 8, 14, 8, 3, light);
            line_px(&mut image, 11, 13, 11, 7, mid);
            put_px(&mut image, 6, 5, light);
            put_px(&mut image, 9, 2, light);
        }
        ElementStyle::Earth => {
            ellipse_px(&mut image, 8, 11, 6, 3, dark);
            ellipse_px(&mut image, 7, 10, 5, 3, mid);
            rect_px(&mut image, 5, 8, 8, 10, light);
            line_px(&mut image, 8, 8, 8, 4, mid);
        }
        ElementStyle::Stone => {
            diamond(&mut image, 8, 4, 6, 8, dark);
            diamond(&mut image, 8, 5, 5, 6, mid);
            triangle(&mut image, 5, 6, 8, 4, 10, 6, light);
        }
        ElementStyle::Plant => {
            line_px(&mut image, 8, 14, 8, 5, dark);
            ellipse_px(&mut image, 5, 8, 4, 2, mid);
            ellipse_px(&mut image, 11, 7, 4, 2, light);
            ellipse_px(&mut image, 8, 4, 3, 2, light);
        }
        ElementStyle::Metal => {
            diamond(&mut image, 8, 5, 6, 6, dark);
            diamond(&mut image, 8, 6, 5, 4, mid);
            rect_px(&mut image, 5, 6, 11, 8, light);
            rect_px(&mut image, 10, 9, 13, 11, light);
        }
        ElementStyle::Light => {
            triangle(&mut image, 9, 1, 4, 9, 8, 9, light);
            triangle(&mut image, 8, 7, 12, 7, 6, 15, mid);
            put_px(&mut image, 3, 3, light);
            put_px(&mut image, 13, 12, dark);
        }
        ElementStyle::Organic => {
            ellipse_px(&mut image, 8, 9, 5, 4, dark);
            ellipse_px(&mut image, 7, 8, 4, 3, mid);
            rect_px(&mut image, 6, 5, 10, 7, light);
            put_px(&mut image, 11, 10, light);
        }
        ElementStyle::Container => {
            rect_px(&mut image, 6, 2, 10, 5, light);
            rect_px(&mut image, 5, 5, 11, 14, dark);
            rect_px(&mut image, 6, 6, 10, 12, mid);
            rect_px(&mut image, 7, 7, 9, 9, light);
        }
        ElementStyle::Neutral => {
            diamond(&mut image, 8, 4, 5, 8, dark);
            diamond(&mut image, 8, 5, 4, 6, mid);
            rect_px(&mut image, 6, 6, 10, 8, light);
        }
    }

    image
}

fn style_palette(style: ElementStyle, seed: u64) -> [image::Rgba<u8>; 3] {
    let jitter = (seed % 16) as u8;
    let colors: [[u8; 3]; 3] = match style {
        ElementStyle::Air => [[64, 108, 190], [102, 172, 235], [220, 242, 255]],
        ElementStyle::Earth => [[72, 52, 42], [135, 98, 55], [220, 188, 107]],
        ElementStyle::Fire => [[126, 47, 35], [232, 82, 30], [255, 219, 68]],
        ElementStyle::Water => [[25, 73, 170], [45, 143, 234], [147, 220, 255]],
        ElementStyle::Steam => [[116, 139, 177], [180, 198, 229], [245, 248, 255]],
        ElementStyle::Stone => [[70, 76, 96], [133, 142, 160], [226, 231, 239]],
        ElementStyle::Plant => [[18, 86, 35], [39, 170, 59], [143, 229, 65]],
        ElementStyle::Metal => [[82, 95, 112], [154, 169, 184], [238, 248, 255]],
        ElementStyle::Light => [[153, 89, 23], [255, 195, 50], [255, 255, 202]],
        ElementStyle::Organic => [[92, 57, 51], [172, 94, 74], [235, 160, 118]],
        ElementStyle::Container => [[45, 98, 126], [92, 184, 206], [230, 255, 255]],
        ElementStyle::Neutral => [[83, 62, 51], [158, 126, 70], [226, 207, 138]],
    };

    colors.map(|[r, g, b]| {
        image::Rgba([
            r.saturating_add(jitter / 3),
            g.saturating_add(jitter / 4),
            b.saturating_add(jitter / 5),
            255,
        ])
    })
}

fn put_px(image: &mut RgbaImage, x: i32, y: i32, color: image::Rgba<u8>) {
    if x >= 0 && y >= 0 && x < image.width() as i32 && y < image.height() as i32 {
        image.put_pixel(x as u32, y as u32, color);
    }
}

fn rect_px(image: &mut RgbaImage, x0: i32, y0: i32, x1: i32, y1: i32, color: image::Rgba<u8>) {
    for y in y0..y1 {
        for x in x0..x1 {
            put_px(image, x, y, color);
        }
    }
}

fn line_px(image: &mut RgbaImage, x0: i32, y0: i32, x1: i32, y1: i32, color: image::Rgba<u8>) {
    let steps = (x1 - x0).abs().max((y1 - y0).abs()).max(1);
    for step in 0..=steps {
        let x = x0 + (x1 - x0) * step / steps;
        let y = y0 + (y1 - y0) * step / steps;
        put_px(image, x, y, color);
        put_px(image, x, y + 1, color);
    }
}

fn ellipse_px(image: &mut RgbaImage, cx: i32, cy: i32, rx: i32, ry: i32, color: image::Rgba<u8>) {
    for y in cy - ry..=cy + ry {
        for x in cx - rx..=cx + rx {
            if (x - cx).pow(2) * ry.pow(2) + (y - cy).pow(2) * rx.pow(2) <= rx.pow(2) * ry.pow(2) {
                put_px(image, x, y, color);
            }
        }
    }
}

fn diamond(image: &mut RgbaImage, cx: i32, cy: i32, rx: i32, ry: i32, color: image::Rgba<u8>) {
    for y in cy - ry..=cy + ry {
        for x in cx - rx..=cx + rx {
            if (x - cx).abs() * ry + (y - cy).abs() * rx <= rx * ry {
                put_px(image, x, y, color);
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn triangle(
    image: &mut RgbaImage,
    ax: i32,
    ay: i32,
    bx: i32,
    by: i32,
    cx: i32,
    cy: i32,
    color: image::Rgba<u8>,
) {
    let min_x = ax.min(bx).min(cx);
    let max_x = ax.max(bx).max(cx);
    let min_y = ay.min(by).min(cy);
    let max_y = ay.max(by).max(cy);
    let area = edge(ax, ay, bx, by, cx, cy).abs().max(1);
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let w0 = edge(bx, by, cx, cy, x, y);
            let w1 = edge(cx, cy, ax, ay, x, y);
            let w2 = edge(ax, ay, bx, by, x, y);
            if w0.abs() + w1.abs() + w2.abs() <= area {
                put_px(image, x, y, color);
            }
        }
    }
}

const fn edge(ax: i32, ay: i32, bx: i32, by: i32, px: i32, py: i32) -> i32 {
    (px - ax) * (by - ay) - (py - ay) * (bx - ax)
}

fn quadrant_block_lines(image: &RgbaImage) -> Vec<Line<'static>> {
    let mut rows = Vec::new();
    let width = image.width();
    let height = image.height();
    let pairs = (height / 2).max(1);
    let columns = (width / 2).max(1);

    for y in 0..pairs {
        let top_y = y * 2;
        let bottom_y = (top_y + 1).min(height.saturating_sub(1));
        let mut spans = Vec::new();

        for cell_x in 0..columns {
            spans.push(quadrant_span(
                image,
                cell_x * 2,
                top_y,
                (cell_x * 2 + 1).min(width.saturating_sub(1)),
                bottom_y,
            ));
        }

        rows.push(Line::from(spans));
    }

    rows
}

fn quadrant_span(
    image: &RgbaImage,
    left_x: u32,
    top_y: u32,
    right_x: u32,
    bottom_y: u32,
) -> Span<'static> {
    let top_left = image.get_pixel(left_x, top_y);
    let top_right = image.get_pixel(right_x, top_y);
    let bottom_left = image.get_pixel(left_x, bottom_y);
    let bottom_right = image.get_pixel(right_x, bottom_y);

    let mask = ((top_left[3] > 16) as u8) << 3
        | ((top_right[3] > 16) as u8) << 2
        | ((bottom_left[3] > 16) as u8) << 1
        | ((bottom_right[3] > 16) as u8);

    match mask {
        0 => Span::raw(" "),
        15 => Span::styled(
            "▀",
            Style::default()
                .fg(average_color(&[top_left, top_right]))
                .bg(average_color(&[bottom_left, bottom_right])),
        ),
        _ => Span::styled(
            quadrant_symbol(mask),
            Style::default().fg(average_occupied_color(
                mask,
                top_left,
                top_right,
                bottom_left,
                bottom_right,
            )),
        ),
    }
}

const fn quadrant_symbol(mask: u8) -> &'static str {
    match mask {
        1 => "▗",
        2 => "▖",
        3 => "▄",
        4 => "▝",
        5 => "▐",
        6 => "▞",
        7 => "▟",
        8 => "▘",
        9 => "▚",
        10 => "▌",
        11 => "▙",
        12 => "▀",
        13 => "▜",
        14 => "▛",
        _ => " ",
    }
}

fn average_occupied_color(
    mask: u8,
    top_left: &image::Rgba<u8>,
    top_right: &image::Rgba<u8>,
    bottom_left: &image::Rgba<u8>,
    bottom_right: &image::Rgba<u8>,
) -> Color {
    let mut red = 0u32;
    let mut green = 0u32;
    let mut blue = 0u32;
    let mut count = 0u32;

    for (bit, pixel) in [
        (8, top_left),
        (4, top_right),
        (2, bottom_left),
        (1, bottom_right),
    ] {
        if mask & bit != 0 {
            red += u32::from(pixel[0]);
            green += u32::from(pixel[1]);
            blue += u32::from(pixel[2]);
            count += 1;
        }
    }

    if count == 0 {
        return Color::Reset;
    }

    Color::Rgb(
        (red / count) as u8,
        (green / count) as u8,
        (blue / count) as u8,
    )
}

fn average_color(pixels: &[&image::Rgba<u8>]) -> Color {
    let count = pixels.len().max(1) as u32;
    let (red, green, blue) = pixels.iter().fold((0u32, 0u32, 0u32), |acc, pixel| {
        (
            acc.0 + u32::from(pixel[0]),
            acc.1 + u32::from(pixel[1]),
            acc.2 + u32::from(pixel[2]),
        )
    });

    Color::Rgb(
        (red / count) as u8,
        (green / count) as u8,
        (blue / count) as u8,
    )
}

fn fxhash(value: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::GameCatalog;
    use image::{Rgba, RgbaImage};
    use std::path::Path;

    fn text_of(lines: &[Line<'static>]) -> String {
        lines
            .iter()
            .flat_map(|line| line.spans.iter())
            .map(|span| span.content.as_ref())
            .collect()
    }

    #[test]
    fn renderer_preserves_diagonal_subcell_shapes() {
        let mut image = RgbaImage::new(4, 4);
        let ink = Rgba([240, 240, 240, 255]);
        image.put_pixel(0, 0, ink);
        image.put_pixel(1, 1, ink);
        image.put_pixel(2, 2, ink);
        image.put_pixel(3, 3, ink);

        let lines = raster_to_sprite(&image, 2, 4);
        assert!(
            text_of(&lines).contains('▚'),
            "diagonal sprite details should survive as quadrant cells"
        );
    }

    #[test]
    fn quadrant_symbol_maps_masks_to_block_glyphs() {
        assert_eq!(quadrant_symbol(12), "▀"); // top half
        assert_eq!(quadrant_symbol(3), "▄"); // bottom half
        assert_eq!(quadrant_symbol(10), "▌"); // left half
        assert_eq!(quadrant_symbol(5), "▐"); // right half
        assert_eq!(quadrant_symbol(9), "▚");
        // Empty and (unused) full masks fall back to a blank cell.
        assert_eq!(quadrant_symbol(0), " ");
        assert_eq!(quadrant_symbol(15), " ");
        // Every painted mask 1..=14 yields a non-space glyph.
        for mask in 1u8..=14 {
            assert_ne!(quadrant_symbol(mask), " ", "mask {mask} should be a glyph");
        }
    }

    #[test]
    fn fully_transparent_image_renders_as_blank_cells() {
        let image = RgbaImage::new(4, 4); // all (0,0,0,0)
        let lines = raster_to_sprite(&image, 2, 2);
        assert!(
            text_of(&lines).trim().is_empty(),
            "transparent sprite should produce only blank cells"
        );
    }

    fn placeholder_catalog() -> GameCatalog {
        // "Zzz Unmapped" has no pixel-sprite file, forcing the placeholder path.
        const JSON: &str = r#"{
            "source": "unit-test",
            "total": 1,
            "elements": [{"name": "Zzz Unmapped", "base": true}]
        }"#;
        GameCatalog::from_raw_json(JSON)
    }

    #[test]
    fn missing_sprite_file_falls_back_to_named_placeholder() {
        let catalog = placeholder_catalog();
        let element = &catalog.elements[0];
        assert!(matches!(
            sprite_source_for_element(element),
            SpriteSource::NamedPlaceholder
        ));
        let lines = sprite_lines_for_element_frame(element, 8, 10, 0);
        assert!(!lines.is_empty(), "placeholder must still draw something");
        // An 8×10 sprite collapses to 5 quadrant rows (2px tall each).
        assert_eq!(lines.len(), 5);
    }

    #[test]
    fn placeholder_is_deterministic_for_a_fixed_tick() {
        let catalog = placeholder_catalog();
        let element = &catalog.elements[0];
        let a = sprite_lines_for_element_frame(element, 8, 10, 3);
        let b = sprite_lines_for_element_frame(element, 8, 10, 3);
        assert_eq!(text_of(&a), text_of(&b));
    }

    #[test]
    fn taller_sprites_produce_more_rows() {
        let catalog = placeholder_catalog();
        let element = &catalog.elements[0];
        let short = sprite_lines_for_element_frame(element, 8, 10, 0);
        let tall = sprite_lines_for_element_frame(element, 8, 20, 0);
        assert!(tall.len() > short.len());
    }

    #[test]
    fn missing_path_renders_the_named_fallback() {
        let lines = sprite_lines_for_path(Path::new("/no/such/sprite.png"), "Mystery");
        assert!(!lines.is_empty());
    }

    #[test]
    fn placeholder_draws_a_shape_for_every_element_style() {
        // One name per style family exercises each drawing branch (and the
        // diamond/triangle/ellipse/line primitives behind them).
        let names = [
            "Sea",          // Water
            "Lava",         // Fire
            "Storm",        // Air
            "Smoke",        // Steam
            "Sand",         // Earth
            "Mountain",     // Stone
            "Flower",       // Plant
            "Gold",         // Metal
            "Energy",       // Light
            "Bird",         // Organic
            "Vase",         // Container
            "Quintessence", // Neutral
        ];
        for name in names {
            for tick in [0u64, 7, 18] {
                let lines = named_placeholder(name, 8, 16, tick);
                assert!(
                    !text_of(&lines).trim().is_empty(),
                    "{name} placeholder (tick {tick}) should draw a visible shape"
                );
            }
        }
    }
}
