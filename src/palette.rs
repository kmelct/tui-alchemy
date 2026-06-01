use image::Rgba;
use ratatui::style::Color;

pub const LOST_CENTURY: [Color; 16] = [
    Color::Rgb(209, 177, 135),
    Color::Rgb(199, 123, 88),
    Color::Rgb(174, 93, 64),
    Color::Rgb(121, 68, 74),
    Color::Rgb(75, 61, 68),
    Color::Rgb(186, 145, 88),
    Color::Rgb(146, 116, 65),
    Color::Rgb(77, 69, 57),
    Color::Rgb(119, 116, 59),
    Color::Rgb(179, 165, 85),
    Color::Rgb(210, 201, 165),
    Color::Rgb(140, 171, 161),
    Color::Rgb(75, 114, 110),
    Color::Rgb(87, 72, 82),
    Color::Rgb(132, 120, 117),
    Color::Rgb(171, 155, 142),
];

pub fn palette_color(index: usize) -> Color {
    LOST_CENTURY[index % LOST_CENTURY.len()]
}

pub fn palette_color_for_seed(seed: u64) -> Color {
    let mut value = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^= value >> 31;
    palette_color(value as usize)
}

pub fn nearest_palette_color(pixel: &Rgba<u8>) -> Color {
    let rgb = [pixel[0], pixel[1], pixel[2]];
    let mut best_index = 0usize;
    let mut best_distance = u32::MAX;

    for (index, color) in LOST_CENTURY.iter().enumerate() {
        let (r, g, b) = color_to_rgb(*color);
        let dr = rgb[0] as i32 - r as i32;
        let dg = rgb[1] as i32 - g as i32;
        let db = rgb[2] as i32 - b as i32;
        let distance = (dr * dr + dg * dg + db * db) as u32;
        if distance < best_distance {
            best_distance = distance;
            best_index = index;
        }
    }

    LOST_CENTURY[best_index]
}

fn color_to_rgb(color: Color) -> (u8, u8, u8) {
    match color {
        Color::Rgb(r, g, b) => (r, g, b),
        Color::Indexed(index) => {
            let value = index;
            (value, value, value)
        }
        Color::Reset => (0, 0, 0),
        _ => (0, 0, 0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn palette_color_wraps_modulo_sixteen() {
        assert_eq!(palette_color(0), LOST_CENTURY[0]);
        assert_eq!(palette_color(15), LOST_CENTURY[15]);
        assert_eq!(palette_color(16), LOST_CENTURY[0]);
        assert_eq!(palette_color(17), LOST_CENTURY[1]);
        assert_eq!(palette_color(usize::MAX), LOST_CENTURY[usize::MAX % 16]);
    }

    #[test]
    fn palette_color_for_seed_is_deterministic_and_in_range() {
        for seed in 0..512u64 {
            let a = palette_color_for_seed(seed);
            let b = palette_color_for_seed(seed);
            assert_eq!(a, b, "the same seed must always map to the same colour");
            assert!(
                LOST_CENTURY.contains(&a),
                "seed colour must be a palette member"
            );
        }
    }

    #[test]
    fn palette_color_for_seed_spreads_across_the_palette() {
        let mut seen = HashSet::new();
        for seed in 0..256u64 {
            seen.insert(palette_color_for_seed(seed));
        }
        assert!(
            seen.len() >= 12,
            "a good hash should touch most of the 16 buckets, got {}",
            seen.len()
        );
    }

    #[test]
    fn nearest_palette_color_returns_the_exact_member_for_palette_input() {
        for color in LOST_CENTURY {
            let (r, g, b) = color_to_rgb(color);
            assert_eq!(nearest_palette_color(&Rgba([r, g, b, 255])), color);
        }
    }

    #[test]
    fn nearest_palette_color_picks_the_closest_swatch_and_ignores_alpha() {
        // Pure black is nearest to the darkest olive swatch (77, 69, 57).
        assert_eq!(
            nearest_palette_color(&Rgba([0, 0, 0, 255])),
            Color::Rgb(77, 69, 57)
        );
        // The alpha channel must not influence the match.
        let opaque = nearest_palette_color(&Rgba([209, 177, 135, 255]));
        let transparent = nearest_palette_color(&Rgba([209, 177, 135, 0]));
        assert_eq!(opaque, transparent);
    }

    #[test]
    fn color_to_rgb_handles_non_rgb_variants() {
        assert_eq!(color_to_rgb(Color::Rgb(1, 2, 3)), (1, 2, 3));
        assert_eq!(color_to_rgb(Color::Indexed(42)), (42, 42, 42));
        assert_eq!(color_to_rgb(Color::Reset), (0, 0, 0));
        assert_eq!(color_to_rgb(Color::Red), (0, 0, 0));
    }
}
