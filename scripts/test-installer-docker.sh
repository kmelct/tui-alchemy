#!/usr/bin/env sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
image=${TUI_ALCHEMY_INSTALLER_TEST_IMAGE:-debian:bookworm-slim}

version=""
while IFS= read -r line; do
  case "$line" in
    'version = "'*'"')
      version=${line#'version = "'}
      version=${version%'"'}
      break
      ;;
  esac
done < "$root/Cargo.toml"
if [ -z "$version" ]; then
  printf '%s\n' "Cargo.toml package version not found" >&2
  exit 1
fi

scripts/package-linux-binary-in-docker.sh

docker run --rm \
  -v "$root:/work" \
  -w /work \
  "$image" \
  sh -eu -c '
    if command -v curl >/dev/null 2>&1 || command -v wget >/dev/null 2>&1; then
      printf "%s\n" "installer test image unexpectedly has a download tool" >&2
      exit 1
    fi
    if command -v cargo >/dev/null 2>&1 || command -v rustc >/dev/null 2>&1 || command -v python >/dev/null 2>&1 || command -v python3 >/dev/null 2>&1; then
      printf "%s\n" "installer test image unexpectedly has Rust or Python" >&2
      exit 1
    fi
    arch=$(uname -m)
    case "$arch" in
      x86_64|amd64) triple=x86_64-unknown-linux-gnu ;;
      aarch64|arm64) triple=aarch64-unknown-linux-gnu ;;
      *) printf "%s\n" "unsupported docker arch: $arch" >&2; exit 1 ;;
    esac
    TUI_ALCHEMY_YES=1 \
    TUI_ALCHEMY_INSTALL_DIR=/usr/local/bin \
      sh scripts/install-tui-alchemy.sh
    version_output=$(tui-alchemy --version)
    case "$version_output" in
      "tui-alchemy '"$version"'") ;;
      *) printf "%s\n" "unexpected version output: $version_output" >&2; exit 1 ;;
    esac
  '
