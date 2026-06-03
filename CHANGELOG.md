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
- Standalone README screenshot updater under `scripts/`.

### Fixed

- PNG screenshot rendering now initializes empty buffers with the opaque scene background instead of transparent pixels.
- Workbench result slots stay empty until a pair resolves, so single-selection screenshots no longer imply a result.

### Removed

- Preview-only `examples/` and unsupported Python tooling.

### Verified release lanes

- `cargo test`
- `cargo ci-clippy`
- `scripts/update-readme-screenshots.sh`
- `cargo package`
