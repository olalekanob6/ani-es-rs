# AGENTS.md

## Proyecto
`ani-es-rs` es una CLI Rust (edition 2021) para buscar y reproducir anime con mpv; fzf se usa en los menús interactivos. Proveedor actual: JKAnime. Mantén los cambios pequeños y evita introducir dependencias sin necesidad.

## Estructura
- `src/main.rs`: Clap CLI y orquestación de búsqueda, continuación, reproducción, historial y watchlist.
- `src/lib.rs`: módulos públicos y reexports.
- `src/models.rs`: modelos de anime, historial y watchlist.
- `src/providers/jkanime.rs`: trait `AnimeProvider`, HTTP, parsing y extracción de fuentes.
- `src/providers/mod.rs`: export del proveedor.
- `src/scraper.rs`: fachada del provider y caché de respuestas.
- `src/player.rs`: lanzamiento de mpv, IPC y lectura de progreso (Unix socket / named pipe Windows).
- `src/ui.rs`: prompts y selección fzf.
- `src/history.rs`: historial JSON y migración/importación legacy SQLite.
- `src/watchlist.rs`: lista de seguimiento JSON.
- `src/cache.rs`: caché scraper con TTL.
- `src/storage.rs`: rutas de datos y escritura atómica compartida.
- `src/config.rs`, `src/logger.rs`, `src/colors.rs`: configuración, logs y color.
- `config.example.toml`: ejemplo de configuración; `README*.md`, `CHANGELOG.md`, `PKGBUILD`, `install.*`: documentación y distribución.
- `.github/workflows/ci.yml`: comandos de calidad usados en CI.

## Reglas para trabajar
- Antes de editar, inspecciona solo los módulos afectados y sus llamadores/tests; no vuelques todo el repositorio al contexto.
- Sigue el flujo existente y reutiliza `storage::atomic_write` para persistencia. No cambies formatos/rutas persistidas sin compatibilidad con datos legacy.
- Para parsing y networking, prioriza pruebas con fixtures o servidor local; no dependas de que JKAnime esté disponible en tests.
- No imprimas ni registres URL firmadas, cookies, tokens CSRF ni datos sensibles.
- Respeta compatibilidad Linux, macOS y Windows; usa `#[cfg(...)]` donde las APIs difieran. No asumas que tests Linux validan los otros sistemas.
- No hagas refactors amplios ni cambios de comportamiento no pedidos. Actualiza README/changelog solo cuando el cambio lo requiera.
- No ejecutes instaladores ni comandos que instalen paquetes o modifiquen el sistema salvo que el usuario lo pida expresamente.
- No crees commits salvo petición explícita.

## Validación
Desde la raíz del repositorio:
```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo check --all-targets
cargo test --all-targets
```
La CI también compila release con `cargo build --release --locked`. Ejecuta las validaciones pertinentes al cambio y reporta resultados reales; si una falla, conserva el error concreto y no la presentes como pasada.

## Rutas del proyecto
El árbol de trabajo más reciente identificado es `/home/olale/Proyectos/ani-es-rs`. Existe otra copia distinta en `/home/olale/ani-es-rs`; confirma la raíz activa antes de actuar si el directorio de trabajo no coincide. No uses `target/` como fuente ni inspecciones `.git/` salvo que la tarea lo requiera.
