#!/usr/bin/env sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$root"

if [ -f .env ]; then
  set -a
  . ./.env
  set +a
fi

: "${CLOUDFLARE_PAGES_ACCOUNT_ID:=e9c375806f33a6c2a42c7d5ca9729105}"
: "${CLOUDFLARE_R2_ACCOUNT_ID:=279a8319536bf8f797e9d25954fe445c}"
: "${CLOUDFLARE_ACCOUNT_ID:=$CLOUDFLARE_PAGES_ACCOUNT_ID}"
: "${CLOUDFLARE_PAGES_PROJECT:=tui-alchemy}"
: "${CLOUDFLARE_PAGES_BRANCH:=master}"
: "${CLOUDFLARE_R2_BUCKET:=tui-alchemy-assets}"

need() {
  if ! command -v "$1" >/dev/null 2>&1; then
    printf '%s\n' "error: $2 is required." >&2
    exit 1
  fi
}

need cargo "Cargo"
need node "Node.js"
need npx "npx"

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
  printf '%s\n' "error: Cargo.toml package version not found" >&2
  exit 1
fi

cargo package --allow-dirty
scripts/build-website.sh

node scripts/package-current-binary.mjs
if command -v docker >/dev/null 2>&1 && [ "${TUI_ALCHEMY_SKIP_DOCKER_BINARY:-0}" != "1" ]; then
  if docker info >/dev/null 2>&1; then
    scripts/package-linux-binary-in-docker.sh
  else
    printf '%s\n' "warning: Docker is installed but the daemon is unavailable; skipping Linux binary package." >&2
  fi
fi
node scripts/upload-r2-assets.mjs

CLOUDFLARE_ACCOUNT_ID="$CLOUDFLARE_PAGES_ACCOUNT_ID" node scripts/provision-cloudflare-pages.mjs

CLOUDFLARE_ACCOUNT_ID="$CLOUDFLARE_PAGES_ACCOUNT_ID" npx wrangler@latest pages deploy website/dist \
  --project-name "$CLOUDFLARE_PAGES_PROJECT" \
  --branch "$CLOUDFLARE_PAGES_BRANCH" \
  --commit-dirty=true
