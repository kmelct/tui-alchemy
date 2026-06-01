import json
import os
import subprocess
import sys
import tempfile
import unittest
import struct
import zlib
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "tools" / "generate_pixel_sprites.py"

EARLY_GAME_SLUGS = [
    "air",
    "earth",
    "fire",
    "water",
    "dust",
    "energy",
    "lava",
    "mud",
    "pressure",
    "rain",
    "sea",
    "steam",
    "atmosphere",
    "brick",
    "cloud",
    "plant",
    "stone",
    "volcano",
    "wind",
    "grass",
    "metal",
    "mountain",
    "sand",
    "sky",
    "storm",
]

OBJECT_STYLE_SLUGS = [
    "glass",
    "time",
    "life",
    "human",
    "tool",
    "book",
    "bird",
    "fish",
    "house",
    "tree",
    "vase",
    "bottle",
    "jar",
]

ICONIC_REFERENCE_SLUGS = [
    "sun",
    "coal",
    "moon",
    "flower",
    "egg",
    "honey",
    "paper",
    "hammer",
    "wheat",
    "wood",
    "snow",
    "ice",
    "cotton",
    "needle",
    "chain",
    "web",
    "spider",
    "bee",
]

EXPANDED_OBJECT_SLUGS = [
    "glasses",
    "clock",
    "boat",
    "car",
    "cat",
    "dog",
    "scissors",
    "wheel",
    "blade",
    "newspaper",
]

REFERENCE_EXTRA_SLUGS = [
    "lizard",
    "bread",
    "fishing-rod",
    "crystal-ball",
    "butterfly",
    "flying-fish",
    "shovel",
]

CRAFT_OBJECT_SLUGS = [
    "axe",
    "clay",
    "pottery",
    "knife",
]

NATURE_COSMOS_SLUGS = [
    "seaweed",
    "hay",
    "bacteria",
    "wool",
    "cow",
    "horse",
    "rainbow",
    "star",
    "lightning",
]

COSMIC_ELECTRIC_SLUGS = [
    "planet",
    "space",
    "electricity",
    "wire",
    "light-bulb",
    "solar-system",
    "galaxy",
    "telescope",
    "rocket",
    "astronaut",
]

NATURAL_FORCE_SLUGS = [
    "earthquake",
    "flood",
    "geyser",
    "granite",
    "gunpowder",
    "obsidian",
    "ocean",
    "salt",
    "algae",
    "ash",
    "eruption",
    "explosion",
    "fog",
    "hurricane",
    "tsunami",
    "wave",
]

CONSTRUCTED_BOTANICAL_SLUGS = [
    "wall",
    "archipelago",
    "atomic-bomb",
    "beach",
    "boiler",
    "bullet",
    "cactus",
    "desert",
    "dew",
    "diamond",
    "dune",
    "fireworks",
    "garden",
    "ivy",
    "moss",
    "pond",
]

MIDGAME_DEVICE_WORLD_SLUGS = [
    "aquarium",
    "blender",
    "bridge",
    "dam",
    "day",
    "eclipse",
    "gold",
    "golem",
    "greenhouse",
    "gun",
    "hourglass",
    "mirror",
    "night",
    "oasis",
    "oxygen",
    "plankton",
]

CIVILIZATION_TRANSPORT_SLUGS = [
    "airplane",
    "bank",
    "castle",
    "city",
    "farm",
    "farmer",
    "field",
    "forest",
    "helicopter",
    "hospital",
    "lake",
    "river",
    "sailboat",
    "swamp",
    "train",
    "village",
]

MATERIAL_ICONIC_SLUGS = [
    "isle",
    "grenade",
    "horizon",
    "mountain-range",
    "quicksand",
    "rust",
    "sandstone",
    "sandstorm",
    "sound",
    "steel",
    "perfume",
    "pyramid",
    "ring",
    "robot",
    "scythe",
    "sunflower",
]

COMMON_OBJECT_SCENERY_SLUGS = [
    "skyscraper",
    "sword",
    "tide",
    "water-lily",
    "waterfall",
    "windmill",
    "window",
    "barn",
    "birdhouse",
    "dynamite",
    "eagle",
    "lamp",
    "lawn-mower",
    "microscope",
    "oil",
    "paint",
]

FANTASY_CHARACTER_SLUGS = [
    "angel",
    "corpse",
    "cyborg",
    "fireman",
    "gardener",
    "grim-reaper",
    "nerd",
    "phoenix",
    "scarecrow",
    "surfer",
    "unicorn",
    "warrior",
    "wizard",
    "alligator",
    "armor",
    "dragon",
]

EARLY_MISSING_CHARACTER_OBJECT_SLUGS = [
    "tobacco",
    "allergy",
    "bayonet",
    "blood",
    "carbon-dioxide",
    "cold",
    "double-rainbow",
    "duck",
    "electrician",
    "excalibur",
    "family",
    "flamethrower",
    "hard-roe",
    "hay-bale",
    "hummingbird",
    "idea",
]

LIGHT_BIRD_OBJECT_SLUGS = [
    "light",
    "lightsaber",
    "love",
    "music",
    "nest",
    "omelette",
    "ostrich",
    "owl",
    "ozone",
    "peacock",
    "prism",
    "ruins",
    "safe",
    "safety-glasses",
    "seagull",
    "sickness",
]

ACCESSORY_DEVICE_NATURE_SLUGS = [
    "sunglasses",
    "swim-goggles",
    "taser",
    "the-one-ring",
    "toucan",
    "turtle",
    "twilight",
    "water-gun",
    "wind-turbine",
    "alarm-clock",
    "black-hole",
    "bone",
    "bonsai-tree",
    "caviar",
    "chameleon",
    "charcoal",
]

ANIMAL_TECH_MONSTER_SLUGS = [
    "chicken",
    "christmas-tree",
    "computer",
    "constellation",
    "crow",
    "cuckoo",
    "dinosaur",
    "drone",
    "dry-ice",
    "duckling",
    "egg-timer",
    "engineer",
    "family-tree",
    "fire-extinguisher",
    "flashlight",
    "frankenstein",
]

FOOD_GRAVE_MAGIC_SLUGS = [
    "fridge",
    "fruit",
    "grave",
    "harp",
    "herb",
    "jedi",
    "lava-lamp",
    "leaf",
    "lighthouse",
    "livestock",
    "mayonnaise",
    "monarch",
    "mummy",
    "narwhal",
    "oil-lamp",
    "optical-fiber",
]

SHORE_TOOL_SKY_SLUGS = [
    "palm",
    "pegasus",
    "pigeon",
    "pilot",
    "pitchfork",
    "rose",
    "seaplane",
    "seasickness",
    "sewing-machine",
    "shark",
    "shuriken",
    "skeleton",
    "smog",
    "soap",
    "soda",
    "solar-cell",
]

SPACE_TIME_UNDEAD_WEATHER_SLUGS = [
    "spaceship",
    "starfish",
    "statue",
    "steam-engine",
    "sundial",
    "super-nova",
    "swimmer",
    "thread",
    "treehouse",
    "umbrella",
    "vampire",
    "vulture",
    "watch",
    "zombie",
    "acid-rain",
    "alcohol",
]

ALIEN_WINTER_FOOD_CHARACTER_SLUGS = [
    "alien",
    "antarctica",
    "avalanche",
    "blizzard",
    "broom",
    "bulletproof-vest",
    "camel",
    "campfire",
    "chicken-soup",
    "chicken-wing",
    "coconut",
    "coffin",
    "crown",
    "darth-vader",
    "doctor",
    "electric-eel",
]

MATERIAL_NATURE_CREATURE_SLUGS = [
    "fabric",
    "fence",
    "flour",
    "flute",
    "fossil",
    "fountain",
    "fruit-tree",
    "glacier",
    "gnome",
    "goat",
    "godzilla",
    "gravestone",
    "graveyard",
    "hail",
    "iceberg",
    "igloo",
]


class PixelSpriteGeneratorTests(unittest.TestCase):
    def run_script(self, *args, env=None, cwd=None):
        full_env = os.environ.copy()
        full_env.pop("OPENAI_API_KEY", None)
        full_env.pop("OPENAI_KEY", None)
        full_env.update(env or {})
        return subprocess.run(
            [sys.executable, str(SCRIPT), *args],
            cwd=cwd or ROOT,
            env=full_env,
            text=True,
            capture_output=True,
            check=False,
        )

    def test_dry_run_uses_openai_key_without_api_call(self):
        with tempfile.TemporaryDirectory() as tmp:
            workspace = Path(tmp)
            (workspace / ".env").write_text("OPENAI_KEY=sk-test\n")
            result = self.run_script(
                "--catalog",
                "la1",
                "--dry-run",
                "--limit",
                "4",
                cwd=workspace,
            )

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("dry-run", result.stdout)
        self.assertIn("assets/pixel-sprites/little-alchemy-1/air.png", result.stdout)
        self.assertIn("assets/pixel-sprites/little-alchemy-1/water.png", result.stdout)
        self.assertNotIn("sk-test", result.stdout + result.stderr)

    def test_existing_sprites_are_skipped_unless_forced(self):
        with tempfile.TemporaryDirectory() as tmp:
            workspace = Path(tmp)
            sprite = workspace / "assets/pixel-sprites/little-alchemy-1/air.png"
            sprite.parent.mkdir(parents=True)
            sprite.write_bytes(b"existing")

            result = self.run_script(
                "--catalog",
                "la1",
                "--dry-run",
                "--only",
                "air",
                env={"OPENAI_API_KEY": "sk-test"},
                cwd=workspace,
            )
            forced = self.run_script(
                "--catalog",
                "la1",
                "--dry-run",
                "--force",
                "--only",
                "air",
                env={"OPENAI_API_KEY": "sk-test"},
                cwd=workspace,
            )

        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("skip existing", result.stdout)
        self.assertEqual(forced.returncode, 0, forced.stderr)
        self.assertIn("would generate", forced.stdout)

    def test_dry_run_writes_manifest_with_resumable_entries(self):
        with tempfile.TemporaryDirectory() as tmp:
            workspace = Path(tmp)
            result = self.run_script(
                "--catalog",
                "ui",
                "--dry-run",
                "--limit",
                "2",
                env={"OPENAI_API_KEY": "sk-test"},
                cwd=workspace,
            )
            manifest_path = workspace / "assets/pixel-sprites/manifest.json"

            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertTrue(manifest_path.exists())
            manifest = json.loads(manifest_path.read_text())
            self.assertEqual(manifest["model"], "gpt-image-2")
            self.assertEqual(len(manifest["entries"]), 2)
            self.assertEqual(manifest["entries"][0]["status"], "dry-run")

    def test_generation_prompt_requests_small_terminal_readability_and_material_style(self):
        with tempfile.TemporaryDirectory() as tmp:
            workspace = Path(tmp)
            result = self.run_script(
                "--catalog",
                "la1",
                "--dry-run",
                "--only",
                "water",
                env={"OPENAI_API_KEY": "sk-test"},
                cwd=workspace,
            )
            manifest_path = workspace / "assets/pixel-sprites/manifest.json"

            self.assertEqual(result.returncode, 0, result.stderr)
            prompt = json.loads(manifest_path.read_text())["entries"][0]["prompt"]
            self.assertIn("readable when sampled to 8x8", prompt)
            self.assertIn("material-specific", prompt)

    def test_seeded_first_pass_sprites_have_readable_pixel_art_density(self):
        for relative in [
            "assets/pixel-sprites/little-alchemy-1/fire.png",
            "assets/pixel-sprites/little-alchemy-1/water.png",
            "assets/pixel-sprites/little-alchemy-1/earth.png",
            "assets/pixel-sprites/little-alchemy-1/air.png",
        ]:
            pixels = read_png_pixels(ROOT / relative)
            opaque = [pixel for pixel in pixels if pixel[3] > 0]
            colors = {pixel for pixel in opaque}

            self.assertGreaterEqual(len(opaque), 150, relative)
            self.assertGreaterEqual(len(colors), 5, relative)

    def test_seeded_early_game_atlas_covers_first_real_combination_chain(self):
        missing = []
        weak = []
        fingerprints = set()
        for slug in EARLY_GAME_SLUGS:
            path = ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}.png"
            if not path.exists():
                missing.append(slug)
                continue

            pixels = read_png_pixels(path)
            opaque = [pixel for pixel in pixels if pixel[3] > 0]
            colors = {pixel for pixel in opaque}
            if len(opaque) < 90 or len(colors) < 3:
                weak.append(slug)
            fingerprints.add(tuple(pixels))

        self.assertEqual(missing, [])
        self.assertEqual(weak, [])
        self.assertGreaterEqual(
            len(fingerprints),
            len(EARLY_GAME_SLUGS) - 2,
            "early-game sprites should be distinct authored objects, not one reusable placeholder",
        )

    def test_core_material_sprites_use_rich_highlight_mid_shadow_palettes(self):
        weak_material_slugs = ["sea", "metal", "pressure", "atmosphere", "mud", "sand", "wind"]
        too_flat = []
        for slug in weak_material_slugs:
            pixels = read_png_pixels(ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}.png")
            opaque = [pixel for pixel in pixels if pixel[3] > 0]
            colors = {pixel for pixel in opaque}
            if len(colors) < 5:
                too_flat.append((slug, len(colors)))

        self.assertEqual(
            too_flat,
            [],
            "core materials need enough palette depth to read like authored 16-bit sprites",
        )

    def test_all_catalog_elements_have_seeded_pixel_sprite_files(self):
        missing = []
        missing_frames = []
        for data_file, folder in [
            ("little_alchemy_wiki.json", "little-alchemy-1"),
            ("little_alchemy_2.json", "little-alchemy-2"),
        ]:
            data = json.loads((ROOT / "data" / data_file).read_text())
            for element in data["elements"]:
                slug = slugify(element.get("slug") or element["name"])
                base = ROOT / f"assets/pixel-sprites/{folder}/{slug}.png"
                if not base.exists():
                    missing.append(f"{folder}/{slug}.png")
                for frame in range(1, 4):
                    frame_path = ROOT / f"assets/pixel-sprites/{folder}/{slug}_idle_{frame}.png"
                    if not frame_path.exists():
                        missing_frames.append(f"{folder}/{slug}_idle_{frame}.png")

        self.assertEqual(missing, [])
        self.assertEqual(missing_frames, [])

    def test_all_seeded_catalog_sprites_are_visible_pixel_art(self):
        weak = []
        for data_file, folder in [
            ("little_alchemy_wiki.json", "little-alchemy-1"),
            ("little_alchemy_2.json", "little-alchemy-2"),
        ]:
            data = json.loads((ROOT / "data" / data_file).read_text())
            for element in data["elements"]:
                slug = slugify(element.get("slug") or element["name"])
                path = ROOT / f"assets/pixel-sprites/{folder}/{slug}.png"
                if not path.exists():
                    weak.append((folder, slug, "missing", 0, 0))
                    continue
                pixels = read_png_pixels(path)
                opaque = [pixel for pixel in pixels if pixel[3] > 0]
                colors = {pixel for pixel in opaque}
                if len(opaque) < 70 or len(colors) < 3:
                    weak.append((folder, slug, "weak", len(opaque), len(colors)))

        self.assertEqual(
            weak,
            [],
            "every catalog element needs a visible authored pixel sprite, not a blank or fallback-only icon",
        )


    def test_seeded_first_pass_sprites_have_subcell_pixel_detail(self):
        for relative in [
            "assets/pixel-sprites/little-alchemy-1/fire.png",
            "assets/pixel-sprites/little-alchemy-1/water.png",
            "assets/pixel-sprites/little-alchemy-1/air.png",
        ]:
            pixels = read_png_pixels(ROOT / relative)
            mixed_macro_cells = 0
            for macro_y in range(0, 32, 4):
                for macro_x in range(0, 32, 4):
                    opaque = 0
                    for y in range(macro_y, macro_y + 4):
                        for x in range(macro_x, macro_x + 4):
                            if pixels[y * 32 + x][3] > 0:
                                opaque += 1
                    if 0 < opaque < 16:
                        mixed_macro_cells += 1

            self.assertGreaterEqual(
                mixed_macro_cells,
                8,
                f"{relative} should not be built only from chunky 4x4 blocks",
            )

    def test_catalog_switch_icons_read_as_open_books(self):
        for slug in ["catalog-la1", "catalog-la2"]:
            pixels = read_png_pixels(ROOT / f"assets/pixel-sprites/ui/{slug}.png")
            gutter = 0
            left_page = 0
            right_page = 0

            for y in range(8, 25):
                if pixels[y * 32 + 15][3] == 0 or pixels[y * 32 + 16][3] == 0:
                    gutter += 1
                for x in range(7, 14):
                    if pixels[y * 32 + x][3] > 0:
                        left_page += 1
                for x in range(18, 25):
                    if pixels[y * 32 + x][3] > 0:
                        right_page += 1

            self.assertGreaterEqual(
                gutter,
                8,
                f"{slug} should have a visible spine/gutter, not a plain filled rectangle",
            )
            self.assertGreaterEqual(left_page, 45, f"{slug} needs a readable left page")
            self.assertGreaterEqual(right_page, 45, f"{slug} needs a readable right page")

    def test_air_sprite_uses_curled_wisps_instead_of_solid_bars(self):
        pixels = read_png_pixels(ROOT / "assets/pixel-sprites/little-alchemy-1/air.png")
        longest_runs = []
        row_starts = set()
        for y in range(32):
            xs = [x for x in range(32) if pixels[y * 32 + x][3] > 0]
            if xs:
                row_starts.add(min(xs))
                run_start = xs[0]
                previous = xs[0]
                for x in xs[1:]:
                    if x == previous + 1:
                        previous = x
                    else:
                        longest_runs.append(previous - run_start + 1)
                        run_start = previous = x
                longest_runs.append(previous - run_start + 1)

        self.assertLessEqual(
            max(longest_runs),
            16,
            "air should read as separated wind curls, not broad solid horizontal bars",
        )
        self.assertGreaterEqual(
            len(row_starts),
            5,
            "air wisps should have staggered curved starts instead of a flat stacked stripe",
        )

    def test_steam_sprite_reads_as_one_coherent_plume(self):
        pixels = read_png_pixels(ROOT / "assets/pixel-sprites/little-alchemy-1/steam.png")
        components = opaque_components(pixels)
        component_sizes = sorted((len(component) for component in components), reverse=True)

        self.assertLessEqual(
            len(components),
            5,
            f"steam should be a coherent rising plume, not scattered disconnected marks: {component_sizes}",
        )
        self.assertGreaterEqual(
            component_sizes[0],
            110,
            "steam needs a dominant readable body so the first crafted result is recognizable",
        )

    def test_steam_sprite_is_airy_not_a_heavy_based_container(self):
        pixels = read_png_pixels(ROOT / "assets/pixel-sprites/little-alchemy-1/steam.png")
        heavy_base_colors = {
            (83, 88, 108, 255),
            (148, 151, 166, 255),
        }

        heavy_base_pixels = sum(
            1
            for y in range(25, 31)
            for x in range(5, 28)
            if pixels[y * 32 + x] in heavy_base_colors
        )
        transparent_pockets = sum(
            1
            for y in range(9, 27)
            for x in range(9, 24)
            if pixels[y * 32 + x][3] == 0
        )

        self.assertLessEqual(
            heavy_base_pixels,
            12,
            "steam should be an airy plume, not a heavy-bottomed object or container",
        )
        self.assertGreaterEqual(
            transparent_pockets,
            80,
            "steam needs negative space inside the curl so it reads as vapor",
        )

    def test_bottle_sprite_reads_as_corked_liquid_container(self):
        pixels = read_png_pixels(ROOT / "assets/pixel-sprites/little-alchemy-1/bottle.png")
        cork_colors = {
            (122, 94, 61, 255),
            (232, 202, 142, 255),
            (255, 166, 49, 255),
        }
        water_colors = {
            (37, 63, 159, 255),
            (44, 124, 232, 255),
            (84, 199, 255, 255),
        }

        cork_pixels = sum(
            1
            for y in range(2, 7)
            for x in range(13, 20)
            if pixels[y * 32 + x] in cork_colors
        )
        liquid_pixels = sum(
            1
            for y in range(19, 27)
            for x in range(9, 24)
            if pixels[y * 32 + x] in water_colors
        )
        highlight_pixels = sum(
            1
            for y in range(6, 27)
            for x in range(10, 23)
            if pixels[y * 32 + x] == (244, 246, 255, 255)
        )

        self.assertGreaterEqual(
            cork_pixels,
            10,
            "bottle should have a readable cork/stopper so it is not just a blue blob",
        )
        self.assertGreaterEqual(
            liquid_pixels,
            70,
            "bottle should clearly show colored liquid inside the glass",
        )
        self.assertGreaterEqual(
            highlight_pixels,
            16,
            "bottle needs glass highlights to read as a container",
        )

    def test_vase_sprite_reads_as_ceramic_vessel_with_handles(self):
        pixels = read_png_pixels(ROOT / "assets/pixel-sprites/little-alchemy-1/vase.png")
        ceramic_colors = {
            (122, 94, 61, 255),
            (78, 58, 48, 255),
            (255, 166, 49, 255),
            (232, 202, 142, 255),
        }

        left_handle = sum(
            1
            for y in range(16, 25)
            for x in range(4, 8)
            if pixels[y * 32 + x] in ceramic_colors
        )
        right_handle = sum(
            1
            for y in range(16, 25)
            for x in range(24, 28)
            if pixels[y * 32 + x] in ceramic_colors
        )
        left_handle_hole = sum(
            1
            for y in range(17, 24)
            for x in range(8, 11)
            if pixels[y * 32 + x][3] == 0
        )
        right_handle_hole = sum(
            1
            for y in range(17, 24)
            for x in range(21, 24)
            if pixels[y * 32 + x][3] == 0
        )

        self.assertGreaterEqual(left_handle, 8, "vase needs a readable left handle")
        self.assertGreaterEqual(right_handle, 8, "vase needs a readable right handle")
        self.assertGreaterEqual(
            left_handle_hole,
            6,
            "vase handle should use negative space, not a solid side blob",
        )
        self.assertGreaterEqual(
            right_handle_hole,
            6,
            "vase handle should use negative space, not a solid side blob",
        )

    def test_jar_sprite_reads_as_lidded_labeled_contents(self):
        pixels = read_png_pixels(ROOT / "assets/pixel-sprites/little-alchemy-1/jar.png")
        dark_lid_pixels = sum(
            1
            for y in range(5, 10)
            for x in range(8, 24)
            if pixels[y * 32 + x] == (83, 88, 108, 255)
        )
        lid_ridges = sum(
            1
            for x in [9, 12, 15, 18, 21]
            for y in range(5, 10)
            if pixels[y * 32 + x] == (83, 88, 108, 255)
        )
        label_pixels = sum(
            1
            for y in range(21, 26)
            for x in range(11, 21)
            if pixels[y * 32 + x] == (232, 202, 142, 255)
        )
        preserve_pixels = sum(
            1
            for y in range(16, 22)
            for x in range(10, 23)
            if pixels[y * 32 + x] == (255, 166, 49, 255)
        )

        self.assertGreaterEqual(
            dark_lid_pixels,
            14,
            "jar needs a dark screw-top lid outline, not only a pale glass band",
        )
        self.assertGreaterEqual(
            lid_ridges,
            5,
            "jar lid should have small vertical ridges so it reads as a cap",
        )
        self.assertGreaterEqual(label_pixels, 18, "jar needs a readable paper label")
        self.assertGreaterEqual(
            preserve_pixels,
            24,
            "jar contents should read as colored preserves inside the glass",
        )

    def test_glass_sprite_reads_as_transparent_faceted_material(self):
        pixels = read_png_pixels(ROOT / "assets/pixel-sprites/little-alchemy-1/glass.png")
        interior_holes = sum(
            1
            for y in range(12, 23)
            for x in range(10, 23)
            if pixels[y * 32 + x][3] == 0
        )
        bright_highlights = sum(
            1
            for y in range(7, 26)
            for x in range(7, 26)
            if pixels[y * 32 + x] == (244, 246, 255, 255)
        )
        edge_pixels = sum(
            1
            for y in range(7, 27)
            for x in range(5, 28)
            if pixels[y * 32 + x] in {
                (37, 63, 159, 255),
                (44, 124, 232, 255),
            }
        )

        self.assertGreaterEqual(
            interior_holes,
            12,
            "glass should use transparent interior facets, not a filled blue crystal",
        )
        self.assertGreaterEqual(
            bright_highlights,
            18,
            "glass needs sharp white highlights to read as reflective material",
        )
        self.assertGreaterEqual(edge_pixels, 70, "glass needs a clear blue silhouette edge")

    def test_seeded_elements_include_multiple_idle_animation_frames(self):
        base = ROOT / "assets/pixel-sprites/little-alchemy-1/water.png"
        frame_one = ROOT / "assets/pixel-sprites/little-alchemy-1/water_idle_1.png"
        frame_two = ROOT / "assets/pixel-sprites/little-alchemy-1/water_idle_2.png"

        self.assertTrue(frame_one.exists())
        self.assertTrue(frame_two.exists())
        self.assertNotEqual(read_png_pixels(base), read_png_pixels(frame_one))
        self.assertNotEqual(read_png_pixels(frame_one), read_png_pixels(frame_two))

    def test_seeded_early_game_sprites_have_living_idle_frames(self):
        static = []
        unstable = []
        for slug in EARLY_GAME_SLUGS:
            base = ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}.png"
            base_pixels = read_png_pixels(base)
            base_bounds = opaque_bounds(base_pixels)
            frames = [
                read_png_pixels(ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}_idle_{frame}.png")
                for frame in range(1, 4)
            ]
            if all(frame == base_pixels for frame in frames):
                static.append(slug)
            if any(opaque_bounds(frame) != base_bounds for frame in frames):
                unstable.append(slug)

        self.assertEqual(static, [])
        self.assertEqual(unstable, [])

    def test_idle_frames_keep_sprite_anchor_stable(self):
        for slug in ["fire", "water", "air", "steam"]:
            base = ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}.png"
            frame = ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}_idle_1.png"

            self.assertEqual(opaque_bounds(read_png_pixels(base)), opaque_bounds(read_png_pixels(frame)))

    def test_idle_animation_changes_are_clustered_not_random_pixel_flicker(self):
        flickery = []
        slugs = sorted(
            set(
                EARLY_GAME_SLUGS
                + OBJECT_STYLE_SLUGS
                + ICONIC_REFERENCE_SLUGS
                + EXPANDED_OBJECT_SLUGS
                + REFERENCE_EXTRA_SLUGS
                + CRAFT_OBJECT_SLUGS
                + NATURE_COSMOS_SLUGS
                + COSMIC_ELECTRIC_SLUGS
                + NATURAL_FORCE_SLUGS
                + CONSTRUCTED_BOTANICAL_SLUGS
                + MIDGAME_DEVICE_WORLD_SLUGS
                + CIVILIZATION_TRANSPORT_SLUGS
                + MATERIAL_ICONIC_SLUGS
                + COMMON_OBJECT_SCENERY_SLUGS
                + FANTASY_CHARACTER_SLUGS
                + EARLY_MISSING_CHARACTER_OBJECT_SLUGS
                + LIGHT_BIRD_OBJECT_SLUGS
                + ACCESSORY_DEVICE_NATURE_SLUGS
                + ANIMAL_TECH_MONSTER_SLUGS
                + FOOD_GRAVE_MAGIC_SLUGS
                + SHORE_TOOL_SKY_SLUGS
                + SPACE_TIME_UNDEAD_WEATHER_SLUGS
                + ALIEN_WINTER_FOOD_CHARACTER_SLUGS
                + MATERIAL_NATURE_CREATURE_SLUGS
            )
        )

        for slug in slugs:
            base = read_png_pixels(ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}.png")
            for frame in range(1, 4):
                frame_pixels = read_png_pixels(
                    ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}_idle_{frame}.png"
                )
                diff_indexes = [
                    index
                    for index, (base_pixel, frame_pixel) in enumerate(zip(base, frame_pixels))
                    if base_pixel != frame_pixel
                ]
                if not diff_indexes:
                    flickery.append((slug, frame, "static"))
                    continue
                isolated = isolated_diff_count(diff_indexes)
                if isolated > max(3, len(diff_indexes) // 2):
                    flickery.append((slug, frame, isolated, len(diff_indexes)))

        self.assertEqual(
            flickery,
            [],
            "idle frames should animate as clustered glints, ripples, flame tongues, or device lights, not scattered debug blink pixels",
        )

    def test_authored_object_sprites_replace_generic_fallbacks(self):
        missing = []
        weak = []
        static = []
        fingerprints = set()
        for slug in OBJECT_STYLE_SLUGS:
            path = ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}.png"
            if not path.exists():
                missing.append(slug)
                continue

            pixels = read_png_pixels(path)
            opaque = [pixel for pixel in pixels if pixel[3] > 0]
            colors = {pixel for pixel in opaque}
            if len(opaque) < 75 or len(colors) < 4:
                weak.append((slug, len(opaque), len(colors)))
            fingerprints.add(tuple(pixels))

            base_bounds = opaque_bounds(pixels)
            frames = [
                read_png_pixels(ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}_idle_{frame}.png")
                for frame in range(1, 4)
            ]
            if all(frame == pixels for frame in frames):
                static.append(slug)
            for frame_pixels in frames:
                self.assertEqual(
                    opaque_bounds(frame_pixels),
                    base_bounds,
                    f"{slug} idle frames should glow in place, not wobble the object silhouette",
                )

        self.assertEqual(missing, [])
        self.assertEqual(weak, [])
        self.assertEqual(static, [])
        self.assertGreaterEqual(
            len(fingerprints),
            len(OBJECT_STYLE_SLUGS) - 1,
            "object sprites should be authored silhouettes, not one reused generic fallback",
        )

    def test_iconic_reference_board_sprites_are_authored(self):
        missing = []
        weak = []
        static = []
        fingerprints = set()
        for slug in ICONIC_REFERENCE_SLUGS:
            path = ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}.png"
            if not path.exists():
                missing.append(slug)
                continue

            pixels = read_png_pixels(path)
            opaque = [pixel for pixel in pixels if pixel[3] > 0]
            colors = {pixel for pixel in opaque}
            if len(opaque) < 70 or len(colors) < 4:
                weak.append((slug, len(opaque), len(colors)))
            fingerprints.add(tuple(pixels))

            base_bounds = opaque_bounds(pixels)
            frames = [
                read_png_pixels(ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}_idle_{frame}.png")
                for frame in range(1, 4)
            ]
            if all(frame == pixels for frame in frames):
                static.append(slug)
            for frame_pixels in frames:
                self.assertEqual(
                    opaque_bounds(frame_pixels),
                    base_bounds,
                    f"{slug} idle frames should preserve the sprite silhouette",
                )

        self.assertEqual(missing, [])
        self.assertEqual(weak, [])
        self.assertEqual(static, [])
        self.assertGreaterEqual(
            len(fingerprints),
            len(ICONIC_REFERENCE_SLUGS) - 1,
            "reference-board sprites should be distinct authored icons",
        )

    def test_expanded_object_sprites_replace_preview_fallbacks(self):
        missing = []
        weak = []
        static = []
        fingerprints = set()
        for slug in EXPANDED_OBJECT_SLUGS:
            path = ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}.png"
            if not path.exists():
                missing.append(slug)
                continue

            pixels = read_png_pixels(path)
            opaque = [pixel for pixel in pixels if pixel[3] > 0]
            colors = {pixel for pixel in opaque}
            if len(opaque) < 65 or len(colors) < 4:
                weak.append((slug, len(opaque), len(colors)))
            fingerprints.add(tuple(pixels))

            base_bounds = opaque_bounds(pixels)
            frames = [
                read_png_pixels(ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}_idle_{frame}.png")
                for frame in range(1, 4)
            ]
            if all(frame == pixels for frame in frames):
                static.append(slug)
            for frame_pixels in frames:
                self.assertEqual(
                    opaque_bounds(frame_pixels),
                    base_bounds,
                    f"{slug} idle frames should preserve the object silhouette",
                )

        self.assertEqual(missing, [])
        self.assertEqual(weak, [])
        self.assertEqual(static, [])
        self.assertGreaterEqual(
            len(fingerprints),
            len(EXPANDED_OBJECT_SLUGS) - 1,
            "expanded preview objects should be distinct authored icons, not generic fallback silhouettes",
        )

    def test_reference_extra_sprites_are_authored(self):
        missing = []
        weak = []
        static = []
        fingerprints = set()
        for slug in REFERENCE_EXTRA_SLUGS:
            path = ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}.png"
            if not path.exists():
                missing.append(slug)
                continue

            pixels = read_png_pixels(path)
            opaque = [pixel for pixel in pixels if pixel[3] > 0]
            colors = {pixel for pixel in opaque}
            if len(opaque) < 65 or len(colors) < 4:
                weak.append((slug, len(opaque), len(colors)))
            fingerprints.add(tuple(pixels))

            base_bounds = opaque_bounds(pixels)
            frames = [
                read_png_pixels(ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}_idle_{frame}.png")
                for frame in range(1, 4)
            ]
            if all(frame == pixels for frame in frames):
                static.append(slug)
            for frame_pixels in frames:
                self.assertEqual(
                    opaque_bounds(frame_pixels),
                    base_bounds,
                    f"{slug} idle frames should preserve the object silhouette",
                )

        self.assertEqual(missing, [])
        self.assertEqual(weak, [])
        self.assertEqual(static, [])
        self.assertGreaterEqual(
            len(fingerprints),
            len(REFERENCE_EXTRA_SLUGS) - 1,
            "reference extra sprites should be distinct authored icons",
        )

    def test_craft_object_sprites_are_authored(self):
        missing = []
        weak = []
        static = []
        fingerprints = set()
        for slug in CRAFT_OBJECT_SLUGS:
            path = ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}.png"
            if not path.exists():
                missing.append(slug)
                continue

            pixels = read_png_pixels(path)
            opaque = [pixel for pixel in pixels if pixel[3] > 0]
            colors = {pixel for pixel in opaque}
            if len(opaque) < 65 or len(colors) < 4:
                weak.append((slug, len(opaque), len(colors)))
            fingerprints.add(tuple(pixels))

            base_bounds = opaque_bounds(pixels)
            frames = [
                read_png_pixels(ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}_idle_{frame}.png")
                for frame in range(1, 4)
            ]
            if all(frame == pixels for frame in frames):
                static.append(slug)
            for frame_pixels in frames:
                self.assertEqual(
                    opaque_bounds(frame_pixels),
                    base_bounds,
                    f"{slug} idle frames should preserve the craft object silhouette",
                )

        self.assertEqual(missing, [])
        self.assertEqual(weak, [])
        self.assertEqual(static, [])
        self.assertGreaterEqual(
            len(fingerprints),
            len(CRAFT_OBJECT_SLUGS),
            "craft object sprites should all be distinct authored icons",
        )

    def test_nature_cosmos_sprites_are_authored(self):
        missing = []
        weak = []
        static = []
        fingerprints = set()
        for slug in NATURE_COSMOS_SLUGS:
            path = ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}.png"
            if not path.exists():
                missing.append(slug)
                continue

            pixels = read_png_pixels(path)
            opaque = [pixel for pixel in pixels if pixel[3] > 0]
            colors = {pixel for pixel in opaque}
            if len(opaque) < 60 or len(colors) < 4:
                weak.append((slug, len(opaque), len(colors)))
            fingerprints.add(tuple(pixels))

            base_bounds = opaque_bounds(pixels)
            frames = [
                read_png_pixels(ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}_idle_{frame}.png")
                for frame in range(1, 4)
            ]
            if all(frame == pixels for frame in frames):
                static.append(slug)
            for frame_pixels in frames:
                self.assertEqual(
                    opaque_bounds(frame_pixels),
                    base_bounds,
                    f"{slug} idle frames should preserve the nature/cosmos silhouette",
                )

        self.assertEqual(missing, [])
        self.assertEqual(weak, [])
        self.assertEqual(static, [])
        self.assertGreaterEqual(
            len(fingerprints),
            len(NATURE_COSMOS_SLUGS) - 1,
            "nature/cosmos sprites should be distinct authored icons",
        )

    def test_cosmic_electric_sprites_are_authored(self):
        missing = []
        weak = []
        static = []
        fingerprints = set()
        for slug in COSMIC_ELECTRIC_SLUGS:
            path = ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}.png"
            if not path.exists():
                missing.append(slug)
                continue

            pixels = read_png_pixels(path)
            opaque = [pixel for pixel in pixels if pixel[3] > 0]
            colors = {pixel for pixel in opaque}
            if len(opaque) < 60 or len(colors) < 4:
                weak.append((slug, len(opaque), len(colors)))
            fingerprints.add(tuple(pixels))

            base_bounds = opaque_bounds(pixels)
            frames = [
                read_png_pixels(ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}_idle_{frame}.png")
                for frame in range(1, 4)
            ]
            if all(frame == pixels for frame in frames):
                static.append(slug)
            for frame_pixels in frames:
                self.assertEqual(
                    opaque_bounds(frame_pixels),
                    base_bounds,
                    f"{slug} idle frames should preserve the cosmic/electric silhouette",
                )

        self.assertEqual(missing, [])
        self.assertEqual(weak, [])
        self.assertEqual(static, [])
        self.assertGreaterEqual(
            len(fingerprints),
            len(COSMIC_ELECTRIC_SLUGS) - 1,
            "cosmic/electric sprites should be distinct authored icons",
        )

    def test_natural_force_sprites_are_authored(self):
        missing = []
        weak = []
        static = []
        fingerprints = set()
        for slug in NATURAL_FORCE_SLUGS:
            path = ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}.png"
            if not path.exists():
                missing.append(slug)
                continue

            pixels = read_png_pixels(path)
            opaque = [pixel for pixel in pixels if pixel[3] > 0]
            colors = {pixel for pixel in opaque}
            if len(opaque) < 60 or len(colors) < 4:
                weak.append((slug, len(opaque), len(colors)))
            fingerprints.add(tuple(pixels))

            base_bounds = opaque_bounds(pixels)
            frames = [
                read_png_pixels(ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}_idle_{frame}.png")
                for frame in range(1, 4)
            ]
            if all(frame == pixels for frame in frames):
                static.append(slug)
            for frame_pixels in frames:
                self.assertEqual(
                    opaque_bounds(frame_pixels),
                    base_bounds,
                    f"{slug} idle frames should preserve the natural-force silhouette",
                )

        self.assertEqual(missing, [])
        self.assertEqual(weak, [])
        self.assertEqual(static, [])
        self.assertGreaterEqual(
            len(fingerprints),
            len(NATURAL_FORCE_SLUGS) - 1,
            "natural-force sprites should be distinct authored icons",
        )

    def test_constructed_botanical_sprites_are_authored(self):
        missing = []
        weak = []
        static = []
        fingerprints = set()
        for slug in CONSTRUCTED_BOTANICAL_SLUGS:
            path = ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}.png"
            if not path.exists():
                missing.append(slug)
                continue

            pixels = read_png_pixels(path)
            opaque = [pixel for pixel in pixels if pixel[3] > 0]
            colors = {pixel for pixel in opaque}
            if len(opaque) < 60 or len(colors) < 4:
                weak.append((slug, len(opaque), len(colors)))
            fingerprints.add(tuple(pixels))

            base_bounds = opaque_bounds(pixels)
            frames = [
                read_png_pixels(ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}_idle_{frame}.png")
                for frame in range(1, 4)
            ]
            if all(frame == pixels for frame in frames):
                static.append(slug)
            for frame_pixels in frames:
                self.assertEqual(
                    opaque_bounds(frame_pixels),
                    base_bounds,
                    f"{slug} idle frames should preserve the constructed/botanical silhouette",
                )

        self.assertEqual(missing, [])
        self.assertEqual(weak, [])
        self.assertEqual(static, [])
        self.assertGreaterEqual(
            len(fingerprints),
            len(CONSTRUCTED_BOTANICAL_SLUGS) - 1,
            "constructed/botanical sprites should be distinct authored icons",
        )

    def test_midgame_device_world_sprites_are_authored(self):
        missing = []
        weak = []
        static = []
        fingerprints = set()
        for slug in MIDGAME_DEVICE_WORLD_SLUGS:
            path = ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}.png"
            if not path.exists():
                missing.append(slug)
                continue

            pixels = read_png_pixels(path)
            opaque = [pixel for pixel in pixels if pixel[3] > 0]
            colors = {pixel for pixel in opaque}
            if len(opaque) < 60 or len(colors) < 4:
                weak.append((slug, len(opaque), len(colors)))
            fingerprints.add(tuple(pixels))

            base_bounds = opaque_bounds(pixels)
            frames = [
                read_png_pixels(ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}_idle_{frame}.png")
                for frame in range(1, 4)
            ]
            if all(frame == pixels for frame in frames):
                static.append(slug)
            for frame_pixels in frames:
                self.assertEqual(
                    opaque_bounds(frame_pixels),
                    base_bounds,
                    f"{slug} idle frames should preserve the device/world silhouette",
                )

        self.assertEqual(missing, [])
        self.assertEqual(weak, [])
        self.assertEqual(static, [])
        self.assertGreaterEqual(
            len(fingerprints),
            len(MIDGAME_DEVICE_WORLD_SLUGS) - 1,
            "mid-game device/world sprites should be distinct authored icons",
        )

    def test_civilization_transport_sprites_are_authored(self):
        missing = []
        weak = []
        static = []
        fingerprints = set()
        for slug in CIVILIZATION_TRANSPORT_SLUGS:
            path = ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}.png"
            if not path.exists():
                missing.append(slug)
                continue

            pixels = read_png_pixels(path)
            opaque = [pixel for pixel in pixels if pixel[3] > 0]
            colors = {pixel for pixel in opaque}
            if len(opaque) < 60 or len(colors) < 4:
                weak.append((slug, len(opaque), len(colors)))
            fingerprints.add(tuple(pixels))

            base_bounds = opaque_bounds(pixels)
            frames = [
                read_png_pixels(ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}_idle_{frame}.png")
                for frame in range(1, 4)
            ]
            if all(frame == pixels for frame in frames):
                static.append(slug)
            for frame_pixels in frames:
                self.assertEqual(
                    opaque_bounds(frame_pixels),
                    base_bounds,
                    f"{slug} idle frames should preserve the civilization/transport silhouette",
                )

        self.assertEqual(missing, [])
        self.assertEqual(weak, [])
        self.assertEqual(static, [])
        self.assertGreaterEqual(
            len(fingerprints),
            len(CIVILIZATION_TRANSPORT_SLUGS) - 1,
            "civilization/transport sprites should be distinct authored icons",
        )

    def test_material_iconic_sprites_are_authored(self):
        missing = []
        weak = []
        static = []
        fingerprints = set()
        for slug in MATERIAL_ICONIC_SLUGS:
            path = ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}.png"
            if not path.exists():
                missing.append(slug)
                continue

            pixels = read_png_pixels(path)
            opaque = [pixel for pixel in pixels if pixel[3] > 0]
            colors = {pixel for pixel in opaque}
            if len(opaque) < 60 or len(colors) < 4:
                weak.append((slug, len(opaque), len(colors)))
            fingerprints.add(tuple(pixels))

            base_bounds = opaque_bounds(pixels)
            frames = [
                read_png_pixels(ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}_idle_{frame}.png")
                for frame in range(1, 4)
            ]
            if all(frame == pixels for frame in frames):
                static.append(slug)
            for frame_pixels in frames:
                self.assertEqual(
                    opaque_bounds(frame_pixels),
                    base_bounds,
                    f"{slug} idle frames should preserve the material/iconic silhouette",
                )

        self.assertEqual(missing, [])
        self.assertEqual(weak, [])
        self.assertEqual(static, [])
        self.assertGreaterEqual(
            len(fingerprints),
            len(MATERIAL_ICONIC_SLUGS) - 1,
            "material/iconic sprites should be distinct authored icons",
        )

    def test_common_object_scenery_sprites_are_authored(self):
        missing = []
        weak = []
        static = []
        fingerprints = set()
        for slug in COMMON_OBJECT_SCENERY_SLUGS:
            path = ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}.png"
            if not path.exists():
                missing.append(slug)
                continue

            pixels = read_png_pixels(path)
            opaque = [pixel for pixel in pixels if pixel[3] > 0]
            colors = {pixel for pixel in opaque}
            if len(opaque) < 60 or len(colors) < 4:
                weak.append((slug, len(opaque), len(colors)))
            fingerprints.add(tuple(pixels))

            base_bounds = opaque_bounds(pixels)
            frames = [
                read_png_pixels(ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}_idle_{frame}.png")
                for frame in range(1, 4)
            ]
            if all(frame == pixels for frame in frames):
                static.append(slug)
            for frame_pixels in frames:
                self.assertEqual(
                    opaque_bounds(frame_pixels),
                    base_bounds,
                    f"{slug} idle frames should preserve the common object/scenery silhouette",
                )

        self.assertEqual(missing, [])
        self.assertEqual(weak, [])
        self.assertEqual(static, [])
        self.assertGreaterEqual(
            len(fingerprints),
            len(COMMON_OBJECT_SCENERY_SLUGS) - 1,
            "common object/scenery sprites should be distinct authored icons",
        )

    def test_fantasy_character_sprites_are_authored(self):
        missing = []
        weak = []
        static = []
        fingerprints = set()
        for slug in FANTASY_CHARACTER_SLUGS:
            path = ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}.png"
            if not path.exists():
                missing.append(slug)
                continue

            pixels = read_png_pixels(path)
            opaque = [pixel for pixel in pixels if pixel[3] > 0]
            colors = {pixel for pixel in opaque}
            if len(opaque) < 60 or len(colors) < 4:
                weak.append((slug, len(opaque), len(colors)))
            fingerprints.add(tuple(pixels))

            base_bounds = opaque_bounds(pixels)
            frames = [
                read_png_pixels(ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}_idle_{frame}.png")
                for frame in range(1, 4)
            ]
            if all(frame == pixels for frame in frames):
                static.append(slug)
            for frame_pixels in frames:
                self.assertEqual(
                    opaque_bounds(frame_pixels),
                    base_bounds,
                    f"{slug} idle frames should preserve the fantasy/character silhouette",
                )

        self.assertEqual(missing, [])
        self.assertEqual(weak, [])
        self.assertEqual(static, [])
        self.assertGreaterEqual(
            len(fingerprints),
            len(FANTASY_CHARACTER_SLUGS) - 1,
            "fantasy/character sprites should be distinct authored icons",
        )

    def test_early_missing_character_object_sprites_are_authored(self):
        missing = []
        weak = []
        static = []
        fingerprints = set()
        for slug in EARLY_MISSING_CHARACTER_OBJECT_SLUGS:
            path = ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}.png"
            if not path.exists():
                missing.append(slug)
                continue

            pixels = read_png_pixels(path)
            opaque = [pixel for pixel in pixels if pixel[3] > 0]
            colors = {pixel for pixel in opaque}
            if len(opaque) < 60 or len(colors) < 4:
                weak.append((slug, len(opaque), len(colors)))
            fingerprints.add(tuple(pixels))

            base_bounds = opaque_bounds(pixels)
            frames = [
                read_png_pixels(ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}_idle_{frame}.png")
                for frame in range(1, 4)
            ]
            if all(frame == pixels for frame in frames):
                static.append(slug)
            for frame_pixels in frames:
                self.assertEqual(
                    opaque_bounds(frame_pixels),
                    base_bounds,
                    f"{slug} idle frames should preserve the authored silhouette",
                )

        self.assertEqual(missing, [])
        self.assertEqual(weak, [])
        self.assertEqual(static, [])
        self.assertGreaterEqual(
            len(fingerprints),
            len(EARLY_MISSING_CHARACTER_OBJECT_SLUGS) - 1,
            "early missing character/object sprites should be distinct authored icons",
        )

    def test_light_bird_object_sprites_are_authored(self):
        missing = []
        weak = []
        static = []
        fingerprints = set()
        for slug in LIGHT_BIRD_OBJECT_SLUGS:
            path = ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}.png"
            if not path.exists():
                missing.append(slug)
                continue

            pixels = read_png_pixels(path)
            opaque = [pixel for pixel in pixels if pixel[3] > 0]
            colors = {pixel for pixel in opaque}
            if len(opaque) < 60 or len(colors) < 4:
                weak.append((slug, len(opaque), len(colors)))
            fingerprints.add(tuple(pixels))

            base_bounds = opaque_bounds(pixels)
            frames = [
                read_png_pixels(ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}_idle_{frame}.png")
                for frame in range(1, 4)
            ]
            if all(frame == pixels for frame in frames):
                static.append(slug)
            for frame_pixels in frames:
                self.assertEqual(
                    opaque_bounds(frame_pixels),
                    base_bounds,
                    f"{slug} idle frames should preserve the light/bird/object silhouette",
                )

        self.assertEqual(missing, [])
        self.assertEqual(weak, [])
        self.assertEqual(static, [])
        self.assertGreaterEqual(
            len(fingerprints),
            len(LIGHT_BIRD_OBJECT_SLUGS) - 1,
            "light/bird/object sprites should be distinct authored icons",
        )

    def test_accessory_device_nature_sprites_are_authored(self):
        missing = []
        weak = []
        static = []
        fingerprints = set()
        for slug in ACCESSORY_DEVICE_NATURE_SLUGS:
            path = ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}.png"
            if not path.exists():
                missing.append(slug)
                continue

            pixels = read_png_pixels(path)
            opaque = [pixel for pixel in pixels if pixel[3] > 0]
            colors = {pixel for pixel in opaque}
            if len(opaque) < 60 or len(colors) < 4:
                weak.append((slug, len(opaque), len(colors)))
            fingerprints.add(tuple(pixels))

            base_bounds = opaque_bounds(pixels)
            frames = [
                read_png_pixels(ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}_idle_{frame}.png")
                for frame in range(1, 4)
            ]
            if all(frame == pixels for frame in frames):
                static.append(slug)
            for frame_pixels in frames:
                self.assertEqual(
                    opaque_bounds(frame_pixels),
                    base_bounds,
                    f"{slug} idle frames should preserve the accessory/device/nature silhouette",
                )

        self.assertEqual(missing, [])
        self.assertEqual(weak, [])
        self.assertEqual(static, [])
        self.assertGreaterEqual(
            len(fingerprints),
            len(ACCESSORY_DEVICE_NATURE_SLUGS) - 1,
            "accessory/device/nature sprites should be distinct authored icons",
        )

    def test_animal_tech_monster_sprites_are_authored(self):
        missing = []
        weak = []
        static = []
        fingerprints = set()
        for slug in ANIMAL_TECH_MONSTER_SLUGS:
            path = ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}.png"
            if not path.exists():
                missing.append(slug)
                continue

            pixels = read_png_pixels(path)
            opaque = [pixel for pixel in pixels if pixel[3] > 0]
            colors = {pixel for pixel in opaque}
            if len(opaque) < 60 or len(colors) < 4:
                weak.append((slug, len(opaque), len(colors)))
            fingerprints.add(tuple(pixels))

            base_bounds = opaque_bounds(pixels)
            frames = [
                read_png_pixels(ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}_idle_{frame}.png")
                for frame in range(1, 4)
            ]
            if all(frame == pixels for frame in frames):
                static.append(slug)
            for frame_pixels in frames:
                self.assertEqual(
                    opaque_bounds(frame_pixels),
                    base_bounds,
                    f"{slug} idle frames should preserve the animal/tech/monster silhouette",
                )

        self.assertEqual(missing, [])
        self.assertEqual(weak, [])
        self.assertEqual(static, [])
        self.assertGreaterEqual(
            len(fingerprints),
            len(ANIMAL_TECH_MONSTER_SLUGS) - 1,
            "animal/tech/monster sprites should be distinct authored icons",
        )

    def test_food_grave_magic_sprites_are_authored(self):
        missing = []
        weak = []
        static = []
        fingerprints = set()
        for slug in FOOD_GRAVE_MAGIC_SLUGS:
            path = ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}.png"
            if not path.exists():
                missing.append(slug)
                continue

            pixels = read_png_pixels(path)
            opaque = [pixel for pixel in pixels if pixel[3] > 0]
            colors = {pixel for pixel in opaque}
            if len(opaque) < 60 or len(colors) < 4:
                weak.append((slug, len(opaque), len(colors)))
            fingerprints.add(tuple(pixels))

            base_bounds = opaque_bounds(pixels)
            frames = [
                read_png_pixels(ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}_idle_{frame}.png")
                for frame in range(1, 4)
            ]
            if all(frame == pixels for frame in frames):
                static.append(slug)
            for frame_pixels in frames:
                self.assertEqual(
                    opaque_bounds(frame_pixels),
                    base_bounds,
                    f"{slug} idle frames should preserve the food/grave/magic silhouette",
                )

        self.assertEqual(missing, [])
        self.assertEqual(weak, [])
        self.assertEqual(static, [])
        self.assertGreaterEqual(
            len(fingerprints),
            len(FOOD_GRAVE_MAGIC_SLUGS) - 1,
            "food/grave/magic sprites should be distinct authored icons",
        )

    def test_shore_tool_sky_sprites_are_authored(self):
        missing = []
        weak = []
        static = []
        fingerprints = set()
        for slug in SHORE_TOOL_SKY_SLUGS:
            path = ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}.png"
            if not path.exists():
                missing.append(slug)
                continue

            pixels = read_png_pixels(path)
            opaque = [pixel for pixel in pixels if pixel[3] > 0]
            colors = {pixel for pixel in opaque}
            if len(opaque) < 60 or len(colors) < 4:
                weak.append((slug, len(opaque), len(colors)))
            fingerprints.add(tuple(pixels))

            base_bounds = opaque_bounds(pixels)
            frames = [
                read_png_pixels(ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}_idle_{frame}.png")
                for frame in range(1, 4)
            ]
            if all(frame == pixels for frame in frames):
                static.append(slug)
            for frame_pixels in frames:
                self.assertEqual(
                    opaque_bounds(frame_pixels),
                    base_bounds,
                    f"{slug} idle frames should preserve the shore/tool/sky silhouette",
                )

        self.assertEqual(missing, [])
        self.assertEqual(weak, [])
        self.assertEqual(static, [])
        self.assertGreaterEqual(
            len(fingerprints),
            len(SHORE_TOOL_SKY_SLUGS) - 1,
            "shore/tool/sky sprites should be distinct authored icons",
        )

    def test_space_time_undead_weather_sprites_are_authored(self):
        missing = []
        weak = []
        static = []
        fingerprints = set()
        for slug in SPACE_TIME_UNDEAD_WEATHER_SLUGS:
            path = ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}.png"
            if not path.exists():
                missing.append(slug)
                continue

            pixels = read_png_pixels(path)
            opaque = [pixel for pixel in pixels if pixel[3] > 0]
            colors = {pixel for pixel in opaque}
            if len(opaque) < 60 or len(colors) < 4:
                weak.append((slug, len(opaque), len(colors)))
            fingerprints.add(tuple(pixels))

            base_bounds = opaque_bounds(pixels)
            frames = [
                read_png_pixels(ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}_idle_{frame}.png")
                for frame in range(1, 4)
            ]
            if all(frame == pixels for frame in frames):
                static.append(slug)
            for frame_pixels in frames:
                self.assertEqual(
                    opaque_bounds(frame_pixels),
                    base_bounds,
                    f"{slug} idle frames should preserve the space/time/undead/weather silhouette",
                )

        self.assertEqual(missing, [])
        self.assertEqual(weak, [])
        self.assertEqual(static, [])
        self.assertGreaterEqual(
            len(fingerprints),
            len(SPACE_TIME_UNDEAD_WEATHER_SLUGS) - 1,
            "space/time/undead/weather sprites should be distinct authored icons",
        )

    def test_alien_winter_food_character_sprites_are_authored(self):
        missing = []
        weak = []
        static = []
        fingerprints = set()
        for slug in ALIEN_WINTER_FOOD_CHARACTER_SLUGS:
            path = ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}.png"
            if not path.exists():
                missing.append(slug)
                continue

            pixels = read_png_pixels(path)
            opaque = [pixel for pixel in pixels if pixel[3] > 0]
            colors = {pixel for pixel in opaque}
            if len(opaque) < 60 or len(colors) < 4:
                weak.append((slug, len(opaque), len(colors)))
            fingerprints.add(tuple(pixels))

            base_bounds = opaque_bounds(pixels)
            frames = [
                read_png_pixels(ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}_idle_{frame}.png")
                for frame in range(1, 4)
            ]
            if all(frame == pixels for frame in frames):
                static.append(slug)
            for frame_pixels in frames:
                self.assertEqual(
                    opaque_bounds(frame_pixels),
                    base_bounds,
                    f"{slug} idle frames should preserve the alien/winter/food/character silhouette",
                )

        self.assertEqual(missing, [])
        self.assertEqual(weak, [])
        self.assertEqual(static, [])
        self.assertGreaterEqual(
            len(fingerprints),
            len(ALIEN_WINTER_FOOD_CHARACTER_SLUGS) - 1,
            "alien/winter/food/character sprites should be distinct authored icons",
        )

    def test_material_nature_creature_sprites_are_authored(self):
        missing = []
        weak = []
        static = []
        fingerprints = set()
        for slug in MATERIAL_NATURE_CREATURE_SLUGS:
            path = ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}.png"
            if not path.exists():
                missing.append(slug)
                continue

            pixels = read_png_pixels(path)
            opaque = [pixel for pixel in pixels if pixel[3] > 0]
            colors = {pixel for pixel in opaque}
            if len(opaque) < 60 or len(colors) < 4:
                weak.append((slug, len(opaque), len(colors)))
            fingerprints.add(tuple(pixels))

            base_bounds = opaque_bounds(pixels)
            frames = [
                read_png_pixels(ROOT / f"assets/pixel-sprites/little-alchemy-1/{slug}_idle_{frame}.png")
                for frame in range(1, 4)
            ]
            if all(frame == pixels for frame in frames):
                static.append(slug)
            for frame_pixels in frames:
                self.assertEqual(
                    opaque_bounds(frame_pixels),
                    base_bounds,
                    f"{slug} idle frames should preserve the material/nature/creature silhouette",
                )

        self.assertEqual(missing, [])
        self.assertEqual(weak, [])
        self.assertEqual(static, [])
        self.assertGreaterEqual(
            len(fingerprints),
            len(MATERIAL_NATURE_CREATURE_SLUGS) - 1,
            "material/nature/creature sprites should be distinct authored icons",
        )

def slugify(value):
    out = []
    last_was_dash = False
    for char in value.strip().lower():
        if char.isascii() and char.isalnum():
            out.append(char)
            last_was_dash = False
        elif not last_was_dash:
            out.append("-")
            last_was_dash = True
    slug = "".join(out).strip("-")
    return slug or "element"


def read_png_pixels(path):
    data = path.read_bytes()
    if data[:8] != b"\x89PNG\r\n\x1a\n":
        raise AssertionError(f"{path} is not a PNG")

    offset = 8
    width = height = None
    idat = bytearray()
    while offset < len(data):
        length = struct.unpack(">I", data[offset : offset + 4])[0]
        kind = data[offset + 4 : offset + 8]
        payload = data[offset + 8 : offset + 8 + length]
        offset += 12 + length
        if kind == b"IHDR":
            width, height, bit_depth, color_type, *_ = struct.unpack(">IIBBBBB", payload)
            if bit_depth != 8 or color_type != 6:
                raise AssertionError(f"{path} must be 8-bit RGBA")
        elif kind == b"IDAT":
            idat.extend(payload)
        elif kind == b"IEND":
            break

    if width is None or height is None:
        raise AssertionError(f"{path} has no IHDR")

    raw = zlib.decompress(bytes(idat))
    stride = width * 4
    pixels = []
    cursor = 0
    previous = [0] * stride
    for _ in range(height):
        filter_type = raw[cursor]
        cursor += 1
        row = list(raw[cursor : cursor + stride])
        cursor += stride
        if filter_type != 0:
            raise AssertionError(f"{path} uses unsupported PNG filter {filter_type}")
        previous = row
        pixels.extend(tuple(row[index : index + 4]) for index in range(0, stride, 4))
    return pixels


def opaque_bounds(pixels):
    coordinates = [
        (index % 32, index // 32)
        for index, pixel in enumerate(pixels)
        if pixel[3] > 0
    ]
    if not coordinates:
        return None
    xs = [x for x, _ in coordinates]
    ys = [y for _, y in coordinates]
    return min(xs), min(ys), max(xs), max(ys)


def opaque_components(pixels):
    points = {
        (index % 32, index // 32)
        for index, pixel in enumerate(pixels)
        if pixel[3] > 0
    }
    seen = set()
    components = []

    for point in points:
        if point in seen:
            continue
        stack = [point]
        seen.add(point)
        component = []
        while stack:
            x, y = stack.pop()
            component.append((x, y))
            for neighbor in [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)]:
                if neighbor in points and neighbor not in seen:
                    seen.add(neighbor)
                    stack.append(neighbor)
        components.append(component)

    return components


def isolated_diff_count(diff_indexes):
    diff_set = set(diff_indexes)
    isolated = 0
    for index in diff_indexes:
        x = index % 32
        y = index // 32
        for dx, dy in [(1, 0), (-1, 0), (0, 1), (0, -1)]:
            nx = x + dx
            ny = y + dy
            if 0 <= nx < 32 and 0 <= ny < 32 and ny * 32 + nx in diff_set:
                break
        else:
            isolated += 1
    return isolated


if __name__ == "__main__":
    unittest.main()
