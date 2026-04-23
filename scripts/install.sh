#!/usr/bin/env bash

set -euo pipefail

REPO_OWNER="${REPO_OWNER:-jeverett32}"
REPO_NAME="${REPO_NAME:-frugal}"
BIN_NAME="${BIN_NAME:-fgl}"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
VERSION="${VERSION:-latest}"

need_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "missing required command: $1" >&2
    exit 1
  }
}

detect_target() {
  local os arch
  os="$(uname -s)"
  arch="$(uname -m)"

  case "$os" in
    Linux) os="unknown-linux-gnu" ;;
    Darwin) os="apple-darwin" ;;
    *)
      echo "unsupported operating system: $os" >&2
      exit 1
      ;;
  esac

  case "$arch" in
    x86_64|amd64) arch="x86_64" ;;
    arm64|aarch64) arch="aarch64" ;;
    *)
      echo "unsupported architecture: $arch" >&2
      exit 1
      ;;
  esac

  printf '%s-%s\n' "$arch" "$os"
}

resolve_version() {
  if [ "$VERSION" != "latest" ]; then
    printf '%s\n' "$VERSION"
    return
  fi

  curl -fsSLI -o /dev/null -w '%{url_effective}' \
    "https://github.com/${REPO_OWNER}/${REPO_NAME}/releases/latest" \
    | sed 's#.*/tag/##'
}

main() {
  need_cmd curl
  need_cmd tar

  local target version archive_name url tmpdir
  target="$(detect_target)"
  version="$(resolve_version)"
  archive_name="frugal-${version}-${target}.tar.gz"
  url="https://github.com/${REPO_OWNER}/${REPO_NAME}/releases/download/${version}/${archive_name}"
  tmpdir="$(mktemp -d)"

  trap "rm -rf '$tmpdir'" EXIT

  echo "Installing ${BIN_NAME} ${version} for ${target}..."
  curl -fsSL "$url" -o "$tmpdir/$archive_name"
  tar -xzf "$tmpdir/$archive_name" -C "$tmpdir"

  mkdir -p "$INSTALL_DIR"
  cp "$tmpdir/frugal-${version}-${target}/${BIN_NAME}" "$INSTALL_DIR/${BIN_NAME}"
  chmod +x "$INSTALL_DIR/${BIN_NAME}"

  echo "Installed to $INSTALL_DIR/${BIN_NAME}"

  case ":$PATH:" in
    *":$INSTALL_DIR:"*) ;;
    *)
      echo "warning: $INSTALL_DIR is not in PATH" >&2
      echo "add this to your shell profile:" >&2
      echo "  export PATH=\"$INSTALL_DIR:\$PATH\"" >&2
      ;;
  esac
}

main "$@"
