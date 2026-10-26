#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
BIN_NAME="ani-es-rs"

if [[ ! -f "$SCRIPT_DIR/target/release/$BIN_NAME" ]]; then
    echo "Error: se requiere el binario release. Ejecuta: cargo build --release"
    exit 1
fi

if [[ -w "$INSTALL_DIR" ]]; then
    cp "$SCRIPT_DIR/target/release/$BIN_NAME" "$INSTALL_DIR/$BIN_NAME"
else
    sudo cp "$SCRIPT_DIR/target/release/$BIN_NAME" "$INSTALL_DIR/$BIN_NAME"
fi

chmod +x "$INSTALL_DIR/$BIN_NAME"
echo "Instalado en $INSTALL_DIR/$BIN_NAME"
