# Changelog

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
