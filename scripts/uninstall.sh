#!/usr/bin/env bash

set -euo pipefail

BIN_NAME="${BIN_NAME:-fgl}"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

BIN_PATH="$INSTALL_DIR/$BIN_NAME"

if [ ! -f "$BIN_PATH" ]; then
  echo "$BIN_PATH not found — nothing to uninstall" >&2
  exit 0
fi

rm "$BIN_PATH"
echo "Removed $BIN_PATH"
