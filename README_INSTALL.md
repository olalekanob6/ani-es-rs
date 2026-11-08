# Instalación

ani-es-rs es compatible con **Linux**, **macOS** y **Windows**.

## Requisitos previos

- **Rust estable** ([rustup.rs](https://rustup.rs/))
- **mpv** ([mpv.io](https://mpv.io/))
- **fzf** ([junegunn/fzf](https://github.com/junegunn/fzf); solo para los menús interactivos)

## Instalación automática

### Linux / WSL

```bash
git clone https://github.com/olalekanob6/ani-es-rs.git
cd ani-es-rs
chmod +x install.sh
./install.sh
```

El script detectará tu gestor de paquetes (`apt`, `dnf`, `yum`, `zypper`, `pacman`) e instalará `mpv` y `fzf`. Rust debe estar instalado antes de ejecutar el script.

### macOS

```bash
git clone https://github.com/olalekanob6/ani-es-rs.git
cd ani-es-rs
chmod +x install.sh
./install.sh
```

El script usará **Homebrew** para instalar `mpv` y `fzf`. Rust debe estar instalado antes de compilar.

### Windows

```powershell
git clone https://github.com/olalekanob6/ani-es-rs.git
cd ani-es-rs
.\install.ps1
```

El script instalará Rust y fzf si hace falta y `winget` está disponible. Instala mpv manualmente.

**Nota:** En Windows, asegúrate de que `mpv` y `fzf` estén en el `PATH` del sistema.

## Instalación manual

Si prefieres instalar las dependencias manualmente:

### Linux

```bash
# Debian/Ubuntu
sudo apt install mpv fzf

# Fedora/RHEL
sudo dnf install mpv fzf

# Arch
sudo pacman -S mpv fzf
```

### macOS

```bash
brew install mpv fzf
```

### Windows

1. **mpv**: descarga desde [mpv.io/installation](https://mpv.io/installation/) y añade la ruta al `PATH`.
2. **fzf** (opcional para `--no-fzf`): descarga desde [junegunn/fzf](https://github.com/junegunn/fzf) y añade la ruta al `PATH`.

## Compilar

```bash
cargo build --release
```

El binario estará en `target/release/ani-es-rs` (o `ani-es-rs.exe` en Windows).

## Uso

```bash
# Buscar anime
ani-es-rs bleach

# Continuar último anime visto
ani-es-rs --continue

# Ayuda
ani-es-rs --help
```

En Windows:
```powershell
.\target\release\ani-es-rs.exe bleach
```
