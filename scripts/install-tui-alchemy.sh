#!/usr/bin/env sh
set -eu

APP_NAME="tui-alchemy"
APP_VERSION="${TUI_ALCHEMY_VERSION:-0.2.0}"
ASSET_BASE_URL="${TUI_ALCHEMY_ASSET_BASE_URL:-https://pub-ec563771aa2c4e0f942506be4f1593ce.r2.dev}"
BINARY_BASE_URL="${TUI_ALCHEMY_BINARY_BASE_URL:-$ASSET_BASE_URL/downloads}"
BINARY_URL="${TUI_ALCHEMY_BINARY_URL:-}"
INSTALL_DIR="${TUI_ALCHEMY_INSTALL_DIR:-}"
AUTO_YES="${TUI_ALCHEMY_YES:-0}"


has_command() {
  command -v "$1" >/dev/null 2>&1
}

die() {
  printf '%s\n' "error: $*" >&2
  exit 1
}
prompt_yes_no() {
  prompt=$1
  if [ "$AUTO_YES" = "1" ] || [ "$AUTO_YES" = "true" ]; then
    printf '%s\n' "$prompt yes"
    return 0
  fi
  if [ ! -t 0 ]; then
    return 1
  fi
  printf '%s [y/N] ' "$prompt" >&2
  read answer
  case "$answer" in
    y|Y|yes|YES) return 0 ;;
    *) return 1 ;;
  esac
}

run_privileged() {
  if [ "$(id -u 2>/dev/null || printf 1)" = "0" ]; then
    "$@"
    return $?
  fi
  if has_command sudo; then
    sudo "$@"
    return $?
  fi
  return 1
}

install_with_package_manager() {
  package=$1
  if has_command apt-get; then
    run_privileged apt-get update && run_privileged apt-get install -y ca-certificates "$package"
    return $?
  fi
  if has_command dnf; then
    run_privileged dnf install -y ca-certificates "$package"
    return $?
  fi
  if has_command yum; then
    run_privileged yum install -y ca-certificates "$package"
    return $?
  fi
  if has_command apk; then
    run_privileged apk add --no-cache ca-certificates "$package"
    return $?
  fi
  if has_command pacman; then
    run_privileged pacman -Sy --noconfirm ca-certificates "$package"
    return $?
  fi
  if has_command zypper; then
    run_privileged zypper --non-interactive install ca-certificates "$package"
    return $?
  fi
  if has_command brew; then
    brew install "$package"
    return $?
  fi
  return 1
}

install_missing_dependency() {
  label=$1
  package=$2
  prompt_yes_no "$APP_NAME needs $label to install the prebuilt binary. Install $package now?" || return 1
  install_with_package_manager "$package"
}

ensure_download_tool() {
  if has_command curl || has_command wget; then
    return 0
  fi
  install_missing_dependency "curl or wget" "curl" || return 1
  has_command curl || has_command wget
}

ensure_archive_tool() {
  if has_command tar; then
    return 0
  fi
  install_missing_dependency "tar" "tar" || return 1
  has_command tar
}


download_file() {
  url=$1
  destination=$2
  case "$url" in
    file://*)
      cp "${url#file://}" "$destination"
      return $?
      ;;
  esac
  if has_command curl; then
    curl -fsSL --retry 2 --connect-timeout 10 --output "$destination" "$url"
    return $?
  fi
  if has_command wget; then
    wget -q -O "$destination" "$url"
    return $?
  fi
  return 127
}

host_triple() {
  os=$(uname -s 2>/dev/null || printf unknown)
  arch=$(uname -m 2>/dev/null || printf unknown)
  case "$os:$arch" in
    Linux:x86_64|Linux:amd64) printf '%s\n' x86_64-unknown-linux-gnu ;;
    Linux:aarch64|Linux:arm64) printf '%s\n' aarch64-unknown-linux-gnu ;;
    Darwin:x86_64|Darwin:amd64) printf '%s\n' x86_64-apple-darwin ;;
    Darwin:aarch64|Darwin:arm64) printf '%s\n' aarch64-apple-darwin ;;
    *) return 1 ;;
  esac
}

install_dir() {
  if [ -n "$INSTALL_DIR" ]; then
    printf '%s\n' "$INSTALL_DIR"
    return
  fi
  if [ -d /usr/local/bin ] && [ -w /usr/local/bin ]; then
    printf '%s\n' /usr/local/bin
    return
  fi
  printf '%s\n' "$HOME/.local/bin"
}

install_binary_file() {
  source_file=$1
  destination_dir=$(install_dir)
  mkdir -p "$destination_dir" || return 1
  destination="$destination_dir/$APP_NAME"
  if has_command install; then
    install -m 755 "$source_file" "$destination"
  else
    cp "$source_file" "$destination"
    chmod 755 "$destination"
  fi
  printf '%s\n' "$destination"
}

install_from_binary() {
  triple=$(host_triple) || return 1
  archive_name="$APP_NAME-$APP_VERSION-$triple.tar.gz"
  url=${BINARY_URL:-$BINARY_BASE_URL/$archive_name}
  tmp_dir=$(mktemp -d 2>/dev/null || mktemp -d -t tui-alchemy)
  archive="$tmp_dir/$archive_name"
  trap 'rm -rf "$tmp_dir"' EXIT HUP INT TERM
  ensure_archive_tool || return 1
  case "$url" in
    file://*) ;;
    *) ensure_download_tool || return 1 ;;
  esac
  download_file "$url" "$archive" || return 1
  tar -xzf "$archive" -C "$tmp_dir" || return 1
  [ -f "$tmp_dir/$APP_NAME" ] || return 1
  installed_path=$(install_binary_file "$tmp_dir/$APP_NAME") || return 1
  printf '%s %s installed successfully. Run: %s\n' "$APP_NAME" "$APP_VERSION" "$installed_path"
  case ":$PATH:" in
    *":$(dirname "$installed_path"):"*) ;;
    *) printf 'Add this directory to PATH for shorter launches: %s\n' "$(dirname "$installed_path")" >&2 ;;
  esac
}

install_from_cargo_package() {
  has_command cargo || return 1
  cargo install "$APP_NAME" --version "$APP_VERSION" --locked --force
}

case "$(uname -s 2>/dev/null || printf unknown)" in
  Darwin|Linux) ;;
  *) die "Unsupported OS for this shell installer. On Windows, run: irm https://i.tui-alchemy.sh/install.ps1 | iex" ;;
esac

if install_from_binary; then
  exit 0
fi

if install_from_cargo_package; then
  printf '%s %s installed from crates.io. Run: %s\n' "$APP_NAME" "$APP_VERSION" "$APP_NAME"
  exit 0
fi

die "Could not install a prebuilt $APP_NAME binary for this platform, and Cargo is unavailable for crates.io fallback."
