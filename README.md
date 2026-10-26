ani-es-rs
CLI para ver anime en español desde la terminal. Port en Rust de ani-es para jkanime.net.

## Requisitos
- Rust 1.70+
- mpv
- fzf (interfaz interactiva)
- CachyOS u otra distro Linux moderna

## Compilar
cargo build --release

## Instalar
sudo ./install.sh

## Uso
ani-es-rs "naruto"
ani-es-rs -c
ani-es-rs --no-fzf --episode 1 bleach
ani-es-rs --auto-next naruto

## Flujo
1. Busca anime.
2. Selecciona con fzf.
3. Elige episodio.
4. Reproduce en mpv.
5. Guarda progreso en ~/ani-es/history.json.

## Configuracion
Archivo: `~/.config/ani-es-rs/config.toml`

```toml
[app]
log_level = "info"
color = true
timeout_seconds = 15
```

Logs: `~/.local/share/ani-es-rs/logs/ani-es-rs.log`

## Notas
- Modo non-interactive: --no-fzf y/o --episode
- Auto next: --auto-next reproduce todos los episodios seguidos
- Historial con timestamp en ~/ani-es/history.json
