# ani-es-rs

> Port en Rust de [ani-es](https://github.com/Zhuchii/ani-es) para ver anime en español desde la terminal.

CLI tool powered by **mpv + fzf**

⚡ Fast | 🎯 No ads | 🖥️ Lightweight | ❤️ Open Source

⚡ Rápido | 🎯 Sin anuncios | 🖥️ Ligero | ❤️ Open Source

> Inspired by [ani-es](https://github.com/Zhuchii/ani-es), focused on Spanish content. This is a Rust port of the original bash script.

---

**Search, stream and continue watching anime — all from your terminal.**

---

## ✨ Features

* 🔍 Búsqueda de animes desde la terminal
* 📺 Reproducción directa con **mpv**
* ⏭️ Navegación entre episodios (siguiente / anterior)
* 💾 Guarda automáticamente el último episodio visto
* ⚡ Interfaz interactiva con **fzf**
* 🧠 Fallback automático para episodios en emisión
* 📦 Compatible con Linux, macOS y Windows
* 🎯 Modo no interactivo: `--no-fzf`, `--episode`, `--auto-next`
* 📋 Watchlist: agrega, elimina y marca animes como planeado / viendo / completado
* 🔄 Auto-next episode: continúa automáticamente con el siguiente episodio

---

## ⚡ Instalación

<details><summary>Windows</summary>

```powershell
git clone https://github.com/olalekanob6/ani-es-rs.git
cd ani-es-rs
.\install.ps1
```

El script instala Rust y fzf cuando `winget` está disponible. mpv puede requerir instalación manual.

**Nota:** En Windows, asegúrate de que `mpv` y `fzf` estén en el `PATH` del sistema.

**Probado en:** Windows 10/11 con PowerShell 5.1+ y `mpv`/`fzf` en `PATH`.

</details>

<details><summary>Linux</summary>

```bash
git clone https://github.com/olalekanob6/ani-es-rs.git
cd ani-es-rs
chmod +x install.sh
./install.sh
```

Linux y macOS requieren Rust instalado antes de ejecutar el instalador. El binario compilado queda en `target/release/ani-es-rs`:

```bash
./target/release/ani-es-rs
```

</details>

<details><summary>macOS</summary>

```bash
git clone https://github.com/olalekanob6/ani-es-rs.git
cd ani-es-rs
chmod +x install.sh
./install.sh
```

El script usará **Homebrew** para instalar las dependencias.

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
* `mpv`
* `fzf` (solo para los menús interactivos; `--no-fzf` evita esa dependencia)

El instalador comprueba e instala `mpv` y `fzf` en Linux/macOS. La compilación requiere Rust.

---

## 💾 Historial

El script guarda automáticamente:

* Último anime visto
* Último episodio

Para que puedas continuar fácilmente después.

---

## ❌ Desinstalar

```bash
rm -f /usr/local/bin/ani-es-rs
```

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
