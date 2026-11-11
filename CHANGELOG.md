## [Unreleased]

## [0.1.3] - 2026-11-11

### Added
- Comando `ani-es-rs update` con comprobación, consentimiento explícito y validación SHA-256 de assets de GitHub Releases.
- Workflow de publicación que compila assets para Linux, macOS y Windows y adjunta `SHA256SUMS`.
- El actualizador evita reemplazar binarios administrados por pacman; Windows indica actualización manual.
- El historial interactivo permite continuar un anime desde el episodio y la posición guardados, o borrarlo con confirmación.
- El historial muestra el episodio y la posición guardados de cada anime.
- Subcomando `history` para listar el historial sin fzf y vaciarlo con confirmación (`--clear`).
- `watchlist list` muestra el último episodio visto y ordena por fecha de alta.
- Helpers puros de watchlist con pruebas unitarias y `sorted_by_recent` en el historial.
- Pruebas unitarias de parseo del CLI.
- Marcas de tiempo en el archivo de log.

### Changed
- `select_episode` pide el número manualmente cuando la serie supera 5000 episodios.

### Fixed
- Se eliminó la doble lectura de argumentos en `main`.

## [0.1.2] - 2026-10-27

### Added
- Watchlist con subcomandos CLI: `ani-es-rs watchlist add/remove/list/mark-watching/mark-completed/mark-planned`.
- Menú interactivo de watchlist en el modo fzf.
- Auto-next episode ya estaba presente; ahora también se puede invocar desde watchlist.
- Tests unitarios para parsing de scraper (`search`, `detail`, `jkplayer`, `jkphp`, `mediafire`).
- Nota de compatibilidad Windows en README.

### Changed
- IPC de mpv usa puerto dinámico en lugar de `12345` hardcodeado.
- Limpieza de artefactos de debug (`examples/`).
- Mejora de mensajes en `--continue` cuando falta slug en el historial.

### Fixed
- Redirección de salida de mpv y flags de silencio para reducir ruido en terminal.

## [0.1.1] - 2026-10-27

### Added
- Tests unitarios para parsing de scraper (`search`, `detail`, `jkplayer`, `jkphp`, `mediafire`).
- Nota de compatibilidad Windows en README.

### Changed
- IPC de mpv usa puerto dinámico en lugar de `12345` hardcodeado.
- Limpieza de artefactos de debug (`examples/`).
- Mejora de mensajes en `--continue` cuando falta slug en el historial.

### Fixed
- Redirección de salida de mpv y flags de silencio para reducir ruido en terminal.

## [0.1.0] - 2026-10-26

### Added
- Port inicial de ani-es a Rust con soporte para jkanime.net.
- Menu interactivo con fzf: busqueda, continuar ultimo, historial.
- Modo non-interactive: `--no-fzf`, `--episode`, `--auto-next`.
- Logging a archivo en `~/.local/share/ani-es-rs/logs/ani-es-rs.log`.
- Configuracion por archivo en `~/.config/ani-es-rs/config.toml`.
- Colores ANSI y manejo de TTY para evitar colgar mpv en entornos no interactivos.
- Cache con TTL para respuestas del scraper en `~/.local/share/ani-es-rs/cache/scraper.json`.
- Historial con timestamps en `~/ani-es/history.json`.
- Instalador `install.sh` para binario release.
- PKGBUILD para Arch/CachyOS.
- CI basica en GitHub Actions.

### Changed
- Scraper movido a sistema de providers con trait `AnimeProvider`.
- Implementacion inicial para jkanime.net (`JkanimeProvider`).

### Fixed
- Corregida deteccion de TTY en player para entornos sin terminal interactiva.
- Corregido paso de cache a funciones de scraper.
- Corregido ejemplo `integration.rs` para compilar como binario.
