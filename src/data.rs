use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct RawCatalog {
    pub source: String,
    pub total: usize,
    pub elements: Vec<RawElement>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawElement {
    pub name: String,
    #[serde(default)]
    pub slug: String,
    #[serde(default)]
    pub base: bool,
    #[serde(default, rename = "final")]
    pub is_final: bool,
    #[serde(default)]
    pub hidden: bool,
    #[serde(default)]
    pub recipes: Vec<[String; 2]>,
    #[serde(default)]
    pub sprite: Option<PathBuf>,
    #[serde(default)]
    pub unlock: Option<RawUnlock>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RawUnlock {
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub minimum: usize,
}

#[derive(Debug, Clone)]
pub enum UnlockRule {
    None,
    DiscoveredCount(usize),
}

#[derive(Debug, Clone)]
pub struct ElementEntry {
    pub name: String,
    pub slug: String,
    pub base: bool,
    pub is_final: bool,
    pub hidden: bool,
    pub recipes: Vec<[String; 2]>,
    pub unlock: UnlockRule,
    pub pixel_sprite_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct RecipeKey {
    left: String,
    right: String,
}

#[derive(Debug, Clone)]
pub struct GameCatalog {
    pub source: String,
    pub total: usize,
    pub elements: Vec<ElementEntry>,
    recipe_index: HashMap<RecipeKey, Vec<usize>>,
    pub base_indices: Vec<usize>,
    pub count_unlocks: Vec<(usize, usize)>,
    pub name_to_index: HashMap<String, usize>,
}

impl GameCatalog {
    pub fn load() -> Self {
        Self::from_raw_json(include_str!("../data/little_alchemy.json"))
    }

    pub fn load_playable_books() -> Vec<Self> {
        vec![Self::load()]
    }

    pub fn from_raw_json(raw_json: &str) -> Self {
        Self::from_raw(raw_json)
    }

    fn from_raw(raw_json: &str) -> Self {
        let raw: RawCatalog = serde_json::from_str(raw_json)
            .unwrap_or_else(|error| panic!("failed to parse canonical catalog: {error}"));

        let mut elements = Vec::with_capacity(raw.elements.len());
        let mut recipe_index: HashMap<RecipeKey, Vec<usize>> = HashMap::new();
        let mut base_indices = Vec::new();
        let mut count_unlocks = Vec::new();
        let mut name_to_index = HashMap::new();

        for (index, raw_element) in raw.elements.into_iter().enumerate() {
            let slug = if raw_element.slug.is_empty() {
                slugify(&raw_element.name)
            } else {
                slugify(&raw_element.slug)
            };
            let pixel_sprite_path = raw_element.sprite.unwrap_or_else(|| {
                PathBuf::from("assets/pixel-sprites").join(format!("{slug}.png"))
            });
            let unlock = match raw_element.unlock {
                Some(rule) if rule.kind == "discovered_count" => {
                    UnlockRule::DiscoveredCount(rule.minimum)
                }
                _ => UnlockRule::None,
            };

            if raw_element.base {
                base_indices.push(index);
            }
            if let UnlockRule::DiscoveredCount(minimum) = unlock {
                count_unlocks.push((index, minimum));
            }

            name_to_index.insert(normalize(&raw_element.name), index);

            for recipe in &raw_element.recipes {
                let key = RecipeKey::from_pair(&recipe[0], &recipe[1]);
                recipe_index.entry(key).or_default().push(index);
            }

            elements.push(ElementEntry {
                name: raw_element.name,
                slug,
                base: raw_element.base,
                is_final: raw_element.is_final,
                hidden: raw_element.hidden,
                recipes: raw_element.recipes,
                unlock,
                pixel_sprite_path,
            });
        }

        count_unlocks.sort_by_key(|(_, minimum)| *minimum);

        Self {
            source: raw.source,
            total: raw.total,
            elements,
            recipe_index,
            base_indices,
            count_unlocks,
            name_to_index,
        }
    }

    pub const fn title(&self) -> &'static str {
        "Little Alchemy"
    }

    pub fn canonical_name(&self, index: usize) -> &str {
        &self.elements[index].name
    }

    pub fn visible_indices(&self, discovered: &[bool]) -> Vec<usize> {
        self.elements
            .iter()
            .enumerate()
            .filter_map(|(index, _)| {
                discovered
                    .get(index)
                    .copied()
                    .unwrap_or(false)
                    .then_some(index)
            })
            .collect()
    }

    pub fn recipe_outputs(&self, left: usize, right: usize) -> &[usize] {
        let key = RecipeKey::from_pair(&self.elements[left].name, &self.elements[right].name);
        self.recipe_index
            .get(&key)
            .map(|items| items.as_slice())
            .unwrap_or(&[])
    }

    pub fn discoverable_by_count(
        &self,
        discovered_count: usize,
        discovered: &[bool],
    ) -> Vec<usize> {
        self.count_unlocks
            .iter()
            .filter_map(|(index, minimum)| {
                if *minimum <= discovered_count && !discovered.get(*index).copied().unwrap_or(false)
                {
                    Some(*index)
                } else {
                    None
                }
            })
            .collect()
    }
}

impl RecipeKey {
    fn from_pair(left: &str, right: &str) -> Self {
        let left = normalize(left);
        let right = normalize(right);
        if left <= right {
            Self { left, right }
        } else {
            Self {
                left: right,
                right: left,
            }
        }
    }
}

pub fn slugify(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut pending_dash = false;

    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            if pending_dash && !out.is_empty() {
                out.push('-');
            }
            out.push(ch.to_ascii_lowercase());
            pending_dash = false;
        } else {
            pending_dash = true;
        }
    }

    while out.ends_with('-') {
        out.pop();
    }

    if out.is_empty() {
        "element".to_string()
    } else {
        out
    }
}

pub fn normalize(value: &str) -> String {
    value
        .chars()
        .filter_map(|ch| {
            if ch.is_ascii_alphanumeric() {
                Some(ch.to_ascii_lowercase())
            } else if ch.is_whitespace() || ch == '-' || ch == '_' {
                Some(' ')
            } else {
                None
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn unique_visible_names<'a>(catalog: &'a GameCatalog, discovered: &[bool]) -> Vec<&'a str> {
    catalog
        .visible_indices(discovered)
        .into_iter()
        .map(|index| catalog.canonical_name(index))
        .collect()
}

pub fn base_discovery_state(catalog: &GameCatalog) -> Vec<bool> {
    let mut discovered = vec![false; catalog.elements.len()];
    for &index in &catalog.base_indices {
        discovered[index] = true;
    }
    discovered
}

pub fn discovered_count(discovered: &[bool]) -> usize {
    discovered.iter().copied().filter(|flag| *flag).count()
}

pub fn active_palette_indices(catalog: &GameCatalog, discovered: &[bool]) -> Vec<usize> {
    catalog.visible_indices(discovered)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TINY_JSON: &str = r#"{
        "source": "unit-test",
        "total": 7,
        "elements": [
            {"name": "Air", "base": true},
            {"name": "Earth", "base": true},
            {"name": "Fire", "base": true},
            {"name": "Water", "base": true},
            {"name": "Steam", "recipes": [["Water", "Fire"]]},
            {"name": "Energy", "recipes": [["Fire", "Air"]]},
            {"name": "Quintessence", "unlock": {"kind": "discovered_count", "minimum": 5}}
        ]
    }"#;

    fn tiny() -> GameCatalog {
        GameCatalog::from_raw_json(TINY_JSON)
    }

    #[test]
    fn load_uses_the_single_canonical_catalog() {
        let catalog = GameCatalog::load();

        assert_eq!(catalog.title(), "Little Alchemy");
        assert_eq!(catalog.source, "little-alchemy");
        assert_eq!(catalog.total, catalog.elements.len());
        assert!(
            catalog.elements.iter().all(|element| element
                .pixel_sprite_path
                .ends_with(format!("{}.png", element.slug))),
            "every canonical element must carry its resolved runtime sprite path"
        );
    }
    #[test]
    fn slugify_normalises_to_kebab_ascii() {
        assert_eq!(slugify("Primordial Soup"), "primordial-soup");
        assert_eq!(slugify("  Trailing  "), "trailing");
        assert_eq!(slugify("A & B"), "a-b");
        assert_eq!(slugify("café"), "caf"); // non-ascii dropped
        assert_eq!(slugify("multi---dash"), "multi-dash");
        assert_eq!(slugify("!!!"), "element"); // empty fallback
        assert_eq!(slugify("HOT"), "hot");
    }

    #[test]
    fn normalize_lowercases_and_collapses_separators() {
        assert_eq!(normalize("Primordial Soup"), "primordial soup");
        assert_eq!(normalize("under_score"), "under score");
        assert_eq!(normalize("dash-dash"), "dash dash");
        assert_eq!(normalize("  many   spaces "), "many spaces");
        assert_eq!(normalize("Pün!ct"), "pnct");
        assert_eq!(normalize("WATER"), "water");
    }

    #[test]
    fn recipe_key_is_order_invariant() {
        assert_eq!(
            RecipeKey::from_pair("Water", "Fire"),
            RecipeKey::from_pair("Fire", "Water")
        );
        // Normalisation folds into the key too.
        assert_eq!(
            RecipeKey::from_pair("WATER", "fire"),
            RecipeKey::from_pair("fire", "water")
        );
    }

    #[test]
    fn from_raw_json_indexes_base_unlocks_and_names() {
        let catalog = tiny();
        assert_eq!(catalog.total, 7);
        assert_eq!(catalog.title(), "Little Alchemy");
        assert_eq!(catalog.source, "unit-test");
        // Base elements are the first four.
        assert_eq!(catalog.base_indices, vec![0, 1, 2, 3]);
        // Names resolve (normalised).
        assert_eq!(catalog.name_to_index.get("steam"), Some(&4));
        assert_eq!(catalog.name_to_index.get("quintessence"), Some(&6));
        // Count unlocks captured and sorted by minimum.
        assert_eq!(catalog.count_unlocks, vec![(6, 5)]);
    }

    #[test]
    fn recipe_outputs_are_order_invariant_and_empty_for_unknown_pairs() {
        let catalog = tiny();
        // Water(3) + Fire(2) -> Steam(4), regardless of argument order.
        assert_eq!(catalog.recipe_outputs(3, 2), [4]);
        assert_eq!(catalog.recipe_outputs(2, 3), [4]);
        // Air(0) + Water(3) is not a recipe.
        assert!(catalog.recipe_outputs(0, 3).is_empty());
    }

    #[test]
    fn discoverable_by_count_respects_the_threshold() {
        let catalog = tiny();
        let none_discovered = vec![false; catalog.elements.len()];
        // Below the threshold: nothing.
        assert!(
            catalog
                .discoverable_by_count(4, &none_discovered)
                .is_empty()
        );
        // At the threshold: Quintessence (index 6) becomes available.
        assert_eq!(catalog.discoverable_by_count(5, &none_discovered), vec![6]);
        // Already discovered: excluded even above the threshold.
        let mut discovered = none_discovered;
        discovered[6] = true;
        assert!(catalog.discoverable_by_count(9, &discovered).is_empty());
    }

    #[test]
    fn visibility_helpers_track_the_discovered_mask() {
        let catalog = tiny();
        let base = base_discovery_state(&catalog);
        assert_eq!(discovered_count(&base), 4);
        assert_eq!(catalog.visible_indices(&base), vec![0, 1, 2, 3]);
        assert_eq!(active_palette_indices(&catalog, &base), vec![0, 1, 2, 3]);

        let names = unique_visible_names(&catalog, &base);
        assert_eq!(names, vec!["Air", "Earth", "Fire", "Water"]);
        assert_eq!(catalog.canonical_name(4), "Steam");
    }

    /// Distinct recipe-input names that do not resolve to any element entry.
    fn dangling_recipe_inputs(catalog: &GameCatalog) -> std::collections::BTreeSet<String> {
        let mut missing = std::collections::BTreeSet::new();
        for element in &catalog.elements {
            for recipe in &element.recipes {
                for input in recipe {
                    if !catalog.name_to_index.contains_key(&normalize(input)) {
                        missing.insert(input.clone());
                    }
                }
            }
        }
        missing
    }

    #[test]
    fn canonical_recipe_inputs_all_resolve() {
        let catalog = GameCatalog::load();
        let missing = dangling_recipe_inputs(&catalog);
        assert!(
            missing.is_empty(),
            "canonical catalog has unresolved recipe inputs: {missing:?}"
        );
    }

    #[test]
    fn combined_book_is_fully_discoverable_from_base_elements() {
        let catalog = GameCatalog::load();
        let mut discovered = base_discovery_state(&catalog);
        let mut changed = true;

        while changed {
            changed = false;
            for element_index in
                catalog.discoverable_by_count(discovered_count(&discovered), &discovered)
            {
                discovered[element_index] = true;
                changed = true;
            }

            for left in 0..catalog.elements.len() {
                if !discovered[left] {
                    continue;
                }
                for right in left..catalog.elements.len() {
                    if !discovered[right] {
                        continue;
                    }
                    for &output in catalog.recipe_outputs(left, right) {
                        if !discovered[output] {
                            discovered[output] = true;
                            changed = true;
                        }
                    }
                }
            }
        }

        let missing: Vec<_> = catalog
            .elements
            .iter()
            .enumerate()
            .filter_map(|(index, element)| (!discovered[index]).then_some(element.name.as_str()))
            .collect();
        assert!(
            missing.is_empty(),
            "combined recipe book should be playable to completion; unreachable elements: {missing:?}"
        );
    }

    #[test]
    fn canonical_catalog_has_base_elements_and_sorted_unlocks() {
        let catalog = GameCatalog::load();
        assert!(
            !catalog.base_indices.is_empty(),
            "canonical catalog should define base elements"
        );
        // count_unlocks must be sorted ascending by minimum (reconcile relies on it).
        let minimums: Vec<usize> = catalog.count_unlocks.iter().map(|(_, m)| *m).collect();
        let mut sorted = minimums.clone();
        sorted.sort_unstable();
        assert_eq!(minimums, sorted, "canonical unlocks must be sorted");
    }
}
