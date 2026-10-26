# ani-es-rs

Port en Rust de [ani-es](https://github.com/Zhuchii/ani-es) para usar [jkanime.net](https://jkanime.net) desde la terminal. Menu interactivo con fzf, logging, colores y reproduccion en mpv.

## Creditos

Este proyecto es un fork/port en Rust de [ani-es](https://github.com/Zhuchii/ani-es) creado por [Zhuchii](https://github.com/Zhuchii). Todo el merito del scraper y la logica original es de el.

## Requisitos

- Rust 1.70+
- mpv
- fzf

## Instalar

```bash
git clone https://github.com/olalekanob6/ani-es-rs.git && cd ani-es-rs && sudo ./install.sh
```

O compilar manual:

```bash
cargo build --release
sudo ./install.sh
```

## Uso

```bash
ani-es-rs                  # menu interactivo
ani-es-rs "naruto"         # busqueda directa
ani-es-rs -c               # continuar ultimo
ani-es-rs --no-fzf --episode 1 bleach
ani-es-rs --auto-next naruto
```

## Configuracion

`~/.config/ani-es-rs/config.toml`

```toml
[app]
log_level = "info"
color = true
timeout_seconds = 15
```

Logs en `~/.local/share/ani-es-rs/logs/ani-es-rs.log`.

## Licencia

MIT
