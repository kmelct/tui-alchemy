# Installing tui-alchemy

`tui-alchemy` is published as both a prebuilt command-line binary and a crates.io package.

## Recommended install

Use the installer when you just want to play:

```sh
curl -fsSL https://i.tui-alchemy.sh | sh
```

The installer first downloads the matching prebuilt binary for your OS and CPU. It does not require Rust, Cargo, Python, Git, or a compiler toolchain for that primary path. On Unix-like systems it checks for the small runtime tools it needs (`tar` plus `curl` or `wget`); when one is missing, it detects the available package manager, asks for approval, installs the dependency, and continues. Set `TUI_ALCHEMY_YES=1` for non-interactive approval in automation.

On Windows PowerShell:

```powershell
irm https://i.tui-alchemy.sh/install.ps1 | iex
```

## Cargo install path

If you already use Rust, install the published package directly from crates.io:

```sh
cargo install tui-alchemy --locked
```

Cargo's install root is selected by `--root`, then `CARGO_INSTALL_ROOT`, then Cargo config, then `CARGO_HOME`, then `$HOME/.cargo`. The executable lands in the install root's `bin` directory. Add that directory to `PATH` if your shell cannot find `tui-alchemy` after installation.

The website installer falls back to the published package with:

```sh
cargo install tui-alchemy --version 0.2.0 --locked --force
```

This follows the Cargo Book guidance for reproducible installs: `--locked` uses the packaged `Cargo.lock` instead of resolving newer dependency versions at install time.

## Release package checks

Before publishing a new version, run the same checks Cargo recommends:

```sh
cargo publish --dry-run
cargo package --list
cargo package
```

`cargo publish --dry-run` performs Cargo's package verification without uploading. `cargo package --list` shows the exact files that will be included in the `.crate` archive so large website artifacts, generated binaries, local environment files, and unrelated assets do not accidentally ship inside the crates.io source package.

A published crates.io version is permanent: the uploaded source cannot be overwritten or deleted. If a version is broken, use `cargo yank --version <version>` to prevent new dependency resolution while preserving existing lockfiles.

## Binary asset naming

Prebuilt archives are served from Cloudflare R2 under:

```text
https://pub-ec563771aa2c4e0f942506be4f1593ce.r2.dev/downloads/tui-alchemy-<version>-<target>.tar.gz
```

Examples:

```text
tui-alchemy-0.2.0-x86_64-unknown-linux-gnu.tar.gz
tui-alchemy-0.2.0-aarch64-unknown-linux-gnu.tar.gz
tui-alchemy-0.2.0-aarch64-apple-darwin.tar.gz
tui-alchemy-0.2.0-x86_64-pc-windows-msvc.tar.gz
```

Each archive contains one executable at the archive root: `tui-alchemy` on Unix-like platforms and `tui-alchemy.exe` on Windows.
