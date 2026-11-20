# ani-es-rs — Watch anime from your terminal

> A Rust CLI to search and watch Spanish-language anime from your terminal with mpv.
>
> Mira anime en español desde tu terminal con una CLI escrita en Rust.

Inspired by [ani-es](https://github.com/Zhuchii/ani-es), focused on Spanish-language anime. ani-es-rs is a command-line interface for supported streaming sites; it does not host video content.

**Search, stream and continue watching anime — all from your terminal.**

---

## ✨ Features

* 🔍 Búsqueda de animes desde la terminal
* 📺 Reproducción directa con **mpv**
* ⏭️ Navegación entre episodios (siguiente / anterior)
* 💾 Guarda automáticamente el último episodio visto
* 📜 Historial: continúa cualquier anime desde su episodio y posición, o bórralo; `ani-es-rs history` lo lista sin fzf y `ani-es-rs history --clear` lo vacía con confirmación
* ⚡ Interfaz interactiva con **fzf**
* 🧠 Fallback automático para episodios en emisión
* 📦 Compatible con Linux, macOS y Windows
* 🎯 Modo no interactivo: `--no-fzf`, `--episode`, `--auto-next`
* 📋 Watchlist: `ani-es-rs watchlist list` muestra el estado y el último episodio visto, ordenado por fecha de alta
* 🔄 Auto-next episode: continúa automáticamente con el siguiente episodio
* 🔌 Proveedores seleccionables desde el menú: `jkanime` (predeterminado) y `animeav1` (`--provider animeav1`)
* 🌐 Menú por capítulo para elegir pistas de audio y subtítulos que mpv detecte

---

## ⚡ Instalación

Descarga rápida del instalador para Linux/macOS (revísalo antes de ejecutarlo):

```bash
curl -fsSLO https://raw.githubusercontent.com/olalekanob6/ani-es-rs/master/install.sh
less install.sh
bash install.sh --interactive
```

<details><summary>Windows</summary>

```powershell
git clone https://github.com/olalekanob6/ani-es-rs.git
cd ani-es-rs
.\install.ps1
```

El instalador descarga el binario publicado desde GitHub Releases y comprueba SHA-256 (no requiere Rust para el binario precompilado), no instala programas ni modifica el `PATH` automáticamente. Instala mpv para reproducir y fzf si quieres los menús interactivos. El binario queda en `%LOCALAPPDATA%\Programs\ani-es-rs`; puedes indicarle otra carpeta con `-InstallDir`.

El binario publicado para Windows es x86_64 y no requiere Cargo. El instalador por defecto no modifica `PATH`; usa `-Interactive` para elegir instalar, desinstalar o cancelar. mpv es necesario para reproducir y fzf es opcional.

</details>

<details><summary>Linux</summary>

```bash
git clone https://github.com/olalekanob6/ani-es-rs.git
cd ani-es-rs
chmod +x install.sh
./install.sh
```

El instalador descarga el binario de GitHub Releases, comprueba SHA-256 y lo coloca en `~/.local/bin/ani-es-rs`; no requiere Rust para el binario de Linux x86_64. Usa `--install-dir` para elegir otra ruta. No instala paquetes del sistema automáticamente. Si quieres que intente instalar las dependencias detectadas, usa `./install.sh --install-deps`; mpv es necesario para reproducir y fzf solo para menús interactivos. Si la carpeta de instalación no está en tu `PATH`, añádela según la configuración de tu shell.

```bash
ani-es-rs
```

</details>

<details><summary>macOS</summary>

```bash
git clone https://github.com/olalekanob6/ani-es-rs.git
cd ani-es-rs
chmod +x install.sh
./install.sh
```

La release ofrece actualmente un binario precompilado para macOS Apple Silicon (arm64); el instalador verifica su SHA-256 y no requiere Rust para esa arquitectura. Para Intel u otra arquitectura, clona el repo y compila con `bash install.sh --from-source` (requiere Rust/Cargo). No instala dependencias automáticamente.

</details>

---

## 🖥️ Uso

```bash
ani-es-rs
```

O directamente:

```bash
ani-es-rs naruto
```

Selecciona AnimeAV1 explícitamente; JKAnime sigue siendo el proveedor predeterminado:

```bash
ani-es-rs --provider animeav1 bleach
ani-es-rs --provider animeav1 --no-fzf --episode 3 bleach
```

En el menú principal puedes elegir **Cambiar proveedor**; el cambio se mantiene durante esa sesión. Al reproducir un capítulo en modo interactivo, si mpv detecta varias pistas de audio o al menos una pista de subtítulos, el reproductor queda pausado mientras eliges. Si no detecta pistas o usas `--no-fzf`, conserva la selección predeterminada de mpv.

Si guardas historial o watchlist con AnimeAV1, usa `--provider animeav1` para reanudar o administrar esas entradas. Los registros antiguos sin proveedor se interpretan como JKAnime; `--continue` elige el registro más reciente del proveedor activo. La persistencia usa el título como clave y no admite dos entradas con el mismo título exacto en proveedores distintos: el CLI rechazará ese conflicto en vez de sobrescribirlo o eliminarlo silenciosamente. Los enlaces de AnimeAV1 pueden ser páginas de reproductores externos; no todos se pueden abrir directamente con mpv y la reproducción aún depende del servidor/embed.

```bash
ani-es-rs history
ani-es-rs history --clear
ani-es-rs watchlist list
```

### Actualizaciones

`ani-es-rs update --check` consulta la release publicada más reciente sin instalar
ni modificar datos locales. `ani-es-rs update` pide confirmación antes de instalar;
`ani-es-rs update --yes` acepta explícitamente la instalación para automatizaciones.
El binario descargado se verifica con `SHA256SUMS` antes de reemplazar el ejecutable.
La actualización automática está disponible para Linux x86_64 y macOS arm64;
Windows y otras arquitecturas requieren descarga y reemplazo manual desde GitHub Releases.
Si pacman administra el ejecutable, actualízalo mediante pacman o tu helper AUR.
La release debe incluir los assets de plataforma y `SHA256SUMS`.

---

## 📦 Compatibilidad

Funciona en:

* Debian / Ubuntu
* Fedora / RHEL / CentOS
* Arch Linux
* macOS
* Windows

Gestores soportados:

* `apt-get`
* `dnf`
* `yum`
* `zypper`
* `pacman`
* `brew` (macOS)
* `winget` (Windows)

---

## 🔧 Dependencias

* Rust estable (`rustc` y `cargo`) para compilar desde el código fuente
* `mpv` para reproducir vídeo
* `fzf` (opcional; solo para menús interactivos)

Los instaladores detectan dependencias, pero no instalan paquetes del sistema automáticamente.

---

## 💾 Historial

El script guarda automáticamente:

* Último anime visto
* Último episodio

Para que puedas continuar fácilmente después.

---

## ❌ Desinstalar

Los instaladores ofrecen una opción de desinstalación que elimina únicamente el ejecutable, no los datos ni el historial.

```bash
rm -f "$HOME/.local/bin/ani-es-rs"
```

En Windows, elimina `ani-es-rs.exe` de `%LOCALAPPDATA%\Programs\ani-es-rs` (o de la carpeta indicada con `-InstallDir`).

---

## ⚠️ Nota

Este proyecto es solo una interfaz CLI para interactuar con sitios de streaming.
No aloja contenido propio.

---

## ❤️ Contribuir

Las contribuciones son bienvenidas:

* Reportar bugs
* Sugerir features
* Hacer pull requests

---

## ⭐ Apoya el proyecto

Si te gusta el proyecto, dale una estrella ⭐

Esto ayuda mucho a que más gente lo descubra 🙌

---

## 📝 Changelog

### 0.1.3
- Subcomando `history` para listar y vaciar el historial sin fzf.
- `watchlist list` con el último episodio visto y orden por fecha de alta.
- Comando `update` con verificación SHA-256 de assets y workflow de releases.
- Limpiezas internas: fzf compartido, parseo centralizado y marcas de tiempo en el log.

### 0.1.2
- Watchlist con subcomandos CLI y menú interactivo.
- Auto-next episode.
- IPC dinámico de mpv y menor ruido en terminal.
- Tests unitarios de scraper.
- Nota de compatibilidad Windows en README.

### 0.1.0
- Port inicial desde ani-es (bash) a Rust
- Búsqueda de anime en jkanime.net
- Reproducción con mpv
- Historial de episodios vistos
- Soporte para Linux, macOS y Windows
