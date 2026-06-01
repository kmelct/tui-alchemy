use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CatalogKind {
    LittleAlchemy1,
    LittleAlchemy2,
    Combined,
}

impl CatalogKind {
    pub fn title(self) -> &'static str {
        match self {
            Self::LittleAlchemy1 => "Little Alchemy 1",
            Self::LittleAlchemy2 => "Little Alchemy 2",
            Self::Combined => "Little Alchemy",
        }
    }

    pub fn asset_dir(self) -> &'static str {
        match self {
            Self::LittleAlchemy1 | Self::Combined => "assets/icons/little-alchemy-1",
            Self::LittleAlchemy2 => "assets/icons/little-alchemy-2",
        }
    }

    pub fn asset_extension(self) -> &'static str {
        match self {
            Self::LittleAlchemy1 | Self::Combined => "png",
            Self::LittleAlchemy2 => "svg",
        }
    }

    pub fn pixel_sprite_dir(self) -> &'static str {
        match self {
            Self::LittleAlchemy1 | Self::Combined => "assets/pixel-sprites/little-alchemy-1",
            Self::LittleAlchemy2 => "assets/pixel-sprites/little-alchemy-2",
        }
    }
}

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
    pub icon_path: PathBuf,
    pub pixel_sprite_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct RecipeKey {
    left: String,
    right: String,
}

#[derive(Debug, Clone)]
pub struct GameCatalog {
    pub kind: CatalogKind,
    pub source: String,
    pub total: usize,
    pub elements: Vec<ElementEntry>,
    recipe_index: HashMap<RecipeKey, Vec<usize>>,
    pub base_indices: Vec<usize>,
    pub count_unlocks: Vec<(usize, usize)>,
    pub name_to_index: HashMap<String, usize>,
}

impl GameCatalog {
    pub fn load_all() -> Vec<Self> {
        vec![
            Self::from_raw(
                CatalogKind::LittleAlchemy1,
                include_str!("../data/little_alchemy_wiki.json"),
            ),
            Self::from_raw(
                CatalogKind::LittleAlchemy2,
                include_str!("../data/little_alchemy_2.json"),
            ),
        ]
    }

    pub fn load_combined_book() -> Self {
        let source_catalogs = [
            Self::from_raw(
                CatalogKind::LittleAlchemy1,
                include_str!("../data/little_alchemy_wiki.json"),
            ),
            Self::from_raw(
                CatalogKind::LittleAlchemy2,
                include_str!("../data/little_alchemy_2.json"),
            ),
        ];
        Self::combine_sources(source_catalogs)
    }

    pub fn load_playable_books() -> Vec<Self> {
        vec![Self::load_combined_book()]
    }

    fn combine_sources<const N: usize>(sources: [Self; N]) -> Self {
        let mut elements: Vec<ElementEntry> = Vec::new();
        let mut name_to_index: HashMap<String, usize> = HashMap::new();

        for source in sources {
            for element in source.elements {
                let normalized = normalize(&element.name);
                if let Some(&index) = name_to_index.get(&normalized) {
                    let existing = &mut elements[index];
                    existing.base |= element.base;
                    existing.is_final |= element.is_final;
                    existing.hidden |= element.hidden;
                    existing.unlock = merge_unlock_rules(&existing.unlock, &element.unlock);
                    if !existing.pixel_sprite_path.exists() && element.pixel_sprite_path.exists() {
                        existing.pixel_sprite_path = element.pixel_sprite_path.clone();
                    }
                    if !existing.icon_path.exists() && element.icon_path.exists() {
                        existing.icon_path = element.icon_path.clone();
                    }
                    for recipe in element.recipes {
                        push_unique_recipe(&mut existing.recipes, recipe);
                    }
                } else {
                    let index = elements.len();
                    name_to_index.insert(normalized, index);
                    elements.push(element);
                }
            }
        }

        let mut recipe_index: HashMap<RecipeKey, Vec<usize>> = HashMap::new();
        let mut base_indices = Vec::new();
        let mut count_unlocks = Vec::new();
        for (index, element) in elements.iter().enumerate() {
            if element.base {
                base_indices.push(index);
            }
            if let UnlockRule::DiscoveredCount(minimum) = element.unlock {
                count_unlocks.push((index, minimum));
            }
            for recipe in &element.recipes {
                let key = RecipeKey::from_pair(&recipe[0], &recipe[1]);
                let outputs = recipe_index.entry(key).or_default();
                if !outputs.contains(&index) {
                    outputs.push(index);
                }
            }
        }
        count_unlocks.sort_by_key(|(_, minimum)| *minimum);

        Self {
            kind: CatalogKind::Combined,
            source: "little-alchemy-combined".to_string(),
            total: elements.len(),
            elements,
            recipe_index,
            base_indices,
            count_unlocks,
            name_to_index,
        }
    }

    pub fn from_raw_json(kind: CatalogKind, raw_json: &str) -> Self {
        Self::from_raw(kind, raw_json)
    }

    fn from_raw(kind: CatalogKind, raw_json: &str) -> Self {
        let raw: RawCatalog = serde_json::from_str(raw_json)
            .unwrap_or_else(|error| panic!("failed to parse {}: {error}", kind.title()));

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
            let icon_path = PathBuf::from(kind.asset_dir()).join(format!(
                "{}.{}",
                slug,
                kind.asset_extension()
            ));
            let pixel_sprite_path =
                PathBuf::from(kind.pixel_sprite_dir()).join(format!("{slug}.png"));
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
                icon_path,
                pixel_sprite_path,
            });
        }

        count_unlocks.sort_by_key(|(_, minimum)| *minimum);

        Self {
            kind,
            source: raw.source,
            total: raw.total,
            elements,
            recipe_index,
            base_indices,
            count_unlocks,
            name_to_index,
        }
    }

    pub fn title(&self) -> &'static str {
        self.kind.title()
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

fn push_unique_recipe(recipes: &mut Vec<[String; 2]>, recipe: [String; 2]) {
    let incoming = RecipeKey::from_pair(&recipe[0], &recipe[1]);
    if recipes
        .iter()
        .any(|existing| RecipeKey::from_pair(&existing[0], &existing[1]) == incoming)
    {
        return;
    }
    recipes.push(recipe);
}

fn merge_unlock_rules(left: &UnlockRule, right: &UnlockRule) -> UnlockRule {
    match (left, right) {
        (UnlockRule::None, _) | (_, UnlockRule::None) => UnlockRule::None,
        (UnlockRule::DiscoveredCount(a), UnlockRule::DiscoveredCount(b)) => {
            UnlockRule::DiscoveredCount((*a).min(*b))
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

pub fn icon_path_for(kind: CatalogKind, name: &str) -> PathBuf {
    let slug = slugify(name);
    PathBuf::from(kind.asset_dir()).join(format!("{}.{}", slug, kind.asset_extension()))
}

pub fn has_icon_file(kind: CatalogKind, name: &str) -> bool {
    icon_path_for(kind, name).exists()
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
        GameCatalog::from_raw_json(CatalogKind::LittleAlchemy1, TINY_JSON)
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
        assert_eq!(catalog.title(), "Little Alchemy 1");
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
        let mut discovered = none_discovered.clone();
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

    #[test]
    fn icon_path_uses_slug_and_kind_extension() {
        let p1 = icon_path_for(CatalogKind::LittleAlchemy1, "Primordial Soup");
        assert!(p1.ends_with("primordial-soup.png"));
        let p2 = icon_path_for(CatalogKind::LittleAlchemy2, "Primordial Soup");
        assert!(p2.ends_with("primordial-soup.svg"));
        assert!(!has_icon_file(
            CatalogKind::LittleAlchemy1,
            "Definitely Not A Real Element 99"
        ));
    }

    #[test]
    fn catalog_kind_paths_are_consistent_per_kind() {
        assert_eq!(CatalogKind::LittleAlchemy1.asset_extension(), "png");
        assert_eq!(CatalogKind::LittleAlchemy2.asset_extension(), "svg");
        assert!(
            CatalogKind::LittleAlchemy1
                .asset_dir()
                .contains("little-alchemy-1")
        );
        assert!(
            CatalogKind::LittleAlchemy2
                .pixel_sprite_dir()
                .contains("little-alchemy-2")
        );
    }

    fn catalog_named(kind: CatalogKind) -> GameCatalog {
        GameCatalog::load_all()
            .into_iter()
            .find(|catalog| catalog.kind == kind)
            .expect("catalog present")
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
    fn little_alchemy_1_recipe_inputs_all_resolve() {
        // The primary catalog must be referentially complete: every recipe
        // ingredient names an element that actually exists.
        let catalog = catalog_named(CatalogKind::LittleAlchemy1);
        let missing = dangling_recipe_inputs(&catalog);
        assert!(
            missing.is_empty(),
            "Little Alchemy 1 has unresolved recipe inputs: {missing:?}"
        );
    }

    #[test]
    fn little_alchemy_2_dangling_inputs_match_known_baseline() {
        // KNOWN DATA GAP: the scraped LA2 set lists 720 elements but references
        // 29 ingredient names (mostly Myths & Monsters content, plus a couple of
        // likely typos such as "Baast") that have no element entry. Those recipes
        // are inert rather than crashing. Pinning the exact set here documents the
        // gap and guards against regression: if the data is fixed this shrinks
        // (update the list), and any *new* dangling name fails the test loudly.
        const KNOWN_DANGLING: [&str; 29] = [
            "Baast",
            "Baba yaga",
            "Babe the blue ox",
            "Book of the dead",
            "Cockatrice",
            "Cosmic egg",
            "Cupid",
            "Curse",
            "Cyclops",
            "Deity",
            "Demon",
            "Dionysus",
            "Elf",
            "Faerie",
            "Good",
            "Heaven",
            "Holy grail",
            "Holy water",
            "Jiangshi",
            "Maui's fishhook",
            "Monster",
            "Necromancer",
            "Paladin",
            "Paul bunyan",
            "Peach of immortality",
            "Philosopher's stone",
            "Selkie",
            "Troll",
            "Zeus",
        ];
        let catalog = catalog_named(CatalogKind::LittleAlchemy2);
        let missing = dangling_recipe_inputs(&catalog);
        let expected: std::collections::BTreeSet<String> =
            KNOWN_DANGLING.iter().map(|s| s.to_string()).collect();
        assert_eq!(
            missing, expected,
            "the set of unresolved LA2 recipe inputs drifted from the documented baseline"
        );
    }

    #[test]
    fn combined_book_is_fully_discoverable_from_base_elements() {
        let catalog = GameCatalog::load_combined_book();
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
    fn real_catalogs_have_base_elements_and_sorted_unlocks() {
        for catalog in GameCatalog::load_all() {
            assert!(
                !catalog.base_indices.is_empty(),
                "{} should define base elements",
                catalog.title()
            );
            // count_unlocks must be sorted ascending by minimum (reconcile relies on it).
            let minimums: Vec<usize> = catalog.count_unlocks.iter().map(|(_, m)| *m).collect();
            let mut sorted = minimums.clone();
            sorted.sort_unstable();
            assert_eq!(
                minimums,
                sorted,
                "{} unlocks must be sorted",
                catalog.title()
            );
        }
    }
}
