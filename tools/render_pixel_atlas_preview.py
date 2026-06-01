#!/usr/bin/env python3
"""Render a browser-checkable preview sheet for the seeded pixel atlas."""

from __future__ import annotations

from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "output" / "pixel-atlas-preview.html"
SLUGS = [
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
    "lizard",
    "bread",
    "fishing-rod",
    "crystal-ball",
    "butterfly",
    "flying-fish",
    "shovel",
    "axe",
    "clay",
    "pottery",
    "knife",
    "seaweed",
    "hay",
    "bacteria",
    "wool",
    "cow",
    "horse",
    "rainbow",
    "star",
    "lightning",
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


def main() -> int:
    OUT.parent.mkdir(parents=True, exist_ok=True)
    cards = "\n".join(card(slug) for slug in SLUGS)
    OUT.write_text(
        f"""<!doctype html>
<html>
<head>
<meta charset="utf-8">
<style>
body {{
  margin: 0;
  min-height: 100vh;
  background: #241d35;
  color: #eee9ff;
  font-family: "JetBrains Mono", "Menlo", monospace;
}}
main {{
  width: 760px;
  margin: 0 auto;
  padding: 30px 24px 40px;
}}
h1 {{
  margin: 0 0 18px;
  color: #f2d277;
  font-size: 18px;
  letter-spacing: 0;
}}
.grid {{
  display: grid;
  grid-template-columns: repeat(5, 1fr);
  gap: 18px 20px;
}}
.card {{
  min-height: 132px;
  display: grid;
  place-items: center;
  align-content: center;
  gap: 7px;
}}
.hero {{
  width: 64px;
  height: 64px;
  image-rendering: pixelated;
}}
.motion {{
  display: grid;
  grid-template-columns: repeat(4, 18px);
  gap: 4px;
  align-items: center;
  opacity: 0.86;
}}
.motion img {{
  width: 18px;
  height: 18px;
  image-rendering: pixelated;
}}
.label {{
  color: #ded7f7;
  font-size: 18px;
  line-height: 1;
  text-align: center;
}}
</style>
</head>
<body>
<main>
<h1>seeded pixel atlas: materials, life, objects, containers</h1>
<div class="grid">
{cards}
</div>
</main>
</body>
</html>
""",
        encoding="utf-8",
    )
    print(OUT.relative_to(ROOT))
    return 0


def card(slug: str) -> str:
    label = slug.replace("-", " ")
    path = f"../assets/pixel-sprites/little-alchemy-1/{slug}.png"
    frames = [path] + [
        f"../assets/pixel-sprites/little-alchemy-1/{slug}_idle_{frame}.png"
        for frame in range(1, 4)
    ]
    strip = "\n    ".join(
        f'<img src="{frame_path}" alt="{label} frame {index}">'
        for index, frame_path in enumerate(frames)
    )
    return f"""<div class="card">
  <img class="hero" src="{path}" alt="{label}">
  <div class="motion">
    {strip}
  </div>
  <div class="label">{label}</div>
</div>"""


if __name__ == "__main__":
    raise SystemExit(main())
