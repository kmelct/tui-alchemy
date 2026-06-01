use crate::data::slugify;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementStyle {
    Air,
    Earth,
    Fire,
    Water,
    Steam,
    Stone,
    Plant,
    Metal,
    Light,
    Organic,
    Container,
    Neutral,
}

impl ElementStyle {
    pub fn for_name(name: &str) -> Self {
        let slug = slugify(name);
        if contains_any(&slug, &["bottle", "glass", "jar", "vase", "container"]) {
            Self::Container
        } else if contains_any(
            &slug,
            &["water", "rain", "river", "sea", "ocean", "ice", "snow"],
        ) {
            Self::Water
        } else if contains_any(&slug, &["fire", "lava", "sun", "heat", "flame"]) {
            Self::Fire
        } else if contains_any(&slug, &["air", "wind", "storm", "cloud"]) {
            Self::Air
        } else if contains_any(&slug, &["steam", "smoke", "mist", "fog"]) {
            Self::Steam
        } else if contains_any(&slug, &["earth", "soil", "dust", "sand", "mud", "clay"]) {
            Self::Earth
        } else if contains_any(&slug, &["stone", "rock", "mountain", "coal", "granite"]) {
            Self::Stone
        } else if contains_any(&slug, &["plant", "tree", "grass", "flower", "leaf", "seed"]) {
            Self::Plant
        } else if contains_any(&slug, &["metal", "steel", "gold", "silver", "iron", "tool"]) {
            Self::Metal
        } else if contains_any(&slug, &["light", "lightning", "electric", "energy"]) {
            Self::Light
        } else if contains_any(&slug, &["life", "animal", "bird", "fish", "human", "worm"]) {
            Self::Organic
        } else {
            Self::Neutral
        }
    }

    pub fn accent(self) -> Color {
        match self {
            Self::Air => Color::Rgb(174, 220, 255),
            Self::Earth => Color::Rgb(177, 137, 82),
            Self::Fire => Color::Rgb(255, 183, 56),
            Self::Water => Color::Rgb(77, 194, 255),
            Self::Steam => Color::Rgb(232, 236, 255),
            Self::Stone => Color::Rgb(166, 170, 190),
            Self::Plant => Color::Rgb(103, 217, 81),
            Self::Metal => Color::Rgb(214, 224, 236),
            Self::Light => Color::Rgb(255, 240, 116),
            Self::Organic => Color::Rgb(218, 145, 104),
            Self::Container => Color::Rgb(148, 228, 232),
            Self::Neutral => Color::Rgb(210, 201, 165),
        }
    }

    fn secondary(self) -> Color {
        match self {
            Self::Air => Color::Rgb(107, 156, 255),
            Self::Earth => Color::Rgb(93, 70, 54),
            Self::Fire => Color::Rgb(244, 81, 36),
            Self::Water => Color::Rgb(43, 112, 232),
            Self::Steam => Color::Rgb(166, 185, 220),
            Self::Stone => Color::Rgb(92, 96, 118),
            Self::Plant => Color::Rgb(32, 146, 51),
            Self::Metal => Color::Rgb(126, 142, 160),
            Self::Light => Color::Rgb(255, 255, 215),
            Self::Organic => Color::Rgb(145, 86, 70),
            Self::Container => Color::Rgb(74, 156, 176),
            Self::Neutral => Color::Rgb(146, 116, 65),
        }
    }

    fn glyphs(self) -> [&'static str; 3] {
        match self {
            Self::Fire => ["▄", "▀", "▖"],
            Self::Water => ["▄", "▀", "▗"],
            Self::Air => ["▘", "▀", "▝"],
            Self::Steam => ["▗", "▀", "▖"],
            Self::Earth => ["▄", "▖", "▗"],
            Self::Stone => ["▄", "▀", "▘"],
            Self::Plant => ["▖", "▄", "▗"],
            Self::Metal => ["▀", "▄", "▝"],
            Self::Light => ["▀", "▘", "▝"],
            Self::Organic => ["▖", "▄", "▗"],
            Self::Container => ["▀", "▖", "▗"],
            Self::Neutral => ["▄", "▖", "▗"],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectKind {
    Birth,
    Drag,
}

#[derive(Debug, Clone)]
pub struct ElementEffect {
    pub element_index: usize,
    pub style: ElementStyle,
    pub kind: EffectKind,
    pub ttl: u8,
    pub age: u8,
}

impl ElementEffect {
    pub fn birth(element_index: usize, name: &str) -> Self {
        Self {
            element_index,
            style: ElementStyle::for_name(name),
            kind: EffectKind::Birth,
            ttl: 12,
            age: 0,
        }
    }

    pub fn age(&mut self) -> bool {
        self.age = self.age.saturating_add(1);
        self.age < self.ttl
    }
}

pub fn particle_lines(
    style: ElementStyle,
    kind: EffectKind,
    tick: u64,
    width: u16,
) -> Vec<Line<'static>> {
    let height = match kind {
        EffectKind::Birth => 3,
        EffectKind::Drag => 2,
    };
    let mut rows = vec![vec![Span::raw(" "); width as usize]; height];
    let glyphs = style.glyphs();
    let count = match kind {
        EffectKind::Birth => 6,
        EffectKind::Drag => 3,
    };

    let center = width.max(1) as i16 / 2;
    let birth_offsets = [-4, -2, 0, 2, 4, 1];
    let birth_rows = [
        [1usize, 0, 2, 0, 1, 2],
        [0usize, 1, 1, 2, 0, 2],
        [1usize, 2, 0, 1, 2, 0],
        [2usize, 1, 0, 1, 0, 2],
    ];
    let drag_offsets = [-2, 0, 2];

    for index in 0..count {
        let phase = ((tick / 3) as usize) % birth_rows.len();
        let x = match kind {
            EffectKind::Birth => (center + birth_offsets[index % birth_offsets.len()])
                .clamp(0, width.saturating_sub(1) as i16) as usize,
            EffectKind::Drag => (center + drag_offsets[index % drag_offsets.len()])
                .clamp(0, width.saturating_sub(1) as i16) as usize,
        };
        let y = match kind {
            EffectKind::Birth => birth_rows[phase][index % birth_rows[phase].len()].min(height - 1),
            EffectKind::Drag => index % height,
        };
        let color = if index % 3 == 0 {
            style.secondary()
        } else {
            style.accent()
        };
        rows[y][x] = Span::styled(glyphs[index % glyphs.len()], Style::default().fg(color));
    }

    rows.into_iter().map(Line::from).collect()
}

pub fn style_for_element_name(name: &str) -> ElementStyle {
    ElementStyle::for_name(name)
}

fn contains_any(value: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| value.contains(needle))
}

#[cfg(test)]
mod tests {
    use super::{ElementStyle, style_for_element_name};

    #[test]
    fn classifies_element_styles_from_names() {
        assert_eq!(style_for_element_name("Water"), ElementStyle::Water);
        assert_eq!(style_for_element_name("Fire"), ElementStyle::Fire);
        assert_eq!(
            style_for_element_name("Glass Bottle"),
            ElementStyle::Container
        );
        assert_eq!(
            style_for_element_name("Bottle of Water"),
            ElementStyle::Container
        );
        assert_eq!(style_for_element_name("Lightning"), ElementStyle::Light);
    }

    #[test]
    fn particles_use_soft_aura_blocks_not_debug_noise() {
        let lines = super::particle_lines(ElementStyle::Water, super::EffectKind::Birth, 3, 16);
        let text = lines
            .iter()
            .flat_map(|line| line.spans.iter())
            .map(|span| span.content.as_ref())
            .collect::<Vec<_>>()
            .join("");

        assert!(
            !text.contains("~") && !text.contains("*") && !text.contains("."),
            "particle helpers should render as soft aura pixels, not debug noise: {text}"
        );
    }

    #[test]
    fn birth_particles_stay_clustered_around_the_element() {
        let lines = super::particle_lines(ElementStyle::Fire, super::EffectKind::Birth, 0, 24);
        let mut columns = Vec::new();

        for line in &lines {
            let mut column = 0usize;
            for span in &line.spans {
                for ch in span.content.chars() {
                    if ch != ' ' {
                        columns.push(column);
                    }
                    column += 1;
                }
            }
        }

        let min = columns.iter().min().copied().unwrap_or_default();
        let max = columns.iter().max().copied().unwrap_or_default();
        assert!(
            max.saturating_sub(min) <= 9,
            "birth particles should be a local aura, not full-width random scatter: {columns:?}"
        );
    }

    #[test]
    fn for_name_covers_every_style_family() {
        use ElementStyle::*;
        let cases = [
            ("Vase", Container),
            ("Sea", Water),
            ("Lava", Fire),
            ("Storm", Air),
            ("Smoke", Steam),
            ("Sand", Earth),
            ("Mountain", Stone),
            ("Flower", Plant),
            ("Gold", Metal),
            ("Energy", Light),
            ("Bird", Organic),
            ("Quintessence", Neutral),
        ];
        for (name, expected) in cases {
            assert_eq!(ElementStyle::for_name(name), expected, "for {name}");
        }
    }

    #[test]
    fn every_style_exposes_distinct_accent_and_secondary_colours() {
        use ElementStyle::*;
        for style in [
            Air, Earth, Fire, Water, Steam, Stone, Plant, Metal, Light, Organic, Container, Neutral,
        ] {
            // accent() and secondary() are total and must never panic.
            assert_ne!(
                style.accent(),
                style.secondary(),
                "{style:?} accent should differ from its secondary"
            );
            // Three aura glyphs are defined for every style.
            assert_eq!(style.glyphs().len(), 3);
        }
    }

    #[test]
    fn birth_effect_ages_until_its_ttl_expires() {
        let mut effect = super::ElementEffect::birth(7, "Steam");
        assert_eq!(effect.kind, super::EffectKind::Birth);
        assert_eq!(effect.style, ElementStyle::Steam);
        // age() returns true while alive, false once it reaches ttl (12).
        let mut alive_ticks = 0;
        while effect.age() {
            alive_ticks += 1;
            assert!(alive_ticks < 100, "effect must eventually expire");
        }
        assert_eq!(alive_ticks, effect.ttl as usize - 1);
    }
}
