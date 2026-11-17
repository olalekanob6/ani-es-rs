#!/usr/bin/env bash
set -euo pipefail

REPO="olalekanob6/ani-es-rs"
INSTALL_DIR="${HOME}/.local/bin"
REQUESTED_VERSION="latest"
INSTALL_DEPS=0
FROM_SOURCE=0
INTERACTIVE=0
UNINSTALL=0

usage() {
    cat <<'EOF'
ani-es-rs — Watch anime from your terminal

Usage: bash install.sh [options]

Options:
  --version VERSION    Install a stable release (default: latest)
  --install-dir DIR    Install location (default: ~/.local/bin)
  --install-deps       Explicitly install missing mpv/fzf packages
  --from-source        Build with Cargo instead of downloading a release
  --interactive        Show a terminal menu (requires an interactive terminal)
  --uninstall          Remove only the ani-es-rs executable from install dir
  -h, --help           Show this help

mpv is required for playback; fzf is optional for interactive menus.
Release binaries are SHA-256 checked before installation.
EOF
}

while (($#)); do
    case "$1" in
        --version)
            (($# >= 2)) || { echo "--version requires a value" >&2; exit 2; }
            REQUESTED_VERSION="$2"
            shift 2
            ;;
        --install-dir)
            (($# >= 2)) || { echo "--install-dir requires a directory" >&2; exit 2; }
            INSTALL_DIR="$2"
            shift 2
            ;;
        --install-deps) INSTALL_DEPS=1; shift ;;
        --from-source) FROM_SOURCE=1; shift ;;
        --interactive) INTERACTIVE=1; shift ;;
        --uninstall) UNINSTALL=1; shift ;;
        -h|--help) usage; exit 0 ;;
        *) echo "Unknown option: $1" >&2; usage >&2; exit 2 ;;
    esac
done

cd "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if [[ -t 1 && -z "${NO_COLOR:-}" ]]; then
    CYAN=$'\033[1;36m'; GREEN=$'\033[1;32m'; DIM=$'\033[2m'; RESET=$'\033[0m'
else
    CYAN=""; GREEN=""; DIM=""; RESET=""
fi
printf '%s╭────────────────────────────────────────────╮%s\n' "$CYAN" "$RESET"
printf '%s│ ani-es-rs — Watch anime from your terminal │%s\n' "$CYAN" "$RESET"
printf '%s╰────────────────────────────────────────────╯%s\n\n' "$CYAN" "$RESET"

if ((INTERACTIVE)); then
    if [[ ! -t 0 || ! -t 1 ]]; then
        echo "--interactive necesita una terminal interactiva." >&2
        exit 2
    fi
    printf '  1) Instalar la última versión estable\n  2) Elegir versión\n  3) Desinstalar\n  0) Cancelar\n\nOpción [1]: '
    read -r choice
    case "${choice:-1}" in
        1) ;;
        2)
            printf 'Versión (por ejemplo, v0.1.3): '
            read -r REQUESTED_VERSION
            [[ -n "$REQUESTED_VERSION" ]] || { echo "Versión vacía." >&2; exit 2; }
            ;;
        3) UNINSTALL=1 ;;
        0) echo "Cancelado."; exit 0 ;;
        *) echo "Opción no válida." >&2; exit 2 ;;
    esac
fi

if ((UNINSTALL)); then
    target="$INSTALL_DIR/ani-es-rs"
    if [[ -e "$target" || -L "$target" ]]; then
        rm -f -- "$target"
        printf '%sDesinstalado:%s %s\n' "$GREEN" "$RESET" "$target"
    else
        echo "No encontré el binario en $target"
    fi
    exit 0
fi

OS="$(uname -s)"
ARCH="$(uname -m)"
if ((FROM_SOURCE)); then
    case "$OS" in
        Linux|Darwin) ASSET="" ;;
        *) echo "Sistema no compatible con install.sh: $OS." >&2; exit 1 ;;
    esac
else
    case "$OS:$ARCH" in
        Linux:x86_64|Linux:amd64) ASSET="ani-es-rs-linux-x86_64" ;;
        Darwin:arm64|Darwin:aarch64) ASSET="ani-es-rs-macos-aarch64" ;;
        *)
            echo "No hay binario publicado para $OS/$ARCH. Prueba --from-source o revisa las releases." >&2
            exit 1
            ;;
    esac
fi

version_tag="$REQUESTED_VERSION"
if ((!FROM_SOURCE)); then
    if [[ "$REQUESTED_VERSION" == latest ]]; then
        latest_url="$(curl --fail --location --silent --show-error --retry 3 --connect-timeout 15 --max-time 60 --proto '=https' --proto-redir '=https' --tlsv1.2 --output /dev/null --write-out '%{url_effective}' "https://github.com/${REPO}/releases/latest")"
        version_tag="${latest_url##*/}"
    fi
    if [[ "$version_tag" =~ ^([0-9]+\.[0-9]+\.[0-9]+)$ ]]; then
        version_tag="v$version_tag"
    elif [[ ! "$version_tag" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        echo "Versión estable no válida: $version_tag" >&2
        exit 2
    fi
fi

if ((FROM_SOURCE)) && ! command -v cargo >/dev/null 2>&1; then
    echo "Cargo no está instalado; instala Rust desde https://rustup.rs/ y vuelve a intentarlo." >&2
    exit 1
fi

missing=()
command -v mpv >/dev/null 2>&1 || missing+=(mpv)
command -v fzf >/dev/null 2>&1 || missing+=(fzf)
if ((${#missing[@]})); then
    printf 'Dependencias ausentes: %s\n' "${missing[*]}"
    echo "mpv es necesario para reproducir; fzf solo para menús interactivos."
    if ((INTERACTIVE && !INSTALL_DEPS)); then
        printf '¿Intentar instalarlas con el gestor del sistema? [s/N] '
        read -r answer
        case "$answer" in
            s|S|si|Si|SI|sí|Sí|SÍ|y|Y|yes|Yes|YES) INSTALL_DEPS=1 ;;
        esac
    fi
    if ((INSTALL_DEPS)); then
        case "$OS" in
            Linux)
                if command -v apt-get >/dev/null 2>&1; then sudo apt-get install "${missing[@]}"
                elif command -v dnf >/dev/null 2>&1; then sudo dnf install "${missing[@]}"
                elif command -v yum >/dev/null 2>&1; then sudo yum install "${missing[@]}"
                elif command -v zypper >/dev/null 2>&1; then sudo zypper install "${missing[@]}"
                elif command -v pacman >/dev/null 2>&1; then sudo pacman -S "${missing[@]}"
                else echo "No detecté un gestor compatible; instala las dependencias manualmente." >&2; exit 1
                fi
                ;;
            Darwin)
                command -v brew >/dev/null 2>&1 || { echo "Homebrew no está disponible; instala las dependencias manualmente." >&2; exit 1; }
                brew install "${missing[@]}"
                ;;
        esac
    else
        echo "Sigo sin instalar paquetes del sistema."
    fi
fi

workdir="$(mktemp -d "${TMPDIR:-/tmp}/ani-es-rs-install.XXXXXX")"
temp_target=""
cleanup() {
    if [[ -n "$temp_target" ]]; then rm -f -- "$temp_target"; fi
    rm -rf -- "$workdir"
}
trap cleanup EXIT HUP INT TERM
binary="$workdir/ani-es-rs"

if ((FROM_SOURCE)); then
    command -v cargo >/dev/null 2>&1 || { echo "Cargo no está instalado; usa https://rustup.rs/ o instala el binario precompilado." >&2; exit 1; }
    printf '%sCompilando desde el código fuente...%s\n' "$CYAN" "$RESET"
    cargo build --release --locked
    cp target/release/ani-es-rs "$binary"
    version_tag="fuente"
else
    base_url="https://github.com/${REPO}/releases/download/${version_tag}"
    sums="$workdir/SHA256SUMS"
    printf '%sDescargando ani-es-rs %s para %s/%s...%s\n' "$CYAN" "$version_tag" "$OS" "$ARCH" "$RESET"
    curl --fail --location --silent --show-error --retry 3 --connect-timeout 15 --max-time 300 --max-filesize 134217728 --proto '=https' --proto-redir '=https' --tlsv1.2 --output "$sums" "$base_url/SHA256SUMS"
    curl --fail --location --silent --show-error --retry 3 --connect-timeout 15 --max-time 300 --max-filesize 134217728 --proto '=https' --proto-redir '=https' --tlsv1.2 --output "$binary" "$base_url/$ASSET"
    expected="$(awk -v name="$ASSET" '$2 == name { print $1; count++ } END { if (count != 1) exit 1 }' "$sums")" || { echo "SHA256SUMS no contiene exactamente una suma para $ASSET." >&2; exit 1; }
    [[ "$expected" =~ ^[[:xdigit:]]{64}$ ]] || { echo "Checksum inválido para $ASSET." >&2; exit 1; }
    if command -v sha256sum >/dev/null 2>&1; then
        actual="$(sha256sum "$binary" | awk '{print $1}')"
    elif command -v shasum >/dev/null 2>&1; then
        actual="$(shasum -a 256 "$binary" | awk '{print $1}')"
    else
        echo "No encontré sha256sum ni shasum para verificar el binario." >&2
        exit 1
    fi
    actual_lower="$(printf '%s' "$actual" | tr '[:upper:]' '[:lower:]')"
    expected_lower="$(printf '%s' "$expected" | tr '[:upper:]' '[:lower:]')"
    [[ "$actual_lower" == "$expected_lower" ]] || { echo "La verificación SHA-256 falló; no se instalará el binario." >&2; exit 1; }
fi

mkdir -p "$INSTALL_DIR"
temp_target="$INSTALL_DIR/.ani-es-rs.tmp.$$"
install -m 755 "$binary" "$temp_target"
mv -f -- "$temp_target" "$INSTALL_DIR/ani-es-rs"
temp_target=""
printf '\n%sInstalación completada.%s\n' "$GREEN" "$RESET"
printf 'Versión: %s\nBinario: %s/ani-es-rs\n' "$version_tag" "$INSTALL_DIR"
if [[ ":${PATH}:" != *":${INSTALL_DIR}:"* ]]; then
    printf '%sAñade esta carpeta a PATH si aún no está incluida: %s%s\n' "$DIM" "$INSTALL_DIR" "$RESET"
fi
