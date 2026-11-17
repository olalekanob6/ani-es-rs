# Installation / Instalación

`ani-es-rs` is a Rust CLI to search and watch Spanish-language anime from your terminal. It is a frontend for supported streaming sites and does not host video content.

`ani-es-rs` es una CLI en Rust para buscar y ver anime en español desde la terminal. Es una interfaz para sitios compatibles; no aloja contenido.

## Requirements / Requisitos

- `mpv` to play video / para reproducir vídeo.
- `fzf` is optional and only used for interactive menus / es opcional y solo se usa en menús interactivos.
- Rust stable (`rustc` and `cargo`) only when building from source / solo al compilar desde fuentes.

The release installer does not install system packages by default. / El instalador de releases no instala paquetes del sistema por defecto.

## Linux and macOS

Download the script, inspect it, then run its menu:

```bash
curl -fsSLO https://raw.githubusercontent.com/olalekanob6/ani-es-rs/master/install.sh
less install.sh
bash install.sh --interactive
```

SHA-256 detects corruption or mismatched assets, but the checksum is downloaded from the same GitHub Release and is not an independent signature or proof of publisher identity. Current prebuilt targets are Linux x86_64 and macOS Apple Silicon (arm64). Rust is not needed for those targets. The default destination is `~/.local/bin`; use `--install-dir DIR` to choose another directory. The installer does not modify shell startup files or `PATH`.

To explicitly request installation of missing `mpv`/`fzf` through a detected system package manager, pass `--install-deps`. This may require `sudo`; inspect the script and command before opting in:

```bash
bash install.sh --install-deps
```

For other architectures, clone the repository and build from source:

```bash
git clone https://github.com/olalekanob6/ani-es-rs.git
cd ani-es-rs
bash install.sh --from-source
```

## Windows

Clone the repository to obtain the PowerShell installer, then run PowerShell 5.1 or newer:

```powershell
git clone https://github.com/olalekanob6/ani-es-rs.git
cd ani-es-rs
.\install.ps1 -Interactive
```

By default, the installer downloads the Windows x86_64 release asset and verifies its SHA-256. Rust is not needed for the prebuilt binary. The destination defaults to `%LOCALAPPDATA%\Programs\ani-es-rs`; choose another with `-InstallDir`. The script does not install system packages or edit `PATH` automatically.

To build from source instead, use `-FromSource` and install Rust/Cargo first. Install mpv for playback; fzf is optional for interactive menus.

## Explicit versions and non-interactive use

```bash
bash install.sh --version v0.1.3 --install-dir "$HOME/.local/bin"
```

PowerShell equivalent:

```powershell
.\install.ps1 -Version v0.1.3 -InstallDir "$env:LOCALAPPDATA\Programs\ani-es-rs"
```

Without `--interactive` / `-Interactive`, installers do not prompt. Unsupported OS/architecture combinations fail rather than silently selecting a different binary.

## Manual dependency installation / Instalación manual de dependencias

Package names and availability depend on your distribution. Examples:

```bash
# Debian / Ubuntu
sudo apt install mpv fzf

# Fedora
sudo dnf install mpv fzf

# Arch Linux
sudo pacman -S mpv fzf

# macOS with Homebrew
brew install mpv fzf
```

For Windows, see the official [mpv installation page](https://mpv.io/installation/) and [fzf repository](https://github.com/junegunn/fzf).

## Build manually / Compilar manualmente

```bash
cargo build --release --locked
```

The binary is `target/release/ani-es-rs` (`ani-es-rs.exe` on Windows).

## Usage / Uso

```bash
ani-es-rs bleach
ani-es-rs --continue
ani-es-rs history
ani-es-rs watchlist list
ani-es-rs --help
```

## Uninstall / Desinstalación

The installer removes only the executable; it does not delete user data or viewing history.

- Linux/macOS: `bash install.sh --uninstall` (or remove the binary under `~/.local/bin`).
- Windows: run `pwsh -File install.ps1 -Uninstall` from the repository (or remove `ani-es-rs.exe` from `%LOCALAPPDATA%\Programs\ani-es-rs`).

User data and viewing history are stored separately and are not removed by uninstalling the executable.
