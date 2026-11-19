# Checklist de promoción de ani-es-rs

Usa esta lista solo después de que los cambios estén en la rama por defecto y la release pase CI. Publicar anuncios o paquetes es una acción manual del mantenedor.

## Antes de anunciar

- [ ] Confirma que la página principal de GitHub apunta a la release vigente y que README/`README_INSTALL.md` coinciden con los assets de `.github/workflows/release.yml`.
- [ ] Verifica que la release contiene `ani-es-rs-linux-x86_64`, `ani-es-rs-macos-aarch64`, `ani-es-rs-windows-x86_64.exe` y `SHA256SUMS`.
- [ ] Comprueba que CI pasó en Linux, macOS y Windows; ejecuta las pruebas de instaladores y `cargo test --all-targets`.
- [ ] Prepara una demo breve, real y revisada del CLI. Muestra la terminal/interfaz, no reproduzcas ni redistribuyas vídeo protegido, y elimina datos de usuario, cookies y URLs privadas.
- [ ] Escribe un resumen corto en español e inglés con: qué hace la CLI, requisitos (`mpv`, `fzf` opcional), plataformas precompiladas, enlace a README y limitaciones conocidas.

## Dónde compartir

- [ ] Comparte una vez en comunidades de Rust/CLI y anime en español donde las reglas permitan autopromoción; revisa las reglas vigentes el día de la publicación.
- [ ] Usa la release de GitHub como destino canónico. No publiques repetidamente ni presentes gestores de paquetes como oficiales si no están mantenidos.
- [ ] Responde a preguntas y errores con enlaces a la guía o issues. No pidas telemetría ni logs completos que puedan incluir información privada.

## Medir a los 14 días

- [ ] Revisa GitHub Insights → Traffic (visitas y clones) y las descargas de cada asset de la release.
- [ ] Compara con el periodo anterior a la publicación y anota qué enlace/comunidad produjo preguntas o descargas útiles, si la información está disponible.
- [ ] Prioriza mejoras basadas en problemas repetidos; no añadas analytics dentro de la aplicación.

## Decisiones del mantenedor

- AUR: actualizar/publicar `PKGBUILD` solo si alguien acepta mantenerlo y valida cada versión.
- crates.io: verificar disponibilidad del nombre y preparar `cargo publish --dry-run`; publicar solo con aprobación explícita.
- GitHub Discussions: actualmente no está habilitado; activarlo solo si hay disponibilidad para responder.
- Redes/comunidades: la checklist no publica nada automáticamente; aprobación y publicación son manuales.
