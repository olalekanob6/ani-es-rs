# Instalación

ani-es-rs es compatible con **Linux**, **macOS** y **Windows**.

## Requisitos previos

- **Rust** 1.70 o superior ([rustup.rs](https://rustup.rs/))
- **mpv** ([mpv.io](https://mpv.io/))
- **fzf** ([junegunn/fzf](https://github.com/junegunn/fzf))
- **curl** y **jq** (generalmente incluidos en Linux/macOS; en Windows se pueden instalar manualmente)

## Instalación automática

### Linux / WSL

```bash
git clone https://github.com/tu-usuario/ani-es-rs.git
cd ani-es-rs
chmod +x install.sh
./install.sh
```

El script detectará tu gestor de paquetes (`apt`, `dnf`, `yum`, `zypper`, `pacman`) y instalará las dependencias automáticamente.

### macOS

```bash
git clone https://github.com/tu-usuario/ani-es-rs.git
cd ani-es-rs
chmod +x install.sh
./install.sh
```

El script usará **Homebrew** para instalar las dependencias.

### Windows

```powershell
git clone https://github.com/tu-usuario/ani-es-rs.git
cd ani-es-rs
.\install.ps1
```

El script instalará:
- Rust (si no está presente)
- mpv (vía winget o descarga manual)
- fzf (vía winget o descarga manual)

**Nota:** En Windows, asegúrate de que `mpv` y `fzf` estén en el `PATH` del sistema.

## Instalación manual

Si prefieres instalar las dependencias manualmente:

### Linux

```bash
# Debian/Ubuntu
sudo -S -p '' apt install mpv fzf curl jq

# Fedora/RHEL
sudo -S -p '' dnf install mpv fzf curl jq

# Arch
sudo -S -p '' pacman -S mpv fzf curl jq
```

### macOS

```bash
brew install mpv fzf curl jq
```

### Windows

1. **mpv**: descarga desde [mpv.io/installation](https://mpv.io/installation/) y añade la ruta al `PATH`.
2. **fzf**: descarga desde [junegunn/fzf](https://github.com/junegunn/fzf) y añade la ruta al `PATH`.

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
