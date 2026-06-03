# Landing page redesign — "The Alchemist's Console"

Design spec for the tui-alchemy landing page (`website/index.html` — a single static
page, no framework). Supersedes the
"hero + relic field" layout. Goal stated by owner: make it read as a **hand-made,
modern pixel-fantasy scene** — "less AI-ish" — with the live demo housed in a retro
home-computer, and **factually correct** game content.

## Concept

A single-screen (no-scroll) pixel-fantasy **alchemist's workshop scene** that fills the
viewport. On the workbench sits a **Commodore-64-style retro computer** whose CRT screen
is the **live WASM ratatui demo** — honest, because the game really runs a shell. Around
it: a parchment with the wordmark + a real recipe, the copper install bar, a wax seal,
candlelight, and restrained diegetic animation.

It fuses the three researched directions: parchment-craft restraint (A) + a custom
fantasy workshop scene (B) + the retro-machine game identity (C).

## Locked decisions

1. **Single viewport, no scroll.** `100dvh`, the whole screen is the scene. The demo must
   **stack-but-stay** on small screens — never `display:none` (the old code deleted it
   under 600px height; that bug is fixed by keeping it visible/letterboxed).
2. **Recipes must be correct.** The game combines **exactly two** elements. The old
   `air+earth+fire+water → steam` four-into-one formula is wrong and is removed. Use real
   two-element recipes from `data/little_alchemy.json`:
   - `Water + Fire → Steam`  ·  `Earth + Fire → Lava`  ·  `Water + Air → Rain`
   - (also valid: `Water + Earth → Mud`, `Air + Fire → Energy`, `Earth + Air → Dust`)
3. **Commodore-64 flavor** for the machine (owner preference), built in **CSS** so the live
   xterm sits in it pixel-perfectly across breakpoints. Generated art surrounds it.
4. **Custom imagery: full set** via gpt-image-2, palette-remapped to the Lost Century 16.

## The `dash://` fix (owner priority #1/#2)

Delete the `.hero-console` block (`dash:// terminal alchemy / mode pixel-art crafting
game`) entirely. Replace with a **real recipe**, rendered in actual element sprites, that
animates **once** on load then rests: two reagents settle in, the `→` draws, the result
pops in with the page's single gold glow. Cycles through the correct recipes above (or
shows one). It says something true; it is literally the game.

Also remove the same-family slop: the `POWER-ON SELF TEST / DASH:// BIOS / ARCANE MEMORY
OK` boot text → one honest `loading alchemy.wasm…`; the `DASH:// ALCHEMY LINK` frame title
→ `tui-alchemy · live` (or real `80×24`).

## Visual language

- **Two surface worlds.** Warm parchment (`--paper #d2c9a5`, ink `--paper-ink #4b3d44`,
  ~7:1 AA) for human/copy; cool void (`#0c0e16`) for the machine/scene. This split is the
  art direction.
- **Type.** Silkscreen = wordmark/headings; Press Start 2P = micro-labels ≤12px only;
  VT323 = all prose/terminal. **Integer pixel sizes** for pixel fonts (fractional clamp
  smears them). Wordmark uses one hard `3px 3px 0` offset (carved relief), not triple glow.
- **Palette tokens added:** `--paper`, `--paper-2 #c9bd92`, `--paper-ink #4b3d44`,
  `--brick #79444a` (seal), `--teal-deep #4b726e` (CRT rim), `--olive #77743b`. Bump
  `--muted` `#847875 → #9a8d88` (AA).
- **Motion budget ≤4 continuous:** sprite bob · steam/ember from the candle · CRT
  flicker · cursor blink. Fire-once: recipe reveal, wasm load line. Copy button **stamps**
  like a seal. Everything stops under `prefers-reduced-motion`.
- **De-slop (net effect-code must go DOWN):** remove particle canvas, star twinkle,
  glow-ring, relic parallax field, mouse parallax. One glow color, one CTA, one seal.

## Generated assets (gpt-image-2 → palette-remapped, `image-rendering:pixelated`)

| File | Purpose | Transparent |
|---|---|---|
| `workshop-backdrop.png` | full-bleed scene: stone wall, shelves of glowing flasks, hanging herbs, candle, mortar; calm center for overlays | no |
| `c64-keyboard.png` | Commodore-64-style beige keyboard unit beneath the CRT | yes (magenta key) |
| `wax-seal.png` | brick-red maker's seal w/ alembic emblem | yes (magenta key) |
| `parchment-tile.png` | seamless aged-paper texture for the card | no |
| `og-card.png` | 1200×630 social hero (or final page screenshot) | no |

Tooling: `scripts/landing-art/generate.py` (gpt-image-2) + `scripts/landing-art/pixelate.py`
(downscale → magenta chroma-key → Lost Century palette remap).

## Implementation (minimalistic static)

A single static page — no web framework. `website/index.html` (markup + the click-to-copy
script) links `website/assets/main.css` (design system + scene) and the xterm bridge
(`website/packages/web-terminal/`, bundled by esbuild). The live demo is the real game
compiled to wasm (`website/packages/alchemy-wasm/`). `scripts/build-website.sh` assembles
`website/dist` (copy page + assets, `cargo build --target wasm32-unknown-unknown`, esbuild
the bridge) and Cloudflare Pages serves it — `pages_build_output_dir = website/dist`.

> A Dioxus rewrite was attempted for "cargo everywhere" but reverted as over-engineered for
> a one-page site (and Dioxus 0.7.9 fullstack/SSG is blocked upstream by an
> `axum 0.8.9 ↔ axum-macros 0.5.1` conflict). Static keeps the build trivial and the same
> deployment.

## Verification (end-to-end)

- `cargo test` — game logic (incl. the low-res layout rebalance).
- `cargo ci-clippy` (`clippy --all-targets -- -D warnings`) and `cargo check`.
- `npm --prefix website run check` — bridge JS syntax.
- `sh scripts/build-website.sh` then `node scripts/test-website.mjs` — builds + validates the
  static output (SEO head, real copy, no AI-slop regressions, demo wiring, deploy config).
- Serve `website/dist`, drive Chrome: desktop + mobile screenshots, zero console errors,
  demo boots + is interactive, copy works, reduced-motion honored, the demo stays visible on
  mobile (never `display:none`).
