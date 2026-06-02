# Changelog

All notable changes to `tui-alchemy` are tracked here. The project follows semantic versioning for release tags.

## [0.1.0] - 2026-06-02

### Added

- Terminal alchemy game with a 755-element combined recipe catalog.
- Ratatui/Crossterm application loop with keyboard and mouse interaction.
- Terminal-native pixel-art sprite renderer using checked-in runtime assets.
- Discovery atlas, progress rail, and three-slot recipe table workbench.
- Deterministic screenshot harness for release and visual QA evidence.
- Tutorial README with curated screenshots and maintenance instructions.
- Package metadata for GitHub and crate packaging.
- Release instructions in `CLAUDE.md`.

### Fixed

- PNG screenshot rendering now initializes empty buffers with the opaque scene background instead of transparent pixels.

### Verified release lanes

- `cargo test`
- `cargo ci-clippy`
- `cargo run --example screenshot`
- `cargo package`
