#!/usr/bin/env sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$root"

version=""
while IFS= read -r line; do
  case "$line" in
    'version = "'*'"')
      version=${line#'version = "'}
      version=${version%'"'}
      break
      ;;
  esac
done < Cargo.toml
if [ -z "$version" ]; then
  printf '%s\n' "Cargo.toml package version not found" >&2
  exit 1
fi

DIST="website/dist"
rm -rf "$DIST"
mkdir -p "$DIST/assets" "$DIST/downloads"

# 1) Static page + static assets (HTML, CSS, art, sprites, fonts, icons).
cp website/index.html "$DIST/index.html"
cp website/_worker.js "$DIST/_worker.js"
cp website/site.webmanifest "$DIST/site.webmanifest"
cp -R website/assets/. "$DIST/assets/"
rm -rf "$DIST/assets/gen/raw"  # never ship the raw pre-pixelated generation sources

# 2) The real ratatui game, compiled to WebAssembly for the live demo.
cargo build \
  --manifest-path website/packages/alchemy-wasm/Cargo.toml \
  --target wasm32-unknown-unknown --release --locked
cp "website/packages/alchemy-wasm/target/wasm32-unknown-unknown/release/alchemy_terminal_wasm.wasm" \
  "$DIST/assets/alchemy_terminal_wasm.wasm"

# 3) The xterm.js <-> wasm bridge, bundled with esbuild (required — the live demo is the
#    hero feature, so a missing bundler must fail the build rather than ship a dead demo).
if [ ! -d website/node_modules ]; then
  if [ -f website/package-lock.json ]; then
    npm --prefix website ci
  else
    npm --prefix website install
  fi
fi
if [ ! -x website/node_modules/.bin/esbuild ]; then
  printf '%s\n' "esbuild not found in website/node_modules — cannot bundle the live demo bridge" >&2
  exit 1
fi
website/node_modules/.bin/esbuild website/packages/web-terminal/src/index.js \
  --bundle --format=esm --target=es2020 --outfile="$DIST/assets/terminal.js"
website/node_modules/.bin/esbuild website/packages/web-terminal/src/terminal.css \
  --bundle --minify --outfile="$DIST/assets/terminal.css"

# 4) Installer scripts served from the installer subdomain.
cp scripts/install-tui-alchemy.sh "$DIST/i.tui-alchemy.sh"
cp scripts/install-tui-alchemy.ps1 "$DIST/install.ps1"

crate="target/package/tui-alchemy-$version.crate"
if [ -f "$crate" ]; then
  cp "$crate" "$DIST/downloads/tui-alchemy-$version.crate"
fi

printf '%s\n' "Built website/dist (static landing page + live wasm demo)."
