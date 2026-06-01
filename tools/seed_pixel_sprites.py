#!/usr/bin/env python3
"""Seed readable local 32x32 pixel sprites for the first playable atlas pass."""

from __future__ import annotations

import io
import json
import math
import shutil
import struct
import subprocess
import time
import zlib
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "assets" / "pixel-sprites"

TRANSPARENT = (0, 0, 0, 0)
COLORS = {
    "ink": (34, 25, 44, 255),
    "white": (244, 246, 255, 255),
    "ice": (171, 211, 255, 255),
    "sky": (84, 199, 255, 255),
    "blue": (44, 124, 232, 255),
    "deep_blue": (37, 63, 159, 255),
    "yellow": (255, 229, 77, 255),
    "gold": (255, 166, 49, 255),
    "orange": (245, 88, 30, 255),
    "red": (191, 47, 38, 255),
    "lime": (130, 226, 64, 255),
    "green": (39, 174, 57, 255),
    "dark_green": (23, 102, 42, 255),
    "dirt": (122, 94, 61, 255),
    "dark_dirt": (78, 58, 48, 255),
    "stone": (148, 151, 166, 255),
    "dark_stone": (83, 88, 108, 255),
    "violet": (136, 112, 224, 255),
    "panel": (79, 92, 90, 255),
    "paper": (232, 202, 142, 255),
}

ICON_COLOR_KEYS = {
    "W": "white",
    "I": "ice",
    "S": "sky",
    "B": "blue",
    "N": "deep_blue",
    "Y": "yellow",
    "G": "gold",
    "O": "orange",
    "R": "red",
    "L": "lime",
    "E": "green",
    "T": "dark_green",
    "D": "dirt",
    "K": "dark_dirt",
    "P": "paper",
    "A": "stone",
    "Q": "dark_stone",
    "V": "violet",
}

ICON_GRIDS = {
    "fire": [
        "................",
        ".......R........",
        "......ROR.......",
        ".....ROOR.......",
        ".....ROOOR......",
        "....ROYOOR......",
        "...ROOYYOR......",
        "...ROYYYYR......",
        "..ROOYYYYOR.....",
        "..ROOYYGGOR.....",
        "...RRGGGOR......",
        "....RRGOR.......",
        ".....KKK........",
        "....KKKKK.......",
        "................",
        "................",
    ],
    "water": [
        "................",
        ".......I........",
        "......IWI.......",
        ".....IWWI.......",
        ".....IBSI.......",
        "....IBSSBI......",
        "...INBSSBNI.....",
        "...INBSSBNI.....",
        "..INBBSSBBNI....",
        "..NBBSSISBBN....",
        "..NBSSSISBBN....",
        "...NBBSSBBN.....",
        "....NNBBNN......",
        "......SS........",
        "................",
        "................",
    ],
    "earth": [
        "................",
        ".......E........",
        "......ELE.......",
        ".......T........",
        "......TT........",
        ".....DDDDD......",
        "...DDDPDDDD.....",
        "..DDPPPDDDDD....",
        ".DDPPPPDKDDD....",
        ".DDDPDDDKKDD....",
        "..KKDDKKKKK.....",
        "...KKKKKKK......",
        "....K.K.K.......",
        "................",
        "................",
        "................",
    ],
    "air": [
        "................",
        ".....IIISSS.....",
        "...IWWW..SSS....",
        "..IWW.....SS....",
        "...II......S....",
        ".........VV.....",
        "......VVVV......",
        "...SSS....III...",
        ".SSS......IWI...",
        "..SSS...IIIW....",
        "....SSSII.......",
        ".N..............",
        "..N.............",
        "................",
        "................",
        "................",
    ],
    "dust": [
        "................",
        "...G.....P......",
        ".....Y..D.......",
        "..P....G...A....",
        "....GDDDY.......",
        "...PDDGDDD......",
        ".....DDYD..P....",
        "..A....D........",
        "......Y....G....",
        "....P.....A.....",
        "................",
        "................",
        "................",
        "................",
        "................",
        "................",
    ],
    "steam": [
        ".....W...W......",
        "......I...W.....",
        ".....W....I.....",
        "....W...I.......",
        ".....I...W......",
        "......W..I......",
        ".....I...W......",
        ".......I........",
        "......I.........",
        ".....SAS........",
        "....SAQAS.......",
        "...SAAQQAS......",
        "................",
        "................",
        "................",
        "................",
    ],
}

SEED_ELEMENT_SLUGS = [
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
    entries = []
    for folder, catalog, data_file, source_ext in [
        ("little-alchemy-1", "la1", "little_alchemy_wiki.json", "png"),
        ("little-alchemy-2", "la2", "little_alchemy_2.json", "svg"),
    ]:
        for slug, label in catalog_seed_jobs(data_file):
            path = OUT / folder / f"{slug}.png"
            source_icon = Path("assets") / "icons" / folder / f"{slug}.{source_ext}"
            pixels = source_icon_pixels(ROOT / source_icon) if slug not in SEED_ELEMENT_SLUGS else None
            if pixels is None:
                pixels = make_sprite(slug)
            write_png(path, pixels)
            entries.append(
                manifest_entry(
                    catalog,
                    slug,
                    label,
                    path,
                    source_icon,
                )
            )
            for frame in range(1, 4):
                frame_path = OUT / folder / f"{slug}_idle_{frame}.png"
                write_png(frame_path, animate_idle(slug, pixels, frame))
                entries.append(
                    manifest_entry(
                        catalog,
                        f"{slug}_idle_{frame}",
                        f"{label} idle {frame}",
                        frame_path,
                        source_icon,
                    )
                )

    for slug, label in [
        ("catalog-la1", "Little Alchemy 1"),
        ("catalog-la2", "Little Alchemy 2"),
        ("combine", "Mix"),
        ("clear", "Clear"),
        ("reset", "Reset"),
        ("hint", "Hint"),
    ]:
        path = OUT / "ui" / f"{slug}.png"
        write_png(path, make_sprite(slug))
        entries.append(manifest_entry("ui", slug, label, path, None))

    manifest = {
        "prompt_version": "pixel-atlas-v1",
        "model": "gpt-image-2",
        "entries": entries,
    }
    (OUT / "manifest.json").write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n")
    return 0

def catalog_seed_jobs(data_file):
    data = json.loads((ROOT / "data" / data_file).read_text())
    seen = set()
    for element in data["elements"]:
        slug = slugify(element.get("slug") or element["name"])
        if slug in seen:
            continue
        seen.add(slug)
        yield slug, element["name"]


def slugify(value):
    out = []
    pending_dash = False
    for ch in value.strip().lower():
        if ch.isascii() and ch.isalnum():
            if pending_dash and out:
                out.append("-")
            out.append(ch)
            pending_dash = False
        else:
            pending_dash = True
    return "".join(out) or "element"

def source_icon_pixels(path):
    if not path.exists():
        return None
    try:
        from PIL import Image
    except ImportError:
        return None

    try:
        if path.suffix.lower() == ".svg":
            converter = shutil.which("rsvg-convert")
            if converter is None:
                return None
            rendered = subprocess.run(
                [converter, "-w", "128", "-h", "128", "-f", "png", str(path)],
                check=True,
                capture_output=True,
            ).stdout
            image = Image.open(io.BytesIO(rendered)).convert("RGBA")
        else:
            image = Image.open(path).convert("RGBA")
    except Exception:
        return None

    bbox = icon_content_bbox(image)
    if bbox is None:
        return None

    image = image.crop(bbox)
    image.thumbnail((30, 30), Image.Resampling.LANCZOS)
    sheet = Image.new("RGBA", (32, 32), TRANSPARENT)
    sheet.alpha_composite(image, ((32 - image.width) // 2, (32 - image.height) // 2))

    pixels = canvas()
    for y in range(32):
        for x in range(32):
            pixel = sheet.getpixel((x, y))
            if pixel[3] < 32 or is_background_black(pixel):
                continue
            pixels[y][x] = nearest_palette_color(pixel)
    return pixels


def icon_content_bbox(image):
    coordinates = []
    for y in range(image.height):
        for x in range(image.width):
            pixel = image.getpixel((x, y))
            if pixel[3] >= 32 and not is_background_black(pixel):
                coordinates.append((x, y))
    if not coordinates:
        return None
    xs = [x for x, _ in coordinates]
    ys = [y for _, y in coordinates]
    return min(xs), min(ys), max(xs) + 1, max(ys) + 1


def is_background_black(pixel):
    red, green, blue, alpha = pixel
    return alpha > 0 and red <= 12 and green <= 12 and blue <= 12


def nearest_palette_color(pixel):
    red, green, blue, alpha = pixel
    best = min(
        COLORS.values(),
        key=lambda color: (red - color[0]) ** 2 + (green - color[1]) ** 2 + (blue - color[2]) ** 2,
    )
    return (best[0], best[1], best[2], min(255, max(96, alpha)))




def canvas():
    return [[TRANSPARENT for _ in range(32)] for _ in range(32)]


def copy_pixels(pixels):
    return [row[:] for row in pixels]


def put(pixels, x, y, color):
    if 0 <= x < 32 and 0 <= y < 32:
        pixels[y][x] = COLORS[color]


def erase(pixels, x, y):
    if 0 <= x < 32 and 0 <= y < 32:
        pixels[y][x] = TRANSPARENT


def rect(pixels, x0, y0, x1, y1, color):
    for y in range(y0, y1):
        for x in range(x0, x1):
            put(pixels, x, y, color)


def circle(pixels, cx, cy, r, color):
    for y in range(cy - r, cy + r + 1):
        for x in range(cx - r, cx + r + 1):
            if (x - cx) ** 2 + (y - cy) ** 2 <= r * r:
                put(pixels, x, y, color)


def ellipse(pixels, cx, cy, rx, ry, color):
    for y in range(cy - ry, cy + ry + 1):
        for x in range(cx - rx, cx + rx + 1):
            if ((x - cx) ** 2) * (ry**2) + ((y - cy) ** 2) * (rx**2) <= (rx**2) * (ry**2):
                put(pixels, x, y, color)


def erase_ellipse(pixels, cx, cy, rx, ry):
    for y in range(cy - ry, cy + ry + 1):
        for x in range(cx - rx, cx + rx + 1):
            if ((x - cx) ** 2) * (ry**2) + ((y - cy) ** 2) * (rx**2) <= (rx**2) * (ry**2):
                erase(pixels, x, y)


def line(pixels, x0, y0, x1, y1, color, width=1):
    steps = max(abs(x1 - x0), abs(y1 - y0), 1)
    for i in range(steps + 1):
        x = round(x0 + (x1 - x0) * i / steps)
        y = round(y0 + (y1 - y0) * i / steps)
        rect(pixels, x - width // 2, y - width // 2, x + width // 2 + 1, y + width // 2 + 1, color)


def polygon(pixels, points, color):
    min_y = max(0, min(y for _, y in points))
    max_y = min(31, max(y for _, y in points))
    for y in range(min_y, max_y + 1):
        xs = []
        for index, (x1, y1) in enumerate(points):
            x2, y2 = points[(index + 1) % len(points)]
            if y1 == y2:
                continue
            if min(y1, y2) <= y < max(y1, y2):
                xs.append(x1 + (y - y1) * (x2 - x1) / (y2 - y1))
        xs.sort()
        for left, right in zip(xs[0::2], xs[1::2]):
            for x in range(math.ceil(left), math.floor(right) + 1):
                put(pixels, x, y, color)


def make_sprite(slug):
    if slug in ICON_GRIDS and slug not in {"air", "earth", "fire", "water", "steam"}:
        return icon_from_grid(ICON_GRIDS[slug])

    p = canvas()
    if slug == "fire":
        polygon(p, [(5, 27), (8, 17), (11, 20), (13, 12), (16, 5), (21, 15), (25, 10), (27, 27)], "red")
        polygon(p, [(4, 25), (7, 15), (10, 22), (10, 27)], "orange")
        polygon(p, [(25, 27), (26, 17), (29, 22), (29, 27)], "orange")
        polygon(p, [(10, 27), (13, 15), (16, 19), (18, 8), (22, 18), (21, 27)], "orange")
        polygon(p, [(13, 27), (15, 18), (17, 20), (18, 12), (20, 22), (19, 27)], "gold")
        polygon(p, [(15, 26), (16, 20), (18, 23), (18, 27)], "yellow")
        line(p, 6, 27, 27, 27, "dark_dirt", 2)
    elif slug == "water":
        polygon(p, [(16, 5), (8, 17), (9, 24), (16, 29), (23, 24), (24, 17)], "deep_blue")
        polygon(p, [(16, 7), (10, 17), (11, 23), (16, 27), (22, 22), (22, 17)], "blue")
        ellipse(p, 16, 22, 8, 4, "sky")
        line(p, 11, 17, 15, 11, "ice", 2)
        line(p, 17, 25, 22, 21, "ice", 1)
        rect(p, 13, 13, 15, 15, "white")
    elif slug == "earth":
        ellipse(p, 16, 21, 11, 6, "dark_dirt")
        ellipse(p, 15, 19, 10, 5, "dirt")
        rect(p, 9, 22, 24, 26, "dark_dirt")
        line(p, 16, 17, 16, 9, "green", 2)
        line(p, 16, 12, 11, 8, "green", 2)
        line(p, 17, 13, 22, 10, "lime", 2)
        rect(p, 11, 21, 15, 23, "paper")
        rect(p, 18, 19, 22, 21, "dirt")
    elif slug == "air":
        # Layered, curling wind ribbons. Short staggered strokes keep the icon
        # airy at terminal size instead of reading as flat stacked bars.
        for x0, y0, x1, y1, color, width in [
            (4, 10, 11, 8, "white", 2),
            (11, 8, 18, 9, "ice", 2),
            (18, 9, 24, 6, "sky", 2),
            (24, 6, 28, 8, "white", 1),
            (7, 15, 15, 14, "ice", 2),
            (15, 14, 22, 16, "sky", 2),
            (22, 16, 27, 13, "white", 1),
            (3, 22, 10, 20, "deep_blue", 2),
            (10, 20, 18, 21, "blue", 2),
            (18, 21, 25, 18, "ice", 1),
        ]:
            line(p, x0, y0, x1, y1, color, width)
        for cx, cy, color in [(9, 9, "white"), (20, 8, "sky"), (12, 15, "ice"), (23, 15, "white"), (8, 21, "blue")]:
            put(p, cx, cy, color)
            put(p, cx + 1, cy - 1, color)
        for gap_x, gap_y in [
            (13, 8), (14, 8), (15, 8), (13, 9), (14, 9), (15, 9), (16, 9),
            (13, 15), (14, 15), (15, 15), (12, 20), (13, 20), (14, 20),
            (11, 21), (12, 21), (13, 21), (14, 21),
        ]:
            erase(p, gap_x, gap_y)
    elif slug == "dust":
        for x, y, r, color in [
            (9, 13, 3, "gold"),
            (16, 10, 2, "paper"),
            (21, 15, 3, "dirt"),
            (13, 22, 3, "gold"),
            (23, 24, 2, "white"),
            (18, 20, 2, "stone"),
        ]:
            circle(p, x, y, r, color)
    elif slug == "steam":
        ellipse(p, 15, 26, 6, 3, "sky")
        ellipse(p, 18, 24, 7, 3, "ice")
        ellipse(p, 13, 21, 6, 3, "white")
        ellipse(p, 19, 18, 7, 3, "ice")
        ellipse(p, 14, 15, 6, 3, "white")
        ellipse(p, 20, 12, 6, 2, "ice")
        ellipse(p, 15, 9, 4, 2, "white")
        line(p, 11, 27, 20, 23, "white", 2)
        line(p, 21, 23, 12, 20, "ice", 2)
        line(p, 11, 18, 21, 15, "white", 2)
        line(p, 22, 15, 14, 12, "ice", 2)
        line(p, 14, 10, 22, 9, "white", 1)
        line(p, 19, 8, 24, 7, "ice", 1)
        erase_ellipse(p, 12, 25, 3, 2)
        erase_ellipse(p, 22, 21, 3, 2)
        erase_ellipse(p, 13, 17, 3, 2)
        erase_ellipse(p, 20, 14, 3, 1)
        erase_ellipse(p, 14, 10, 2, 1)
        line(p, 17, 28, 15, 30, "ice", 1)
        line(p, 19, 27, 21, 29, "white", 1)
        put(p, 10, 22, "white")
        put(p, 23, 16, "white")
        put(p, 17, 11, "ice")
    elif slug == "energy":
        polygon(p, [(17, 3), (9, 17), (15, 17), (12, 29), (23, 13), (17, 13)], "yellow")
        polygon(p, [(18, 5), (13, 15), (18, 15), (15, 24), (21, 12), (17, 12)], "white")
        line(p, 8, 8, 5, 5, "sky", 1)
        line(p, 24, 22, 28, 25, "sky", 1)
    elif slug == "lava":
        polygon(p, [(6, 21), (10, 12), (16, 8), (23, 13), (27, 22), (22, 28), (10, 28)], "dark_dirt")
        polygon(p, [(10, 21), (14, 13), (18, 15), (22, 22), (20, 25), (12, 25)], "red")
        polygon(p, [(12, 21), (15, 16), (18, 18), (20, 22), (18, 24), (14, 24)], "orange")
        rect(p, 8, 25, 24, 28, "gold")
        rect(p, 14, 22, 20, 24, "yellow")
    elif slug == "mud":
        ellipse(p, 16, 22, 11, 6, "dark_dirt")
        ellipse(p, 15, 20, 10, 5, "dirt")
        ellipse(p, 11, 19, 5, 3, "gold")
        ellipse(p, 12, 18, 3, 2, "paper")
        ellipse(p, 21, 21, 3, 2, "dark_dirt")
        rect(p, 20, 18, 24, 20, "stone")
        rect(p, 7, 23, 25, 27, "dark_dirt")
    elif slug == "pressure":
        rect(p, 8, 8, 24, 12, "stone")
        rect(p, 8, 21, 24, 25, "dark_stone")
        line(p, 16, 13, 16, 20, "ice", 2)
        polygon(p, [(16, 12), (12, 16), (20, 16)], "sky")
        polygon(p, [(16, 24), (12, 20), (20, 20)], "sky")
        rect(p, 11, 16, 21, 18, "white")
    elif slug == "rain":
        draw_cloud(p, 15, 11)
        for x, y in [(10, 21), (15, 23), (20, 21), (23, 24)]:
            line(p, x, y, x - 2, y + 4, "blue", 2)
            put(p, x - 2, y + 5, "sky")
    elif slug == "sea":
        rect(p, 4, 18, 28, 26, "deep_blue")
        rect(p, 4, 16, 28, 19, "blue")
        wave(p, 5, 17, "white")
        wave(p, 4, 21, "sky")
        wave(p, 8, 24, "ice")
        rect(p, 5, 23, 27, 25, "blue")
        rect(p, 5, 26, 27, 28, "deep_blue")
    elif slug == "atmosphere":
        circle(p, 16, 20, 9, "deep_blue")
        circle(p, 16, 20, 8, "blue")
        circle(p, 13, 17, 4, "sky")
        circle(p, 19, 23, 3, "ice")
        line(p, 4, 13, 28, 9, "ice", 1)
        line(p, 3, 18, 29, 15, "white", 1)
        line(p, 6, 25, 26, 28, "sky", 1)
    elif slug == "brick":
        rect(p, 6, 9, 26, 25, "red")
        for y in [10, 16, 22]:
            line(p, 6, y, 26, y, "orange", 1)
        for x0, y0, y1 in [(15, 10, 16), (11, 16, 22), (21, 16, 22), (15, 22, 25)]:
            line(p, x0, y0, x0, y1, "dark_dirt", 1)
        rect(p, 8, 11, 13, 14, "orange")
    elif slug == "cloud":
        draw_cloud(p, 16, 17)
        rect(p, 8, 21, 25, 24, "ice")
        rect(p, 13, 14, 17, 16, "white")
    elif slug == "plant":
        line(p, 16, 27, 16, 13, "green", 2)
        ellipse(p, 11, 18, 6, 3, "green")
        ellipse(p, 21, 15, 6, 3, "lime")
        ellipse(p, 13, 10, 4, 2, "lime")
        rect(p, 13, 26, 20, 28, "dark_dirt")
    elif slug == "stone":
        polygon(p, [(8, 25), (7, 17), (13, 9), (22, 8), (27, 17), (24, 26), (13, 28)], "dark_stone")
        polygon(p, [(10, 23), (10, 17), (14, 11), (21, 10), (24, 17), (21, 24)], "stone")
        polygon(p, [(14, 11), (20, 10), (17, 18), (10, 18)], "white")
        rect(p, 8, 25, 23, 28, "dark_stone")
    elif slug == "volcano":
        polygon(p, [(5, 27), (13, 11), (18, 11), (27, 27)], "dark_dirt")
        polygon(p, [(10, 25), (15, 13), (18, 14), (23, 25)], "dirt")
        rect(p, 13, 10, 20, 13, "red")
        line(p, 16, 13, 14, 24, "orange", 2)
        line(p, 18, 14, 21, 25, "red", 2)
        rect(p, 12, 8, 20, 10, "orange")
        put(p, 10, 7, "gold")
        put(p, 22, 7, "gold")
    elif slug == "wind":
        line(p, 6, 7, 18, 7, "violet", 1)
        line(p, 4, 10, 23, 10, "ice", 2)
        line(p, 18, 10, 27, 14, "sky", 2)
        line(p, 6, 14, 15, 14, "blue", 1)
        line(p, 7, 17, 26, 17, "white", 2)
        line(p, 20, 17, 25, 21, "ice", 1)
        line(p, 10, 24, 21, 24, "sky", 2)
    elif slug == "grass":
        for x, h, color in [(8, 9, "dark_green"), (11, 14, "green"), (15, 17, "lime"), (19, 15, "green"), (23, 10, "dark_green")]:
            line(p, x, 28, x - 3, 28 - h, color, 2)
            line(p, x, 28, x + 3, 28 - h + 2, color, 2)
        rect(p, 6, 27, 26, 29, "dark_green")
    elif slug == "metal":
        polygon(p, [(8, 19), (13, 12), (24, 13), (27, 20), (21, 26), (10, 25)], "blue")
        polygon(p, [(8, 19), (13, 12), (24, 13), (27, 20), (21, 26), (10, 25)], "dark_stone")
        polygon(p, [(10, 18), (14, 14), (23, 15), (25, 20), (20, 23), (11, 22)], "stone")
        polygon(p, [(15, 15), (23, 15), (21, 18), (13, 18)], "ice")
        rect(p, 13, 15, 22, 17, "white")
        rect(p, 20, 18, 25, 20, "sky")
        rect(p, 10, 21, 22, 23, "dark_stone")
    elif slug == "mountain":
        polygon(p, [(4, 28), (13, 9), (22, 28)], "dark_stone")
        polygon(p, [(13, 9), (19, 28), (29, 28)], "dirt")
        polygon(p, [(13, 9), (10, 16), (16, 16)], "white")
        polygon(p, [(20, 14), (16, 22), (24, 22)], "stone")
        line(p, 7, 27, 28, 27, "dark_dirt", 2)
    elif slug == "sand":
        polygon(p, [(4, 25), (11, 15), (18, 25)], "gold")
        polygon(p, [(12, 25), (22, 13), (30, 25)], "dirt")
        polygon(p, [(7, 25), (13, 18), (18, 25)], "yellow")
        polygon(p, [(17, 25), (23, 16), (28, 25)], "paper")
        rect(p, 4, 25, 30, 28, "gold")
        rect(p, 10, 26, 25, 27, "white")
        line(p, 8, 21, 18, 20, "paper", 1)
        line(p, 18, 17, 27, 18, "paper", 1)
    elif slug == "sky":
        rect(p, 6, 10, 26, 24, "blue")
        circle(p, 22, 12, 4, "yellow")
        draw_cloud(p, 12, 19)
        line(p, 7, 12, 13, 9, "sky", 1)
    elif slug == "storm":
        draw_cloud(p, 15, 11)
        polygon(p, [(17, 17), (12, 26), (17, 25), (14, 31), (23, 21), (18, 22)], "yellow")
        for x, y in [(8, 22), (24, 21)]:
            line(p, x, y, x - 2, y + 4, "blue", 1)
    elif slug == "glass":
        polygon(p, [(9, 9), (22, 6), (27, 12), (23, 25), (10, 27), (5, 18)], "deep_blue")
        polygon(p, [(10, 10), (21, 8), (25, 13), (21, 23), (11, 25), (7, 18)], "sky")
        polygon(p, [(13, 11), (21, 10), (23, 14), (19, 21), (12, 23), (9, 18)], "ice")
        erase_ellipse(p, 16, 17, 4, 3)
        erase_ellipse(p, 20, 14, 2, 2)
        erase_ellipse(p, 12, 20, 2, 2)
        line(p, 10, 10, 22, 6, "white", 1)
        line(p, 22, 6, 27, 12, "blue", 1)
        line(p, 27, 12, 23, 25, "deep_blue", 1)
        line(p, 23, 25, 10, 27, "blue", 1)
        line(p, 10, 27, 5, 18, "deep_blue", 1)
        line(p, 10, 11, 23, 23, "white", 1)
        line(p, 8, 18, 22, 8, "blue", 1)
        line(p, 12, 22, 23, 13, "white", 1)
        rect(p, 13, 13, 16, 16, "white")
    elif slug == "time":
        rect(p, 11, 4, 21, 7, "gold")
        rect(p, 12, 7, 20, 10, "paper")
        polygon(p, [(11, 10), (21, 10), (18, 16), (14, 16)], "ice")
        polygon(p, [(14, 16), (18, 16), (22, 25), (10, 25)], "paper")
        rect(p, 10, 25, 22, 28, "gold")
        rect(p, 12, 13, 20, 16, "white")
        rect(p, 14, 20, 18, 24, "gold")
        line(p, 10, 7, 22, 25, "dark_dirt", 1)
        line(p, 22, 7, 10, 25, "dark_dirt", 1)
    elif slug == "life":
        line(p, 16, 27, 16, 13, "green", 2)
        ellipse(p, 10, 17, 6, 4, "green")
        ellipse(p, 22, 15, 6, 4, "lime")
        circle(p, 16, 10, 4, "yellow")
        circle(p, 16, 10, 2, "white")
        line(p, 8, 8, 6, 5, "sky", 1)
        line(p, 24, 8, 27, 5, "sky", 1)
        rect(p, 12, 26, 20, 28, "dark_dirt")
    elif slug == "human":
        circle(p, 16, 8, 5, "paper")
        rect(p, 13, 13, 19, 16, "paper")
        polygon(p, [(10, 16), (22, 16), (25, 27), (7, 27)], "blue")
        polygon(p, [(12, 17), (20, 17), (22, 26), (10, 26)], "violet")
        rect(p, 13, 6, 19, 8, "dark_dirt")
        rect(p, 11, 8, 13, 11, "dark_dirt")
        rect(p, 19, 8, 21, 11, "dark_dirt")
        rect(p, 10, 18, 13, 24, "paper")
        rect(p, 19, 18, 22, 24, "paper")
    elif slug == "tool":
        rect(p, 7, 8, 22, 12, "stone")
        rect(p, 19, 10, 25, 16, "dark_stone")
        line(p, 13, 13, 24, 27, "dark_dirt", 4)
        line(p, 14, 14, 24, 27, "dirt", 2)
        rect(p, 6, 10, 11, 15, "ice")
        rect(p, 19, 8, 24, 10, "white")
    elif slug == "book":
        polygon(p, [(7, 9), (16, 7), (16, 25), (7, 27)], "red")
        polygon(p, [(16, 7), (26, 10), (26, 27), (16, 25)], "orange")
        polygon(p, [(9, 11), (16, 10), (16, 23), (9, 24)], "paper")
        polygon(p, [(17, 10), (24, 12), (24, 24), (17, 23)], "paper")
        line(p, 16, 8, 16, 25, "dark_dirt", 1)
        line(p, 10, 15, 15, 14, "gold", 1)
        line(p, 18, 16, 23, 17, "gold", 1)
        rect(p, 7, 25, 26, 28, "dark_dirt")
    elif slug == "bird":
        ellipse(p, 17, 18, 8, 5, "orange")
        ellipse(p, 12, 16, 6, 4, "paper")
        polygon(p, [(23, 17), (29, 15), (24, 20)], "gold")
        circle(p, 11, 12, 4, "paper")
        rect(p, 12, 11, 14, 13, "ink")
        polygon(p, [(9, 18), (3, 14), (7, 22)], "violet")
        polygon(p, [(18, 15), (15, 8), (22, 13)], "red")
        line(p, 16, 23, 14, 27, "dark_dirt", 1)
        line(p, 19, 23, 22, 27, "dark_dirt", 1)
    elif slug == "fish":
        ellipse(p, 15, 17, 9, 5, "green")
        ellipse(p, 13, 16, 6, 3, "sky")
        polygon(p, [(23, 17), (30, 11), (29, 23)], "dark_green")
        polygon(p, [(10, 13), (14, 6), (18, 13)], "blue")
        polygon(p, [(11, 21), (15, 28), (18, 21)], "deep_blue")
        circle(p, 8, 16, 2, "white")
        put(p, 8, 16, "ink")
        line(p, 10, 19, 20, 19, "ice", 1)
    elif slug == "house":
        rect(p, 8, 15, 25, 27, "paper")
        polygon(p, [(5, 16), (16, 6), (28, 16)], "red")
        polygon(p, [(9, 16), (16, 9), (24, 16)], "orange")
        rect(p, 13, 20, 18, 27, "dark_dirt")
        rect(p, 20, 18, 24, 22, "sky")
        rect(p, 21, 19, 23, 21, "white")
        rect(p, 7, 26, 26, 29, "dark_dirt")
        rect(p, 10, 16, 12, 25, "gold")
    elif slug == "tree":
        rect(p, 14, 16, 19, 28, "dirt")
        rect(p, 12, 24, 21, 28, "dark_dirt")
        circle(p, 13, 13, 7, "dark_green")
        circle(p, 19, 11, 8, "green")
        circle(p, 22, 17, 6, "lime")
        circle(p, 10, 18, 6, "green")
        rect(p, 14, 10, 18, 13, "lime")
        line(p, 16, 18, 11, 24, "dark_dirt", 1)
        line(p, 17, 18, 23, 23, "dark_dirt", 1)
    elif slug == "vase":
        rect(p, 12, 5, 20, 9, "gold")
        rect(p, 10, 9, 22, 13, "dirt")
        ellipse(p, 8, 20, 4, 6, "dirt")
        ellipse(p, 24, 20, 4, 6, "dirt")
        polygon(p, [(10, 13), (22, 13), (25, 25), (20, 29), (12, 29), (7, 25)], "dirt")
        polygon(p, [(12, 14), (20, 14), (22, 24), (19, 27), (13, 27), (10, 24)], "paper")
        line(p, 8, 18, 24, 18, "gold", 2)
        line(p, 9, 24, 23, 24, "gold", 1)
        rect(p, 13, 6, 19, 8, "paper")
        rect(p, 20, 21, 23, 24, "dark_dirt")
        erase_ellipse(p, 9, 20, 2, 4)
        erase_ellipse(p, 23, 20, 2, 4)
        rect(p, 11, 27, 21, 29, "dark_dirt")
    elif slug == "bottle":
        rect(p, 14, 2, 18, 6, "dirt")
        rect(p, 15, 2, 17, 4, "paper")
        rect(p, 13, 5, 19, 11, "sky")
        rect(p, 12, 10, 20, 13, "ice")
        line(p, 13, 5, 18, 5, "white", 1)
        polygon(p, [(10, 13), (22, 13), (25, 27), (7, 27)], "deep_blue")
        polygon(p, [(12, 14), (20, 14), (22, 25), (10, 25)], "sky")
        rect(p, 11, 20, 22, 26, "blue")
        line(p, 11, 20, 22, 20, "ice", 1)
        line(p, 13, 6, 21, 25, "white", 1)
        rect(p, 14, 6, 18, 8, "white")
        rect(p, 8, 26, 24, 28, "deep_blue")
    elif slug == "jar":
        rect(p, 9, 5, 23, 7, "dark_stone")
        rect(p, 10, 6, 22, 9, "stone")
        for ridge_x in [10, 13, 16, 19, 22]:
            rect(p, ridge_x, 5, ridge_x + 1, 10, "dark_stone")
        rect(p, 11, 9, 21, 12, "ice")
        polygon(p, [(9, 12), (23, 12), (25, 26), (21, 29), (11, 29), (7, 26)], "sky")
        polygon(p, [(11, 13), (21, 13), (23, 25), (20, 27), (12, 27), (9, 25)], "ice")
        rect(p, 10, 18, 22, 26, "gold")
        rect(p, 11, 21, 21, 25, "paper")
        line(p, 13, 23, 19, 23, "dirt", 1)
        line(p, 10, 13, 20, 27, "white", 1)
        rect(p, 9, 7, 23, 8, "white")
    elif slug == "sun":
        for x0, y0, x1, y1 in [
            (16, 2, 16, 8),
            (16, 24, 16, 30),
            (2, 16, 8, 16),
            (24, 16, 30, 16),
            (6, 6, 10, 10),
            (22, 22, 26, 26),
            (26, 6, 22, 10),
            (6, 26, 10, 22),
        ]:
            line(p, x0, y0, x1, y1, "orange", 2)
        circle(p, 16, 16, 8, "gold")
        circle(p, 16, 16, 5, "yellow")
        circle(p, 13, 13, 2, "white")
        rect(p, 8, 21, 24, 23, "orange")
    elif slug == "coal":
        polygon(p, [(7, 23), (9, 13), (16, 7), (25, 12), (27, 23), (20, 28), (11, 27)], "ink")
        polygon(p, [(9, 22), (11, 14), (17, 10), (23, 14), (24, 22), (19, 25), (12, 25)], "dark_stone")
        polygon(p, [(12, 15), (17, 10), (22, 15), (17, 18)], "stone")
        polygon(p, [(10, 22), (15, 18), (20, 24), (13, 25)], "dark_dirt")
        rect(p, 14, 12, 18, 14, "white")
        rect(p, 21, 20, 24, 22, "stone")
    elif slug == "moon":
        circle(p, 16, 16, 10, "deep_blue")
        circle(p, 14, 15, 10, "ice")
        circle(p, 19, 12, 8, "white")
        circle(p, 20, 13, 7, "ice")
        circle(p, 12, 19, 2, "stone")
        circle(p, 17, 10, 1, "stone")
        circle(p, 18, 22, 1, "blue")
    elif slug == "flower":
        line(p, 16, 27, 16, 14, "green", 2)
        ellipse(p, 11, 20, 5, 2, "green")
        ellipse(p, 21, 18, 5, 2, "lime")
        circle(p, 16, 12, 3, "gold")
        for cx, cy, color in [(16, 7, "paper"), (16, 17, "paper"), (11, 12, "red"), (21, 12, "red"), (13, 9, "orange"), (19, 15, "orange")]:
            ellipse(p, cx, cy, 3, 4 if cx == 16 else 3, color)
        circle(p, 16, 12, 2, "yellow")
        rect(p, 13, 26, 20, 28, "dark_dirt")
    elif slug == "egg":
        ellipse(p, 16, 18, 8, 11, "paper")
        ellipse(p, 16, 17, 7, 10, "white")
        ellipse(p, 13, 13, 3, 4, "ice")
        rect(p, 11, 25, 22, 28, "gold")
        rect(p, 12, 26, 21, 28, "paper")
        put(p, 20, 20, "stone")
    elif slug == "honey":
        for cx, cy in [(11, 12), (18, 12), (15, 18), (22, 18), (9, 22)]:
            polygon(p, [(cx, cy - 4), (cx + 4, cy - 2), (cx + 4, cy + 2), (cx, cy + 4), (cx - 4, cy + 2), (cx - 4, cy - 2)], "gold")
            polygon(p, [(cx, cy - 3), (cx + 3, cy - 1), (cx + 3, cy + 1), (cx, cy + 3), (cx - 3, cy + 1), (cx - 3, cy - 1)], "yellow")
            rect(p, cx - 1, cy - 1, cx + 2, cy + 2, "paper")
        line(p, 6, 27, 25, 27, "orange", 2)
        put(p, 26, 24, "white")
    elif slug == "paper":
        rect(p, 9, 5, 24, 27, "dirt")
        rect(p, 7, 7, 22, 29, "paper")
        rect(p, 9, 9, 20, 12, "white")
        line(p, 10, 15, 19, 15, "gold", 1)
        line(p, 10, 18, 19, 18, "gold", 1)
        line(p, 10, 21, 17, 21, "dirt", 1)
        polygon(p, [(17, 24), (22, 24), (22, 29)], "gold")
        line(p, 17, 24, 22, 29, "dark_dirt", 1)
    elif slug == "hammer":
        rect(p, 8, 7, 24, 12, "dark_stone")
        rect(p, 10, 8, 26, 13, "stone")
        rect(p, 21, 10, 26, 15, "dark_stone")
        line(p, 15, 13, 15, 28, "dark_dirt", 4)
        line(p, 16, 13, 16, 28, "dirt", 2)
        rect(p, 11, 8, 21, 10, "white")
        rect(p, 13, 25, 18, 29, "gold")
    elif slug == "wheat":
        for x in [11, 16, 21]:
            line(p, x, 28, x, 8, "gold", 1)
            for y in range(10, 24, 4):
                ellipse(p, x - 3, y, 3, 2, "yellow")
                ellipse(p, x + 3, y + 2, 3, 2, "paper")
        line(p, 8, 28, 24, 28, "dark_dirt", 2)
        put(p, 16, 6, "white")
    elif slug == "wood":
        circle(p, 16, 17, 10, "dark_dirt")
        circle(p, 16, 17, 8, "dirt")
        circle(p, 16, 17, 5, "paper")
        circle(p, 16, 17, 2, "gold")
        line(p, 6, 17, 26, 17, "dark_dirt", 1)
        line(p, 12, 9, 21, 25, "gold", 1)
        rect(p, 7, 24, 25, 28, "dirt")
        rect(p, 10, 25, 22, 27, "paper")
    elif slug == "snow":
        ellipse(p, 12, 21, 7, 5, "ice")
        ellipse(p, 20, 20, 8, 6, "white")
        ellipse(p, 15, 16, 6, 5, "white")
        rect(p, 6, 21, 27, 27, "ice")
        rect(p, 8, 18, 16, 21, "white")
        line(p, 9, 25, 25, 25, "sky", 1)
        rect(p, 7, 26, 26, 28, "stone")
        rect(p, 11, 27, 22, 28, "white")
        put(p, 21, 13, "ice")
    elif slug == "ice":
        polygon(p, [(8, 10), (18, 5), (26, 11), (24, 24), (14, 28), (6, 21)], "deep_blue")
        polygon(p, [(10, 11), (18, 7), (24, 12), (22, 22), (14, 26), (8, 20)], "blue")
        polygon(p, [(13, 10), (20, 9), (23, 13), (18, 17), (11, 16)], "sky")
        line(p, 10, 12, 22, 23, "ice", 1)
        rect(p, 13, 12, 17, 15, "white")
        rect(p, 19, 19, 23, 22, "ice")
    elif slug == "cotton":
        circle(p, 10, 16, 5, "paper")
        circle(p, 15, 12, 6, "white")
        circle(p, 21, 16, 5, "ice")
        circle(p, 16, 20, 6, "paper")
        rect(p, 8, 18, 24, 24, "ice")
        rect(p, 12, 10, 17, 13, "white")
        line(p, 9, 23, 24, 23, "stone", 1)
    elif slug == "needle":
        line(p, 16, 4, 16, 28, "stone", 2)
        line(p, 17, 5, 17, 28, "ice", 1)
        circle(p, 16, 7, 3, "stone")
        circle(p, 16, 7, 1, "ink")
        line(p, 10, 16, 24, 25, "sky", 2)
        line(p, 9, 17, 22, 27, "violet", 1)
        rect(p, 14, 26, 19, 29, "white")
    elif slug == "chain":
        for cx, cy, color in [(11, 13, "stone"), (18, 18, "ice"), (23, 13, "stone")]:
            ellipse(p, cx, cy, 6, 4, "dark_stone")
            ellipse(p, cx, cy, 4, 2, "sky")
            rect(p, cx - 2, cy - 1, cx + 3, cy + 2, "deep_blue")
            line(p, cx - 5, cy, cx + 5, cy, color, 1)
        rect(p, 8, 23, 24, 26, "stone")
        rect(p, 10, 24, 22, 25, "white")
    elif slug == "web":
        circle(p, 16, 16, 12, "paper")
        circle(p, 16, 16, 10, "ink")
        for x1, y1 in [(16, 3), (16, 29), (3, 16), (29, 16), (7, 7), (25, 7), (7, 25), (25, 25)]:
            line(p, 16, 16, x1, y1, "paper", 1)
        for r, color in [(4, "white"), (7, "ice"), (10, "paper")]:
            for x, y in [(16 - r, 16), (16, 16 - r), (16 + r, 16), (16, 16 + r)]:
                put(p, x, y, color)
            line(p, 16 - r, 16, 16, 16 - r, color, 1)
            line(p, 16, 16 - r, 16 + r, 16, color, 1)
            line(p, 16 + r, 16, 16, 16 + r, color, 1)
            line(p, 16, 16 + r, 16 - r, 16, color, 1)
    elif slug == "spider":
        circle(p, 16, 18, 6, "ink")
        circle(p, 16, 16, 5, "red")
        circle(p, 16, 10, 4, "ink")
        circle(p, 16, 17, 3, "orange")
        rect(p, 14, 10, 18, 13, "red")
        for x0, y0, x1, y1 in [(12, 16, 4, 10), (12, 18, 3, 18), (12, 20, 5, 26), (20, 16, 28, 10), (20, 18, 29, 18), (20, 20, 27, 26)]:
            line(p, x0, y0, x1, y1, "ink", 2)
            put(p, x1, y1, "red")
        rect(p, 14, 7, 15, 8, "white")
        rect(p, 18, 7, 19, 8, "white")
    elif slug == "bee":
        ellipse(p, 17, 18, 8, 5, "gold")
        rect(p, 12, 14, 14, 22, "ink")
        rect(p, 17, 13, 19, 23, "ink")
        rect(p, 21, 15, 23, 21, "orange")
        ellipse(p, 11, 12, 5, 4, "ice")
        ellipse(p, 21, 11, 5, 4, "white")
        circle(p, 9, 18, 3, "ink")
        line(p, 8, 14, 5, 10, "dark_dirt", 1)
        line(p, 10, 14, 13, 10, "dark_dirt", 1)
        polygon(p, [(25, 18), (29, 16), (26, 21)], "yellow")
    elif slug == "glasses":
        ellipse(p, 11, 17, 7, 5, "stone")
        ellipse(p, 22, 17, 7, 5, "stone")
        ellipse(p, 11, 17, 5, 3, "sky")
        ellipse(p, 22, 17, 5, 3, "ice")
        rect(p, 9, 15, 13, 18, "deep_blue")
        rect(p, 20, 15, 24, 18, "blue")
        line(p, 16, 16, 18, 16, "white", 2)
        line(p, 4, 14, 7, 16, "dark_stone", 2)
        line(p, 26, 16, 29, 14, "dark_stone", 2)
        rect(p, 9, 14, 13, 15, "white")
        rect(p, 20, 14, 24, 15, "white")
    elif slug == "clock":
        circle(p, 16, 16, 11, "dark_dirt")
        circle(p, 16, 16, 9, "gold")
        circle(p, 16, 16, 7, "paper")
        circle(p, 16, 16, 1, "ink")
        line(p, 16, 16, 16, 9, "dark_stone", 1)
        line(p, 16, 16, 22, 19, "ink", 1)
        rect(p, 15, 5, 18, 8, "paper")
        rect(p, 15, 25, 18, 28, "dark_dirt")
        rect(p, 7, 15, 10, 18, "dirt")
        rect(p, 23, 15, 26, 18, "dirt")
        rect(p, 12, 10, 14, 12, "white")
    elif slug == "boat":
        polygon(p, [(5, 20), (27, 20), (23, 27), (10, 27)], "dark_dirt")
        polygon(p, [(8, 20), (25, 20), (21, 24), (11, 24)], "dirt")
        rect(p, 15, 7, 17, 21, "dark_stone")
        polygon(p, [(17, 8), (27, 18), (17, 18)], "paper")
        polygon(p, [(15, 9), (7, 19), (15, 19)], "ice")
        line(p, 5, 28, 27, 28, "blue", 2)
        wave(p, 4, 29, "sky")
        rect(p, 18, 10, 21, 12, "white")
    elif slug == "car":
        rect(p, 6, 16, 27, 24, "red")
        polygon(p, [(10, 16), (14, 10), (22, 10), (26, 16)], "orange")
        rect(p, 14, 12, 18, 15, "sky")
        rect(p, 20, 12, 24, 15, "ice")
        circle(p, 11, 25, 4, "ink")
        circle(p, 23, 25, 4, "ink")
        circle(p, 11, 25, 2, "stone")
        circle(p, 23, 25, 2, "stone")
        rect(p, 7, 18, 12, 21, "orange")
        rect(p, 24, 18, 28, 21, "yellow")
    elif slug == "cat":
        ellipse(p, 16, 20, 8, 7, "orange")
        circle(p, 16, 12, 6, "gold")
        polygon(p, [(11, 9), (13, 3), (16, 10)], "orange")
        polygon(p, [(21, 9), (19, 3), (16, 10)], "orange")
        rect(p, 13, 11, 15, 13, "ink")
        rect(p, 18, 11, 20, 13, "ink")
        rect(p, 16, 14, 18, 16, "paper")
        line(p, 10, 15, 4, 13, "paper", 1)
        line(p, 22, 15, 28, 13, "paper", 1)
        line(p, 22, 22, 29, 16, "orange", 3)
        rect(p, 11, 26, 15, 29, "dark_dirt")
        rect(p, 19, 26, 23, 29, "dark_dirt")
    elif slug == "dog":
        ellipse(p, 17, 20, 9, 6, "dirt")
        circle(p, 12, 13, 6, "paper")
        ellipse(p, 7, 14, 3, 6, "dark_dirt")
        ellipse(p, 17, 14, 3, 6, "dark_dirt")
        rect(p, 11, 12, 13, 14, "ink")
        rect(p, 7, 17, 12, 20, "dark_dirt")
        line(p, 25, 18, 29, 13, "dirt", 2)
        rect(p, 10, 25, 14, 29, "dark_dirt")
        rect(p, 20, 25, 24, 29, "dark_dirt")
        rect(p, 14, 16, 17, 18, "white")
    elif slug == "scissors":
        circle(p, 10, 22, 5, "violet")
        circle(p, 20, 22, 5, "violet")
        circle(p, 10, 22, 3, "ink")
        circle(p, 20, 22, 3, "ink")
        circle(p, 15, 17, 2, "gold")
        polygon(p, [(15, 16), (7, 5), (12, 5), (17, 15)], "stone")
        polygon(p, [(16, 17), (26, 5), (21, 5), (14, 16)], "ice")
        line(p, 15, 17, 10, 22, "dark_stone", 2)
        line(p, 16, 17, 20, 22, "dark_stone", 2)
        rect(p, 8, 5, 12, 7, "white")
    elif slug == "wheel":
        circle(p, 16, 16, 11, "ink")
        circle(p, 16, 16, 8, "dark_stone")
        circle(p, 16, 16, 4, "stone")
        circle(p, 16, 16, 2, "ice")
        for x1, y1 in [(16, 6), (16, 26), (6, 16), (26, 16), (9, 9), (23, 9), (9, 23), (23, 23)]:
            line(p, 16, 16, x1, y1, "paper", 1)
        rect(p, 9, 9, 23, 11, "dark_stone")
        rect(p, 8, 21, 24, 23, "dark_stone")
    elif slug == "blade":
        polygon(p, [(15, 4), (22, 8), (18, 24), (12, 29), (11, 16)], "dark_stone")
        polygon(p, [(16, 5), (20, 9), (17, 22), (13, 26), (13, 15)], "ice")
        polygon(p, [(15, 7), (18, 10), (15, 20), (13, 24), (13, 14)], "white")
        rect(p, 10, 24, 19, 27, "gold")
        rect(p, 11, 27, 18, 30, "dark_dirt")
        line(p, 20, 9, 16, 25, "stone", 1)
    elif slug == "newspaper":
        polygon(p, [(7, 8), (23, 5), (26, 25), (10, 29)], "paper")
        polygon(p, [(9, 10), (22, 8), (24, 23), (11, 26)], "white")
        rect(p, 11, 12, 20, 15, "stone")
        line(p, 11, 17, 22, 15, "dark_stone", 1)
        line(p, 12, 20, 23, 18, "dark_stone", 1)
        line(p, 13, 23, 20, 22, "dark_stone", 1)
        rect(p, 12, 13, 15, 15, "blue")
        polygon(p, [(21, 5), (26, 5), (26, 10)], "gold")
        line(p, 21, 5, 26, 10, "dark_dirt", 1)
    elif slug == "lizard":
        ellipse(p, 16, 18, 9, 5, "green")
        ellipse(p, 11, 15, 5, 4, "lime")
        polygon(p, [(23, 18), (31, 14), (26, 21)], "dark_green")
        line(p, 7, 21, 2, 26, "green", 2)
        line(p, 12, 21, 9, 27, "dark_green", 2)
        line(p, 20, 21, 17, 27, "green", 2)
        line(p, 5, 12, 1, 8, "green", 2)
        circle(p, 9, 13, 1, "white")
        put(p, 9, 13, "ink")
        rect(p, 13, 16, 20, 18, "lime")
    elif slug == "bread":
        rect(p, 7, 14, 26, 26, "dirt")
        ellipse(p, 12, 14, 6, 6, "gold")
        ellipse(p, 18, 13, 7, 6, "gold")
        ellipse(p, 23, 15, 5, 5, "orange")
        rect(p, 8, 17, 25, 25, "paper")
        rect(p, 10, 20, 23, 26, "gold")
        line(p, 11, 13, 14, 19, "white", 1)
        line(p, 18, 11, 18, 18, "white", 1)
        line(p, 23, 14, 20, 20, "paper", 1)
    elif slug == "fishing-rod":
        line(p, 7, 28, 22, 5, "dark_dirt", 3)
        line(p, 8, 28, 23, 5, "dirt", 1)
        line(p, 22, 5, 26, 14, "ice", 1)
        line(p, 26, 14, 26, 22, "sky", 1)
        circle(p, 26, 24, 2, "lime")
        line(p, 24, 24, 28, 24, "green", 1)
        rect(p, 5, 26, 10, 30, "gold")
        rect(p, 17, 9, 20, 12, "paper")
    elif slug == "crystal-ball":
        circle(p, 16, 15, 9, "deep_blue")
        circle(p, 16, 15, 8, "sky")
        circle(p, 13, 12, 4, "white")
        circle(p, 19, 18, 4, "blue")
        line(p, 10, 20, 22, 9, "ice", 1)
        rect(p, 11, 23, 22, 26, "gold")
        rect(p, 8, 26, 25, 29, "dark_dirt")
        rect(p, 12, 24, 21, 25, "paper")
        put(p, 22, 11, "white")
    elif slug == "butterfly":
        ellipse(p, 10, 14, 6, 7, "violet")
        ellipse(p, 22, 14, 6, 7, "sky")
        ellipse(p, 11, 22, 5, 5, "blue")
        ellipse(p, 21, 22, 5, 5, "paper")
        rect(p, 15, 11, 18, 25, "ink")
        circle(p, 16, 9, 2, "ink")
        line(p, 15, 8, 11, 5, "ice", 1)
        line(p, 18, 8, 22, 5, "ice", 1)
        rect(p, 8, 13, 11, 16, "white")
        rect(p, 21, 13, 24, 16, "white")
    elif slug == "flying-fish":
        ellipse(p, 15, 18, 8, 4, "blue")
        polygon(p, [(22, 18), (29, 12), (29, 24)], "deep_blue")
        polygon(p, [(13, 15), (6, 6), (18, 13)], "sky")
        polygon(p, [(14, 21), (7, 28), (19, 23)], "ice")
        circle(p, 9, 17, 2, "white")
        put(p, 9, 17, "ink")
        line(p, 10, 20, 20, 20, "white", 1)
        line(p, 4, 10, 8, 7, "ice", 1)
        line(p, 5, 26, 9, 29, "sky", 1)
    elif slug == "shovel":
        line(p, 16, 5, 16, 22, "dark_dirt", 3)
        line(p, 17, 5, 17, 22, "dirt", 1)
        rect(p, 13, 4, 20, 7, "red")
        line(p, 13, 4, 20, 4, "orange", 1)
        polygon(p, [(11, 21), (22, 21), (20, 29), (16, 31), (12, 29)], "stone")
        polygon(p, [(13, 22), (20, 22), (18, 28), (16, 29), (14, 28)], "ice")
        rect(p, 14, 22, 19, 24, "white")
    elif slug == "axe":
        line(p, 12, 27, 21, 10, "dark_dirt", 4)
        line(p, 13, 27, 22, 10, "dirt", 2)
        polygon(p, [(16, 8), (25, 6), (28, 12), (22, 17), (17, 14)], "dark_stone")
        polygon(p, [(17, 9), (24, 8), (26, 12), (21, 15), (18, 13)], "ice")
        polygon(p, [(14, 8), (7, 9), (5, 15), (12, 16), (17, 12)], "stone")
        polygon(p, [(13, 10), (8, 11), (7, 14), (12, 14), (16, 12)], "white")
        rect(p, 10, 24, 15, 29, "gold")
    elif slug == "clay":
        ellipse(p, 16, 21, 11, 7, "dark_dirt")
        ellipse(p, 15, 19, 10, 6, "dirt")
        ellipse(p, 11, 17, 5, 3, "paper")
        rect(p, 7, 23, 25, 28, "dark_dirt")
        line(p, 8, 20, 24, 18, "gold", 1)
        line(p, 11, 24, 22, 25, "paper", 1)
        rect(p, 18, 15, 22, 18, "stone")
    elif slug == "pottery":
        rect(p, 12, 6, 21, 10, "dark_dirt")
        rect(p, 10, 10, 23, 14, "dirt")
        polygon(p, [(10, 14), (23, 14), (25, 25), (21, 29), (12, 29), (7, 25)], "dirt")
        polygon(p, [(12, 15), (21, 15), (22, 24), (19, 27), (13, 27), (10, 24)], "paper")
        rect(p, 12, 7, 21, 9, "gold")
        line(p, 9, 18, 24, 18, "gold", 2)
        line(p, 10, 23, 23, 23, "dark_dirt", 1)
        rect(p, 20, 20, 23, 24, "dark_dirt")
    elif slug == "knife":
        polygon(p, [(15, 3), (21, 7), (18, 21), (13, 25), (12, 13)], "dark_stone")
        polygon(p, [(16, 5), (19, 8), (17, 19), (14, 22), (14, 13)], "ice")
        polygon(p, [(16, 6), (18, 9), (16, 16), (14, 20), (14, 13)], "white")
        rect(p, 10, 22, 19, 26, "dark_dirt")
        rect(p, 11, 23, 18, 28, "dirt")
        rect(p, 12, 23, 17, 24, "gold")
        put(p, 16, 26, "paper")
    elif slug == "seaweed":
        for x, h, color in [(8, 14, "dark_green"), (12, 20, "green"), (17, 23, "lime"), (22, 18, "green"), (26, 12, "dark_green")]:
            line(p, x, 29, x - 3, 29 - h, color, 2)
            line(p, x, 29, x + 3, 27 - h, color, 2)
        rect(p, 5, 28, 28, 31, "deep_blue")
        wave(p, 5, 27, "sky")
        circle(p, 13, 20, 2, "lime")
        circle(p, 21, 15, 2, "green")
    elif slug == "hay":
        polygon(p, [(7, 23), (11, 11), (25, 11), (29, 23), (24, 29), (11, 29)], "gold")
        polygon(p, [(10, 22), (13, 14), (23, 14), (26, 23), (22, 26), (12, 26)], "yellow")
        for y in [15, 18, 21, 24]:
            line(p, 9, y, 27, y + 1, "paper", 1)
        for x in [12, 16, 20, 24]:
            line(p, x, 12, x - 2, 28, "dirt", 1)
        rect(p, 8, 26, 27, 29, "dark_dirt")
    elif slug == "bacteria":
        ellipse(p, 16, 17, 8, 6, "lime")
        ellipse(p, 16, 17, 6, 4, "green")
        for x, y in [(8, 10), (24, 10), (6, 18), (26, 19), (11, 25), (22, 25)]:
            line(p, 16, 17, x, y, "dark_green", 1)
            circle(p, x, y, 2, "lime")
        circle(p, 13, 15, 2, "white")
        circle(p, 19, 19, 2, "dark_green")
        put(p, 16, 17, "yellow")
    elif slug == "wool":
        circle(p, 10, 17, 5, "paper")
        circle(p, 15, 13, 6, "white")
        circle(p, 22, 16, 5, "ice")
        circle(p, 17, 21, 7, "paper")
        rect(p, 7, 20, 25, 26, "ice")
        rect(p, 12, 11, 18, 14, "white")
        line(p, 8, 25, 25, 25, "stone", 1)
        rect(p, 11, 26, 14, 29, "dark_dirt")
        rect(p, 21, 25, 24, 29, "dark_dirt")
    elif slug == "cow":
        ellipse(p, 17, 19, 10, 6, "white")
        circle(p, 9, 15, 5, "paper")
        rect(p, 14, 16, 18, 20, "ink")
        rect(p, 21, 18, 25, 22, "ink")
        rect(p, 7, 14, 9, 16, "ink")
        line(p, 6, 12, 3, 8, "paper", 1)
        line(p, 12, 12, 15, 8, "paper", 1)
        rect(p, 10, 26, 13, 30, "dark_dirt")
        rect(p, 21, 25, 24, 30, "dark_dirt")
        line(p, 26, 18, 30, 14, "dark_dirt", 1)
        rect(p, 7, 17, 11, 19, "paper")
    elif slug == "horse":
        ellipse(p, 17, 19, 10, 6, "dirt")
        polygon(p, [(8, 16), (11, 8), (17, 12), (15, 18)], "dirt")
        rect(p, 12, 9, 16, 12, "dark_dirt")
        rect(p, 10, 14, 13, 16, "paper")
        line(p, 24, 17, 30, 10, "dark_dirt", 2)
        rect(p, 10, 24, 13, 30, "dark_dirt")
        rect(p, 21, 24, 24, 30, "dark_dirt")
        line(p, 14, 13, 8, 9, "dark_dirt", 2)
        put(p, 11, 12, "ink")
    elif slug == "rainbow":
        for r, color in [(12, "red"), (10, "orange"), (8, "yellow"), (6, "green"), (4, "blue"), (2, "violet")]:
            for x in range(4, 29):
                y = 26 - int((max(0, r * r - (x - 16) * (x - 16))) ** 0.5)
                rect(p, x, y, x + 1, y + 2, color)
        draw_cloud(p, 8, 21)
        draw_cloud(p, 24, 21)
    elif slug == "star":
        polygon(p, [(16, 3), (19, 12), (29, 12), (21, 18), (24, 28), (16, 22), (8, 28), (11, 18), (3, 12), (13, 12)], "gold")
        polygon(p, [(16, 7), (18, 14), (24, 14), (19, 18), (21, 24), (16, 20), (11, 24), (13, 18), (8, 14), (14, 14)], "yellow")
        rect(p, 14, 12, 17, 15, "white")
        put(p, 23, 9, "ice")
        put(p, 7, 23, "paper")
    elif slug == "lightning":
        polygon(p, [(17, 3), (8, 17), (14, 17), (11, 30), (25, 12), (18, 12)], "yellow")
        polygon(p, [(18, 5), (13, 15), (18, 15), (15, 24), (22, 11), (18, 11)], "white")
        line(p, 8, 7, 5, 4, "sky", 1)
        line(p, 25, 23, 29, 26, "sky", 1)
        line(p, 6, 24, 3, 27, "blue", 1)
    elif slug == "planet":
        circle(p, 16, 16, 9, "deep_blue")
        circle(p, 16, 16, 7, "blue")
        ellipse(p, 13, 14, 4, 3, "green")
        ellipse(p, 20, 19, 5, 3, "lime")
        ellipse(p, 16, 16, 13, 4, "stone")
        ellipse(p, 16, 16, 11, 2, "paper")
        rect(p, 9, 12, 12, 15, "sky")
        put(p, 24, 9, "white")
    elif slug == "space":
        for x, y, color in [(6, 6, "white"), (14, 5, "yellow"), (26, 8, "ice"), (9, 18, "sky"), (23, 22, "white"), (17, 27, "violet")]:
            circle(p, x, y, 1, color)
        circle(p, 16, 16, 6, "deep_blue")
        circle(p, 17, 15, 4, "blue")
        line(p, 5, 25, 28, 9, "violet", 1)
        line(p, 6, 26, 29, 10, "sky", 1)
        rect(p, 13, 14, 19, 17, "white")
    elif slug == "electricity":
        polygon(p, [(17, 3), (8, 17), (14, 17), (11, 30), (25, 12), (18, 12)], "yellow")
        polygon(p, [(18, 5), (13, 15), (18, 15), (15, 24), (22, 11), (18, 11)], "white")
        line(p, 6, 8, 2, 5, "sky", 1)
        line(p, 25, 24, 30, 28, "sky", 1)
        line(p, 6, 25, 2, 29, "blue", 1)
        rect(p, 20, 7, 22, 10, "gold")
    elif slug == "wire":
        line(p, 5, 20, 11, 14, "red", 3)
        line(p, 11, 14, 18, 20, "red", 3)
        line(p, 18, 20, 27, 10, "red", 3)
        line(p, 5, 24, 12, 17, "blue", 3)
        line(p, 12, 17, 19, 24, "blue", 3)
        line(p, 19, 24, 28, 14, "blue", 3)
        rect(p, 3, 19, 7, 23, "gold")
        rect(p, 25, 9, 29, 14, "gold")
        rect(p, 3, 23, 7, 27, "ice")
        rect(p, 26, 13, 30, 18, "ice")
    elif slug == "light-bulb":
        circle(p, 16, 13, 8, "yellow")
        circle(p, 14, 10, 3, "white")
        rect(p, 12, 20, 21, 23, "stone")
        rect(p, 13, 23, 20, 26, "dark_stone")
        rect(p, 14, 26, 19, 28, "gold")
        line(p, 8, 4, 5, 1, "yellow", 1)
        line(p, 24, 4, 27, 1, "yellow", 1)
        line(p, 16, 2, 16, 0, "yellow", 1)
    elif slug == "solar-system":
        circle(p, 16, 16, 5, "yellow")
        ellipse(p, 16, 16, 14, 5, "stone")
        ellipse(p, 16, 16, 10, 3, "violet")
        circle(p, 5, 15, 2, "blue")
        circle(p, 26, 17, 2, "red")
        circle(p, 20, 9, 1, "white")
        circle(p, 12, 23, 1, "ice")
        line(p, 4, 26, 28, 5, "sky", 1)
    elif slug == "galaxy":
        ellipse(p, 16, 16, 13, 6, "violet")
        ellipse(p, 16, 16, 10, 4, "blue")
        circle(p, 16, 16, 4, "white")
        line(p, 4, 23, 28, 9, "sky", 2)
        line(p, 5, 9, 27, 23, "deep_blue", 1)
        for x, y in [(7, 6), (24, 6), (5, 18), (26, 19), (12, 27), (21, 26)]:
            put(p, x, y, "white")
    elif slug == "telescope":
        line(p, 9, 19, 24, 10, "dark_stone", 4)
        line(p, 10, 18, 24, 10, "ice", 2)
        rect(p, 22, 8, 28, 12, "stone")
        rect(p, 6, 18, 12, 22, "blue")
        line(p, 14, 20, 10, 29, "dark_dirt", 2)
        line(p, 14, 20, 18, 29, "dark_dirt", 2)
        line(p, 14, 20, 14, 29, "dirt", 1)
        circle(p, 14, 20, 2, "gold")
    elif slug == "rocket":
        polygon(p, [(16, 3), (22, 11), (20, 24), (12, 24), (10, 11)], "stone")
        polygon(p, [(16, 5), (20, 12), (18, 22), (14, 22), (12, 12)], "white")
        circle(p, 16, 13, 3, "sky")
        polygon(p, [(11, 20), (5, 27), (12, 25)], "red")
        polygon(p, [(21, 20), (27, 27), (20, 25)], "red")
        polygon(p, [(14, 24), (18, 24), (16, 31)], "orange")
        polygon(p, [(15, 24), (17, 24), (16, 29)], "yellow")
    elif slug == "astronaut":
        circle(p, 16, 9, 6, "white")
        circle(p, 16, 9, 4, "deep_blue")
        rect(p, 11, 15, 22, 24, "white")
        rect(p, 13, 17, 20, 22, "ice")
        rect(p, 9, 17, 12, 25, "stone")
        rect(p, 21, 17, 24, 25, "stone")
        rect(p, 12, 24, 15, 30, "white")
        rect(p, 18, 24, 21, 30, "white")
        rect(p, 14, 18, 19, 20, "blue")
        line(p, 8, 16, 5, 12, "sky", 1)
    elif slug == "earthquake":
        polygon(p, [(4, 18), (12, 13), (27, 15), (30, 24), (22, 29), (7, 27)], "dark_dirt")
        polygon(p, [(5, 18), (13, 15), (25, 16), (28, 23), (21, 26), (8, 25)], "dirt")
        line(p, 14, 15, 12, 20, "ink", 2)
        line(p, 12, 20, 17, 23, "ink", 2)
        line(p, 17, 23, 15, 28, "ink", 2)
        rect(p, 5, 25, 12, 28, "stone")
        rect(p, 19, 14, 25, 16, "paper")
        circle(p, 8, 12, 2, "stone")
        circle(p, 25, 11, 2, "paper")
        line(p, 4, 10, 9, 8, "gold", 1)
        line(p, 23, 6, 29, 8, "gold", 1)
    elif slug == "flood":
        rect(p, 3, 17, 30, 29, "deep_blue")
        rect(p, 3, 17, 30, 22, "blue")
        wave(p, 4, 18, "sky")
        wave(p, 7, 24, "ice")
        polygon(p, [(10, 13), (16, 8), (23, 13)], "red")
        rect(p, 12, 13, 21, 20, "paper")
        rect(p, 15, 15, 18, 20, "dark_dirt")
        rect(p, 20, 16, 23, 20, "sky")
        line(p, 6, 14, 3, 20, "dark_dirt", 2)
        circle(p, 5, 12, 3, "green")
    elif slug == "geyser":
        ellipse(p, 16, 25, 10, 4, "dark_stone")
        ellipse(p, 16, 24, 7, 3, "stone")
        rect(p, 14, 10, 19, 25, "sky")
        rect(p, 15, 5, 18, 14, "ice")
        rect(p, 12, 8, 15, 16, "white")
        rect(p, 19, 7, 22, 15, "white")
        circle(p, 11, 5, 2, "ice")
        circle(p, 22, 4, 2, "white")
        circle(p, 8, 11, 1, "sky")
        circle(p, 25, 12, 1, "sky")
    elif slug == "granite":
        polygon(p, [(8, 12), (15, 6), (25, 10), (29, 20), (23, 28), (10, 27), (4, 20)], "dark_stone")
        polygon(p, [(10, 13), (16, 8), (24, 11), (27, 19), (22, 25), (11, 24), (7, 19)], "stone")
        polygon(p, [(14, 10), (21, 12), (17, 18), (10, 18)], "ice")
        line(p, 9, 22, 23, 13, "dark_stone", 1)
        line(p, 13, 25, 25, 20, "paper", 1)
        rect(p, 16, 11, 20, 13, "white")
    elif slug == "gunpowder":
        ellipse(p, 15, 22, 10, 5, "ink")
        ellipse(p, 15, 21, 8, 4, "dark_stone")
        for x, y in [(8, 17), (12, 15), (18, 16), (23, 18), (10, 24), (20, 24)]:
            circle(p, x, y, 1, "stone")
        line(p, 21, 16, 27, 8, "dark_dirt", 1)
        circle(p, 28, 7, 2, "orange")
        rect(p, 27, 5, 29, 7, "yellow")
        line(p, 24, 6, 22, 3, "red", 1)
    elif slug == "obsidian":
        polygon(p, [(15, 3), (25, 10), (23, 24), (15, 30), (7, 22), (9, 9)], "ink")
        polygon(p, [(16, 5), (22, 11), (20, 23), (15, 27), (10, 21), (11, 10)], "deep_blue")
        polygon(p, [(14, 7), (18, 10), (15, 22), (11, 20), (11, 11)], "dark_stone")
        line(p, 18, 8, 21, 18, "violet", 1)
        rect(p, 13, 9, 16, 12, "white")
        rect(p, 18, 22, 20, 25, "sky")
    elif slug == "ocean":
        rect(p, 3, 14, 30, 28, "deep_blue")
        rect(p, 3, 14, 30, 20, "blue")
        for y, color in [(15, "sky"), (19, "ice"), (23, "sky"), (27, "white")]:
            wave(p, 4, y, color)
        polygon(p, [(22, 6), (28, 11), (22, 11)], "paper")
        rect(p, 20, 6, 22, 19, "dark_dirt")
        rect(p, 18, 25, 27, 29, "blue")
    elif slug == "salt":
        ellipse(p, 16, 25, 11, 3, "stone")
        polygon(p, [(7, 24), (12, 13), (16, 24)], "ice")
        polygon(p, [(13, 24), (18, 9), (23, 24)], "white")
        polygon(p, [(20, 24), (25, 16), (29, 24)], "paper")
        line(p, 12, 15, 15, 23, "sky", 1)
        line(p, 18, 11, 21, 23, "ice", 1)
        rect(p, 15, 13, 18, 16, "white")
        put(p, 8, 11, "white")
        put(p, 25, 9, "ice")
    elif slug == "algae":
        rect(p, 3, 23, 30, 29, "deep_blue")
        for cx, cy, rx, ry, color in [
            (9, 19, 5, 4, "dark_green"),
            (16, 17, 6, 5, "green"),
            (23, 20, 5, 4, "lime"),
            (13, 24, 5, 3, "green"),
        ]:
            ellipse(p, cx, cy, rx, ry, color)
        wave(p, 4, 24, "sky")
        circle(p, 10, 16, 1, "lime")
        circle(p, 20, 15, 1, "white")
        line(p, 6, 28, 26, 28, "blue", 1)
    elif slug == "ash":
        ellipse(p, 16, 24, 11, 4, "dark_stone")
        ellipse(p, 15, 22, 8, 3, "stone")
        circle(p, 10, 19, 2, "paper")
        circle(p, 20, 18, 2, "ice")
        rect(p, 8, 23, 25, 27, "stone")
        line(p, 11, 10, 8, 4, "dark_stone", 1)
        line(p, 16, 13, 17, 6, "stone", 1)
        line(p, 22, 11, 25, 5, "dark_stone", 1)
        put(p, 14, 20, "white")
    elif slug == "eruption":
        polygon(p, [(7, 28), (14, 11), (18, 11), (27, 28)], "dark_dirt")
        polygon(p, [(10, 27), (15, 13), (17, 13), (24, 27)], "dirt")
        rect(p, 14, 7, 19, 15, "orange")
        polygon(p, [(16, 3), (23, 12), (10, 12)], "red")
        polygon(p, [(16, 5), (20, 11), (13, 11)], "yellow")
        line(p, 12, 16, 9, 26, "orange", 2)
        line(p, 19, 15, 22, 27, "red", 2)
        circle(p, 8, 8, 2, "stone")
        circle(p, 25, 7, 2, "dark_stone")
    elif slug == "explosion":
        polygon(p, [(16, 2), (19, 11), (29, 6), (23, 15), (31, 19), (21, 20), (24, 30), (16, 23), (8, 30), (11, 20), (1, 19), (9, 15), (3, 6), (13, 11)], "orange")
        polygon(p, [(16, 6), (18, 13), (24, 11), (20, 16), (26, 18), (19, 19), (20, 25), (16, 21), (12, 25), (13, 19), (6, 18), (12, 16), (8, 11), (14, 13)], "yellow")
        circle(p, 16, 17, 5, "white")
        circle(p, 16, 17, 3, "gold")
        put(p, 5, 26, "red")
        put(p, 27, 26, "red")
    elif slug == "fog":
        for y, color in [(10, "ice"), (14, "white"), (18, "sky"), (22, "ice")]:
            line(p, 5, y, 25, y, color, 2)
            line(p, 24, y, 28, y - 2, color, 2)
        ellipse(p, 10, 20, 6, 3, "stone")
        ellipse(p, 21, 13, 5, 3, "white")
        rect(p, 7, 24, 25, 26, "dark_stone")
        put(p, 14, 8, "white")
    elif slug == "hurricane":
        ellipse(p, 16, 16, 13, 9, "deep_blue")
        ellipse(p, 16, 16, 10, 7, "blue")
        ellipse(p, 16, 16, 6, 4, "sky")
        circle(p, 16, 16, 2, "ink")
        line(p, 5, 15, 14, 11, "white", 2)
        line(p, 18, 20, 29, 17, "white", 2)
        line(p, 10, 23, 4, 28, "ice", 1)
        line(p, 22, 9, 28, 4, "ice", 1)
    elif slug == "tsunami":
        polygon(p, [(4, 26), (4, 17), (12, 8), (22, 6), (28, 11), (24, 18), (30, 26)], "deep_blue")
        polygon(p, [(6, 25), (7, 18), (14, 11), (22, 9), (26, 12), (21, 16), (27, 25)], "blue")
        polygon(p, [(17, 9), (25, 11), (20, 15), (15, 14)], "white")
        rect(p, 7, 22, 28, 28, "sky")
        wave(p, 8, 23, "ice")
        wave(p, 6, 27, "white")
    elif slug == "wave":
        polygon(p, [(5, 25), (6, 17), (13, 11), (22, 9), (28, 15), (24, 22), (29, 25)], "blue")
        polygon(p, [(8, 24), (9, 18), (15, 13), (22, 12), (25, 15), (21, 19), (26, 24)], "sky")
        polygon(p, [(18, 11), (25, 14), (21, 17), (16, 16)], "white")
        rect(p, 7, 24, 28, 28, "deep_blue")
        wave(p, 8, 23, "ice")
        line(p, 6, 28, 27, 28, "white", 1)
    elif slug == "wall":
        rect(p, 5, 9, 28, 28, "red")
        for y in [9, 14, 19, 24]:
            line(p, 5, y, 28, y, "dark_dirt", 1)
        for y, offset in [(10, 0), (15, 4), (20, 0), (25, 4)]:
            for x in range(5 + offset, 28, 8):
                line(p, x, y, x, y + 4, "dark_dirt", 1)
        rect(p, 7, 10, 14, 13, "orange")
        rect(p, 17, 15, 25, 18, "paper")
        rect(p, 9, 21, 16, 23, "orange")
        rect(p, 18, 25, 26, 27, "paper")
    elif slug == "archipelago":
        rect(p, 3, 18, 30, 28, "deep_blue")
        wave(p, 4, 20, "sky")
        wave(p, 8, 25, "ice")
        for points in [
            [(6, 17), (10, 12), (15, 17)],
            [(15, 21), (20, 15), (26, 21)],
            [(7, 26), (11, 22), (16, 26)],
        ]:
            polygon(p, points, "gold")
            polygon(p, [(x, y + 2) for x, y in points], "dirt")
        circle(p, 10, 13, 2, "green")
        circle(p, 21, 16, 2, "lime")
    elif slug == "atomic-bomb":
        circle(p, 16, 20, 7, "dark_stone")
        circle(p, 16, 20, 5, "stone")
        polygon(p, [(16, 3), (20, 14), (12, 14)], "yellow")
        polygon(p, [(16, 5), (18, 13), (14, 13)], "white")
        rect(p, 14, 12, 19, 20, "orange")
        line(p, 8, 9, 5, 5, "yellow", 1)
        line(p, 24, 9, 28, 5, "yellow", 1)
        rect(p, 12, 24, 21, 27, "ink")
        circle(p, 16, 20, 2, "red")
    elif slug == "beach":
        rect(p, 4, 19, 29, 29, "gold")
        rect(p, 4, 10, 29, 20, "blue")
        wave(p, 4, 18, "white")
        wave(p, 7, 14, "sky")
        circle(p, 23, 7, 4, "yellow")
        rect(p, 9, 22, 14, 24, "paper")
        rect(p, 18, 25, 25, 27, "dirt")
        put(p, 11, 18, "green")
    elif slug == "boiler":
        rect(p, 8, 10, 25, 26, "dark_stone")
        rect(p, 10, 12, 23, 24, "stone")
        circle(p, 16, 18, 5, "deep_blue")
        circle(p, 16, 18, 3, "sky")
        rect(p, 12, 6, 21, 10, "dark_dirt")
        rect(p, 11, 26, 14, 29, "dark_dirt")
        rect(p, 20, 26, 23, 29, "dark_dirt")
        line(p, 25, 15, 29, 15, "gold", 2)
        line(p, 12, 4, 9, 1, "white", 1)
        line(p, 18, 4, 19, 1, "ice", 1)
    elif slug == "bullet":
        polygon(p, [(7, 18), (20, 9), (28, 13), (24, 19), (10, 24)], "gold")
        polygon(p, [(9, 18), (20, 11), (25, 13), (22, 17), (11, 21)], "yellow")
        rect(p, 6, 18, 11, 24, "dark_dirt")
        rect(p, 8, 19, 12, 22, "paper")
        line(p, 13, 18, 23, 13, "white", 1)
        rect(p, 23, 13, 26, 15, "orange")
    elif slug == "cactus":
        rect(p, 14, 8, 19, 28, "green")
        rect(p, 15, 9, 18, 27, "lime")
        rect(p, 7, 16, 12, 21, "green")
        rect(p, 9, 12, 12, 18, "green")
        rect(p, 21, 14, 26, 19, "dark_green")
        rect(p, 21, 10, 24, 16, "green")
        rect(p, 11, 27, 23, 30, "dirt")
        for x, y in [(16, 12), (17, 18), (15, 24), (10, 15), (23, 13)]:
            put(p, x, y, "white")
        circle(p, 17, 7, 2, "red")
    elif slug == "desert":
        rect(p, 3, 18, 30, 29, "gold")
        polygon(p, [(3, 23), (12, 16), (22, 23)], "paper")
        polygon(p, [(12, 24), (22, 17), (31, 25)], "dirt")
        circle(p, 24, 7, 4, "yellow")
        rect(p, 8, 15, 10, 24, "green")
        rect(p, 6, 18, 12, 20, "green")
        wave(p, 5, 27, "paper")
        line(p, 14, 21, 24, 18, "orange", 1)
    elif slug == "dew":
        rect(p, 6, 23, 27, 27, "green")
        for x in [8, 13, 18, 23]:
            line(p, x, 25, x + 3, 18, "lime", 2)
        for cx, cy in [(10, 17), (16, 14), (22, 17)]:
            circle(p, cx, cy, 3, "sky")
            circle(p, cx - 1, cy - 1, 1, "white")
        line(p, 6, 27, 27, 27, "dark_green", 1)
    elif slug == "diamond":
        polygon(p, [(8, 10), (14, 5), (22, 5), (28, 11), (18, 29)], "deep_blue")
        polygon(p, [(9, 11), (15, 7), (21, 7), (26, 12), (18, 26)], "sky")
        polygon(p, [(15, 7), (18, 12), (21, 7), (18, 26)], "ice")
        polygon(p, [(10, 11), (18, 12), (15, 7)], "white")
        line(p, 10, 12, 26, 12, "white", 1)
        line(p, 14, 6, 18, 12, "deep_blue", 1)
        line(p, 22, 6, 18, 12, "blue", 1)
    elif slug == "dune":
        rect(p, 3, 23, 30, 29, "gold")
        polygon(p, [(3, 24), (15, 11), (29, 24)], "paper")
        polygon(p, [(10, 24), (22, 15), (31, 25)], "dirt")
        line(p, 8, 22, 25, 18, "gold", 1)
        line(p, 5, 27, 28, 26, "paper", 1)
        circle(p, 24, 8, 3, "yellow")
        rect(p, 21, 7, 27, 9, "orange")
    elif slug == "fireworks":
        for cx, cy, color in [(10, 11, "red"), (22, 10, "sky"), (16, 22, "yellow")]:
            circle(p, cx, cy, 2, "white")
            for x, y in [(cx, cy - 6), (cx + 6, cy), (cx, cy + 6), (cx - 6, cy), (cx + 4, cy + 4), (cx - 4, cy - 4)]:
                line(p, cx, cy, x, y, color, 1)
        put(p, 5, 24, "violet")
        put(p, 27, 22, "gold")
    elif slug == "garden":
        rect(p, 5, 22, 28, 29, "dirt")
        for x in [8, 13, 18, 23]:
            rect(p, x, 16, x + 2, 25, "green")
            circle(p, x + 1, 14, 3, "lime")
        circle(p, 9, 13, 1, "red")
        circle(p, 14, 12, 1, "yellow")
        circle(p, 19, 14, 1, "violet")
        circle(p, 24, 13, 1, "white")
        line(p, 6, 26, 27, 26, "paper", 1)
    elif slug == "ivy":
        line(p, 13, 4, 12, 28, "dark_green", 2)
        line(p, 18, 6, 20, 29, "green", 2)
        for cx, cy in [(11, 9), (15, 13), (11, 18), (16, 23), (21, 10), (18, 16), (23, 22)]:
            polygon(p, [(cx, cy - 3), (cx + 4, cy), (cx, cy + 3), (cx - 4, cy)], "green")
            polygon(p, [(cx, cy - 2), (cx + 2, cy), (cx, cy + 2), (cx - 2, cy)], "lime")
        rect(p, 9, 28, 23, 30, "dark_dirt")
    elif slug == "moss":
        ellipse(p, 16, 23, 12, 5, "dark_green")
        ellipse(p, 12, 20, 7, 4, "green")
        ellipse(p, 21, 20, 7, 4, "lime")
        rect(p, 7, 24, 25, 28, "green")
        circle(p, 9, 18, 1, "white")
        circle(p, 19, 16, 1, "yellow")
        circle(p, 25, 22, 1, "lime")
        line(p, 6, 28, 26, 28, "dark_dirt", 1)
    elif slug == "pond":
        ellipse(p, 16, 21, 13, 7, "deep_blue")
        ellipse(p, 16, 20, 11, 5, "blue")
        wave(p, 6, 19, "sky")
        wave(p, 9, 23, "ice")
        ellipse(p, 9, 17, 4, 2, "green")
        ellipse(p, 23, 18, 4, 2, "lime")
        circle(p, 10, 16, 1, "white")
        rect(p, 5, 27, 27, 29, "dirt")
    elif slug == "aquarium":
        rect(p, 6, 8, 27, 26, "deep_blue")
        rect(p, 7, 9, 26, 25, "sky")
        rect(p, 8, 20, 25, 25, "blue")
        rect(p, 8, 24, 25, 27, "gold")
        line(p, 6, 8, 27, 8, "white", 1)
        line(p, 6, 8, 6, 26, "ice", 1)
        line(p, 27, 8, 27, 26, "stone", 1)
        ellipse(p, 15, 18, 5, 3, "orange")
        polygon(p, [(20, 18), (25, 14), (25, 22)], "gold")
        circle(p, 12, 17, 1, "white")
        line(p, 10, 24, 10, 16, "green", 1)
        line(p, 23, 24, 21, 15, "lime", 1)
        for bx, by in [(18, 13), (21, 11), (23, 16)]:
            circle(p, bx, by, 1, "white")
    elif slug == "blender":
        rect(p, 11, 8, 22, 23, "deep_blue")
        polygon(p, [(10, 8), (23, 8), (21, 23), (12, 23)], "ice")
        polygon(p, [(12, 10), (21, 10), (20, 21), (13, 21)], "sky")
        rect(p, 12, 19, 21, 22, "blue")
        line(p, 14, 17, 20, 17, "stone", 1)
        line(p, 17, 14, 17, 20, "white", 1)
        rect(p, 9, 23, 25, 29, "dark_stone")
        rect(p, 11, 25, 23, 28, "red")
        rect(p, 15, 5, 19, 8, "stone")
        rect(p, 13, 7, 21, 9, "white")
        circle(p, 21, 26, 1, "yellow")
    elif slug == "bridge":
        rect(p, 4, 22, 29, 27, "deep_blue")
        wave(p, 5, 23, "sky")
        polygon(p, [(4, 20), (10, 12), (16, 10), (23, 12), (29, 20)], "dark_dirt")
        polygon(p, [(6, 19), (11, 14), (16, 12), (22, 14), (27, 19)], "dirt")
        ellipse(p, 16, 20, 7, 6, "deep_blue")
        rect(p, 5, 19, 28, 22, "gold")
        for x in [7, 12, 20, 25]:
            rect(p, x, 18, x + 2, 26, "dark_dirt")
        line(p, 6, 17, 27, 17, "paper", 1)
    elif slug == "dam":
        rect(p, 5, 9, 18, 27, "dark_stone")
        rect(p, 7, 10, 19, 26, "stone")
        for y in [13, 17, 21]:
            line(p, 6, y, 20, y, "dark_stone", 1)
        for x in [10, 15]:
            line(p, x, 10, x - 1, 26, "white", 1)
        rect(p, 19, 13, 28, 27, "deep_blue")
        line(p, 20, 14, 28, 17, "sky", 2)
        line(p, 20, 19, 28, 22, "ice", 2)
        wave(p, 19, 26, "white")
    elif slug == "day":
        rect(p, 5, 8, 27, 22, "sky")
        circle(p, 22, 11, 5, "yellow")
        circle(p, 22, 11, 3, "white")
        draw_cloud(p, 12, 14)
        rect(p, 5, 22, 27, 27, "green")
        line(p, 6, 22, 26, 18, "lime", 1)
        rect(p, 7, 25, 26, 28, "dark_green")
    elif slug == "eclipse":
        circle(p, 16, 16, 10, "gold")
        circle(p, 16, 16, 7, "yellow")
        circle(p, 13, 16, 8, "ink")
        circle(p, 12, 15, 5, "dark_stone")
        for x, y in [(16, 3), (28, 15), (16, 29), (4, 17)]:
            line(p, 16, 16, x, y, "orange", 1)
        rect(p, 18, 9, 21, 12, "white")
    elif slug == "gold":
        polygon(p, [(7, 18), (16, 14), (27, 18), (24, 25), (10, 25)], "gold")
        polygon(p, [(9, 17), (16, 14), (24, 17), (21, 21), (12, 21)], "yellow")
        rect(p, 6, 23, 18, 28, "orange")
        rect(p, 14, 21, 27, 27, "gold")
        rect(p, 10, 24, 16, 26, "yellow")
        rect(p, 17, 22, 24, 24, "white")
        line(p, 7, 28, 27, 28, "dark_dirt", 1)
    elif slug == "golem":
        rect(p, 12, 7, 21, 15, "dark_stone")
        rect(p, 10, 15, 23, 25, "stone")
        rect(p, 8, 17, 12, 25, "dark_stone")
        rect(p, 22, 17, 26, 25, "dark_stone")
        rect(p, 11, 25, 15, 30, "dark_stone")
        rect(p, 18, 25, 22, 30, "dark_stone")
        rect(p, 14, 10, 16, 12, "yellow")
        rect(p, 18, 10, 20, 12, "yellow")
        line(p, 12, 16, 22, 24, "white", 1)
        rect(p, 13, 17, 20, 20, "stone")
    elif slug == "greenhouse":
        rect(p, 6, 15, 27, 27, "dark_green")
        polygon(p, [(5, 16), (16, 6), (28, 16)], "ice")
        polygon(p, [(8, 16), (16, 9), (25, 16)], "sky")
        rect(p, 8, 16, 25, 25, "ice")
        line(p, 16, 7, 16, 26, "white", 1)
        line(p, 7, 16, 26, 16, "white", 1)
        line(p, 10, 13, 23, 13, "sky", 1)
        for x in [10, 15, 21]:
            rect(p, x, 20, x + 2, 26, "green")
            circle(p, x + 1, 18, 3, "lime")
        rect(p, 6, 26, 27, 29, "dirt")
    elif slug == "gun":
        rect(p, 7, 13, 24, 17, "dark_stone")
        rect(p, 23, 14, 29, 16, "stone")
        rect(p, 11, 17, 17, 20, "dark_stone")
        polygon(p, [(14, 20), (20, 20), (18, 28), (12, 28)], "dark_dirt")
        rect(p, 8, 11, 14, 13, "stone")
        rect(p, 18, 17, 22, 19, "gold")
        rect(p, 25, 12, 28, 14, "white")
        line(p, 7, 18, 12, 18, "ice", 1)
    elif slug == "hourglass":
        rect(p, 10, 5, 23, 8, "gold")
        rect(p, 10, 25, 23, 28, "gold")
        polygon(p, [(11, 8), (22, 8), (18, 16), (15, 16)], "ice")
        polygon(p, [(15, 16), (18, 16), (22, 25), (11, 25)], "paper")
        polygon(p, [(14, 10), (19, 10), (17, 15), (16, 15)], "gold")
        polygon(p, [(13, 24), (20, 24), (18, 19), (15, 19)], "gold")
        line(p, 10, 8, 22, 25, "dark_dirt", 1)
        line(p, 23, 8, 11, 25, "dark_dirt", 1)
        rect(p, 15, 16, 18, 19, "yellow")
    elif slug == "mirror":
        ellipse(p, 16, 14, 8, 10, "gold")
        ellipse(p, 16, 14, 6, 8, "deep_blue")
        ellipse(p, 15, 13, 5, 7, "sky")
        line(p, 12, 9, 21, 19, "white", 1)
        rect(p, 14, 23, 19, 29, "dark_dirt")
        rect(p, 12, 28, 21, 30, "gold")
        rect(p, 12, 6, 20, 8, "yellow")
        put(p, 21, 11, "ice")
    elif slug == "night":
        rect(p, 5, 6, 27, 25, "deep_blue")
        circle(p, 12, 12, 6, "ice")
        circle(p, 15, 10, 6, "deep_blue")
        for x, y, color in [(22, 8, "white"), (25, 13, "yellow"), (18, 18, "ice"), (8, 20, "white")]:
            put(p, x, y, color)
            put(p, x + 1, y, color)
        rect(p, 5, 24, 27, 28, "dark_stone")
        line(p, 7, 24, 25, 21, "violet", 1)
    elif slug == "oasis":
        rect(p, 3, 22, 30, 29, "gold")
        ellipse(p, 16, 22, 10, 5, "deep_blue")
        ellipse(p, 16, 21, 8, 3, "sky")
        wave(p, 9, 22, "white")
        line(p, 9, 22, 13, 10, "dirt", 3)
        for x1, y1 in [(6, 9), (12, 7), (16, 10), (10, 13)]:
            line(p, 13, 10, x1, y1, "green", 2)
            ellipse(p, x1, y1, 3, 2, "lime")
        polygon(p, [(20, 24), (27, 17), (31, 24)], "paper")
        rect(p, 5, 27, 28, 29, "dirt")
    elif slug == "oxygen":
        circle(p, 12, 16, 6, "sky")
        circle(p, 21, 16, 6, "ice")
        circle(p, 12, 16, 4, "blue")
        circle(p, 21, 16, 4, "sky")
        circle(p, 12, 16, 2, "white")
        circle(p, 21, 16, 2, "white")
        line(p, 16, 16, 17, 16, "white", 2)
        rect(p, 9, 24, 12, 27, "sky")
        rect(p, 14, 25, 19, 27, "white")
        rect(p, 21, 24, 24, 27, "ice")
        put(p, 26, 10, "white")
    elif slug == "plankton":
        rect(p, 4, 22, 28, 28, "deep_blue")
        wave(p, 5, 23, "sky")
        for cx, cy, color in [(10, 17, "lime"), (17, 14, "green"), (23, 18, "sky")]:
            ellipse(p, cx, cy, 3, 5, color)
            circle(p, cx, cy - 1, 1, "white")
            line(p, cx, cy + 4, cx - 4, cy + 8, color, 1)
            line(p, cx, cy + 4, cx + 4, cy + 8, color, 1)
        circle(p, 7, 11, 1, "ice")
        circle(p, 26, 13, 1, "lime")
    elif slug == "airplane":
        rect(p, 8, 14, 25, 18, "ice")
        polygon(p, [(24, 14), (30, 16), (24, 18)], "white")
        polygon(p, [(13, 14), (7, 7), (18, 14)], "sky")
        polygon(p, [(14, 18), (8, 25), (20, 18)], "blue")
        polygon(p, [(8, 14), (4, 12), (8, 18)], "stone")
        rect(p, 19, 12, 23, 14, "deep_blue")
        rect(p, 18, 18, 21, 20, "dark_stone")
        line(p, 10, 16, 25, 16, "white", 1)
    elif slug == "bank":
        rect(p, 7, 14, 26, 27, "stone")
        polygon(p, [(5, 14), (16, 6), (28, 14)], "gold")
        rect(p, 8, 12, 25, 15, "paper")
        for x in [9, 14, 19, 24]:
            rect(p, x, 15, x + 2, 25, "white")
            rect(p, x - 1, 25, x + 3, 27, "dark_stone")
        rect(p, 6, 27, 28, 30, "dark_stone")
        rect(p, 14, 9, 19, 12, "yellow")
    elif slug == "castle":
        rect(p, 7, 13, 26, 28, "dark_stone")
        rect(p, 9, 15, 24, 28, "stone")
        for x in [7, 13, 20]:
            rect(p, x, 8, x + 6, 16, "dark_stone")
            rect(p, x + 1, 10, x + 5, 16, "stone")
            rect(p, x, 7, x + 2, 9, "stone")
            rect(p, x + 4, 7, x + 6, 9, "stone")
        polygon(p, [(13, 28), (16, 20), (20, 28)], "ink")
        rect(p, 11, 17, 14, 20, "sky")
        rect(p, 21, 17, 24, 20, "sky")
    elif slug == "city":
        rect(p, 4, 14, 10, 28, "dark_stone")
        rect(p, 11, 9, 17, 28, "stone")
        rect(p, 18, 12, 26, 28, "dark_stone")
        rect(p, 24, 17, 29, 28, "stone")
        for x in [6, 13, 20, 25]:
            for y in [16, 20, 24]:
                rect(p, x, y, x + 2, y + 2, "yellow")
        rect(p, 3, 28, 30, 30, "ink")
        line(p, 11, 9, 14, 5, "ice", 1)
    elif slug == "farm":
        rect(p, 4, 22, 30, 29, "green")
        rect(p, 6, 17, 19, 28, "red")
        polygon(p, [(5, 17), (12, 10), (20, 17)], "dark_dirt")
        rect(p, 10, 21, 15, 28, "paper")
        rect(p, 21, 18, 28, 28, "gold")
        line(p, 21, 20, 28, 26, "paper", 1)
        for x in [5, 9, 13, 17, 23, 27]:
            line(p, x, 29, x + 4, 23, "lime", 1)
    elif slug == "farmer":
        circle(p, 16, 9, 4, "paper")
        rect(p, 11, 6, 21, 8, "gold")
        rect(p, 9, 8, 23, 10, "yellow")
        rect(p, 12, 13, 20, 23, "blue")
        rect(p, 10, 15, 13, 23, "paper")
        rect(p, 19, 15, 22, 23, "paper")
        rect(p, 12, 23, 15, 29, "dark_dirt")
        rect(p, 17, 23, 20, 29, "dark_dirt")
        line(p, 22, 13, 27, 27, "dirt", 2)
        polygon(p, [(25, 26), (30, 25), (27, 30)], "stone")
    elif slug == "field":
        rect(p, 3, 19, 30, 29, "gold")
        for x in range(5, 29, 4):
            line(p, x, 29, x + 3, 17, "yellow", 1)
            ellipse(p, x + 1, 18, 2, 3, "paper")
        rect(p, 3, 24, 30, 27, "dirt")
        line(p, 4, 22, 29, 19, "paper", 1)
        circle(p, 24, 8, 4, "yellow")
        draw_cloud(p, 10, 8)
    elif slug == "forest":
        rect(p, 4, 25, 29, 29, "dark_dirt")
        for cx, cy, size in [(8, 17, 6), (14, 14, 7), (21, 16, 6), (26, 19, 5)]:
            rect(p, cx - 1, cy + 6, cx + 2, 28, "dirt")
            polygon(p, [(cx, cy - size), (cx - size, cy + 7), (cx + size, cy + 7)], "dark_green")
            polygon(p, [(cx, cy - size + 2), (cx - size + 4, cy + 5), (cx + size - 4, cy + 5)], "green")
        rect(p, 10, 26, 24, 28, "green")
    elif slug == "helicopter":
        ellipse(p, 16, 17, 9, 5, "blue")
        rect(p, 12, 14, 20, 17, "sky")
        polygon(p, [(24, 17), (30, 13), (29, 20)], "deep_blue")
        line(p, 16, 12, 16, 8, "stone", 1)
        line(p, 5, 8, 27, 8, "white", 1)
        line(p, 9, 22, 24, 22, "dark_stone", 1)
        rect(p, 10, 21, 13, 23, "dark_stone")
        rect(p, 20, 21, 23, 23, "dark_stone")
        line(p, 28, 12, 31, 9, "ice", 1)
    elif slug == "hospital":
        rect(p, 6, 10, 27, 28, "white")
        rect(p, 8, 12, 25, 28, "ice")
        rect(p, 14, 16, 19, 27, "red")
        rect(p, 11, 19, 22, 24, "red")
        rect(p, 9, 14, 12, 17, "sky")
        rect(p, 22, 14, 25, 17, "sky")
        rect(p, 9, 24, 12, 27, "sky")
        rect(p, 22, 24, 25, 27, "sky")
        rect(p, 5, 28, 28, 30, "stone")
    elif slug == "lake":
        ellipse(p, 16, 21, 13, 7, "deep_blue")
        ellipse(p, 16, 20, 11, 5, "blue")
        wave(p, 6, 19, "sky")
        wave(p, 8, 23, "ice")
        rect(p, 4, 25, 28, 28, "green")
        polygon(p, [(4, 18), (10, 12), (16, 18)], "dark_green")
        polygon(p, [(22, 18), (28, 12), (31, 18)], "green")
    elif slug == "river":
        polygon(p, [(12, 4), (20, 4), (17, 12), (22, 20), (18, 30), (9, 30), (13, 20), (8, 12)], "deep_blue")
        polygon(p, [(14, 5), (18, 5), (15, 12), (20, 20), (16, 28), (11, 28), (15, 20), (10, 12)], "blue")
        wave(p, 10, 13, "sky")
        wave(p, 12, 22, "ice")
        rect(p, 4, 25, 12, 29, "green")
        rect(p, 20, 23, 29, 29, "dark_green")
    elif slug == "sailboat":
        polygon(p, [(6, 21), (28, 21), (24, 28), (10, 28)], "dark_dirt")
        polygon(p, [(9, 21), (25, 21), (21, 25), (11, 25)], "dirt")
        rect(p, 15, 6, 17, 22, "stone")
        polygon(p, [(17, 7), (27, 19), (17, 19)], "paper")
        polygon(p, [(15, 9), (7, 20), (15, 20)], "ice")
        rect(p, 4, 28, 30, 30, "deep_blue")
        wave(p, 5, 28, "sky")
    elif slug == "swamp":
        rect(p, 3, 21, 30, 29, "dark_green")
        ellipse(p, 16, 23, 12, 4, "deep_blue")
        wave(p, 5, 23, "green")
        for x in [7, 12, 21, 26]:
            line(p, x, 28, x + 2, 15, "green", 2)
            ellipse(p, x + 2, 17, 4, 2, "lime")
        rect(p, 8, 24, 13, 26, "dirt")
        rect(p, 18, 24, 24, 26, "dark_dirt")
        circle(p, 24, 12, 2, "white")
    elif slug == "train":
        rect(p, 5, 15, 26, 24, "red")
        rect(p, 8, 10, 18, 15, "dark_stone")
        rect(p, 19, 12, 25, 15, "stone")
        rect(p, 9, 12, 12, 15, "sky")
        rect(p, 14, 12, 17, 15, "sky")
        rect(p, 22, 16, 27, 20, "gold")
        circle(p, 9, 25, 3, "ink")
        circle(p, 18, 25, 3, "ink")
        circle(p, 24, 25, 3, "ink")
        line(p, 4, 29, 29, 29, "stone", 1)
        rect(p, 4, 20, 28, 22, "orange")
    elif slug == "village":
        rect(p, 4, 24, 30, 29, "green")
        for x, roof, wall in [(6, "red", "paper"), (16, "orange", "dirt"), (24, "red", "paper")]:
            rect(p, x, 18, x + 7, 27, wall)
            polygon(p, [(x - 1, 18), (x + 3, 12), (x + 8, 18)], roof)
            rect(p, x + 2, 22, x + 5, 27, "dark_dirt")
            rect(p, x + 5, 20, x + 7, 22, "sky")
        line(p, 4, 28, 30, 28, "dirt", 1)
    elif slug == "isle":
        ellipse(p, 16, 23, 13, 5, "deep_blue")
        wave(p, 5, 22, "sky")
        ellipse(p, 16, 21, 9, 4, "gold")
        rect(p, 13, 14, 16, 23, "dirt")
        line(p, 15, 15, 10, 9, "green", 2)
        line(p, 15, 15, 20, 9, "lime", 2)
        ellipse(p, 9, 9, 4, 2, "green")
        ellipse(p, 20, 9, 4, 2, "lime")
        circle(p, 24, 6, 3, "yellow")
    elif slug == "grenade":
        circle(p, 16, 18, 8, "dark_green")
        circle(p, 15, 17, 6, "green")
        rect(p, 13, 7, 20, 12, "dark_stone")
        rect(p, 15, 5, 22, 8, "stone")
        line(p, 19, 7, 25, 11, "gold", 1)
        rect(p, 12, 16, 20, 18, "lime")
        rect(p, 14, 21, 22, 23, "dark_green")
        put(p, 8, 12, "white")
    elif slug == "horizon":
        rect(p, 4, 8, 28, 19, "sky")
        rect(p, 4, 19, 28, 28, "deep_blue")
        circle(p, 16, 19, 5, "gold")
        rect(p, 4, 19, 28, 21, "orange")
        wave(p, 5, 23, "ice")
        line(p, 4, 19, 28, 19, "white", 1)
        draw_cloud(p, 9, 8)
    elif slug == "mountain-range":
        polygon(p, [(2, 28), (9, 12), (17, 28)], "dark_stone")
        polygon(p, [(9, 28), (17, 7), (27, 28)], "stone")
        polygon(p, [(18, 28), (25, 14), (31, 28)], "dark_stone")
        polygon(p, [(9, 12), (7, 18), (12, 18)], "white")
        polygon(p, [(17, 7), (14, 15), (21, 15)], "ice")
        polygon(p, [(25, 14), (23, 19), (28, 19)], "white")
        rect(p, 3, 27, 30, 30, "dark_dirt")
    elif slug == "quicksand":
        ellipse(p, 16, 22, 12, 6, "dirt")
        ellipse(p, 16, 21, 10, 4, "gold")
        ellipse(p, 16, 21, 6, 2, "paper")
        for r, color in [(11, "dark_dirt"), (8, "dirt"), (5, "gold")]:
            ellipse(p, 16, 21, r, max(1, r // 3), color)
        rect(p, 14, 12, 19, 22, "dark_dirt")
        rect(p, 15, 10, 18, 13, "paper")
        line(p, 6, 28, 27, 28, "gold", 1)
    elif slug == "rust":
        polygon(p, [(8, 12), (18, 8), (26, 15), (24, 25), (12, 27), (6, 20)], "dark_dirt")
        polygon(p, [(10, 13), (18, 10), (24, 16), (22, 23), (13, 25), (8, 20)], "orange")
        rect(p, 12, 15, 16, 18, "red")
        rect(p, 18, 19, 23, 22, "dirt")
        rect(p, 10, 22, 14, 25, "gold")
        line(p, 8, 12, 24, 25, "dark_stone", 1)
    elif slug == "sandstone":
        polygon(p, [(7, 24), (9, 12), (18, 7), (26, 14), (25, 25), (14, 28)], "dirt")
        polygon(p, [(9, 23), (11, 14), (18, 9), (24, 15), (23, 23), (14, 26)], "gold")
        for y in [14, 18, 22]:
            line(p, 9, y, 25, y + 1, "paper", 1)
        rect(p, 12, 10, 18, 12, "white")
        rect(p, 17, 24, 23, 26, "orange")
    elif slug == "sandstorm":
        rect(p, 3, 22, 30, 29, "gold")
        for y, color in [(9, "paper"), (13, "gold"), (17, "dirt"), (21, "paper")]:
            line(p, 3, y, 24, y, color, 2)
            line(p, 22, y, 29, y + 4, color, 1)
        circle(p, 23, 8, 3, "yellow")
        line(p, 7, 25, 27, 24, "dirt", 1)
    elif slug == "sound":
        rect(p, 7, 16, 12, 22, "stone")
        polygon(p, [(12, 15), (20, 10), (20, 28), (12, 23)], "blue")
        polygon(p, [(14, 16), (18, 14), (18, 24), (14, 22)], "sky")
        for offset, color in [(0, "ice"), (4, "sky"), (8, "blue")]:
            line(p, 22 + offset // 4, 12 - offset // 4, 26 + offset // 4, 16, color, 1)
            line(p, 26 + offset // 4, 16, 22 + offset // 4, 20 + offset // 4, color, 1)
        rect(p, 8, 17, 11, 21, "white")
    elif slug == "steel":
        polygon(p, [(7, 19), (13, 11), (24, 12), (28, 20), (22, 27), (10, 26)], "dark_stone")
        polygon(p, [(10, 18), (14, 14), (23, 15), (25, 20), (20, 24), (12, 23)], "stone")
        rect(p, 13, 15, 23, 18, "ice")
        rect(p, 10, 21, 21, 24, "dark_stone")
        line(p, 12, 16, 24, 22, "white", 1)
        rect(p, 18, 18, 24, 20, "sky")
    elif slug == "perfume":
        rect(p, 12, 12, 22, 26, "violet")
        rect(p, 13, 13, 21, 25, "sky")
        rect(p, 14, 17, 20, 22, "paper")
        rect(p, 14, 8, 20, 12, "gold")
        rect(p, 11, 6, 23, 8, "stone")
        rect(p, 15, 4, 19, 6, "white")
        line(p, 10, 7, 6, 5, "ice", 1)
        line(p, 22, 7, 27, 4, "white", 1)
        circle(p, 9, 4, 1, "sky")
    elif slug == "pyramid":
        polygon(p, [(4, 28), (16, 7), (29, 28)], "dirt")
        polygon(p, [(16, 7), (29, 28), (17, 28)], "gold")
        polygon(p, [(16, 10), (9, 23), (17, 23)], "paper")
        for y in [16, 20, 24]:
            line(p, 8, y, 25, y, "dark_dirt", 1)
        line(p, 16, 8, 16, 28, "orange", 1)
        rect(p, 4, 28, 30, 30, "gold")
    elif slug == "ring":
        circle(p, 16, 19, 9, "gold")
        circle(p, 16, 19, 6, "ink")
        circle(p, 16, 8, 5, "deep_blue")
        polygon(p, [(16, 2), (22, 8), (16, 14), (10, 8)], "sky")
        polygon(p, [(16, 4), (20, 8), (16, 12), (12, 8)], "ice")
        rect(p, 14, 17, 19, 20, "yellow")
        rect(p, 13, 7, 16, 9, "white")
    elif slug == "robot":
        rect(p, 10, 9, 23, 21, "dark_stone")
        rect(p, 12, 11, 21, 19, "stone")
        rect(p, 13, 13, 15, 15, "yellow")
        rect(p, 18, 13, 20, 15, "sky")
        rect(p, 14, 18, 19, 20, "ink")
        rect(p, 8, 21, 25, 28, "blue")
        rect(p, 6, 22, 9, 27, "dark_stone")
        rect(p, 24, 22, 27, 27, "dark_stone")
        line(p, 16, 9, 14, 5, "stone", 1)
        line(p, 17, 9, 21, 5, "stone", 1)
        put(p, 14, 5, "red")
        put(p, 21, 5, "red")
    elif slug == "scythe":
        line(p, 12, 28, 22, 7, "dark_dirt", 3)
        line(p, 13, 28, 23, 7, "dirt", 1)
        polygon(p, [(18, 7), (29, 4), (26, 8), (21, 13), (18, 18), (16, 14)], "dark_stone")
        polygon(p, [(20, 7), (27, 5), (25, 8), (21, 12), (19, 15), (18, 13)], "ice")
        line(p, 20, 7, 25, 16, "stone", 1)
        rect(p, 10, 25, 15, 29, "gold")
    elif slug == "sunflower":
        line(p, 16, 28, 16, 14, "green", 2)
        ellipse(p, 11, 23, 5, 2, "green")
        ellipse(p, 21, 21, 5, 2, "lime")
        for cx, cy in [(16, 7), (16, 17), (9, 12), (23, 12), (11, 8), (21, 8), (11, 16), (21, 16)]:
            ellipse(p, cx, cy, 3, 4 if cx == 16 else 3, "yellow")
        circle(p, 16, 12, 5, "dark_dirt")
        circle(p, 16, 12, 3, "dirt")
        rect(p, 13, 27, 20, 29, "dark_dirt")
    elif slug == "skyscraper":
        rect(p, 9, 6, 23, 29, "dark_stone")
        rect(p, 11, 8, 21, 29, "stone")
        for x in [12, 17]:
            for y in [10, 14, 18, 22]:
                rect(p, x, y, x + 3, y + 2, "yellow")
        rect(p, 14, 25, 18, 29, "ink")
        rect(p, 8, 29, 24, 31, "dark_dirt")
        line(p, 16, 6, 16, 2, "ice", 1)
        polygon(p, [(9, 6), (16, 2), (23, 6)], "dark_stone")
    elif slug == "sword":
        polygon(p, [(16, 3), (21, 8), (18, 23), (14, 23), (11, 8)], "dark_stone")
        polygon(p, [(16, 5), (19, 9), (17, 21), (15, 21), (13, 9)], "ice")
        line(p, 16, 6, 16, 22, "white", 1)
        rect(p, 9, 22, 23, 25, "gold")
        rect(p, 14, 24, 18, 30, "dark_dirt")
        rect(p, 13, 28, 19, 31, "dirt")
    elif slug == "tide":
        rect(p, 4, 20, 29, 28, "deep_blue")
        polygon(p, [(4, 21), (11, 13), (18, 20)], "blue")
        polygon(p, [(11, 20), (20, 9), (29, 20)], "sky")
        polygon(p, [(18, 19), (24, 13), (30, 19)], "ice")
        wave(p, 5, 22, "white")
        wave(p, 4, 26, "sky")
        rect(p, 4, 28, 29, 30, "deep_blue")
    elif slug == "water-lily":
        ellipse(p, 16, 22, 13, 5, "deep_blue")
        ellipse(p, 16, 21, 11, 4, "blue")
        ellipse(p, 11, 18, 6, 3, "green")
        ellipse(p, 22, 20, 5, 2, "lime")
        for cx, cy in [(16, 13), (13, 15), (19, 15), (16, 17)]:
            ellipse(p, cx, cy, 3, 2, "paper")
        circle(p, 16, 15, 2, "yellow")
        wave(p, 7, 24, "sky")
    elif slug == "waterfall":
        polygon(p, [(7, 8), (25, 6), (27, 13), (23, 29), (8, 29), (5, 15)], "dark_stone")
        rect(p, 12, 8, 21, 28, "deep_blue")
        rect(p, 14, 8, 20, 28, "sky")
        line(p, 15, 9, 13, 27, "white", 1)
        line(p, 19, 9, 20, 27, "ice", 1)
        rect(p, 7, 26, 26, 30, "blue")
        wave(p, 7, 28, "white")
        rect(p, 6, 11, 12, 16, "green")
    elif slug == "windmill":
        rect(p, 13, 14, 20, 29, "paper")
        polygon(p, [(12, 14), (16, 7), (21, 14)], "red")
        rect(p, 15, 21, 18, 29, "dark_dirt")
        circle(p, 16, 13, 2, "gold")
        line(p, 16, 13, 16, 3, "white", 2)
        line(p, 16, 13, 27, 13, "ice", 2)
        line(p, 16, 13, 6, 13, "ice", 2)
        line(p, 16, 13, 16, 24, "white", 2)
        rect(p, 8, 29, 25, 31, "green")
    elif slug == "window":
        rect(p, 8, 7, 25, 27, "dark_dirt")
        rect(p, 10, 9, 23, 25, "gold")
        rect(p, 11, 10, 17, 17, "sky")
        rect(p, 18, 10, 22, 17, "ice")
        rect(p, 11, 18, 17, 24, "deep_blue")
        rect(p, 18, 18, 22, 24, "blue")
        line(p, 17, 9, 17, 25, "dark_dirt", 1)
        line(p, 10, 17, 23, 17, "dark_dirt", 1)
        line(p, 12, 11, 21, 23, "white", 1)
    elif slug == "barn":
        rect(p, 7, 16, 26, 29, "red")
        polygon(p, [(5, 16), (16, 7), (28, 16)], "dark_dirt")
        rect(p, 11, 20, 21, 29, "paper")
        line(p, 11, 20, 21, 29, "red", 1)
        line(p, 21, 20, 11, 29, "red", 1)
        rect(p, 9, 18, 13, 21, "white")
        rect(p, 19, 18, 23, 21, "white")
        rect(p, 6, 29, 28, 31, "green")
    elif slug == "birdhouse":
        rect(p, 11, 14, 22, 25, "dirt")
        polygon(p, [(9, 14), (16, 7), (24, 14)], "red")
        rect(p, 13, 16, 20, 23, "paper")
        circle(p, 16, 18, 3, "ink")
        rect(p, 15, 25, 18, 30, "dark_dirt")
        line(p, 8, 30, 25, 30, "green", 1)
        circle(p, 23, 21, 3, "orange")
        polygon(p, [(26, 21), (30, 19), (27, 23)], "gold")
    elif slug == "dynamite":
        for x, color in [(10, "red"), (15, "orange"), (20, "red")]:
            rect(p, x, 11, x + 4, 26, color)
            rect(p, x, 10, x + 4, 12, "paper")
            rect(p, x, 24, x + 4, 27, "dark_dirt")
        line(p, 8, 15, 26, 15, "dark_dirt", 1)
        line(p, 18, 10, 24, 5, "dark_dirt", 1)
        circle(p, 25, 4, 2, "yellow")
        put(p, 28, 3, "white")
    elif slug == "eagle":
        ellipse(p, 17, 18, 8, 5, "dark_dirt")
        circle(p, 20, 11, 5, "white")
        polygon(p, [(24, 11), (30, 13), (24, 16)], "gold")
        polygon(p, [(11, 17), (2, 9), (8, 23)], "dirt")
        polygon(p, [(20, 17), (30, 8), (27, 24)], "dark_dirt")
        rect(p, 18, 10, 20, 12, "ink")
        rect(p, 14, 21, 18, 24, "paper")
        line(p, 16, 23, 13, 28, "gold", 1)
        line(p, 19, 23, 22, 28, "gold", 1)
    elif slug == "lamp":
        polygon(p, [(10, 11), (22, 11), (25, 21), (7, 21)], "yellow")
        polygon(p, [(12, 12), (20, 12), (22, 19), (10, 19)], "paper")
        rect(p, 14, 21, 19, 27, "dark_stone")
        rect(p, 10, 27, 23, 30, "gold")
        circle(p, 16, 16, 3, "white")
        line(p, 8, 13, 5, 10, "yellow", 1)
        line(p, 24, 13, 28, 10, "yellow", 1)
    elif slug == "lawn-mower":
        rect(p, 8, 17, 24, 24, "green")
        rect(p, 10, 15, 20, 18, "lime")
        circle(p, 10, 25, 3, "ink")
        circle(p, 22, 25, 3, "ink")
        circle(p, 10, 25, 1, "stone")
        circle(p, 22, 25, 1, "stone")
        line(p, 22, 17, 28, 9, "dark_stone", 2)
        rect(p, 5, 27, 28, 30, "dark_green")
        line(p, 5, 23, 25, 23, "white", 1)
    elif slug == "microscope":
        rect(p, 10, 25, 25, 29, "dark_stone")
        rect(p, 13, 22, 21, 25, "stone")
        line(p, 16, 22, 21, 12, "dark_stone", 3)
        rect(p, 19, 8, 25, 13, "ice")
        rect(p, 17, 11, 23, 15, "sky")
        circle(p, 13, 18, 5, "blue")
        circle(p, 13, 18, 3, "deep_blue")
        rect(p, 9, 17, 17, 20, "white")
        rect(p, 12, 7, 17, 10, "stone")
    elif slug == "oil":
        polygon(p, [(16, 5), (9, 17), (10, 25), (16, 29), (23, 25), (24, 17)], "ink")
        polygon(p, [(16, 8), (11, 18), (12, 24), (16, 27), (21, 24), (22, 18)], "dark_stone")
        rect(p, 13, 18, 20, 22, "violet")
        line(p, 12, 16, 18, 10, "white", 1)
        ellipse(p, 16, 27, 7, 2, "dark_dirt")
    elif slug == "paint":
        rect(p, 8, 15, 24, 27, "gold")
        ellipse(p, 16, 15, 8, 4, "paper")
        rect(p, 10, 12, 22, 16, "blue")
        rect(p, 11, 16, 15, 24, "red")
        rect(p, 16, 16, 20, 24, "sky")
        line(p, 22, 10, 28, 4, "dark_dirt", 3)
        line(p, 23, 9, 29, 3, "paper", 1)
        rect(p, 7, 26, 25, 29, "dark_dirt")
    elif slug == "angel":
        ellipse(p, 9, 17, 7, 9, "ice")
        ellipse(p, 23, 17, 7, 9, "ice")
        ellipse(p, 10, 18, 4, 7, "white")
        ellipse(p, 22, 18, 4, 7, "white")
        circle(p, 16, 8, 4, "paper")
        rect(p, 13, 12, 20, 26, "white")
        polygon(p, [(13, 16), (8, 27), (24, 27), (20, 16)], "ice")
        rect(p, 13, 6, 20, 7, "yellow")
        rect(p, 15, 3, 18, 4, "gold")
        line(p, 12, 25, 20, 25, "gold", 1)
    elif slug == "corpse":
        ellipse(p, 16, 24, 12, 4, "dark_stone")
        rect(p, 7, 18, 23, 25, "stone")
        rect(p, 10, 16, 19, 22, "ice")
        circle(p, 23, 17, 4, "paper")
        rect(p, 22, 16, 24, 18, "ink")
        line(p, 8, 19, 21, 25, "white", 1)
        rect(p, 5, 21, 9, 23, "dark_stone")
        rect(p, 12, 13, 15, 17, "violet")
    elif slug == "cyborg":
        rect(p, 9, 8, 23, 22, "stone")
        rect(p, 9, 8, 16, 22, "paper")
        rect(p, 16, 8, 23, 22, "dark_stone")
        rect(p, 12, 13, 14, 15, "ink")
        rect(p, 18, 12, 21, 15, "red")
        rect(p, 12, 19, 21, 21, "ink")
        line(p, 16, 8, 16, 22, "ice", 1)
        rect(p, 7, 22, 25, 29, "blue")
        rect(p, 20, 3, 22, 8, "dark_stone")
        put(p, 21, 2, "sky")
    elif slug == "fireman":
        circle(p, 16, 10, 4, "paper")
        polygon(p, [(10, 9), (16, 4), (23, 9)], "red")
        rect(p, 11, 9, 22, 11, "gold")
        rect(p, 10, 14, 23, 27, "red")
        rect(p, 13, 14, 20, 27, "orange")
        rect(p, 15, 16, 18, 26, "yellow")
        line(p, 7, 19, 3, 25, "paper", 2)
        line(p, 23, 18, 29, 24, "stone", 2)
        rect(p, 11, 27, 15, 30, "dark_stone")
        rect(p, 19, 27, 23, 30, "dark_stone")
    elif slug == "gardener":
        circle(p, 16, 10, 4, "paper")
        rect(p, 10, 7, 23, 9, "yellow")
        rect(p, 12, 4, 21, 7, "gold")
        rect(p, 11, 14, 22, 25, "green")
        rect(p, 13, 15, 20, 25, "lime")
        line(p, 9, 18, 5, 24, "paper", 2)
        line(p, 23, 17, 27, 23, "dark_dirt", 2)
        polygon(p, [(27, 23), (31, 21), (29, 27)], "stone")
        rect(p, 12, 25, 15, 30, "dirt")
        rect(p, 19, 25, 22, 30, "dirt")
        rect(p, 4, 27, 28, 30, "dark_green")
    elif slug == "grim-reaper":
        ellipse(p, 16, 15, 8, 10, "ink")
        rect(p, 10, 15, 23, 28, "dark_stone")
        polygon(p, [(10, 15), (16, 5), (24, 15)], "ink")
        circle(p, 15, 14, 2, "ice")
        circle(p, 18, 14, 2, "ice")
        line(p, 7, 30, 23, 5, "dark_dirt", 2)
        polygon(p, [(20, 5), (30, 3), (27, 7), (22, 11), (19, 14)], "stone")
        polygon(p, [(22, 5), (29, 4), (26, 6), (22, 9)], "ice")
    elif slug == "nerd":
        circle(p, 16, 10, 5, "paper")
        rect(p, 10, 9, 15, 13, "ink")
        rect(p, 17, 9, 22, 13, "ink")
        rect(p, 15, 10, 17, 11, "ink")
        rect(p, 12, 18, 21, 28, "blue")
        rect(p, 8, 19, 14, 27, "paper")
        line(p, 10, 21, 13, 26, "red", 1)
        rect(p, 20, 18, 25, 24, "white")
        rect(p, 21, 19, 24, 23, "sky")
        rect(p, 13, 14, 20, 16, "gold")
    elif slug == "phoenix":
        polygon(p, [(16, 5), (22, 15), (18, 28), (14, 28), (10, 15)], "orange")
        polygon(p, [(7, 18), (2, 8), (14, 14), (13, 23)], "red")
        polygon(p, [(25, 18), (31, 8), (18, 14), (19, 23)], "red")
        polygon(p, [(16, 7), (20, 15), (16, 24), (12, 15)], "yellow")
        polygon(p, [(16, 4), (19, 9), (13, 9)], "gold")
        rect(p, 15, 12, 17, 15, "white")
        line(p, 12, 26, 8, 31, "gold", 1)
        line(p, 20, 26, 24, 31, "gold", 1)
    elif slug == "scarecrow":
        rect(p, 8, 8, 24, 10, "dark_dirt")
        rect(p, 12, 4, 20, 8, "gold")
        circle(p, 16, 12, 4, "paper")
        rect(p, 12, 17, 21, 27, "dirt")
        line(p, 4, 18, 28, 18, "dark_dirt", 2)
        rect(p, 5, 16, 10, 21, "yellow")
        rect(p, 22, 16, 27, 21, "yellow")
        line(p, 16, 16, 16, 30, "dark_dirt", 2)
        rect(p, 13, 22, 20, 27, "green")
        rect(p, 14, 12, 18, 14, "ink")
    elif slug == "surfer":
        wave(p, 4, 24, "sky")
        rect(p, 3, 24, 29, 28, "deep_blue")
        line(p, 7, 22, 27, 17, "red", 3)
        line(p, 8, 21, 28, 16, "gold", 1)
        circle(p, 16, 11, 3, "paper")
        rect(p, 14, 14, 19, 20, "lime")
        line(p, 14, 18, 9, 22, "paper", 2)
        line(p, 19, 19, 24, 17, "paper", 2)
    elif slug == "unicorn":
        ellipse(p, 17, 19, 10, 6, "white")
        circle(p, 24, 12, 5, "white")
        polygon(p, [(23, 7), (25, 1), (27, 7)], "gold")
        rect(p, 21, 11, 24, 15, "ice")
        rect(p, 8, 18, 11, 28, "ice")
        rect(p, 16, 20, 19, 29, "ice")
        rect(p, 23, 19, 26, 29, "ice")
        line(p, 8, 15, 3, 10, "violet", 2)
        rect(p, 22, 12, 24, 14, "ink")
        line(p, 13, 14, 21, 10, "violet", 1)
    elif slug == "warrior":
        circle(p, 16, 9, 4, "paper")
        polygon(p, [(10, 9), (16, 3), (23, 9)], "dark_stone")
        rect(p, 10, 14, 23, 27, "stone")
        rect(p, 12, 16, 21, 24, "dark_stone")
        circle(p, 8, 20, 5, "blue")
        rect(p, 6, 18, 10, 22, "sky")
        line(p, 23, 17, 29, 7, "ice", 2)
        rect(p, 25, 6, 31, 8, "gold")
        rect(p, 12, 27, 15, 30, "dark_dirt")
        rect(p, 19, 27, 22, 30, "dark_dirt")
    elif slug == "wizard":
        circle(p, 16, 11, 4, "paper")
        polygon(p, [(10, 10), (16, 2), (23, 10)], "violet")
        rect(p, 12, 14, 21, 28, "violet")
        polygon(p, [(12, 15), (6, 29), (27, 29), (21, 15)], "deep_blue")
        rect(p, 14, 13, 19, 18, "white")
        line(p, 25, 28, 25, 7, "dark_dirt", 2)
        circle(p, 25, 6, 2, "yellow")
        put(p, 28, 4, "white")
    elif slug == "alligator":
        rect(p, 5, 17, 24, 23, "green")
        rect(p, 20, 14, 29, 21, "lime")
        polygon(p, [(27, 17), (31, 19), (27, 21)], "dark_green")
        rect(p, 7, 14, 21, 17, "dark_green")
        rect(p, 23, 15, 25, 17, "ink")
        for x in [22, 25, 28]:
            put(p, x, 21, "white")
        line(p, 5, 22, 2, 28, "dark_green", 2)
        rect(p, 8, 23, 11, 27, "dark_green")
        rect(p, 18, 23, 21, 27, "dark_green")
    elif slug == "armor":
        polygon(p, [(10, 8), (22, 8), (25, 17), (21, 29), (11, 29), (7, 17)], "dark_stone")
        polygon(p, [(12, 9), (20, 9), (23, 17), (19, 27), (13, 27), (9, 17)], "stone")
        rect(p, 12, 5, 20, 10, "dark_stone")
        rect(p, 13, 6, 19, 9, "ice")
        rect(p, 14, 11, 18, 29, "ice")
        line(p, 10, 16, 23, 16, "white", 1)
        line(p, 13, 24, 20, 24, "white", 1)
    elif slug == "dragon":
        ellipse(p, 16, 19, 9, 6, "green")
        polygon(p, [(9, 15), (2, 7), (13, 10)], "dark_green")
        polygon(p, [(20, 15), (30, 6), (26, 18)], "dark_green")
        circle(p, 23, 10, 4, "green")
        polygon(p, [(26, 9), (31, 11), (26, 13)], "red")
        rect(p, 23, 9, 25, 11, "ink")
        polygon(p, [(18, 13), (21, 6), (24, 13)], "lime")
        line(p, 8, 20, 2, 25, "green", 2)
        rect(p, 12, 23, 15, 29, "dark_green")
        rect(p, 20, 22, 23, 29, "dark_green")
        line(p, 27, 12, 31, 15, "orange", 1)
    elif slug == "tobacco":
        ellipse(p, 12, 18, 7, 4, "dark_green")
        ellipse(p, 18, 17, 8, 4, "green")
        ellipse(p, 21, 21, 6, 3, "lime")
        line(p, 8, 25, 25, 11, "dark_dirt", 2)
        line(p, 10, 24, 24, 13, "dirt", 1)
        rect(p, 20, 9, 27, 12, "paper")
        rect(p, 26, 9, 29, 12, "orange")
        line(p, 22, 7, 19, 3, "stone", 1)
        line(p, 25, 7, 27, 3, "ice", 1)
    elif slug == "allergy":
        circle(p, 14, 14, 7, "paper")
        rect(p, 10, 12, 12, 14, "ink")
        rect(p, 17, 12, 19, 14, "ink")
        rect(p, 13, 17, 17, 20, "red")
        rect(p, 21, 17, 28, 21, "white")
        rect(p, 24, 15, 30, 23, "ice")
        for cx, cy in [(6, 7), (24, 6), (27, 10), (8, 23), (22, 27)]:
            circle(p, cx, cy, 2, "yellow")
            put(p, cx, cy, "lime")
        rect(p, 10, 22, 19, 29, "blue")
    elif slug == "bayonet":
        line(p, 6, 25, 23, 12, "dark_dirt", 3)
        line(p, 8, 24, 22, 13, "dirt", 1)
        rect(p, 4, 23, 11, 27, "dark_stone")
        rect(p, 10, 20, 19, 23, "dark_stone")
        polygon(p, [(20, 12), (30, 6), (27, 11), (22, 16)], "stone")
        polygon(p, [(22, 11), (29, 7), (26, 10), (22, 14)], "ice")
        rect(p, 15, 18, 19, 21, "gold")
    elif slug == "blood":
        polygon(p, [(16, 4), (9, 17), (10, 25), (16, 30), (23, 25), (24, 17)], "red")
        polygon(p, [(16, 7), (11, 18), (12, 24), (16, 27), (21, 24), (22, 18)], "orange")
        rect(p, 13, 17, 20, 23, "red")
        line(p, 13, 14, 17, 8, "white", 1)
        ellipse(p, 16, 28, 7, 2, "dark_dirt")
    elif slug == "carbon-dioxide":
        circle(p, 11, 18, 6, "stone")
        circle(p, 22, 14, 5, "ice")
        circle(p, 22, 23, 5, "ice")
        circle(p, 11, 18, 3, "dark_stone")
        circle(p, 22, 14, 2, "white")
        circle(p, 22, 23, 2, "white")
        line(p, 15, 17, 18, 15, "white", 1)
        line(p, 15, 20, 18, 22, "white", 1)
        circle(p, 7, 8, 2, "sky")
        circle(p, 27, 7, 2, "stone")
        circle(p, 5, 26, 2, "ice")
    elif slug == "cold":
        circle(p, 16, 17, 8, "ice")
        circle(p, 16, 17, 5, "white")
        rect(p, 13, 15, 15, 17, "deep_blue")
        rect(p, 18, 15, 20, 17, "deep_blue")
        line(p, 12, 22, 20, 22, "deep_blue", 1)
        line(p, 16, 4, 16, 30, "sky", 1)
        line(p, 5, 17, 27, 17, "sky", 1)
        line(p, 8, 8, 24, 26, "white", 1)
        line(p, 24, 8, 8, 26, "white", 1)
    elif slug == "double-rainbow":
        for offset, color in [(0, "red"), (2, "orange"), (4, "yellow"), (6, "green"), (8, "blue"), (10, "violet")]:
            ellipse(p, 16, 25, 13 - offset // 2, 13 - offset // 2, color)
        rect(p, 1, 24, 31, 32, "ink")
        for offset, color in [(0, "yellow"), (2, "lime"), (4, "sky"), (6, "violet")]:
            ellipse(p, 16, 28, 9 - offset // 2, 8 - offset // 2, color)
        rect(p, 4, 26, 29, 32, "ink")
        ellipse(p, 6, 25, 5, 3, "white")
        ellipse(p, 25, 25, 5, 3, "ice")
    elif slug == "duck":
        ellipse(p, 16, 20, 10, 6, "yellow")
        circle(p, 22, 13, 5, "yellow")
        polygon(p, [(26, 13), (31, 15), (26, 17)], "orange")
        rect(p, 22, 12, 24, 14, "ink")
        polygon(p, [(9, 17), (3, 12), (7, 23)], "gold")
        rect(p, 12, 25, 15, 29, "orange")
        rect(p, 19, 25, 22, 29, "orange")
        wave(p, 5, 29, "sky")
    elif slug == "electrician":
        circle(p, 16, 9, 4, "paper")
        rect(p, 11, 5, 22, 8, "yellow")
        rect(p, 12, 14, 21, 27, "blue")
        rect(p, 14, 14, 19, 27, "sky")
        line(p, 22, 16, 29, 8, "dark_stone", 2)
        line(p, 24, 10, 29, 15, "yellow", 1)
        line(p, 24, 13, 31, 8, "white", 1)
        rect(p, 7, 18, 12, 21, "gold")
        rect(p, 11, 27, 14, 30, "dark_dirt")
        rect(p, 19, 27, 22, 30, "dark_dirt")
    elif slug == "excalibur":
        polygon(p, [(16, 2), (21, 7), (18, 22), (14, 22), (11, 7)], "ice")
        line(p, 16, 4, 16, 22, "white", 1)
        rect(p, 9, 22, 23, 25, "gold")
        rect(p, 14, 25, 18, 31, "dark_dirt")
        polygon(p, [(5, 30), (9, 23), (14, 30)], "stone")
        polygon(p, [(18, 30), (23, 23), (27, 30)], "dark_stone")
        circle(p, 16, 23, 2, "yellow")
    elif slug == "family":
        for cx, body, height in [(10, "blue", 9), (17, "red", 11), (24, "green", 8)]:
            circle(p, cx, 10, 3, "paper")
            rect(p, cx - 3, 14, cx + 4, 14 + height, body)
            line(p, cx - 3, 17, cx - 7, 22, "paper", 1)
            line(p, cx + 3, 17, cx + 7, 22, "paper", 1)
            rect(p, cx - 2, 23, cx + 1, 29, "dark_dirt")
        line(p, 12, 22, 20, 22, "yellow", 1)
    elif slug == "flamethrower":
        rect(p, 6, 19, 17, 24, "dark_stone")
        rect(p, 8, 16, 15, 20, "blue")
        line(p, 16, 20, 25, 16, "dark_stone", 2)
        rect(p, 4, 22, 9, 28, "dark_dirt")
        polygon(p, [(25, 16), (31, 11), (30, 17)], "red")
        polygon(p, [(24, 16), (30, 14), (28, 19)], "orange")
        polygon(p, [(24, 16), (28, 15), (27, 18)], "yellow")
        rect(p, 12, 24, 16, 27, "gold")
    elif slug == "hard-roe":
        ellipse(p, 16, 23, 10, 4, "deep_blue")
        for cx, cy, color in [(11, 18, "orange"), (16, 16, "gold"), (21, 18, "orange"), (14, 22, "red"), (20, 22, "gold")]:
            circle(p, cx, cy, 4, color)
            circle(p, cx - 1, cy - 1, 1, "white")
        rect(p, 7, 25, 25, 28, "paper")
        wave(p, 8, 27, "sky")
    elif slug == "hay-bale":
        rect(p, 6, 13, 26, 26, "gold")
        ellipse(p, 16, 13, 10, 4, "yellow")
        rect(p, 8, 15, 24, 25, "yellow")
        line(p, 8, 17, 24, 17, "paper", 1)
        line(p, 8, 21, 24, 21, "dirt", 1)
        line(p, 12, 13, 12, 26, "dark_dirt", 1)
        line(p, 20, 13, 20, 26, "dark_dirt", 1)
        rect(p, 5, 26, 27, 29, "dark_dirt")
    elif slug == "hummingbird":
        ellipse(p, 17, 17, 6, 4, "green")
        circle(p, 23, 13, 3, "lime")
        line(p, 25, 13, 31, 10, "dark_dirt", 1)
        polygon(p, [(14, 15), (5, 7), (10, 19)], "sky")
        polygon(p, [(17, 18), (8, 27), (13, 20)], "blue")
        rect(p, 21, 12, 23, 14, "ink")
        line(p, 11, 18, 4, 22, "violet", 1)
        circle(p, 25, 8, 2, "red")
    elif slug == "idea":
        circle(p, 16, 13, 7, "yellow")
        circle(p, 14, 11, 3, "white")
        rect(p, 12, 20, 21, 24, "gold")
        rect(p, 14, 25, 19, 28, "dark_stone")
        rect(p, 13, 28, 20, 30, "stone")
        line(p, 16, 2, 16, 5, "white", 1)
        line(p, 7, 6, 10, 8, "yellow", 1)
        line(p, 25, 6, 22, 8, "yellow", 1)
        line(p, 6, 16, 3, 16, "gold", 1)
        line(p, 26, 16, 29, 16, "gold", 1)
    elif slug == "light":
        circle(p, 16, 15, 7, "yellow")
        circle(p, 14, 13, 3, "white")
        rect(p, 13, 22, 20, 25, "gold")
        rect(p, 14, 26, 19, 28, "stone")
        line(p, 16, 2, 16, 7, "white", 1)
        line(p, 16, 23, 16, 30, "gold", 1)
        line(p, 5, 15, 10, 15, "yellow", 1)
        line(p, 22, 15, 28, 15, "yellow", 1)
        line(p, 8, 7, 11, 10, "gold", 1)
        line(p, 24, 7, 21, 10, "gold", 1)
    elif slug == "lightsaber":
        line(p, 17, 3, 17, 23, "sky", 3)
        line(p, 17, 3, 17, 23, "white", 1)
        rect(p, 14, 23, 20, 30, "dark_stone")
        rect(p, 15, 24, 19, 27, "stone")
        rect(p, 13, 21, 21, 24, "gold")
        line(p, 11, 6, 8, 4, "sky", 1)
        line(p, 24, 6, 27, 4, "ice", 1)
    elif slug == "love":
        circle(p, 12, 12, 5, "red")
        circle(p, 20, 12, 5, "red")
        polygon(p, [(7, 14), (25, 14), (16, 27)], "red")
        circle(p, 13, 11, 2, "orange")
        line(p, 11, 8, 5, 4, "gold", 1)
        polygon(p, [(4, 4), (8, 3), (6, 7)], "yellow")
        line(p, 24, 20, 29, 25, "white", 1)
    elif slug == "music":
        line(p, 20, 5, 20, 22, "sky", 2)
        line(p, 12, 8, 12, 25, "violet", 2)
        rect(p, 12, 7, 21, 10, "sky")
        ellipse(p, 10, 25, 5, 3, "violet")
        ellipse(p, 19, 22, 5, 3, "sky")
        line(p, 5, 11, 8, 8, "yellow", 1)
        line(p, 26, 15, 29, 12, "gold", 1)
        circle(p, 7, 8, 1, "yellow")
        circle(p, 28, 12, 1, "gold")
    elif slug == "nest":
        ellipse(p, 16, 21, 12, 6, "dark_dirt")
        ellipse(p, 16, 20, 10, 4, "dirt")
        for y in [18, 21, 24]:
            line(p, 6, y, 26, y - 3, "paper", 1)
            line(p, 7, y - 2, 25, y + 1, "dark_dirt", 1)
        for cx in [12, 17, 21]:
            ellipse(p, cx, 16, 4, 5, "ice")
            circle(p, cx - 1, 14, 1, "white")
    elif slug == "omelette":
        ellipse(p, 16, 20, 13, 6, "yellow")
        ellipse(p, 16, 20, 10, 4, "gold")
        circle(p, 15, 18, 4, "white")
        circle(p, 15, 18, 2, "orange")
        rect(p, 8, 24, 25, 27, "paper")
        rect(p, 6, 26, 27, 29, "dark_dirt")
        line(p, 9, 16, 24, 21, "white", 1)
    elif slug == "ostrich":
        ellipse(p, 16, 20, 7, 9, "dark_dirt")
        line(p, 19, 13, 23, 5, "paper", 2)
        circle(p, 24, 5, 3, "paper")
        rect(p, 24, 4, 26, 6, "ink")
        polygon(p, [(27, 5), (31, 7), (27, 8)], "orange")
        line(p, 13, 27, 9, 31, "orange", 1)
        line(p, 19, 27, 23, 31, "orange", 1)
        ellipse(p, 11, 20, 5, 7, "stone")
        rect(p, 12, 17, 19, 24, "white")
    elif slug == "owl":
        ellipse(p, 16, 17, 9, 11, "dark_dirt")
        circle(p, 12, 13, 4, "paper")
        circle(p, 20, 13, 4, "paper")
        circle(p, 12, 13, 2, "ink")
        circle(p, 20, 13, 2, "ink")
        polygon(p, [(16, 15), (13, 19), (19, 19)], "orange")
        polygon(p, [(8, 7), (13, 9), (10, 12)], "dirt")
        polygon(p, [(24, 7), (19, 9), (22, 12)], "dirt")
        rect(p, 12, 27, 15, 30, "orange")
        rect(p, 18, 27, 21, 30, "orange")
    elif slug == "ozone":
        circle(p, 10, 18, 5, "sky")
        circle(p, 21, 11, 5, "ice")
        circle(p, 23, 24, 5, "sky")
        circle(p, 10, 18, 2, "white")
        circle(p, 21, 11, 2, "white")
        circle(p, 23, 24, 2, "white")
        line(p, 14, 16, 18, 13, "white", 1)
        line(p, 14, 20, 19, 23, "white", 1)
        line(p, 5, 8, 28, 4, "deep_blue", 1)
        line(p, 4, 27, 27, 30, "deep_blue", 1)
    elif slug == "peacock":
        for cx, cy, color in [(8, 12, "blue"), (12, 8, "green"), (16, 6, "lime"), (20, 8, "green"), (24, 12, "blue")]:
            ellipse(p, cx, cy, 4, 8, color)
            circle(p, cx, cy - 2, 2, "yellow")
        ellipse(p, 16, 20, 6, 8, "deep_blue")
        circle(p, 16, 12, 4, "sky")
        rect(p, 15, 11, 17, 13, "ink")
        polygon(p, [(18, 12), (23, 13), (18, 15)], "gold")
        rect(p, 13, 27, 16, 31, "orange")
        rect(p, 18, 27, 21, 31, "orange")
    elif slug == "prism":
        polygon(p, [(9, 26), (16, 5), (25, 26)], "ice")
        polygon(p, [(12, 24), (16, 9), (22, 24)], "white")
        line(p, 3, 14, 12, 17, "white", 1)
        for index, color in enumerate(["red", "orange", "yellow", "green", "blue", "violet"]):
            line(p, 20, 16 + index, 30, 12 + index, color, 1)
        rect(p, 9, 26, 25, 28, "stone")
    elif slug == "ruins":
        rect(p, 5, 25, 28, 29, "dark_stone")
        rect(p, 8, 12, 12, 25, "stone")
        rect(p, 15, 9, 19, 25, "stone")
        rect(p, 23, 14, 26, 25, "dark_stone")
        rect(p, 6, 9, 14, 12, "dark_stone")
        rect(p, 14, 6, 22, 9, "stone")
        line(p, 10, 13, 8, 25, "white", 1)
        line(p, 18, 11, 16, 25, "dark_stone", 1)
        rect(p, 11, 27, 16, 30, "green")
    elif slug == "safe":
        rect(p, 8, 8, 25, 27, "dark_stone")
        rect(p, 10, 10, 23, 25, "stone")
        rect(p, 12, 12, 21, 23, "dark_stone")
        circle(p, 17, 18, 4, "gold")
        circle(p, 17, 18, 2, "ink")
        rect(p, 14, 5, 20, 9, "stone")
        rect(p, 11, 27, 22, 30, "dark_dirt")
        rect(p, 20, 16, 23, 20, "yellow")
    elif slug == "safety-glasses":
        rect(p, 6, 12, 14, 20, "ice")
        rect(p, 18, 12, 26, 20, "ice")
        rect(p, 8, 14, 13, 19, "sky")
        rect(p, 19, 14, 24, 19, "sky")
        line(p, 14, 15, 18, 15, "dark_stone", 2)
        line(p, 6, 13, 2, 9, "dark_stone", 2)
        line(p, 26, 13, 30, 9, "dark_stone", 2)
        line(p, 9, 13, 13, 18, "white", 1)
        line(p, 20, 13, 24, 18, "white", 1)
    elif slug == "seagull":
        ellipse(p, 17, 19, 7, 4, "white")
        circle(p, 22, 15, 3, "white")
        polygon(p, [(25, 15), (30, 16), (25, 18)], "orange")
        polygon(p, [(14, 18), (3, 10), (10, 22)], "ice")
        polygon(p, [(20, 18), (29, 11), (25, 23)], "ice")
        rect(p, 21, 14, 23, 16, "ink")
        line(p, 14, 22, 11, 28, "orange", 1)
        line(p, 18, 22, 21, 28, "orange", 1)
        wave(p, 6, 29, "sky")
    elif slug == "sickness":
        circle(p, 16, 13, 7, "paper")
        rect(p, 12, 11, 14, 13, "ink")
        rect(p, 18, 11, 20, 13, "ink")
        rect(p, 12, 18, 20, 21, "green")
        rect(p, 11, 23, 22, 29, "blue")
        line(p, 23, 8, 29, 19, "white", 2)
        rect(p, 26, 16, 30, 20, "red")
        for cx, cy in [(6, 8), (25, 5), (8, 25), (27, 27)]:
            circle(p, cx, cy, 2, "lime")
            put(p, cx, cy, "yellow")
    elif slug == "sunglasses":
        rect(p, 5, 12, 14, 20, "ink")
        rect(p, 18, 12, 27, 20, "ink")
        rect(p, 7, 14, 13, 19, "deep_blue")
        rect(p, 19, 14, 25, 19, "deep_blue")
        line(p, 14, 15, 18, 15, "dark_stone", 2)
        line(p, 5, 13, 2, 10, "dark_stone", 2)
        line(p, 27, 13, 30, 10, "dark_stone", 2)
        line(p, 8, 13, 13, 18, "sky", 1)
        line(p, 20, 13, 25, 18, "sky", 1)
    elif slug == "swim-goggles":
        rect(p, 5, 12, 15, 21, "sky")
        rect(p, 17, 12, 27, 21, "sky")
        rect(p, 7, 14, 14, 20, "ice")
        rect(p, 18, 14, 25, 20, "ice")
        line(p, 15, 16, 17, 16, "white", 2)
        line(p, 5, 16, 1, 13, "violet", 2)
        line(p, 27, 16, 31, 13, "violet", 2)
        wave(p, 6, 25, "blue")
        wave(p, 7, 28, "sky")
    elif slug == "taser":
        rect(p, 8, 16, 22, 24, "dark_stone")
        rect(p, 10, 14, 19, 17, "yellow")
        rect(p, 12, 22, 17, 30, "dark_dirt")
        rect(p, 20, 17, 27, 20, "stone")
        rect(p, 25, 15, 29, 18, "ice")
        line(p, 28, 14, 31, 10, "yellow", 1)
        line(p, 27, 20, 31, 25, "sky", 1)
        put(p, 31, 10, "white")
        put(p, 31, 25, "white")
    elif slug == "the-one-ring":
        circle(p, 16, 17, 10, "gold")
        circle(p, 16, 17, 7, "ink")
        circle(p, 16, 17, 5, "dark_dirt")
        line(p, 9, 13, 23, 11, "yellow", 1)
        line(p, 8, 20, 22, 23, "orange", 1)
        rect(p, 13, 6, 19, 8, "white")
        rect(p, 12, 26, 20, 28, "dark_dirt")
        put(p, 24, 8, "red")
    elif slug == "toucan":
        ellipse(p, 15, 19, 7, 8, "ink")
        circle(p, 18, 10, 4, "white")
        polygon(p, [(21, 9), (31, 7), (31, 13), (21, 14)], "orange")
        polygon(p, [(23, 10), (31, 8), (29, 11), (23, 12)], "yellow")
        rect(p, 18, 9, 20, 11, "ink")
        rect(p, 12, 24, 15, 30, "blue")
        rect(p, 18, 24, 21, 30, "blue")
        polygon(p, [(10, 18), (4, 13), (8, 23)], "dark_stone")
        ellipse(p, 15, 20, 4, 5, "white")
    elif slug == "turtle":
        ellipse(p, 15, 20, 10, 6, "green")
        ellipse(p, 15, 19, 8, 5, "dark_green")
        circle(p, 25, 17, 4, "lime")
        rect(p, 25, 16, 27, 18, "ink")
        rect(p, 8, 24, 12, 28, "lime")
        rect(p, 18, 24, 22, 28, "lime")
        rect(p, 7, 15, 11, 18, "lime")
        rect(p, 3, 20, 7, 23, "green")
        line(p, 10, 17, 21, 22, "yellow", 1)
        line(p, 11, 22, 20, 17, "yellow", 1)
    elif slug == "twilight":
        rect(p, 4, 18, 29, 27, "deep_blue")
        polygon(p, [(4, 18), (16, 7), (29, 18)], "violet")
        circle(p, 11, 12, 5, "orange")
        rect(p, 4, 15, 29, 20, "ink")
        line(p, 5, 20, 28, 20, "gold", 1)
        for cx, cy in [(22, 8), (26, 11), (20, 14)]:
            put(p, cx, cy, "white")
            put(p, cx + 1, cy, "ice")
        rect(p, 7, 25, 25, 29, "dark_stone")
    elif slug == "water-gun":
        rect(p, 7, 16, 23, 23, "sky")
        rect(p, 10, 13, 20, 17, "blue")
        rect(p, 22, 17, 29, 20, "ice")
        rect(p, 11, 22, 17, 29, "blue")
        rect(p, 6, 20, 10, 24, "orange")
        line(p, 28, 18, 31, 16, "sky", 1)
        line(p, 29, 20, 31, 22, "white", 1)
        circle(p, 5, 23, 2, "yellow")
    elif slug == "wind-turbine":
        rect(p, 15, 14, 18, 30, "stone")
        circle(p, 16, 13, 2, "white")
        line(p, 16, 13, 16, 3, "ice", 2)
        line(p, 16, 13, 27, 9, "ice", 2)
        line(p, 16, 13, 7, 21, "white", 2)
        polygon(p, [(16, 3), (19, 7), (14, 7)], "sky")
        polygon(p, [(27, 9), (24, 13), (22, 8)], "sky")
        polygon(p, [(7, 21), (12, 20), (10, 25)], "sky")
        rect(p, 8, 30, 25, 31, "green")
    elif slug == "alarm-clock":
        circle(p, 16, 17, 9, "sky")
        circle(p, 16, 17, 7, "white")
        rect(p, 15, 11, 17, 18, "dark_stone")
        line(p, 16, 17, 21, 17, "dark_stone", 1)
        circle(p, 10, 7, 4, "gold")
        circle(p, 22, 7, 4, "gold")
        rect(p, 13, 5, 19, 8, "dark_stone")
        rect(p, 10, 25, 13, 30, "dark_dirt")
        rect(p, 20, 25, 23, 30, "dark_dirt")
    elif slug == "black-hole":
        circle(p, 16, 16, 11, "violet")
        circle(p, 16, 16, 8, "deep_blue")
        circle(p, 16, 16, 5, "ink")
        ellipse(p, 16, 16, 14, 4, "sky")
        ellipse(p, 16, 16, 11, 3, "white")
        circle(p, 7, 7, 1, "white")
        circle(p, 25, 6, 1, "yellow")
        circle(p, 27, 24, 1, "ice")
        circle(p, 6, 24, 1, "violet")
    elif slug == "bone":
        circle(p, 9, 12, 4, "white")
        circle(p, 9, 20, 4, "white")
        circle(p, 23, 12, 4, "ice")
        circle(p, 23, 20, 4, "ice")
        rect(p, 9, 13, 24, 20, "white")
        rect(p, 12, 15, 21, 18, "ice")
        line(p, 11, 14, 21, 19, "stone", 1)
        rect(p, 14, 21, 19, 23, "paper")
    elif slug == "bonsai-tree":
        rect(p, 11, 24, 22, 29, "dirt")
        rect(p, 9, 28, 24, 31, "dark_dirt")
        line(p, 16, 24, 16, 12, "dark_dirt", 3)
        line(p, 16, 17, 10, 12, "dark_dirt", 2)
        line(p, 16, 15, 22, 10, "dark_dirt", 2)
        ellipse(p, 10, 11, 7, 4, "green")
        ellipse(p, 21, 10, 7, 4, "dark_green")
        ellipse(p, 16, 7, 7, 4, "lime")
        ellipse(p, 16, 14, 8, 4, "green")
    elif slug == "caviar":
        ellipse(p, 16, 22, 11, 5, "ice")
        rect(p, 7, 21, 26, 26, "deep_blue")
        for cx, cy in [(11, 16), (15, 14), (19, 16), (13, 19), (21, 20)]:
            circle(p, cx, cy, 3, "ink")
            circle(p, cx - 1, cy - 1, 1, "violet")
        rect(p, 8, 25, 25, 28, "white")
        wave(p, 8, 27, "sky")
    elif slug == "chameleon":
        ellipse(p, 15, 20, 9, 5, "green")
        circle(p, 23, 15, 4, "lime")
        circle(p, 24, 14, 1, "ink")
        line(p, 7, 20, 3, 16, "green", 2)
        circle(p, 4, 16, 3, "dark_green")
        polygon(p, [(26, 15), (31, 13), (27, 18)], "red")
        rect(p, 10, 23, 13, 28, "dark_green")
        rect(p, 18, 23, 21, 28, "dark_green")
        line(p, 12, 16, 19, 13, "yellow", 1)
        line(p, 11, 19, 21, 19, "sky", 1)
    elif slug == "charcoal":
        polygon(p, [(9, 12), (17, 7), (25, 13), (23, 25), (13, 28), (6, 21)], "ink")
        polygon(p, [(12, 13), (17, 10), (22, 14), (20, 23), (14, 25), (9, 20)], "dark_stone")
        rect(p, 13, 14, 17, 17, "stone")
        rect(p, 18, 20, 21, 23, "red")
        line(p, 10, 18, 22, 12, "white", 1)
        ellipse(p, 15, 27, 9, 2, "dark_dirt")
    elif slug == "chicken":
        ellipse(p, 16, 20, 8, 7, "white")
        circle(p, 21, 12, 4, "white")
        polygon(p, [(24, 12), (30, 14), (24, 16)], "orange")
        rect(p, 21, 11, 23, 13, "ink")
        polygon(p, [(18, 8), (20, 4), (22, 8)], "red")
        polygon(p, [(8, 19), (3, 14), (7, 24)], "ice")
        line(p, 13, 26, 10, 31, "orange", 1)
        line(p, 19, 26, 22, 31, "orange", 1)
    elif slug == "christmas-tree":
        polygon(p, [(16, 4), (8, 16), (24, 16)], "green")
        polygon(p, [(16, 10), (5, 23), (27, 23)], "dark_green")
        polygon(p, [(16, 16), (3, 28), (29, 28)], "green")
        rect(p, 13, 27, 20, 31, "dark_dirt")
        put(p, 16, 3, "yellow")
        for cx, cy, color in [(12, 15, "red"), (20, 16, "gold"), (9, 23, "sky"), (23, 24, "red"), (16, 21, "yellow")]:
            circle(p, cx, cy, 1, color)
    elif slug == "computer":
        rect(p, 7, 7, 26, 22, "dark_stone")
        rect(p, 9, 9, 24, 20, "deep_blue")
        rect(p, 11, 11, 22, 17, "sky")
        rect(p, 13, 23, 20, 26, "stone")
        rect(p, 8, 27, 25, 30, "dark_stone")
        line(p, 12, 12, 20, 12, "white", 1)
        line(p, 12, 15, 18, 15, "ice", 1)
        put(p, 22, 18, "yellow")
    elif slug == "constellation":
        for cx, cy in [(8, 9), (15, 6), (22, 10), (19, 18), (10, 22), (25, 25)]:
            circle(p, cx, cy, 2, "white")
        line(p, 8, 9, 15, 6, "sky", 1)
        line(p, 15, 6, 22, 10, "sky", 1)
        line(p, 22, 10, 19, 18, "violet", 1)
        line(p, 19, 18, 10, 22, "sky", 1)
        line(p, 19, 18, 25, 25, "violet", 1)
        circle(p, 4, 4, 1, "yellow")
        circle(p, 27, 6, 1, "ice")
    elif slug == "crow":
        ellipse(p, 16, 19, 8, 5, "ink")
        circle(p, 22, 12, 4, "dark_stone")
        polygon(p, [(25, 12), (31, 14), (25, 16)], "stone")
        polygon(p, [(11, 18), (3, 10), (8, 23)], "dark_stone")
        polygon(p, [(19, 18), (28, 10), (25, 24)], "ink")
        rect(p, 22, 11, 24, 13, "white")
        line(p, 14, 22, 11, 29, "dark_dirt", 1)
        line(p, 18, 22, 21, 29, "dark_dirt", 1)
    elif slug == "cuckoo":
        rect(p, 9, 9, 23, 25, "dirt")
        polygon(p, [(7, 9), (16, 3), (25, 9)], "red")
        circle(p, 16, 15, 5, "paper")
        circle(p, 16, 15, 2, "ink")
        rect(p, 12, 20, 20, 25, "dark_dirt")
        rect(p, 14, 25, 18, 30, "gold")
        circle(p, 25, 15, 3, "sky")
        polygon(p, [(28, 15), (31, 13), (31, 17)], "orange")
    elif slug == "dinosaur":
        ellipse(p, 15, 21, 10, 6, "green")
        circle(p, 23, 14, 5, "lime")
        rect(p, 23, 13, 25, 15, "ink")
        line(p, 8, 20, 3, 15, "green", 2)
        polygon(p, [(20, 10), (23, 4), (26, 11)], "dark_green")
        for x in [10, 14, 18]:
            polygon(p, [(x, 15), (x + 2, 11), (x + 4, 15)], "dark_green")
        rect(p, 11, 24, 14, 30, "dark_green")
        rect(p, 19, 24, 22, 30, "dark_green")
        polygon(p, [(26, 14), (31, 16), (26, 18)], "yellow")
    elif slug == "drone":
        rect(p, 12, 14, 21, 20, "dark_stone")
        rect(p, 14, 15, 19, 18, "sky")
        for cx, cy in [(7, 10), (26, 10), (7, 24), (26, 24)]:
            circle(p, cx, cy, 4, "stone")
            circle(p, cx, cy, 2, "ice")
        line(p, 12, 14, 7, 10, "dark_stone", 1)
        line(p, 21, 14, 26, 10, "dark_stone", 1)
        line(p, 12, 20, 7, 24, "dark_stone", 1)
        line(p, 21, 20, 26, 24, "dark_stone", 1)
        put(p, 16, 21, "red")
    elif slug == "dry-ice":
        rect(p, 9, 17, 24, 25, "ice")
        rect(p, 11, 15, 22, 21, "white")
        rect(p, 7, 24, 26, 28, "deep_blue")
        line(p, 10, 13, 6, 8, "sky", 1)
        line(p, 15, 13, 16, 6, "white", 1)
        line(p, 21, 13, 27, 8, "ice", 1)
        circle(p, 5, 7, 1, "white")
        circle(p, 28, 7, 1, "sky")
        wave(p, 8, 29, "sky")
    elif slug == "duckling":
        ellipse(p, 15, 21, 8, 5, "yellow")
        circle(p, 22, 15, 4, "yellow")
        polygon(p, [(25, 15), (31, 16), (25, 18)], "orange")
        rect(p, 22, 14, 24, 16, "ink")
        polygon(p, [(8, 19), (4, 16), (6, 23)], "gold")
        rect(p, 12, 25, 15, 29, "orange")
        rect(p, 19, 25, 22, 29, "orange")
        wave(p, 6, 29, "sky")
    elif slug == "egg-timer":
        rect(p, 10, 5, 23, 8, "dark_dirt")
        rect(p, 10, 25, 23, 28, "dark_dirt")
        polygon(p, [(12, 8), (21, 8), (18, 16), (21, 25), (12, 25), (15, 16)], "ice")
        polygon(p, [(14, 9), (19, 9), (17, 15), (15, 15)], "sky")
        polygon(p, [(15, 19), (18, 19), (20, 24), (13, 24)], "gold")
        rect(p, 12, 28, 21, 30, "paper")
    elif slug == "engineer":
        circle(p, 16, 9, 4, "paper")
        rect(p, 10, 5, 22, 8, "yellow")
        rect(p, 12, 14, 21, 27, "orange")
        rect(p, 14, 15, 19, 27, "yellow")
        line(p, 22, 18, 29, 24, "dark_stone", 2)
        circle(p, 28, 25, 2, "stone")
        rect(p, 7, 17, 12, 20, "paper")
        rect(p, 11, 27, 14, 30, "dark_dirt")
        rect(p, 19, 27, 22, 30, "dark_dirt")
    elif slug == "family-tree":
        line(p, 16, 27, 16, 10, "dark_dirt", 3)
        line(p, 16, 15, 9, 8, "dark_dirt", 2)
        line(p, 16, 15, 23, 8, "dark_dirt", 2)
        line(p, 16, 20, 9, 24, "dark_dirt", 2)
        line(p, 16, 20, 24, 24, "dark_dirt", 2)
        for cx, cy, color in [(9, 8, "blue"), (23, 8, "red"), (9, 24, "green"), (24, 24, "yellow"), (16, 12, "paper")]:
            circle(p, cx, cy, 4, color)
            circle(p, cx, cy, 2, "paper")
        rect(p, 12, 27, 21, 30, "green")
    elif slug == "fire-extinguisher":
        rect(p, 12, 9, 21, 27, "red")
        rect(p, 14, 5, 19, 10, "dark_stone")
        rect(p, 12, 12, 21, 16, "white")
        rect(p, 14, 13, 19, 15, "yellow")
        line(p, 19, 6, 27, 9, "dark_stone", 2)
        line(p, 27, 9, 29, 15, "dark_stone", 1)
        line(p, 29, 15, 24, 19, "white", 1)
        rect(p, 11, 27, 22, 30, "dark_dirt")
    elif slug == "flashlight":
        rect(p, 8, 16, 22, 23, "dark_stone")
        rect(p, 10, 14, 18, 17, "stone")
        polygon(p, [(21, 15), (29, 11), (29, 27), (21, 23)], "yellow")
        polygon(p, [(22, 17), (29, 14), (29, 24), (22, 21)], "paper")
        rect(p, 11, 18, 18, 21, "blue")
        rect(p, 6, 18, 10, 22, "dark_dirt")
    elif slug == "frankenstein":
        rect(p, 10, 8, 23, 21, "green")
        rect(p, 12, 5, 21, 9, "ink")
        rect(p, 12, 12, 14, 14, "ink")
        rect(p, 19, 12, 21, 14, "ink")
        rect(p, 13, 18, 20, 20, "dark_stone")
        rect(p, 6, 13, 10, 16, "stone")
        rect(p, 23, 13, 27, 16, "stone")
        rect(p, 10, 22, 23, 29, "dark_stone")
        rect(p, 13, 22, 20, 29, "violet")
        put(p, 24, 6, "yellow")
    elif slug == "fridge":
        rect(p, 9, 5, 24, 29, "dark_stone")
        rect(p, 11, 7, 22, 28, "ice")
        rect(p, 12, 8, 21, 17, "white")
        rect(p, 12, 18, 21, 27, "sky")
        line(p, 11, 17, 22, 17, "dark_stone", 1)
        rect(p, 19, 11, 21, 15, "stone")
        rect(p, 19, 21, 21, 25, "stone")
        rect(p, 10, 29, 23, 31, "dark_dirt")
    elif slug == "fruit":
        circle(p, 11, 19, 5, "red")
        circle(p, 18, 17, 5, "orange")
        circle(p, 21, 23, 5, "violet")
        circle(p, 13, 24, 4, "lime")
        line(p, 16, 14, 20, 8, "dark_dirt", 1)
        ellipse(p, 21, 9, 4, 2, "green")
        circle(p, 10, 17, 1, "white")
        circle(p, 17, 15, 1, "yellow")
    elif slug == "grave":
        rect(p, 10, 13, 23, 28, "stone")
        ellipse(p, 16, 13, 7, 5, "stone")
        rect(p, 12, 16, 21, 25, "dark_stone")
        line(p, 16, 15, 16, 23, "ice", 1)
        line(p, 13, 18, 19, 18, "ice", 1)
        rect(p, 7, 28, 27, 31, "dark_dirt")
        rect(p, 8, 25, 12, 28, "green")
        rect(p, 22, 25, 26, 28, "dark_green")
    elif slug == "harp":
        line(p, 10, 28, 20, 5, "gold", 3)
        line(p, 19, 5, 25, 12, "gold", 3)
        line(p, 25, 12, 22, 28, "gold", 3)
        line(p, 10, 28, 23, 28, "dark_dirt", 2)
        for x in [14, 17, 20, 23]:
            line(p, x, 11, x - 3, 26, "white", 1)
        circle(p, 20, 5, 2, "yellow")
        rect(p, 8, 27, 25, 30, "dark_dirt")
    elif slug == "herb":
        line(p, 16, 29, 16, 12, "green", 2)
        ellipse(p, 11, 22, 5, 3, "dark_green")
        ellipse(p, 21, 20, 5, 3, "lime")
        ellipse(p, 12, 16, 4, 3, "green")
        ellipse(p, 20, 13, 4, 3, "lime")
        ellipse(p, 16, 9, 4, 5, "green")
        rect(p, 12, 28, 21, 30, "dark_dirt")
        put(p, 18, 8, "white")
    elif slug == "jedi":
        circle(p, 16, 9, 4, "paper")
        rect(p, 12, 14, 21, 27, "dirt")
        rect(p, 14, 15, 19, 27, "paper")
        line(p, 22, 17, 29, 7, "sky", 2)
        line(p, 22, 17, 29, 7, "white", 1)
        rect(p, 21, 17, 24, 20, "dark_stone")
        rect(p, 7, 18, 12, 21, "paper")
        rect(p, 12, 27, 15, 30, "dark_dirt")
        rect(p, 19, 27, 22, 30, "dark_dirt")
    elif slug == "lava-lamp":
        rect(p, 11, 7, 22, 27, "violet")
        rect(p, 13, 9, 20, 25, "deep_blue")
        polygon(p, [(12, 5), (21, 5), (23, 8), (10, 8)], "stone")
        polygon(p, [(10, 27), (23, 27), (25, 31), (8, 31)], "stone")
        circle(p, 16, 15, 3, "orange")
        ellipse(p, 18, 22, 4, 3, "red")
        ellipse(p, 15, 10, 3, 2, "yellow")
        line(p, 14, 12, 19, 24, "white", 1)
    elif slug == "leaf":
        ellipse(p, 16, 17, 8, 13, "green")
        ellipse(p, 18, 15, 6, 10, "lime")
        line(p, 8, 27, 24, 8, "dark_green", 2)
        line(p, 14, 18, 8, 16, "dark_green", 1)
        line(p, 17, 15, 23, 14, "dark_green", 1)
        line(p, 12, 22, 7, 23, "dark_green", 1)
        put(p, 19, 10, "white")
    elif slug == "lighthouse":
        rect(p, 13, 9, 21, 29, "white")
        rect(p, 14, 13, 20, 17, "red")
        rect(p, 14, 21, 20, 25, "red")
        rect(p, 12, 6, 22, 10, "dark_stone")
        rect(p, 15, 4, 20, 7, "yellow")
        polygon(p, [(20, 5), (31, 1), (31, 10), (20, 8)], "paper")
        polygon(p, [(12, 29), (22, 29), (25, 31), (9, 31)], "dark_dirt")
        wave(p, 4, 30, "sky")
    elif slug == "livestock":
        ellipse(p, 15, 20, 9, 6, "white")
        circle(p, 23, 15, 5, "paper")
        rect(p, 22, 14, 24, 16, "ink")
        rect(p, 10, 18, 14, 22, "dark_stone")
        rect(p, 17, 19, 20, 23, "dark_stone")
        rect(p, 9, 24, 12, 30, "dark_dirt")
        rect(p, 18, 24, 21, 30, "dark_dirt")
        polygon(p, [(7, 15), (3, 11), (6, 19)], "paper")
        polygon(p, [(26, 13), (31, 11), (28, 16)], "paper")
    elif slug == "mayonnaise":
        rect(p, 11, 8, 22, 27, "paper")
        rect(p, 12, 10, 21, 25, "white")
        rect(p, 13, 15, 20, 21, "yellow")
        rect(p, 13, 5, 20, 9, "blue")
        rect(p, 12, 27, 21, 30, "dark_stone")
        line(p, 14, 12, 19, 12, "ice", 1)
        line(p, 14, 18, 19, 18, "gold", 1)
    elif slug == "monarch":
        rect(p, 15, 16, 18, 25, "ink")
        circle(p, 16, 14, 2, "ink")
        polygon(p, [(15, 17), (4, 8), (7, 25), (15, 24)], "orange")
        polygon(p, [(18, 17), (29, 8), (26, 25), (18, 24)], "orange")
        polygon(p, [(11, 13), (6, 10), (8, 19), (14, 20)], "yellow")
        polygon(p, [(22, 13), (27, 10), (25, 19), (19, 20)], "yellow")
        line(p, 7, 14, 14, 21, "ink", 1)
        line(p, 26, 14, 19, 21, "ink", 1)
        put(p, 8, 9, "white")
        put(p, 25, 9, "white")
    elif slug == "mummy":
        rect(p, 11, 7, 22, 27, "paper")
        rect(p, 12, 8, 21, 26, "white")
        rect(p, 13, 12, 20, 15, "ink")
        rect(p, 14, 13, 16, 15, "yellow")
        rect(p, 18, 13, 20, 15, "yellow")
        for y in [10, 17, 21, 25]:
            line(p, 11, y, 22, y - 2, "stone", 1)
        rect(p, 8, 18, 12, 21, "paper")
        rect(p, 21, 17, 25, 20, "paper")
        rect(p, 12, 27, 15, 30, "dark_dirt")
        rect(p, 18, 27, 21, 30, "dark_dirt")
    elif slug == "narwhal":
        ellipse(p, 15, 21, 10, 5, "ice")
        circle(p, 23, 17, 4, "ice")
        polygon(p, [(25, 14), (31, 6), (28, 16)], "white")
        line(p, 26, 14, 30, 8, "sky", 1)
        polygon(p, [(8, 20), (2, 15), (5, 25)], "deep_blue")
        rect(p, 22, 16, 24, 18, "ink")
        wave(p, 5, 27, "sky")
        rect(p, 11, 23, 18, 25, "white")
    elif slug == "oil-lamp":
        ellipse(p, 16, 22, 10, 5, "gold")
        rect(p, 10, 18, 23, 24, "dirt")
        rect(p, 14, 12, 19, 19, "sky")
        rect(p, 13, 11, 20, 13, "stone")
        polygon(p, [(16, 4), (12, 11), (20, 11)], "yellow")
        polygon(p, [(16, 6), (14, 11), (18, 11)], "orange")
        line(p, 23, 20, 29, 16, "gold", 2)
        rect(p, 7, 24, 25, 27, "dark_dirt")
    elif slug == "optical-fiber":
        rect(p, 7, 23, 25, 27, "dark_stone")
        for x, color in [(9, "sky"), (12, "ice"), (15, "violet"), (18, "yellow"), (21, "lime")]:
            line(p, x, 23, x + 4, 7, color, 1)
            circle(p, x + 4, 7, 2, color)
            put(p, x + 4, 6, "white")
        rect(p, 5, 26, 27, 29, "stone")
        line(p, 7, 24, 24, 24, "white", 1)
    elif slug == "palm":
        line(p, 15, 28, 19, 10, "dirt", 3)
        line(p, 17, 28, 21, 12, "dark_dirt", 1)
        for end_x, end_y, color in [
            (7, 11, "dark_green"),
            (12, 7, "green"),
            (20, 6, "lime"),
            (27, 11, "green"),
            (22, 15, "dark_green"),
            (10, 17, "green"),
        ]:
            line(p, 18, 10, end_x, end_y, color, 3)
            line(p, 18, 11, (18 + end_x) // 2, end_y + 2, color, 2)
        circle(p, 16, 13, 2, "dark_dirt")
        circle(p, 20, 13, 2, "dirt")
        rect(p, 10, 27, 24, 29, "gold")
    elif slug == "pegasus":
        ellipse(p, 15, 20, 8, 4, "ice")
        rect(p, 10, 18, 22, 23, "white")
        circle(p, 23, 16, 4, "white")
        rect(p, 24, 15, 26, 17, "ice")
        line(p, 8, 21, 6, 27, "stone", 2)
        line(p, 13, 22, 12, 28, "stone", 2)
        line(p, 19, 22, 20, 28, "stone", 2)
        line(p, 22, 21, 25, 27, "stone", 2)
        polygon(p, [(11, 17), (3, 10), (12, 12)], "ice")
        polygon(p, [(13, 17), (8, 6), (17, 12)], "white")
        line(p, 5, 11, 13, 16, "sky", 1)
        line(p, 10, 8, 17, 14, "sky", 1)
        line(p, 6, 20, 3, 18, "ice", 1)
        put(p, 25, 15, "ink")
    elif slug == "pigeon":
        ellipse(p, 16, 20, 8, 5, "stone")
        circle(p, 22, 15, 4, "ice")
        polygon(p, [(9, 18), (2, 13), (9, 24)], "dark_stone")
        polygon(p, [(14, 18), (8, 10), (20, 17)], "ice")
        polygon(p, [(25, 15), (29, 17), (25, 18)], "gold")
        line(p, 14, 24, 12, 28, "orange", 1)
        line(p, 19, 24, 20, 28, "orange", 1)
        rect(p, 20, 13, 22, 15, "violet")
        put(p, 23, 14, "ink")
    elif slug == "pilot":
        circle(p, 16, 8, 4, "paper")
        rect(p, 11, 5, 21, 8, "dark_stone")
        rect(p, 10, 8, 22, 10, "stone")
        rect(p, 10, 13, 22, 25, "blue")
        rect(p, 12, 15, 20, 20, "sky")
        rect(p, 7, 15, 11, 23, "deep_blue")
        rect(p, 21, 15, 25, 23, "deep_blue")
        rect(p, 13, 25, 16, 29, "dark_stone")
        rect(p, 18, 25, 21, 29, "dark_stone")
        line(p, 11, 13, 22, 24, "white", 1)
        put(p, 14, 8, "ink")
        put(p, 18, 8, "ink")
    elif slug == "pitchfork":
        rect(p, 14, 10, 18, 29, "dirt")
        rect(p, 16, 10, 18, 29, "dark_dirt")
        rect(p, 9, 9, 24, 13, "stone")
        rect(p, 9, 5, 12, 14, "ice")
        rect(p, 14, 4, 17, 14, "white")
        rect(p, 19, 5, 22, 14, "ice")
        rect(p, 24, 7, 27, 14, "stone")
        rect(p, 8, 13, 27, 16, "dark_stone")
        rect(p, 12, 25, 20, 29, "dark_dirt")
        line(p, 10, 5, 10, 2, "white", 1)
        line(p, 15, 4, 15, 1, "white", 1)
        line(p, 20, 5, 20, 2, "white", 1)
    elif slug == "rose":
        line(p, 16, 28, 16, 14, "green", 2)
        ellipse(p, 12, 21, 5, 2, "dark_green")
        ellipse(p, 20, 18, 5, 2, "lime")
        circle(p, 16, 11, 6, "red")
        circle(p, 13, 10, 3, "orange")
        circle(p, 19, 10, 3, "red")
        circle(p, 16, 14, 3, "orange")
        rect(p, 14, 10, 18, 13, "yellow")
        rect(p, 13, 27, 19, 29, "dark_dirt")
    elif slug == "seaplane":
        rect(p, 8, 17, 24, 21, "ice")
        polygon(p, [(24, 17), (30, 19), (24, 21)], "sky")
        rect(p, 11, 14, 22, 16, "blue")
        rect(p, 12, 22, 21, 24, "blue")
        line(p, 7, 16, 3, 12, "stone", 1)
        line(p, 7, 21, 3, 25, "stone", 1)
        rect(p, 9, 24, 15, 26, "deep_blue")
        rect(p, 18, 24, 24, 26, "deep_blue")
        rect(p, 5, 27, 27, 29, "sky")
        wave(p, 4, 28, "white")
    elif slug == "seasickness":
        rect(p, 6, 22, 25, 25, "deep_blue")
        wave(p, 6, 21, "sky")
        polygon(p, [(11, 18), (17, 12), (24, 18)], "paper")
        rect(p, 12, 18, 24, 22, "dirt")
        line(p, 10, 10, 15, 13, "lime", 2)
        line(p, 12, 7, 18, 10, "green", 1)
        circle(p, 23, 10, 3, "paper")
        put(p, 22, 9, "ink")
        line(p, 21, 13, 25, 13, "dark_dirt", 1)
    elif slug == "sewing-machine":
        rect(p, 7, 20, 26, 24, "stone")
        rect(p, 10, 12, 24, 20, "ice")
        rect(p, 14, 15, 28, 18, "stone")
        circle(p, 11, 13, 3, "dark_stone")
        rect(p, 22, 18, 25, 25, "dark_stone")
        line(p, 24, 20, 24, 28, "ice", 1)
        put(p, 24, 28, "white")
        rect(p, 5, 25, 29, 28, "dark_dirt")
        rect(p, 8, 22, 14, 23, "white")
    elif slug == "shark":
        polygon(p, [(4, 20), (13, 14), (25, 15), (30, 20), (23, 24), (10, 24)], "blue")
        polygon(p, [(10, 23), (23, 23), (18, 26), (8, 25)], "ice")
        polygon(p, [(14, 15), (18, 7), (21, 16)], "deep_blue")
        polygon(p, [(5, 20), (0, 15), (2, 24)], "deep_blue")
        polygon(p, [(24, 17), (31, 14), (29, 20)], "blue")
        rect(p, 23, 17, 25, 19, "white")
        put(p, 24, 16, "ink")
        wave(p, 4, 27, "sky")
    elif slug == "shuriken":
        polygon(p, [(16, 3), (19, 13), (16, 16), (13, 13)], "stone")
        polygon(p, [(29, 16), (19, 19), (16, 16), (19, 13)], "ice")
        polygon(p, [(16, 29), (13, 19), (16, 16), (19, 19)], "dark_stone")
        polygon(p, [(3, 16), (13, 13), (16, 16), (13, 19)], "stone")
        circle(p, 16, 16, 3, "ink")
        circle(p, 16, 16, 1, "white")
        line(p, 8, 8, 24, 24, "white", 1)
        line(p, 24, 8, 8, 24, "ice", 1)
    elif slug == "skeleton":
        circle(p, 16, 7, 5, "paper")
        rect(p, 13, 13, 19, 23, "paper")
        for x in [11, 21]:
            line(p, 16, 14, x, 22, "paper", 2)
            line(p, 16, 23, x, 29, "paper", 2)
        rect(p, 14, 16, 18, 18, "dark_stone")
        put(p, 14, 6, "ink")
        put(p, 18, 6, "ink")
        rect(p, 15, 9, 17, 10, "ink")
        line(p, 12, 25, 20, 25, "stone", 1)
    elif slug == "smog":
        draw_cloud(p, 12, 15)
        draw_cloud(p, 21, 18)
        rect(p, 7, 19, 26, 23, "dark_stone")
        rect(p, 10, 15, 24, 18, "stone")
        line(p, 5, 11, 20, 8, "dark_stone", 2)
        line(p, 10, 25, 27, 27, "stone", 1)
        rect(p, 14, 12, 17, 14, "violet")
        rect(p, 22, 20, 25, 22, "green")
    elif slug == "soap":
        ellipse(p, 16, 20, 10, 6, "ice")
        ellipse(p, 16, 19, 8, 4, "sky")
        rect(p, 9, 18, 23, 23, "blue")
        circle(p, 9, 9, 3, "white")
        circle(p, 20, 8, 2, "ice")
        circle(p, 25, 13, 2, "white")
        rect(p, 12, 18, 19, 20, "white")
        line(p, 9, 24, 24, 24, "deep_blue", 1)
    elif slug == "soda":
        rect(p, 11, 6, 21, 27, "red")
        rect(p, 12, 4, 20, 7, "stone")
        rect(p, 11, 9, 21, 13, "white")
        rect(p, 13, 14, 19, 22, "orange")
        rect(p, 14, 16, 18, 19, "yellow")
        line(p, 20, 6, 24, 3, "sky", 1)
        put(p, 24, 3, "white")
        rect(p, 10, 27, 22, 29, "dark_dirt")
    elif slug == "solar-cell":
        polygon(p, [(6, 15), (24, 12), (29, 23), (10, 26)], "deep_blue")
        polygon(p, [(8, 16), (23, 14), (27, 22), (11, 24)], "blue")
        for x in [12, 17, 22]:
            line(p, x, 15, x + 4, 24, "sky", 1)
        for y in [17, 20, 23]:
            line(p, 8, y, 27, y - 2, "ice", 1)
        rect(p, 15, 25, 18, 29, "stone")
        line(p, 4, 7, 10, 7, "yellow", 1)
        line(p, 7, 4, 7, 10, "yellow", 1)
        circle(p, 7, 7, 3, "gold")
    elif slug == "spaceship":
        polygon(p, [(16, 4), (9, 18), (12, 27), (20, 27), (23, 18)], "ice")
        polygon(p, [(16, 6), (12, 18), (14, 25), (18, 25), (20, 18)], "blue")
        rect(p, 13, 13, 19, 17, "sky")
        rect(p, 14, 14, 18, 16, "white")
        polygon(p, [(11, 21), (4, 28), (13, 26)], "deep_blue")
        polygon(p, [(21, 21), (28, 28), (19, 26)], "deep_blue")
        polygon(p, [(13, 27), (16, 31), (19, 27)], "orange")
        line(p, 16, 28, 16, 31, "yellow", 2)
        put(p, 7, 9, "white")
        put(p, 25, 12, "sky")
    elif slug == "starfish":
        polygon(p, [(16, 4), (19, 13), (29, 11), (21, 18), (26, 28), (16, 22), (6, 28), (11, 18), (3, 11), (13, 13)], "orange")
        polygon(p, [(16, 7), (18, 15), (25, 14), (19, 18), (22, 24), (16, 20), (10, 24), (13, 18), (7, 14), (14, 15)], "gold")
        circle(p, 14, 16, 1, "yellow")
        circle(p, 18, 16, 1, "yellow")
        circle(p, 16, 19, 1, "paper")
        wave(p, 5, 29, "sky")
    elif slug == "statue":
        rect(p, 9, 25, 23, 29, "dark_stone")
        rect(p, 11, 21, 21, 25, "stone")
        rect(p, 13, 11, 19, 22, "ice")
        circle(p, 16, 7, 4, "stone")
        rect(p, 12, 5, 20, 7, "dark_stone")
        line(p, 13, 13, 8, 20, "stone", 2)
        line(p, 19, 13, 24, 20, "dark_stone", 2)
        rect(p, 14, 13, 18, 17, "white")
        rect(p, 7, 28, 25, 30, "dark_stone")
    elif slug == "steam-engine":
        rect(p, 7, 18, 24, 25, "dark_stone")
        rect(p, 10, 14, 20, 19, "blue")
        rect(p, 19, 11, 23, 18, "stone")
        rect(p, 22, 8, 25, 12, "dark_stone")
        rect(p, 6, 21, 10, 25, "red")
        circle(p, 10, 25, 4, "stone")
        circle(p, 22, 25, 4, "stone")
        circle(p, 10, 25, 2, "ice")
        circle(p, 22, 25, 2, "ice")
        line(p, 3, 29, 29, 29, "dark_dirt", 1)
        line(p, 24, 8, 27, 4, "white", 2)
        line(p, 27, 4, 29, 2, "ice", 1)
    elif slug == "sundial":
        ellipse(p, 16, 24, 11, 4, "gold")
        ellipse(p, 16, 23, 9, 3, "paper")
        polygon(p, [(16, 9), (18, 22), (13, 22)], "stone")
        line(p, 16, 13, 24, 20, "dark_stone", 1)
        line(p, 8, 23, 24, 23, "dark_dirt", 1)
        for x in [10, 14, 18, 22]:
            line(p, x, 20, x, 23, "dark_dirt", 1)
        circle(p, 6, 8, 3, "yellow")
        line(p, 6, 3, 6, 5, "yellow", 1)
    elif slug == "super-nova":
        circle(p, 16, 16, 8, "orange")
        circle(p, 16, 16, 5, "yellow")
        circle(p, 16, 16, 2, "white")
        for x0, y0, x1, y1, color in [
            (16, 1, 16, 9, "yellow"),
            (16, 23, 16, 31, "orange"),
            (1, 16, 9, 16, "gold"),
            (23, 16, 31, 16, "yellow"),
            (5, 5, 11, 11, "white"),
            (27, 5, 21, 11, "gold"),
            (5, 27, 11, 21, "orange"),
            (27, 27, 21, 21, "white"),
        ]:
            line(p, x0, y0, x1, y1, color, 2)
        put(p, 25, 9, "sky")
        put(p, 7, 22, "violet")
    elif slug == "swimmer":
        rect(p, 4, 22, 29, 26, "deep_blue")
        wave(p, 5, 21, "sky")
        wave(p, 7, 25, "white")
        circle(p, 12, 14, 3, "paper")
        rect(p, 15, 15, 24, 18, "orange")
        line(p, 13, 18, 8, 22, "paper", 2)
        line(p, 20, 18, 25, 22, "paper", 2)
        rect(p, 10, 13, 15, 15, "blue")
        put(p, 13, 13, "ink")
    elif slug == "thread":
        for offset, color in [(0, "red"), (2, "orange"), (4, "paper")]:
            line(p, 6, 22 - offset, 14, 14 - offset, color, 2)
            line(p, 14, 14 - offset, 23, 20 - offset, color, 2)
            line(p, 23, 20 - offset, 28, 11 - offset, color, 2)
        circle(p, 7, 22, 2, "white")
        circle(p, 28, 11, 2, "yellow")
        rect(p, 11, 25, 22, 27, "dark_dirt")
    elif slug == "treehouse":
        rect(p, 13, 16, 19, 29, "dirt")
        circle(p, 16, 11, 8, "dark_green")
        circle(p, 10, 15, 6, "green")
        circle(p, 22, 15, 6, "lime")
        rect(p, 9, 14, 23, 23, "dirt")
        polygon(p, [(8, 14), (16, 8), (24, 14)], "dark_dirt")
        rect(p, 12, 17, 16, 21, "yellow")
        rect(p, 18, 18, 21, 23, "dark_dirt")
        line(p, 8, 23, 4, 29, "paper", 1)
        line(p, 23, 23, 28, 29, "paper", 1)
    elif slug == "umbrella":
        polygon(p, [(4, 17), (10, 8), (16, 5), (22, 8), (28, 17)], "red")
        polygon(p, [(5, 17), (10, 9), (16, 6), (16, 17)], "orange")
        polygon(p, [(16, 6), (22, 9), (27, 17), (16, 17)], "violet")
        line(p, 4, 17, 28, 17, "white", 1)
        line(p, 16, 17, 16, 27, "stone", 2)
        line(p, 16, 27, 20, 27, "stone", 2)
        line(p, 20, 27, 21, 24, "stone", 1)
    elif slug == "vampire":
        circle(p, 16, 8, 4, "paper")
        rect(p, 12, 12, 20, 24, "ink")
        polygon(p, [(10, 12), (3, 27), (13, 22)], "violet")
        polygon(p, [(22, 12), (29, 27), (19, 22)], "violet")
        rect(p, 14, 13, 18, 20, "white")
        rect(p, 15, 20, 17, 24, "red")
        put(p, 14, 8, "ink")
        put(p, 18, 8, "ink")
        put(p, 15, 11, "white")
        put(p, 17, 11, "white")
    elif slug == "vulture":
        ellipse(p, 16, 20, 8, 5, "dark_dirt")
        circle(p, 23, 13, 4, "paper")
        rect(p, 21, 16, 24, 19, "stone")
        polygon(p, [(9, 18), (1, 13), (8, 26)], "dark_stone")
        polygon(p, [(18, 18), (27, 15), (23, 25)], "dirt")
        polygon(p, [(26, 13), (31, 15), (26, 17)], "orange")
        line(p, 14, 24, 12, 29, "stone", 1)
        line(p, 18, 24, 20, 29, "stone", 1)
        put(p, 24, 12, "ink")
    elif slug == "watch":
        rect(p, 13, 3, 19, 9, "dirt")
        rect(p, 13, 23, 19, 29, "dirt")
        circle(p, 16, 16, 8, "stone")
        circle(p, 16, 16, 6, "ice")
        circle(p, 16, 16, 2, "white")
        line(p, 16, 16, 16, 11, "ink", 1)
        line(p, 16, 16, 21, 18, "ink", 1)
        rect(p, 20, 14, 23, 18, "gold")
        rect(p, 14, 5, 18, 8, "paper")
        rect(p, 14, 24, 18, 27, "paper")
    elif slug == "zombie":
        circle(p, 16, 8, 4, "lime")
        rect(p, 11, 12, 21, 24, "dark_stone")
        rect(p, 13, 14, 19, 22, "green")
        line(p, 11, 14, 5, 20, "lime", 2)
        line(p, 21, 14, 27, 18, "lime", 2)
        rect(p, 12, 24, 15, 29, "dirt")
        rect(p, 18, 24, 21, 29, "dirt")
        put(p, 14, 7, "ink")
        put(p, 18, 7, "ink")
        rect(p, 15, 10, 18, 11, "dark_dirt")
    elif slug == "acid-rain":
        draw_cloud(p, 15, 9)
        rect(p, 8, 13, 24, 15, "green")
        for x, y in [(9, 18), (14, 20), (19, 18), (24, 21)]:
            line(p, x, y, x - 2, y + 5, "lime", 2)
            put(p, x - 2, y + 6, "yellow")
        rect(p, 6, 27, 27, 29, "dark_green")
        circle(p, 11, 27, 2, "green")
        circle(p, 21, 28, 2, "lime")
    elif slug == "alcohol":
        rect(p, 13, 5, 19, 11, "ice")
        rect(p, 11, 10, 21, 28, "green")
        rect(p, 12, 12, 20, 26, "dark_green")
        rect(p, 13, 14, 19, 20, "paper")
        rect(p, 14, 15, 18, 19, "yellow")
        rect(p, 14, 6, 18, 9, "white")
        line(p, 21, 12, 25, 8, "sky", 1)
        put(p, 25, 8, "white")
        rect(p, 10, 27, 22, 29, "dark_dirt")
    elif slug == "alien":
        circle(p, 16, 10, 7, "lime")
        ellipse(p, 16, 18, 7, 8, "green")
        rect(p, 12, 20, 20, 27, "dark_green")
        ellipse(p, 13, 9, 3, 2, "ink")
        ellipse(p, 19, 9, 3, 2, "ink")
        put(p, 13, 9, "white")
        put(p, 19, 9, "white")
        line(p, 11, 19, 6, 25, "lime", 2)
        line(p, 21, 19, 26, 25, "lime", 2)
        line(p, 14, 26, 11, 30, "green", 2)
        line(p, 18, 26, 21, 30, "green", 2)
        rect(p, 14, 15, 18, 17, "sky")
    elif slug == "antarctica":
        rect(p, 4, 23, 28, 27, "deep_blue")
        polygon(p, [(5, 23), (12, 11), (19, 23)], "ice")
        polygon(p, [(14, 23), (22, 8), (29, 23)], "white")
        polygon(p, [(9, 21), (12, 14), (16, 21)], "sky")
        polygon(p, [(19, 21), (22, 12), (26, 21)], "ice")
        wave(p, 5, 26, "sky")
        rect(p, 8, 27, 27, 29, "blue")
        put(p, 4, 7, "white")
        put(p, 27, 10, "ice")
    elif slug == "avalanche":
        polygon(p, [(4, 28), (15, 7), (28, 28)], "dark_stone")
        polygon(p, [(15, 7), (10, 17), (20, 17)], "white")
        polygon(p, [(9, 27), (16, 15), (25, 27)], "ice")
        circle(p, 10, 22, 3, "white")
        circle(p, 16, 24, 4, "ice")
        circle(p, 23, 25, 3, "white")
        line(p, 13, 14, 26, 27, "sky", 2)
        rect(p, 5, 28, 29, 30, "stone")
    elif slug == "blizzard":
        draw_cloud(p, 15, 9)
        for x, y in [(8, 18), (13, 21), (18, 18), (23, 22), (27, 17)]:
            line(p, x, y, x + 3, y + 2, "ice", 1)
            line(p, x + 3, y, x, y + 2, "white", 1)
        line(p, 4, 24, 24, 18, "sky", 2)
        line(p, 9, 28, 29, 23, "white", 2)
        line(p, 3, 19, 17, 15, "blue", 1)
        rect(p, 20, 25, 26, 27, "deep_blue")
        put(p, 6, 16, "ice")
        put(p, 25, 14, "white")
    elif slug == "broom":
        line(p, 8, 26, 24, 6, "dirt", 2)
        line(p, 10, 27, 26, 7, "paper", 1)
        polygon(p, [(5, 24), (13, 20), (20, 29), (8, 29)], "gold")
        polygon(p, [(8, 24), (14, 21), (17, 28), (10, 28)], "yellow")
        line(p, 8, 25, 18, 28, "dark_dirt", 1)
        line(p, 11, 22, 14, 29, "dark_dirt", 1)
    elif slug == "bulletproof-vest":
        polygon(p, [(10, 6), (14, 6), (16, 11), (18, 6), (22, 6), (25, 27), (7, 27)], "dark_stone")
        polygon(p, [(11, 8), (14, 8), (16, 13), (18, 8), (21, 8), (23, 25), (9, 25)], "stone")
        rect(p, 10, 15, 22, 18, "dark_stone")
        rect(p, 12, 20, 20, 23, "ice")
        rect(p, 14, 20, 18, 22, "white")
        line(p, 9, 12, 23, 12, "ink", 1)
        rect(p, 12, 26, 20, 28, "dark_dirt")
    elif slug == "camel":
        rect(p, 9, 17, 24, 23, "dirt")
        circle(p, 13, 14, 5, "dirt")
        circle(p, 19, 13, 5, "paper")
        circle(p, 25, 15, 4, "paper")
        rect(p, 24, 17, 28, 20, "paper")
        line(p, 10, 22, 8, 29, "dark_dirt", 2)
        line(p, 15, 22, 15, 29, "dark_dirt", 2)
        line(p, 21, 22, 21, 29, "dark_dirt", 2)
        line(p, 25, 21, 27, 29, "dark_dirt", 2)
        put(p, 26, 14, "ink")
        rect(p, 5, 28, 28, 30, "gold")
    elif slug == "campfire":
        line(p, 7, 26, 25, 20, "dark_dirt", 3)
        line(p, 7, 20, 25, 26, "dirt", 3)
        polygon(p, [(16, 6), (10, 19), (15, 17), (13, 26), (22, 18), (18, 19)], "red")
        polygon(p, [(16, 9), (12, 19), (16, 17), (15, 24), (20, 18), (17, 18)], "orange")
        polygon(p, [(16, 12), (14, 20), (18, 19), (16, 24)], "yellow")
        rect(p, 9, 26, 24, 28, "dark_dirt")
    elif slug == "chicken-soup":
        ellipse(p, 16, 22, 11, 5, "blue")
        rect(p, 6, 18, 26, 23, "ice")
        ellipse(p, 16, 18, 10, 4, "paper")
        circle(p, 13, 17, 2, "orange")
        circle(p, 18, 18, 2, "yellow")
        rect(p, 19, 15, 22, 17, "paper")
        line(p, 8, 13, 6, 8, "white", 1)
        line(p, 16, 13, 16, 7, "ice", 1)
        line(p, 24, 13, 27, 8, "white", 1)
    elif slug == "chicken-wing":
        polygon(p, [(8, 19), (14, 11), (23, 12), (27, 18), (22, 25), (12, 25)], "orange")
        polygon(p, [(10, 19), (15, 14), (22, 14), (24, 18), (20, 22), (13, 23)], "gold")
        rect(p, 4, 20, 11, 24, "paper")
        circle(p, 4, 20, 3, "white")
        circle(p, 4, 24, 3, "white")
        line(p, 14, 13, 23, 22, "red", 1)
        rect(p, 13, 25, 23, 27, "dark_dirt")
    elif slug == "coconut":
        circle(p, 16, 18, 9, "dark_dirt")
        circle(p, 16, 18, 7, "dirt")
        circle(p, 13, 16, 1, "ink")
        circle(p, 17, 15, 1, "ink")
        circle(p, 18, 19, 1, "ink")
        rect(p, 12, 23, 20, 25, "paper")
        line(p, 10, 13, 20, 23, "dark_dirt", 1)
        line(p, 22, 10, 28, 4, "green", 2)
        line(p, 22, 10, 29, 11, "lime", 2)
    elif slug == "coffin":
        polygon(p, [(12, 4), (20, 4), (25, 10), (22, 28), (10, 28), (7, 10)], "dark_dirt")
        polygon(p, [(13, 6), (19, 6), (23, 11), (20, 25), (12, 25), (9, 11)], "dirt")
        line(p, 16, 9, 16, 20, "gold", 2)
        line(p, 12, 14, 20, 14, "gold", 2)
        rect(p, 11, 25, 21, 28, "dark_dirt")
        line(p, 10, 8, 22, 24, "paper", 1)
    elif slug == "crown":
        polygon(p, [(6, 23), (8, 10), (13, 18), (16, 7), (20, 18), (25, 10), (27, 23)], "gold")
        rect(p, 7, 22, 26, 27, "orange")
        rect(p, 10, 23, 14, 25, "yellow")
        rect(p, 18, 23, 22, 25, "yellow")
        circle(p, 8, 10, 2, "red")
        circle(p, 16, 7, 2, "sky")
        circle(p, 25, 10, 2, "violet")
        line(p, 7, 27, 27, 27, "dark_dirt", 1)
    elif slug == "darth-vader":
        circle(p, 16, 8, 5, "ink")
        polygon(p, [(10, 10), (22, 10), (26, 27), (6, 27)], "ink")
        rect(p, 11, 13, 21, 24, "dark_stone")
        rect(p, 13, 15, 19, 20, "stone")
        line(p, 5, 23, 1, 30, "red", 2)
        line(p, 6, 22, 2, 29, "orange", 1)
        put(p, 14, 7, "white")
        put(p, 18, 7, "white")
        rect(p, 13, 10, 19, 11, "dark_stone")
    elif slug == "doctor":
        circle(p, 16, 7, 4, "paper")
        rect(p, 10, 12, 22, 26, "white")
        rect(p, 13, 13, 19, 25, "ice")
        rect(p, 14, 15, 18, 21, "sky")
        line(p, 8, 15, 5, 24, "paper", 2)
        line(p, 24, 15, 27, 24, "paper", 2)
        rect(p, 15, 4, 17, 7, "stone")
        line(p, 13, 5, 19, 5, "stone", 1)
        line(p, 16, 15, 16, 21, "red", 1)
        line(p, 13, 18, 19, 18, "red", 1)
        put(p, 14, 7, "ink")
        put(p, 18, 7, "ink")
    elif slug == "electric-eel":
        rect(p, 4, 22, 28, 26, "deep_blue")
        wave(p, 4, 23, "sky")
        ellipse(p, 16, 18, 11, 4, "green")
        polygon(p, [(5, 18), (0, 14), (3, 23)], "dark_green")
        polygon(p, [(25, 17), (31, 15), (28, 20)], "lime")
        rect(p, 12, 16, 23, 19, "lime")
        polygon(p, [(16, 6), (12, 14), (17, 13), (14, 22), (23, 10), (18, 11)], "yellow")
        put(p, 24, 16, "ink")
        line(p, 8, 27, 26, 28, "sky", 1)
    elif slug == "fabric":
        polygon(p, [(7, 9), (24, 6), (27, 22), (10, 26)], "violet")
        polygon(p, [(8, 10), (22, 8), (24, 20), (11, 23)], "ice")
        rect(p, 10, 13, 25, 17, "sky")
        line(p, 10, 11, 25, 22, "white", 1)
        line(p, 8, 18, 22, 8, "blue", 1)
        line(p, 12, 25, 27, 21, "dark_stone", 1)
    elif slug == "fence":
        rect(p, 6, 12, 10, 28, "dirt")
        rect(p, 14, 8, 18, 28, "paper")
        rect(p, 22, 12, 26, 28, "dirt")
        polygon(p, [(6, 12), (8, 7), (10, 12)], "paper")
        polygon(p, [(14, 8), (16, 3), (18, 8)], "gold")
        polygon(p, [(22, 12), (24, 7), (26, 12)], "paper")
        rect(p, 4, 15, 28, 19, "dark_dirt")
        rect(p, 5, 21, 29, 25, "dirt")
        line(p, 4, 19, 28, 15, "paper", 1)
        rect(p, 3, 28, 30, 30, "dark_green")
    elif slug == "flour":
        polygon(p, [(10, 8), (22, 8), (25, 27), (7, 27)], "paper")
        polygon(p, [(12, 10), (20, 10), (22, 24), (10, 24)], "white")
        rect(p, 11, 13, 21, 18, "ice")
        rect(p, 13, 15, 19, 17, "white")
        line(p, 9, 25, 23, 25, "dark_dirt", 1)
        for cx, cy in [(5, 25), (8, 29), (25, 25), (27, 29), (15, 29)]:
            circle(p, cx, cy, 1, "white")
    elif slug == "flute":
        line(p, 5, 23, 26, 8, "gold", 3)
        line(p, 7, 22, 27, 9, "yellow", 1)
        for cx, cy in [(11, 19), (15, 16), (19, 13), (23, 10)]:
            circle(p, cx, cy, 1, "ink")
            put(p, cx - 1, cy - 1, "white")
        rect(p, 4, 22, 8, 26, "paper")
        line(p, 25, 7, 29, 5, "white", 1)
    elif slug == "fossil":
        polygon(p, [(6, 10), (22, 5), (28, 18), (22, 28), (8, 26), (3, 16)], "stone")
        polygon(p, [(8, 12), (21, 8), (25, 18), (20, 24), (9, 23), (6, 16)], "dark_stone")
        circle(p, 16, 16, 5, "paper")
        circle(p, 16, 16, 3, "stone")
        line(p, 12, 20, 21, 11, "paper", 2)
        line(p, 11, 13, 15, 17, "ice", 1)
        put(p, 23, 20, "white")
    elif slug == "fountain":
        rect(p, 6, 23, 26, 27, "stone")
        ellipse(p, 16, 22, 11, 5, "dark_stone")
        ellipse(p, 16, 20, 9, 4, "ice")
        rect(p, 13, 16, 19, 23, "stone")
        ellipse(p, 16, 15, 6, 3, "dark_stone")
        line(p, 16, 4, 16, 15, "sky", 2)
        line(p, 16, 8, 9, 19, "ice", 1)
        line(p, 16, 8, 23, 19, "white", 1)
        wave(p, 8, 22, "sky")
    elif slug == "fruit-tree":
        rect(p, 14, 16, 19, 29, "dark_dirt")
        line(p, 16, 18, 9, 12, "dirt", 2)
        line(p, 17, 17, 24, 11, "dirt", 2)
        circle(p, 11, 12, 6, "green")
        circle(p, 18, 9, 7, "lime")
        circle(p, 23, 14, 6, "green")
        for cx, cy in [(10, 12), (18, 10), (23, 15), (15, 17)]:
            circle(p, cx, cy, 2, "red")
        rect(p, 6, 29, 27, 31, "dark_green")
    elif slug == "glacier":
        rect(p, 4, 24, 28, 28, "deep_blue")
        polygon(p, [(5, 24), (15, 6), (28, 24)], "ice")
        polygon(p, [(15, 6), (12, 21), (21, 21)], "white")
        polygon(p, [(8, 23), (15, 13), (15, 24)], "sky")
        polygon(p, [(19, 23), (24, 15), (27, 24)], "blue")
        line(p, 10, 25, 29, 26, "sky", 1)
        wave(p, 5, 27, "ice")
    elif slug == "gnome":
        circle(p, 16, 12, 4, "paper")
        polygon(p, [(10, 10), (16, 2), (23, 10)], "red")
        rect(p, 12, 16, 21, 26, "blue")
        polygon(p, [(10, 14), (22, 14), (18, 23), (14, 23)], "white")
        rect(p, 14, 22, 18, 28, "dark_dirt")
        line(p, 10, 18, 6, 24, "paper", 1)
        line(p, 22, 18, 27, 23, "paper", 1)
        rect(p, 11, 28, 22, 30, "dark_green")
    elif slug == "goat":
        ellipse(p, 16, 20, 9, 6, "paper")
        circle(p, 23, 13, 5, "ice")
        line(p, 20, 9, 17, 4, "gold", 2)
        line(p, 25, 9, 28, 4, "gold", 2)
        rect(p, 21, 13, 23, 15, "ink")
        polygon(p, [(27, 14), (31, 16), (27, 17)], "paper")
        line(p, 10, 23, 8, 29, "dark_dirt", 2)
        line(p, 16, 24, 16, 29, "dark_dirt", 2)
        line(p, 22, 23, 24, 29, "dark_dirt", 2)
        rect(p, 6, 29, 28, 31, "dark_green")
    elif slug == "godzilla":
        ellipse(p, 15, 19, 8, 9, "green")
        circle(p, 21, 9, 5, "green")
        polygon(p, [(23, 9), (31, 11), (23, 13)], "dark_green")
        rect(p, 20, 8, 22, 10, "ink")
        for x, y in [(12, 9), (15, 7), (18, 10), (13, 13)]:
            polygon(p, [(x, y), (x + 2, y - 5), (x + 4, y)], "lime")
        line(p, 9, 23, 3, 30, "dark_green", 3)
        rect(p, 11, 26, 14, 31, "dark_green")
        rect(p, 18, 25, 22, 31, "dark_green")
        line(p, 26, 13, 31, 16, "orange", 1)
    elif slug == "gravestone":
        rect(p, 9, 14, 24, 28, "stone")
        ellipse(p, 16, 14, 8, 7, "stone")
        rect(p, 11, 16, 22, 28, "dark_stone")
        line(p, 16, 10, 16, 20, "ice", 1)
        line(p, 12, 15, 20, 15, "ice", 1)
        rect(p, 6, 28, 27, 30, "dark_green")
        for x in [7, 24, 28]:
            line(p, x, 29, x + 2, 24, "green", 1)
    elif slug == "graveyard":
        rect(p, 5, 18, 13, 28, "stone")
        ellipse(p, 9, 18, 4, 4, "stone")
        rect(p, 19, 15, 28, 28, "dark_stone")
        ellipse(p, 23, 15, 5, 5, "dark_stone")
        line(p, 23, 12, 23, 22, "ice", 1)
        line(p, 19, 17, 27, 17, "ice", 1)
        circle(p, 6, 7, 3, "ice")
        draw_cloud(p, 20, 6)
        rect(p, 2, 28, 30, 31, "dark_green")
        line(p, 3, 25, 30, 24, "dark_dirt", 1)
    elif slug == "hail":
        draw_cloud(p, 16, 8)
        for cx, cy in [(8, 18), (14, 21), (20, 18), (25, 23), (11, 26), (22, 27)]:
            circle(p, cx, cy, 2, "ice")
            put(p, cx - 1, cy - 1, "white")
        line(p, 7, 16, 26, 15, "sky", 1)
        line(p, 5, 21, 27, 18, "blue", 1)
        put(p, 29, 24, "deep_blue")
    elif slug == "iceberg":
        rect(p, 4, 24, 28, 28, "deep_blue")
        polygon(p, [(4, 24), (14, 7), (29, 24)], "ice")
        polygon(p, [(14, 7), (11, 21), (20, 21)], "white")
        polygon(p, [(8, 24), (18, 16), (25, 24)], "sky")
        polygon(p, [(10, 28), (22, 28), (18, 31), (13, 31)], "blue")
        wave(p, 4, 27, "sky")
    elif slug == "igloo":
        ellipse(p, 16, 22, 12, 9, "ice")
        rect(p, 4, 22, 28, 29, "ice")
        ellipse(p, 16, 21, 9, 6, "white")
        rect(p, 13, 20, 20, 29, "dark_stone")
        ellipse(p, 16, 20, 4, 4, "dark_stone")
        line(p, 5, 23, 27, 23, "sky", 1)
        line(p, 8, 18, 24, 18, "sky", 1)
        line(p, 10, 14, 22, 14, "sky", 1)
        line(p, 10, 18, 8, 25, "sky", 1)
        line(p, 22, 18, 25, 25, "sky", 1)
    elif slug in {"catalog-la1", "catalog-la2"}:
        cover = "blue" if slug.endswith("la1") else "violet"
        trim = "sky" if slug.endswith("la1") else "gold"
        polygon(p, [(5, 9), (14, 7), (14, 26), (5, 23)], cover)
        polygon(p, [(18, 7), (27, 9), (27, 23), (18, 26)], cover)
        polygon(p, [(7, 10), (14, 9), (14, 23), (7, 22)], "paper")
        polygon(p, [(18, 9), (25, 10), (25, 22), (18, 23)], "white")
        line(p, 8, 13, 13, 12, trim, 1)
        line(p, 19, 13, 24, 14, trim, 1)
        line(p, 8, 17, 13, 16, trim, 1)
        line(p, 19, 17, 24, 18, trim, 1)
        line(p, 7, 23, 14, 25, "dark_stone", 1)
        line(p, 18, 25, 26, 23, "dark_stone", 1)
    elif slug == "combine":
        circle(p, 11, 16, 6, "blue")
        circle(p, 22, 16, 6, "gold")
        line(p, 14, 16, 19, 16, "white", 2)
    elif slug == "clear":
        line(p, 9, 9, 23, 23, "red", 3)
        line(p, 23, 9, 9, 23, "orange", 3)
        line(p, 9, 9, 23, 23, "white", 1)
        line(p, 23, 9, 9, 23, "white", 1)
    elif slug == "reset":
        line(p, 10, 10, 22, 10, "green", 3)
        line(p, 22, 10, 22, 22, "green", 3)
        line(p, 22, 22, 10, 22, "lime", 3)
        line(p, 10, 22, 10, 15, "lime", 3)
        polygon(p, [(8, 14), (13, 14), (10, 18)], "lime")
    elif slug == "hint":
        circle(p, 16, 13, 7, "yellow")
        circle(p, 14, 11, 3, "white")
        rect(p, 13, 20, 20, 24, "gold")
        rect(p, 14, 25, 19, 27, "stone")
    else:
        draw_generic_sprite(p, slug)
    return p


def draw_cloud(pixels, cx, cy):
    ellipse(pixels, cx - 6, cy + 5, 6, 4, "sky")
    ellipse(pixels, cx + 6, cy + 5, 6, 4, "sky")
    ellipse(pixels, cx - 5, cy + 4, 6, 4, "ice")
    ellipse(pixels, cx, cy + 1, 7, 5, "white")
    ellipse(pixels, cx + 6, cy + 4, 6, 4, "ice")
    rect(pixels, cx - 10, cy + 4, cx + 11, cy + 9, "ice")
    rect(pixels, cx - 4, cy, cx + 4, cy + 3, "white")


def wave(pixels, x0, y, color):
    for x in range(x0, min(30, x0 + 20), 4):
        line(pixels, x, y, x + 2, y - 1, color, 1)
        line(pixels, x + 2, y - 1, x + 4, y, color, 1)


def stable_hash(slug):
    value = 2166136261
    for byte in slug.encode("utf-8"):
        value ^= byte
        value = (value * 16777619) & 0xFFFFFFFF
    return value


def draw_generic_sprite(pixels, slug):
    style = generic_style(slug)
    seed = stable_hash(slug)
    if style == "water":
        rect(pixels, 5, 20, 27, 26, "deep_blue")
        rect(pixels, 6, 18, 26, 21, "blue")
        wave(pixels, 6 + seed % 3, 19, "white")
        wave(pixels, 5 + (seed >> 3) % 3, 23, "sky")
        circle(pixels, 11 + seed % 11, 13, 3, "sky")
        rect(pixels, 8, 26, 25, 28, "deep_blue")
    elif style == "air":
        for y, color, offset in [(9, "white", 0), (15, "ice", 2), (22, "sky", 1)]:
            x = 4 + ((seed >> y) & 3) + offset
            line(pixels, x, y, x + 8, y - 2, color, 2)
            line(pixels, x + 8, y - 2, x + 17, y, color, 1)
        put(pixels, 24, 12, "white")
        put(pixels, 9, 20, "ice")
    elif style == "fire":
        polygon(pixels, [(8, 27), (11, 17), (14, 20), (16, 8), (21, 18), (24, 12), (27, 27)], "red")
        polygon(pixels, [(11, 27), (14, 18), (17, 21), (18, 12), (22, 21), (21, 27)], "orange")
        polygon(pixels, [(14, 27), (16, 20), (18, 23), (19, 27)], "yellow")
        put(pixels, 7 + seed % 18, 8 + (seed >> 4) % 7, "gold")
    elif style == "earth":
        polygon(pixels, [(5, 26), (10, 16), (17, 10), (25, 15), (28, 26)], "dark_dirt")
        polygon(pixels, [(8, 25), (12, 18), (18, 13), (24, 17), (25, 25)], "dirt")
        line(pixels, 16, 17, 16, 9, "green", 2)
        ellipse(pixels, 12, 12, 5, 2, "green")
        ellipse(pixels, 21, 11, 5, 2, "lime")
        rect(pixels, 7, 26, 27, 29, "dark_dirt")
    elif style == "plant":
        line(pixels, 16, 28, 16, 10, "green", 2)
        ellipse(pixels, 10, 18, 6, 3, "green")
        ellipse(pixels, 22, 15, 6, 3, "lime")
        circle(pixels, 14 + seed % 6, 10 + (seed >> 5) % 6, 3, "gold")
        rect(pixels, 12, 27, 21, 29, "dark_dirt")
    elif style == "metal":
        polygon(pixels, [(8, 18), (13, 10), (24, 11), (28, 19), (21, 27), (9, 25)], "dark_stone")
        polygon(pixels, [(10, 17), (14, 13), (23, 14), (25, 19), (20, 23), (11, 22)], "stone")
        rect(pixels, 13, 14, 22, 16, "white")
        line(pixels, 10, 22, 23, 14, "ice", 1)
    elif style == "space":
        circle(pixels, 16, 17, 8, "deep_blue")
        circle(pixels, 14, 15, 4, "blue")
        ellipse(pixels, 16, 17, 13, 4, "violet")
        for shift in [0, 7, 13, 21]:
            put(pixels, 4 + ((seed >> shift) % 24), 4 + ((seed >> (shift + 3)) % 22), "white")
    elif style == "tool":
        line(pixels, 9, 27, 23, 8, "dark_dirt", 3)
        line(pixels, 10, 27, 24, 8, "dirt", 1)
        polygon(pixels, [(15, 8), (25, 6), (28, 12), (22, 17), (16, 14)], "stone")
        polygon(pixels, [(17, 9), (24, 8), (26, 12), (21, 15), (18, 13)], "ice")
        rect(pixels, 8, 25, 14, 29, "gold")
    elif style == "creature":
        ellipse(pixels, 16, 20, 9, 6, "dirt")
        circle(pixels, 10 + seed % 13, 13, 5, "paper")
        polygon(pixels, [(23, 18), (30, 15), (26, 22)], "dark_dirt")
        rect(pixels, 10, 25, 13, 29, "dark_dirt")
        rect(pixels, 20, 25, 23, 29, "dark_dirt")
        put(pixels, 12 + seed % 9, 12, "ink")
    elif style == "person":
        circle(pixels, 16, 8, 5, "paper")
        polygon(pixels, [(10, 15), (22, 15), (25, 27), (7, 27)], "blue")
        polygon(pixels, [(12, 16), (20, 16), (22, 26), (10, 26)], "violet")
        rect(pixels, 13, 5, 19, 8, "dark_dirt")
        rect(pixels, 10, 18, 13, 24, "paper")
        rect(pixels, 19, 18, 22, 24, "paper")
    elif style == "food":
        ellipse(pixels, 16, 22, 11, 5, "gold")
        ellipse(pixels, 16, 19, 9, 5, "paper")
        circle(pixels, 12, 17, 3, "red")
        circle(pixels, 20, 18, 3, "green")
        rect(pixels, 7, 24, 26, 27, "dirt")
    elif style == "structure":
        rect(pixels, 8, 16, 25, 28, "paper")
        polygon(pixels, [(5, 16), (16, 6), (28, 16)], "red")
        rect(pixels, 13, 21, 18, 28, "dark_dirt")
        rect(pixels, 20, 18, 24, 22, "sky")
        rect(pixels, 7, 27, 27, 29, "dark_dirt")
    elif style == "container":
        rect(pixels, 13, 4, 19, 10, "stone")
        polygon(pixels, [(10, 10), (22, 10), (25, 27), (7, 27)], "sky")
        polygon(pixels, [(12, 12), (20, 12), (22, 24), (10, 24)], "ice")
        rect(pixels, 11, 19, 21, 24, "gold")
        line(pixels, 12, 12, 22, 24, "white", 1)
    elif style == "magic":
        polygon(pixels, [(16, 4), (24, 12), (20, 26), (12, 28), (6, 17)], "violet")
        polygon(pixels, [(16, 7), (21, 13), (18, 23), (12, 25), (9, 17)], "sky")
        rect(pixels, 13, 13, 17, 17, "white")
        put(pixels, 6, 7, "gold")
        put(pixels, 25, 9, "white")
    else:
        polygon(pixels, [(16, 5), (25, 12), (23, 25), (11, 28), (5, 16)], "stone")
        polygon(pixels, [(16, 8), (22, 13), (20, 22), (12, 24), (8, 16)], "ice")
        rect(pixels, 12, 13, 20, 17, "white")
        line(pixels, 9, 23, 23, 11, "blue", 1)
    add_slug_marks(pixels, seed)


def generic_style(slug):
    if any(word in slug for word in ["rain", "river", "sea", "ocean", "lake", "puddle", "juice", "swim", "fish"]):
        return "water"
    if any(word in slug for word in ["air", "wind", "tornado", "storm", "cloud", "mist", "smoke", "sky", "weather"]):
        return "air"
    if any(word in slug for word in ["fire", "flame", "lava", "heat", "warm", "plasma", "explosion"]):
        return "fire"
    if any(word in slug for word in ["earth", "land", "stone", "rock", "mountain", "sand", "dust", "clay", "mud", "island"]):
        return "earth"
    if any(word in slug for word in ["tree", "plant", "flower", "grass", "leaf", "wood", "forest", "fruit", "vegetable"]):
        return "plant"
    if any(word in slug for word in ["metal", "steel", "iron", "gold", "silver", "blade", "sword", "armor", "wire"]):
        return "metal"
    if any(word in slug for word in ["space", "star", "planet", "moon", "sun", "galaxy", "meteor", "mars", "venus", "jupiter"]):
        return "space"
    if any(word in slug for word in ["tool", "hammer", "axe", "knife", "scissor", "rod", "pencil", "needle", "plow"]):
        return "tool"
    if any(word in slug for word in ["bird", "cat", "dog", "cow", "horse", "fish", "animal", "dragon", "monster", "lizard", "penguin", "sheep", "pig"]):
        return "creature"
    if any(word in slug for word in ["human", "man", "woman", "person", "doctor", "knight", "wizard", "ninja", "santa", "alchemist"]):
        return "person"
    if any(word in slug for word in ["bread", "meat", "cheese", "milk", "cake", "soup", "food", "lemonade", "smoothie", "sugar"]):
        return "food"
    if any(word in slug for word in ["house", "hut", "cabin", "castle", "city", "wall", "factory", "building", "station", "village"]):
        return "structure"
    if any(word in slug for word in ["bottle", "jar", "glass", "cup", "box", "bucket", "pipe", "container"]):
        return "container"
    if any(word in slug for word in ["magic", "crystal", "unicorn", "fairy", "vampire", "zombie", "ghost", "ring", "tardis"]):
        return "magic"
    return "object"


def add_slug_marks(pixels, seed):
    accents = ["white", "gold", "sky", "lime", "violet", "paper"]
    for index in range(5):
        color = accents[(seed >> (index * 4)) % len(accents)]
        x = 5 + ((seed >> (index * 5)) % 22)
        y = 6 + ((seed >> (index * 6 + 3)) % 20)
        rect(pixels, x, y, x + 2, y + 2, color)


def icon_from_grid(rows):
    p = canvas()
    scale = max(1, 32 // len(rows))
    for grid_y, row in enumerate(rows):
        for grid_x, char in enumerate(row):
            if char == ".":
                continue
            color = ICON_COLOR_KEYS[char]
            rect(
                p,
                grid_x * scale,
                grid_y * scale,
                grid_x * scale + scale,
                grid_y * scale + scale,
                color,
            )
    return p


def animate_idle(slug, pixels, frame):
    if slug == "water":
        shifted = copy_pixels(pixels)
        if frame == 1:
            rect(shifted, 10, 17, 16, 19, "ice")
            rect(shifted, 18, 23, 24, 25, "sky")
        elif frame == 2:
            rect(shifted, 8, 19, 14, 21, "sky")
            rect(shifted, 16, 15, 20, 17, "white")
        else:
            rect(shifted, 12, 21, 22, 23, "ice")
        return shifted
    if slug == "steam":
        shifted = copy_pixels(pixels)
        if frame == 1:
            rect(shifted, 11, 20, 17, 24, "sky")
            rect(shifted, 18, 14, 24, 18, "white")
        elif frame == 2:
            rect(shifted, 14, 17, 20, 21, "white")
            rect(shifted, 20, 10, 26, 14, "sky")
        else:
            rect(shifted, 10, 15, 16, 19, "ice")
            rect(shifted, 17, 22, 23, 26, "white")
        return shifted
    if slug == "air":
        shifted = copy_pixels(pixels)
        if frame == 1:
            rect(shifted, 8, 13, 14, 15, "white")
            rect(shifted, 20, 18, 24, 20, "ice")
        elif frame == 2:
            rect(shifted, 13, 8, 19, 10, "sky")
            rect(shifted, 8, 20, 13, 22, "ice")
        else:
            rect(shifted, 16, 9, 24, 11, "sky")
        return shifted
    if slug == "fire":
        shifted = copy_pixels(pixels)
        if frame == 1:
            rect(shifted, 20, 8, 22, 12, "yellow")
        elif frame == 2:
            rect(shifted, 8, 12, 10, 16, "yellow")
        else:
            rect(shifted, 16, 12, 18, 16, "white")
        return shifted
    if slug in {"earth", "dust"}:
        shifted = clustered_idle_pixels(slug, pixels, frame, material_pulse_colors(slug))
        if slug == "earth":
            rect(shifted, 20, 8, 22, 10, "lime")
        else:
            rect(shifted, 4 + frame * 3, 8, 6 + frame * 3, 10, "white")
        return shifted
    return clustered_idle_pixels(slug, pixels, frame, material_pulse_colors(slug))


def material_pulse_colors(slug):
    if slug in {"energy", "pressure", "storm"}:
        return ["white", "yellow", "sky"]
    if slug in {"lava", "volcano"}:
        return ["yellow", "gold", "orange"]
    if slug in {"rain", "sea", "atmosphere", "sky"}:
        return ["white", "ice", "sky"]
    if slug in {"cloud", "wind"}:
        return ["white", "ice", "sky"]
    if slug in {"plant", "grass"}:
        return ["lime", "green", "dark_green"]
    if slug in {"stone", "metal", "mountain"}:
        return ["white", "stone", "dark_stone"]
    if slug in {"mud", "brick", "sand"}:
        return ["paper", "dirt", "gold"]
    if slug in {"glass", "bottle", "jar", "vase"}:
        return ["white", "ice", "sky"]
    if slug in {"life", "tree", "bird", "fish"}:
        return ["lime", "green", "sky"]
    if slug in {"human", "book", "house", "time", "tool"}:
        return ["white", "gold", "paper"]
    if slug in {"sun", "flower", "egg", "honey", "paper", "hammer", "wheat", "wood", "bee"}:
        return ["white", "yellow", "gold"]
    if slug in {"coal", "moon", "snow", "ice", "cotton", "needle", "chain", "web", "spider"}:
        return ["white", "ice", "stone"]
    if slug in {"glasses", "clock", "boat", "car", "scissors", "wheel", "blade", "newspaper"}:
        return ["white", "ice", "gold"]
    if slug in {"cat", "dog"}:
        return ["white", "paper", "gold"]
    if slug in {"lizard", "butterfly", "flying-fish"}:
        return ["white", "lime", "sky"]
    if slug in {"bread", "fishing-rod", "crystal-ball", "shovel"}:
        return ["white", "ice", "gold"]
    if slug in {"axe", "clay", "pottery", "knife"}:
        return ["white", "paper", "gold"]
    if slug in {"seaweed", "bacteria"}:
        return ["white", "lime", "green"]
    if slug in {"hay", "wool", "cow", "horse", "rainbow", "star", "lightning"}:
        return ["white", "yellow", "gold"]
    if slug in {"planet", "space", "solar-system", "galaxy", "telescope", "rocket", "astronaut"}:
        return ["white", "ice", "sky"]
    if slug in {"electricity", "wire", "light-bulb"}:
        return ["white", "yellow", "sky"]
    if slug in {"flood", "geyser", "ocean", "algae", "fog", "hurricane", "tsunami", "wave"}:
        return ["white", "ice", "sky"]
    if slug in {"earthquake", "granite", "obsidian", "ash"}:
        return ["white", "stone", "dark_stone"]
    if slug in {"gunpowder", "eruption", "explosion"}:
        return ["white", "yellow", "orange"]
    if slug == "salt":
        return ["white", "ice", "paper"]
    if slug in {"wall", "boiler", "bullet", "atomic-bomb"}:
        return ["white", "yellow", "gold"]
    if slug in {"archipelago", "beach", "desert", "dune", "pond", "dew"}:
        return ["white", "ice", "sky"]
    if slug in {"cactus", "garden", "ivy", "moss"}:
        return ["white", "lime", "green"]
    if slug == "diamond":
        return ["white", "ice", "sky"]
    if slug == "fireworks":
        return ["white", "yellow", "violet"]
    if slug in {"aquarium", "dam", "oasis", "oxygen", "plankton"}:
        return ["white", "ice", "sky"]
    if slug in {"blender", "bridge", "greenhouse", "gun", "hourglass", "mirror"}:
        return ["white", "ice", "gold"]
    if slug in {"day", "eclipse", "gold"}:
        return ["white", "yellow", "gold"]
    if slug in {"golem", "night"}:
        return ["white", "ice", "stone"]
    if slug in {"airplane", "helicopter", "sailboat", "train"}:
        return ["white", "ice", "sky"]
    if slug in {"bank", "castle", "city", "hospital", "village"}:
        return ["white", "yellow", "gold"]
    if slug in {"farm", "farmer", "field", "forest"}:
        return ["white", "lime", "green"]
    if slug in {"lake", "river", "swamp"}:
        return ["white", "ice", "sky"]
    if slug in {"isle", "horizon", "mountain-range", "quicksand", "sandstone", "sandstorm", "pyramid"}:
        return ["white", "yellow", "gold"]
    if slug in {"grenade", "rust"}:
        return ["white", "orange", "gold"]
    if slug in {"sound", "steel", "perfume", "ring", "robot", "scythe"}:
        return ["white", "ice", "sky"]
    if slug == "sunflower":
        return ["white", "yellow", "lime"]
    if slug in {"skyscraper", "window", "microscope"}:
        return ["white", "ice", "sky"]
    if slug in {"sword", "dynamite", "lamp", "paint"}:
        return ["white", "yellow", "gold"]
    if slug in {"tide", "water-lily", "waterfall", "oil"}:
        return ["white", "ice", "sky"]
    if slug in {"windmill", "barn", "birdhouse", "eagle", "lawn-mower"}:
        return ["white", "lime", "gold"]
    if slug in {"angel", "corpse", "cyborg", "nerd", "armor"}:
        return ["white", "ice", "sky"]
    if slug in {"fireman", "phoenix", "dragon"}:
        return ["white", "yellow", "orange"]
    if slug in {"gardener", "scarecrow", "alligator"}:
        return ["white", "lime", "green"]
    if slug in {"grim-reaper", "surfer", "unicorn", "warrior", "wizard"}:
        return ["white", "ice", "gold"]
    if slug in {"tobacco", "duck", "hay-bale", "hummingbird"}:
        return ["white", "yellow", "lime"]
    if slug in {"allergy", "carbon-dioxide", "cold", "double-rainbow"}:
        return ["white", "ice", "sky"]
    if slug in {"bayonet", "electrician", "excalibur", "flamethrower", "idea"}:
        return ["white", "yellow", "gold"]
    if slug in {"blood", "hard-roe"}:
        return ["white", "orange", "red"]
    if slug == "family":
        return ["white", "paper", "gold"]
    if slug in {"light", "love", "music", "prism"}:
        return ["white", "yellow", "gold"]
    if slug in {"lightsaber", "ozone", "safety-glasses"}:
        return ["white", "ice", "sky"]
    if slug in {"nest", "omelette", "ostrich", "owl", "peacock", "seagull"}:
        return ["white", "yellow", "gold"]
    if slug in {"ruins", "safe"}:
        return ["white", "stone", "dark_stone"]
    if slug == "sickness":
        return ["white", "lime", "green"]
    if slug in {"sunglasses", "swim-goggles", "water-gun"}:
        return ["white", "ice", "sky"]
    if slug in {"taser", "the-one-ring", "alarm-clock"}:
        return ["white", "yellow", "gold"]
    if slug in {"toucan", "turtle", "bonsai-tree", "chameleon"}:
        return ["white", "lime", "green"]
    if slug in {"twilight", "black-hole"}:
        return ["white", "violet", "sky"]
    if slug in {"wind-turbine", "bone", "caviar"}:
        return ["white", "ice", "sky"]
    if slug == "charcoal":
        return ["white", "stone", "orange"]
    if slug in {"chicken", "crow", "cuckoo", "duckling", "dinosaur"}:
        return ["white", "yellow", "gold"]
    if slug in {"christmas-tree", "family-tree"}:
        return ["white", "yellow", "lime"]
    if slug in {"computer", "constellation", "drone", "dry-ice", "flashlight"}:
        return ["white", "ice", "sky"]
    if slug in {"egg-timer", "engineer", "fire-extinguisher"}:
        return ["white", "yellow", "gold"]
    if slug == "frankenstein":
        return ["white", "lime", "green"]
    if slug in {"fridge", "lighthouse", "narwhal", "optical-fiber"}:
        return ["white", "ice", "sky"]
    if slug in {"fruit", "harp", "lava-lamp", "mayonnaise", "oil-lamp"}:
        return ["white", "yellow", "gold"]
    if slug in {"herb", "leaf"}:
        return ["white", "lime", "green"]
    if slug in {"grave", "mummy"}:
        return ["white", "stone", "paper"]
    if slug in {"jedi", "monarch"}:
        return ["white", "yellow", "sky"]
    if slug == "livestock":
        return ["white", "paper", "gold"]
    if slug in {"palm", "rose"}:
        return ["white", "lime", "green"]
    if slug in {"pegasus", "pigeon", "pilot", "seaplane", "seasickness", "shark", "smog", "soap", "solar-cell"}:
        return ["white", "ice", "sky"]
    if slug in {"pitchfork", "sewing-machine", "shuriken", "skeleton"}:
        return ["white", "ice", "stone"]
    if slug in {"soda"}:
        return ["white", "yellow", "orange"]
    if slug in {"spaceship", "statue", "steam-engine", "sundial", "watch"}:
        return ["white", "ice", "sky"]
    if slug in {"starfish", "super-nova", "umbrella", "alcohol"}:
        return ["white", "yellow", "orange"]
    if slug in {"swimmer", "acid-rain"}:
        return ["white", "ice", "sky"]
    if slug in {"thread", "treehouse"}:
        return ["white", "yellow", "lime"]
    if slug in {"vampire", "vulture", "zombie"}:
        return ["white", "paper", "lime"]
    if slug in {"alien", "electric-eel"}:
        return ["white", "yellow", "lime"]
    if slug in {"antarctica", "avalanche", "blizzard"}:
        return ["white", "ice", "sky"]
    if slug in {"broom", "camel", "chicken-wing", "coconut", "coffin", "crown", "campfire"}:
        return ["white", "yellow", "orange"]
    if slug in {"bulletproof-vest", "darth-vader", "doctor"}:
        return ["white", "ice", "sky"]
    if slug == "chicken-soup":
        return ["white", "yellow", "sky"]
    if slug in {"fabric", "flute", "fountain", "glacier", "hail", "iceberg", "igloo"}:
        return ["white", "ice", "sky"]
    if slug in {"fence", "flour", "fossil", "gravestone", "graveyard"}:
        return ["white", "paper", "gold"]
    if slug in {"fruit-tree", "gnome", "goat", "godzilla"}:
        return ["white", "lime", "green"]
    return ["white", "ice", "gold"]


def clustered_idle_pixels(slug, pixels, frame, colors):
    shifted = copy_pixels(pixels)
    if not colors:
        return shifted

    bounds = pixel_bounds(pixels)
    if bounds is None:
        return shifted

    min_x, min_y, max_x, max_y = bounds
    opaque = [
        (x, y)
        for y, row in enumerate(pixels)
        for x, pixel in enumerate(row)
        if pixel[3] > 0
    ]
    if not opaque:
        return shifted

    seed = sum((index + 1) * ord(char) for index, char in enumerate(slug))
    cluster_count = 2 if len(opaque) < 120 else 3
    for cluster in range(cluster_count):
        center_x, center_y = opaque[(seed + frame * 19 + cluster * 41) % len(opaque)]
        paint_cluster(
            shifted,
            center_x,
            center_y,
            colors,
            frame + cluster,
            min_x,
            min_y,
            max_x,
            max_y,
        )

    return shifted


def pixel_bounds(pixels):
    coordinates = [
        (x, y)
        for y, row in enumerate(pixels)
        for x, pixel in enumerate(row)
        if pixel[3] > 0
    ]
    if not coordinates:
        return None
    xs = [x for x, _ in coordinates]
    ys = [y for _, y in coordinates]
    return min(xs), min(ys), max(xs), max(ys)


def paint_cluster(pixels, x, y, colors, color_index, min_x, min_y, max_x, max_y):
    x0 = max(min_x, min(x, max_x))
    y0 = max(min_y, min(y, max_y))
    x1 = min(max_x + 1, x0 + 2)
    y1 = min(max_y + 1, y0 + 2)
    if x1 - x0 < 2 and x0 > min_x:
        x0 -= 1
    if y1 - y0 < 2 and y0 > min_y:
        y0 -= 1
    for py in range(y0, y1):
        for px in range(x0, x1):
            current = pixels[py][px]
            for offset in range(len(colors)):
                color = COLORS[colors[(color_index + offset) % len(colors)]]
                if color != current:
                    pixels[py][px] = color
                    break


def shift_pixels(pixels, dx, dy):
    shifted = canvas()
    for y, row in enumerate(pixels):
        for x, pixel in enumerate(row):
            if pixel[3] == 0:
                continue
            nx = x + dx
            ny = y + dy
            if 0 <= nx < 32 and 0 <= ny < 32:
                shifted[ny][nx] = pixel
    return shifted


def write_png(path, pixels):
    path.parent.mkdir(parents=True, exist_ok=True)
    raw = b"".join(b"\x00" + b"".join(bytes(pixel) for pixel in row) for row in pixels)

    def chunk(kind, payload):
        return (
            struct.pack(">I", len(payload))
            + kind
            + payload
            + struct.pack(">I", zlib.crc32(kind + payload) & 0xFFFFFFFF)
        )

    data = b"\x89PNG\r\n\x1a\n"
    data += chunk(b"IHDR", struct.pack(">IIBBBBB", 32, 32, 8, 6, 0, 0, 0))
    data += chunk(b"IDAT", zlib.compress(raw, 9))
    data += chunk(b"IEND", b"")
    path.write_bytes(data)


def manifest_entry(catalog, slug, name, output_path, source_icon):
    return {
        "catalog": catalog,
        "message": "readable first-pass local atlas seed",
        "model": "local-seed",
        "name": name,
        "output_path": str(output_path.relative_to(ROOT)),
        "prompt_version": "pixel-atlas-v1",
        "slug": slug,
        "source_icon": str(source_icon) if source_icon else None,
        "status": "seeded",
        "updated_at": int(time.time()),
    }


if __name__ == "__main__":
    raise SystemExit(main())
