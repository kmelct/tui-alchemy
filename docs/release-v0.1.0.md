# tui-alchemy v0.1.0

## Summary

`tui-alchemy` is ready for its first GitHub release: a terminal alchemy crafting game with a Ratatui UI, pixel-art sprites, mouse and keyboard controls, and a 755-element recipe catalog.

## Highlights

- Start with `Air`, `Earth`, `Fire`, and `Water`, then discover hundreds of new elements by combining pairs.
- Play with keyboard navigation, direct number selection, or mouse drag/drop.
- Use the recipe table to preview `ingredient + ingredient = result` discoveries.
- Browse discoveries in a paged atlas with terminal-native pixel art.
- Read the root `README.md` for a GitHub-optimized tutorial with screenshots and maintenance instructions.
- Run deterministic screenshot generation for release-quality visual QA.

## Screenshots

Fresh game:

![Fresh game](https://github.com/kmelct/tui-alchemy/raw/v0.1.0/docs/screenshots/01-initial.png)

First discovery:

![Steam discovered](https://github.com/kmelct/tui-alchemy/raw/v0.1.0/docs/screenshots/02-created-steam.png)

Populated atlas:

![Populated atlas](https://github.com/kmelct/tui-alchemy/raw/v0.1.0/docs/screenshots/04-populated-board.png)

Responsive layout:

![Large layout](https://github.com/kmelct/tui-alchemy/raw/v0.1.0/docs/screenshots/06-xlarge.png)

## Package metadata

- Version: `0.1.0`
- Repository: `https://github.com/kmelct/tui-alchemy`
- License: MIT
- README: `README.md`
- Runtime catalog: `data/little_alchemy.json`
- Documentation screenshots: `docs/screenshots/`

## Verification before publishing

Run these commands from the repository root:

```sh
cargo test
cargo ci-clippy
cargo run --example screenshot
cargo package
```

## Suggested GitHub release command

After committing the release changes and tagging the release commit:

```sh
git tag v0.1.0
git push origin v0.1.0
gh release create v0.1.0 --draft --title "tui-alchemy v0.1.0" --notes-file docs/release-v0.1.0.md
```
