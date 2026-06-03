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

rm -rf website/dist
mkdir -p website/dist/assets website/dist/downloads
cp website/index.html website/dist/index.html
cp website/_worker.js website/dist/_worker.js
cp website/site.webmanifest website/dist/site.webmanifest
cp website/assets/* website/dist/assets/
cp scripts/install-tui-alchemy.sh website/dist/i.tui-alchemy.sh
cp scripts/install-tui-alchemy.ps1 website/dist/install.ps1

if [ -f website/package.json ]; then
  if [ -f website/package-lock.json ]; then
    npm --prefix website ci
  else
    npm --prefix website install
  fi
  npm --prefix website run build
fi

crate="target/package/tui-alchemy-$version.crate"
if [ -f "$crate" ]; then
  cp "$crate" "website/dist/downloads/tui-alchemy-$version.crate"
fi
