# tui-alchemy agent instructions

## Release workflow

When preparing a full release, do not stop at documentation or tests. Complete the whole publishing lane.

1. Ensure the root `README.md` is the player-facing GitHub README and references screenshots from `docs/screenshots/`.
2. Keep release-facing docs in `docs/`; remove obsolete generated HTML or preview-only documentation.
3. Update package metadata in `Cargo.toml`:
   - `version`
   - `rust-version`
   - `description`
   - `readme`
   - `repository`
   - `homepage`
   - `documentation`
   - `license`
   - `keywords`
   - `categories`
   - `include`
4. Update `CHANGELOG.md` with the release entry.
5. Update `docs/release-vX.Y.Z.md` with GitHub release notes and screenshot links.
6. Regenerate visual evidence:

   ```sh
   cargo run --example screenshot
   ```

7. Copy curated release screenshots from `output/screenshot/` into `docs/screenshots/`.
8. Run the verification lane:

   ```sh
   cargo test
   cargo ci-clippy
   python -m unittest tests.test_generate_pixel_sprites
   cargo package
   ```

9. Verify CLI metadata:

   ```sh
   cargo run -- --help
   cargo run -- --version
   ```

10. Commit all intended release changes.
11. Push the branch to GitHub.
12. Create the GitHub release draft:

    ```sh
    git tag vX.Y.Z
    git push origin vX.Y.Z
    gh release create vX.Y.Z --draft --title "tui-alchemy vX.Y.Z" --notes-file docs/release-vX.Y.Z.md
    ```

## Release constraints

- Do not publish with failing tests, Clippy warnings, dirty screenshots, stale release notes, or package warnings.
- Do not use `--allow-dirty` for the final package verification after committing; it is only acceptable for pre-commit checks.
- Keep screenshots deterministic and checked in only under `docs/screenshots/` for README and release use.
- Runtime data source is `data/little_alchemy.json`; do not reintroduce legacy catalog files.
- Runtime sprites come from `assets/pixel-sprites/`; do not reintroduce SVG runtime rendering dependencies.
