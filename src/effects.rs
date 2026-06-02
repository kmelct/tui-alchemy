use crate::data::slugify;

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
}
#[derive(Debug, Clone)]
pub struct ElementEffect {
    pub element_index: usize,
    pub ttl: u8,
    pub age: u8,
}

impl ElementEffect {
    pub const fn birth(element_index: usize, _name: &str) -> Self {
        Self {
            element_index,
            ttl: 12,
            age: 0,
        }
    }

    pub const fn age(&mut self) -> bool {
        self.age = self.age.saturating_add(1);
        self.age < self.ttl
    }
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
    fn birth_effect_ages_until_its_ttl_expires() {
        let mut effect = super::ElementEffect::birth(7, "Steam");
        assert_eq!(effect.element_index, 7);
        // age() returns true while alive, false once it reaches ttl (12).
        let mut alive_ticks = 0;
        while effect.age() {
            alive_ticks += 1;
            assert!(alive_ticks < 100, "effect must eventually expire");
        }
        assert_eq!(alive_ticks, effect.ttl as usize - 1);
    }
}
