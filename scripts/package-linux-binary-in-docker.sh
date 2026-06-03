#!/usr/bin/env sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
image=${TUI_ALCHEMY_RUST_DOCKER_IMAGE:-rust:1.88-bookworm}

docker run --rm \
  -v "$root:/work" \
  -w /work \
  "$image" \
  sh -eu -c '
    version=""
    while IFS= read -r line; do
      case "$line" in
        "version = \""*"\"")
          version=${line#"version = \""}
          version=${version%"\""}
          break
          ;;
      esac
    done < Cargo.toml
    if [ -z "$version" ]; then
      printf "%s\n" "Cargo.toml package version not found" >&2
      exit 1
    fi
    host=""
    rustc_info=$(rustc -vV)
    old_ifs=$IFS
    IFS="
"
    for line in $rustc_info; do
      case "$line" in
        "host: "*) host=${line#"host: "}; break ;;
      esac
    done
    IFS=$old_ifs
    if [ -z "$host" ]; then
      printf "%s\n" "rustc host triple not found" >&2
      exit 1
    fi
    cargo build --release --locked
    mkdir -p website/dist/downloads
    tmp_dir=$(mktemp -d)
    cp target/release/tui-alchemy "$tmp_dir/tui-alchemy"
    tar -czf "website/dist/downloads/tui-alchemy-$version-$host.tar.gz" -C "$tmp_dir" tui-alchemy
    rm -rf "$tmp_dir"
    printf "%s\n" "Packaged website/dist/downloads/tui-alchemy-$version-$host.tar.gz"
  '
