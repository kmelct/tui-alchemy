# tui-alchemy v0.2.0

## Summary

`tui-alchemy` v0.2.0 adds a standalone website, public installer delivery, and Cloudflare deployment automation for the project site.

## Highlights

- Added `website/` with a cleaned standalone HTML page and extracted local assets.
- Fixed the website copy button with Clipboard API support and a non-secure-context fallback.
- Added Unix and Windows installers served from `i.tui-alchemy.sh`.
- Added dependency checks that prompt before installing a missing Rust toolchain.
- Added Cloudflare Pages and R2 deployment scripts for `tui-alchemy.sh`.

## Install

Unix-like shells:

```sh
curl -fsSL https://i.tui-alchemy.sh | sh
```

Windows PowerShell:

```powershell
irm https://i.tui-alchemy.sh/install.ps1 | iex
```

## Screenshots

Hero:

![Alchemy TUI hero](https://github.com/kmelct/tui-alchemy/raw/v0.2.0/docs/screenshots/hero.png)

First discovery:

![Steam discovered](https://github.com/kmelct/tui-alchemy/raw/v0.2.0/docs/screenshots/03-get-result.png)

Populated atlas:

![Populated atlas](https://github.com/kmelct/tui-alchemy/raw/v0.2.0/docs/screenshots/04-populated-board.png)

Responsive layout:

![Large layout](https://github.com/kmelct/tui-alchemy/raw/v0.2.0/docs/screenshots/06-xlarge.png)

## Package metadata

- Version: `0.2.0`
- Repository: `https://github.com/kmelct/tui-alchemy`
- License: MIT
- README: `README.md`
- Runtime catalog: `data/little_alchemy.json`
- Documentation screenshots: `docs/screenshots/`
- Website source: `website/`

## Verification before publishing

Run these commands from the repository root:

```sh
cargo test
cargo ci-clippy
scripts/update-readme-screenshots.sh
scripts/build-website.sh
node scripts/test-website.mjs
cargo package
```

## Suggested GitHub release command

After committing the release changes and tagging the release commit:

```sh
git tag v0.2.0
git push origin v0.2.0
gh release create v0.2.0 --draft --title "tui-alchemy v0.2.0" --notes-file docs/release-v0.2.0.md
```
