#!/usr/bin/env sh
set -eu

cargo run \
  --manifest-path scripts/update-readme-screenshots/Cargo.toml \
  --target-dir target/readme-screenshots
