//! Design tokens for the alchemist's-study scene.
//!
//! One coherent palette so the whole screen reads as a single 16-bit picture: a
//! dark night-chamber backdrop (a sparse starfield, never a flat fill), warm
//! bronze-rimmed panels for the stats rail and the recipe table, and cool stone
//! pedestals that hold the discovered elements. Hues derive from the Lost
//! Century palette ([`crate::palette`]); the surfaces below are scene tones.

use ratatui::style::{Color, Modifier, Style};

/// Flat scene surfaces, painted as cell backgrounds.
pub struct Surfaces;

impl Surfaces {
    // --- Night-chamber backdrop (fills everything behind the panels). ---
    /// Deep base tone of the chamber.
    pub const VOID: Color = Color::Rgb(12, 14, 22);
    /// Dim background speck (distant motes).
    pub const SPECK_DIM: Color = Color::Rgb(28, 31, 46);
    /// Brighter background speck (closer sparkles).
    pub const SPECK_LIT: Color = Color::Rgb(56, 62, 88);

    // --- Bronze-rimmed UI panels (rail + recipe table). ---
    pub const PANEL_BG: Color = Color::Rgb(22, 25, 36);
    pub const PANEL_RIM: Color = Color::Rgb(92, 66, 38);
    pub const PANEL_SHADOW: Color = Color::Rgb(8, 9, 15);

    // --- Stone pedestals the element tiles rest on. ---
    /// Lit top surface where the sprite sits (kept distinct from ATLAS_BG).
    pub const PEDESTAL_TOP: Color = Color::Rgb(46, 52, 72);
    /// Active/selected shelf top (warm highlight).
    pub const PEDESTAL_TOP_ACTIVE: Color = Color::Rgb(78, 72, 52);
    /// Front riser face.
    pub const PEDESTAL_FACE: Color = Color::Rgb(28, 32, 46);
    /// Shaded side / depth.
    pub const PEDESTAL_SIDE: Color = Color::Rgb(16, 18, 27);
    /// Cast shadow under a tile (a bg fill, never a glyph).
    pub const DROP_SHADOW: Color = Color::Rgb(9, 10, 16);

    // --- Recipe table sockets. ---
    /// Empty recessed socket bed.
    pub const SOCKET_BED: Color = Color::Rgb(34, 32, 42);
    /// Filled-socket fallback (kept value: tests pin it against this sentinel).
    pub const SLOT_BED: Color = Color::Rgb(64, 50, 55);

    // --- Stats rail aliases (kept names; share the panel palette). ---
    pub const RAIL_BG: Color = Self::PANEL_BG;
    pub const RAIL_RIM: Color = Self::PANEL_RIM;
    pub const RAIL_SHADOW: Color = Self::PANEL_SHADOW;

    /// Legacy atlas tile sentinel — pedestal tops must differ from this.
    pub const ATLAS_BG: Color = Color::Rgb(24, 38, 43);
}

/// Accent ink roles as indices into the Lost Century palette
/// (resolve with [`crate::palette::palette_color`]).
pub struct Ink;

impl Ink {
    pub const TITLE: usize = 10;
    pub const STAT: usize = 9;
    pub const CATALOG: usize = 11;
    pub const FRAME: usize = 5;
    pub const SELECTED: usize = 1;
    pub const MUTED: usize = 7;
    pub const HINT: usize = 14;
}

/// Border / decoration glyphs (never `█`/`░`; depth uses bg fills + these).
pub struct Glyphs;

impl Glyphs {
    pub const CORNER_TL: &'static str = "▛";
    pub const CORNER_TR: &'static str = "▜";
    pub const CORNER_BL: &'static str = "▙";
    pub const CORNER_BR: &'static str = "▟";
    pub const TOP: &'static str = "▀";
    pub const BOTTOM: &'static str = "▄";
    pub const SIDE_L: &'static str = "▌";
    pub const SIDE_R: &'static str = "▐";
    pub const SOCKET: &'static str = "◆";
    pub const SWAP: &'static str = "⇆";
    pub const RUNE: &'static str = "✦";
}

/// A label style with the given accent fg over `bg`, optionally bold.
pub fn label_style(accent: Color, bg: Color, strong: bool) -> Style {
    let style = Style::default().fg(accent).bg(bg);
    if strong {
        style.add_modifier(Modifier::BOLD)
    } else {
        style
    }
}

/// A bold rune/accent glyph style.
pub fn rune_style(accent: Color, bg: Color) -> Style {
    Style::default()
        .fg(accent)
        .bg(bg)
        .add_modifier(Modifier::BOLD)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn label_style_sets_fg_bg_and_optional_bold() {
        let plain = label_style(Color::Rgb(1, 2, 3), Color::Rgb(4, 5, 6), false);
        assert_eq!(plain.fg, Some(Color::Rgb(1, 2, 3)));
        assert_eq!(plain.bg, Some(Color::Rgb(4, 5, 6)));
        assert!(!plain.add_modifier.contains(Modifier::BOLD));

        let strong = label_style(Color::Rgb(1, 2, 3), Color::Rgb(4, 5, 6), true);
        assert!(strong.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn rune_style_is_always_bold() {
        let style = rune_style(Color::Rgb(9, 9, 9), Color::Rgb(0, 0, 0));
        assert_eq!(style.fg, Some(Color::Rgb(9, 9, 9)));
        assert_eq!(style.bg, Some(Color::Rgb(0, 0, 0)));
        assert!(style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn pedestal_top_stays_distinct_from_legacy_atlas_sentinel() {
        // Documented invariant: pedestal tops must never collide with ATLAS_BG.
        assert_ne!(Surfaces::PEDESTAL_TOP, Surfaces::ATLAS_BG);
        assert_ne!(Surfaces::PEDESTAL_TOP_ACTIVE, Surfaces::ATLAS_BG);
    }

    #[test]
    fn ink_roles_index_into_the_sixteen_colour_palette() {
        for index in [
            Ink::TITLE,
            Ink::STAT,
            Ink::CATALOG,
            Ink::FRAME,
            Ink::SELECTED,
            Ink::MUTED,
            Ink::HINT,
        ] {
            assert!(index < 16, "ink role {index} is out of palette range");
        }
    }
}
