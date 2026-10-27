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

---

## ⚡ Instalación

<details><summary>Windows (WSL)</summary>

Abre Powershell y pega esto

```bash
wsl --install
```

Reiniciar el ordenador y luego en el Powershell ejecutar esto:

```bash
wsl --install -d Ubuntu
```

Luego desde la barra de tareas busca ubuntu y ejecuta

Ya por ultimo solo queda instalar el script

```bash
cd ~/
git clone https://github.com/olalekanob6/ani-es-rs.git
cd ani-es-rs
chmod +x install.sh
./install.sh
```

Cerramos el ubuntu

### ⚠️ Nota sobre mpv
Para poder usar mpv has de instalar mpv de descargarte mpv desde su [pagina oficial](https://mpv.io/installation/) y añadirlo al PATH de windows

Si no sabes como hacerlo hecha un vistazo a [esta guia](https://github.com/Zhuchii/ani-es/wiki/C%C3%B3mo-descargar-y-a%C3%B1adir-mpv-al-PATH-en-Windows)

Ya podras ejecutarlo desde el CMD de Windows usando:

```bash
wsl ani-es-rs
```

</details>

<details><summary>Windows (nativo)</summary>

```powershell
git clone https://github.com/olalekanob6/ani-es-rs.git
cd ani-es-rs
.\install.ps1
```

El script instalará Rust (si no está presente), mpv y fzf usando `winget` o descarga manual.

**Nota:** En Windows, asegúrate de que `mpv` y `fzf` estén en el `PATH` del sistema.

</details>

<details><summary>Linux</summary>

```bash
git clone https://github.com/olalekanob6/ani-es-rs.git
cd ani-es-rs
chmod +x install.sh
./install.sh
```

Después podrás usar:

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
* Windows (nativo y WSL)

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

* `rustc` / `cargo` (1.70+)
* `mpv`
* `fzf`
* `curl`
* `jq`

El instalador se encarga de todo automáticamente.

---

## 💾 Historial

El script guarda automáticamente:

* Último anime visto
* Último episodio

Para que puedas continuar fácilmente después.

---

## ❌ Desinstalar

```bash
./uninstall.sh
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

### 0.1.0
- Port inicial desde ani-es (bash) a Rust
- Búsqueda de anime en jkanime.net
- Reproducción con mpv
- Historial de episodios vistos
- Soporte para Linux, macOS y Windows
