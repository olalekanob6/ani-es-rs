# ani-es-rs

CLI para ver anime en español desde la terminal. Port en Rust de [ani-es](https://github.com/Zhuchii/ani-es) para [jkanime.net](https://jkanime.net), con menu interactivo, logging, colores y reproduccion en mpv.

## Creditos

Este proyecto es un fork/port en Rust de [ani-es](https://github.com/Zhuchii/ani-es) creado por [Zhuchii](https://github.com/Zhuchii). Agradezco el trabajo original que hizo posible esta herramienta.

## Requisitos

- Rust 1.70+
- mpv
- fzf (interfaz interactiva)
- CachyOS u otra distro Linux moderna

## Compilar

```bash
cargo build --release
```

## Instalar

### Instalacion automatica

```bash
git clone https://github.com/olalekanob6/ani-es-rs.git && cd ani-es-rs && sudo ./install.sh
```

Eso clona el repo, compila en release y copia el binario a `/usr/local/bin/ani-es-rs`.

### Instalacion manual

```bash
sudo ./install.sh
```

O con symlink personal sin sudo:

```bash
mkdir -p ~/.local/bin
ln -sf /home/olal3/proyectos/ani-es-rs/target/release/ani-es-rs ~/.local/bin/ani-es-rs
```

## Uso

```bash
ani-es-rs
ani-es-rs "naruto"
ani-es-rs -c
ani-es-rs --no-fzf --episode 1 bleach
ani-es-rs --auto-next naruto
```

## Menu interactivo

Al ejecutar `ani-es-rs` sin argumentos se abre un menu con fzf:

- Buscar anime
- Continuar ultimo
- Historial
- Salir

## Configuracion

Archivo: `~/.config/ani-es-rs/config.toml`

```toml
[app]
log_level = "info"
color = true
timeout_seconds = 15
```

Logs: `~/.local/share/ani-es-rs/logs/ani-es-rs.log`

## Flujo

1. Busca anime en jkanime.net
2. Selecciona con fzf
3. Elige episodio
4. Reproduce en mpv con `--input-ipc-server`
5. Guarda progreso en `~/ani-es/history.json`

## Notas

- Modo non-interactive: `--no-fzf` y/o `--episode`
- Auto next: `--auto-next` reproduce todos los episodios seguidos
- Historial con timestamp en `~/ani-es/history.json`
- Manejo de errores robusto con reintentos automaticos
- Logging a archivo para diagnostico

## Licencia

MIT
