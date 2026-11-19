# ani-es-rs — anime en español desde la terminal

CLI en Rust para buscar y ver anime en español desde la terminal con mpv. / A Rust CLI to search and watch Spanish-language anime from your terminal with mpv.

Inspirado en [ani-es](https://github.com/Zhuchii/ani-es). Es una interfaz para sitios compatibles; no aloja contenido de vídeo. / Inspired by [ani-es](https://github.com/Zhuchii/ani-es); it is a frontend for supported streaming sites and does not host video content.

[![CI](https://github.com/olalekanob6/ani-es-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/olalekanob6/ani-es-rs/actions/workflows/ci.yml) [![Latest release](https://img.shields.io/github/v/release/olalekanob6/ani-es-rs)](https://github.com/olalekanob6/ani-es-rs/releases/latest)

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

---

## ⚡ Instalación

Descarga rápida del instalador para Linux/macOS (revísalo antes de ejecutarlo):

```bash
curl -fsSLO https://raw.githubusercontent.com/olalekanob6/ani-es-rs/master/install.sh
less install.sh
bash install.sh --interactive
```

### Windows (x86_64)

```powershell
git clone https://github.com/olalekanob6/ani-es-rs.git
cd ani-es-rs
.\install.ps1
```

El instalador descarga el binario publicado desde GitHub Releases y comprueba SHA-256 (no requiere Rust para el binario precompilado), no instala programas ni modifica el `PATH` automáticamente. Instala mpv para reproducir y fzf si quieres los menús interactivos. El binario queda en `%LOCALAPPDATA%\Programs\ani-es-rs`; puedes indicarle otra carpeta con `-InstallDir`.

El binario publicado para Windows es x86_64 y no requiere Cargo. El instalador por defecto no modifica `PATH`; usa `-Interactive` para elegir instalar, desinstalar o cancelar. mpv es necesario para reproducir y fzf es opcional.

### Linux (x86_64)

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

### macOS (Apple Silicon / arm64)

```bash
git clone https://github.com/olalekanob6/ani-es-rs.git
cd ani-es-rs
chmod +x install.sh
./install.sh
```

La release ofrece actualmente un binario precompilado para macOS Apple Silicon (arm64); el instalador verifica su SHA-256 y no requiere Rust para esa arquitectura. Para Intel u otra arquitectura, clona el repo y compila con `bash install.sh --from-source` (requiere Rust/Cargo). No instala dependencias automáticamente.

Targets precompilados adicionales y notas de seguridad están en [Releases](https://github.com/olalekanob6/ani-es-rs/releases) y la [guía detallada](README_INSTALL.md). Otras arquitecturas requieren compilar desde el código fuente.

---

## 🖥️ Uso

```bash
ani-es-rs
```

O directamente:

```bash
ani-es-rs naruto
```

```bash
ani-es-rs history
ani-es-rs history --clear
ani-es-rs watchlist list
ani-es-rs update --check
```

| Comando | Acción |
| --- | --- |
| `ani-es-rs <búsqueda>` | Busca anime y abre el selector interactivo. |
| `ani-es-rs --continue` | Reanuda el último anime/episodio guardado. |
| `ani-es-rs <búsqueda> --episode 3 --no-fzf` | Reproduce un episodio sin el selector fzf. |
| `ani-es-rs <búsqueda> --auto-next` | Avanza automáticamente entre episodios. |
| `ani-es-rs history` / `ani-es-rs history --clear` | Muestra o limpia el historial (limpiar pide confirmación). |
| `ani-es-rs watchlist list` | Muestra la lista de seguimiento. |
| `ani-es-rs update --check` | Comprueba si hay una actualización, sin instalarla. |
| `ani-es-rs --help` | Muestra todas las opciones y subcomandos. |

`--no-fzf` selecciona el primer resultado de búsqueda; si no se indica `--episode`, empieza por el episodio 1. No permite escoger entre resultados.

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

| Sistema | Binario publicado |
| --- | --- |
| Linux | x86_64 (GNU) |
| macOS | Apple Silicon / aarch64 |
| Windows | x86_64 |

Otras arquitecturas pueden compilarse desde el código fuente con Rust/Cargo. Las dependencias de reproducción son:

* `mpv` para reproducir vídeo (requerido)
* `fzf` para menús interactivos (opcional; `--no-fzf` evita esa dependencia)
* Rust estable (`rustc` y `cargo`) solo para compilar desde el código fuente

---

## 🔧 Instalar dependencias

Los instaladores de release no instalan paquetes del sistema automáticamente. Consulta [Instalación](README_INSTALL.md) para instalarlos manualmente.

---

## 💾 Historial

El historial y la watchlist se guardan en el directorio de datos del usuario, dentro de `ani-es-rs/`; los logs usan `ani-es-rs/logs/`. La ubicación concreta depende del sistema operativo. La configuración opcional está en el directorio de configuración del usuario, en `ani-es-rs/config.toml`; usa [`config.example.toml`](config.example.toml) como referencia.

El historial guarda automáticamente:

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

Las contribuciones y reportes reproducibles son bienvenidos. [Abre un issue](https://github.com/olalekanob6/ani-es-rs/issues/new/choose) para reportar un error o sugerir una mejora; elimina datos personales y URLs privadas antes de adjuntar logs.

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
