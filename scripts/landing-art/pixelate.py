#!/usr/bin/env python3
"""Turn an AI-generated PNG into believable, on-palette pixel art.

Pipeline: downscale (chunky pixels) -> remap to the Lost Century palette
-> keep it small so CSS upscales it with image-rendering:pixelated.
Transparency is preserved (alpha is thresholded to stay crisp).

Usage:
  python3 scripts/landing-art/pixelate.py \
      --in  website/assets/gen/raw/wax-seal.png \
      --out website/assets/gen/wax-seal.png \
      --width 128 --dither
"""
import argparse
import os

from PIL import Image, ImageChops

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))

# Lost Century family used by the in-game theme + landing tokens.
PALETTE_HEX = [
    "0c0e16", "0a0b12", "10131c", "161924", "1c2030", "2e3448", "383e58",
    "4b3d44", "5c4226", "79444a", "ae5d40", "c77b58",
    "ba9158", "b3a555", "d6c97a", "d2c9a5", "c9bd92",
    "8caba1", "4b726e", "77743b", "9a8d88", "847875",
]


def build_palette_image() -> Image.Image:
    pal = []
    for hx in PALETTE_HEX:
        pal += [int(hx[0:2], 16), int(hx[2:4], 16), int(hx[4:6], 16)]
    pal += [0, 0, 0] * (256 - len(PALETTE_HEX))
    palimg = Image.new("P", (1, 1))
    palimg.putpalette(pal)
    return palimg


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--in", dest="src", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--width", type=int, default=480, help="pixel-art width in px")
    ap.add_argument("--dither", action="store_true")
    ap.add_argument("--no-remap", action="store_true", help="skip palette remap")
    ap.add_argument("--alpha-threshold", type=int, default=128)
    ap.add_argument("--chroma", choices=["magenta"], help="key out a flat magenta background")
    ap.add_argument("--chroma-tol", type=int, default=60)
    args = ap.parse_args()

    src = Image.open(os.path.join(ROOT, args.src)).convert("RGBA")

    if args.chroma == "magenta":
        # Magenta == red>>green AND blue>>green. The warm Lost Century palette
        # never satisfies both (its colors are low-blue), so this is safe.
        r, g, b, _ = src.split()
        rg = ImageChops.subtract(r, g)
        bg = ImageChops.subtract(b, g)
        magentaness = ImageChops.darker(rg, bg)  # min of the two
        t = args.chroma_tol
        keyed_alpha = magentaness.point(lambda v: 0 if v > t else 255)
        src.putalpha(keyed_alpha)
        # Zero the RGB of keyed (background) pixels so the later BOX downscale does not
        # average magenta into surviving edge pixels (which would leave a pink fringe).
        src = Image.composite(src, Image.new("RGBA", src.size, (0, 0, 0, 0)), keyed_alpha)

    w, h = src.size
    tw = args.width
    th = max(1, round(h * tw / w))
    small = src.resize((tw, th), Image.BOX)

    rgb = small.convert("RGB")
    alpha = small.getchannel("A")

    if not args.no_remap:
        palimg = build_palette_image()
        dither = Image.Dither.FLOYDSTEINBERG if args.dither else Image.Dither.NONE
        rgb = rgb.quantize(palette=palimg, dither=dither).convert("RGB")

    out = rgb.convert("RGBA")
    # Re-apply crisp alpha (threshold so edges stay hard, not feathered).
    alpha = alpha.point(lambda a: 255 if a >= args.alpha_threshold else 0)
    out.putalpha(alpha)

    dst = os.path.join(ROOT, args.out)
    os.makedirs(os.path.dirname(dst) or ".", exist_ok=True)
    out.save(dst)
    print(f"OK {args.out}  {w}x{h} -> {tw}x{th}  remap={not args.no_remap} dither={args.dither}")


if __name__ == "__main__":
    main()
