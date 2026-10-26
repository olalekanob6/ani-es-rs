use anyhow::Result;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

const IPC_SOCKET: &str = "/tmp/ani-es-mpvsocket";

pub fn play_video(url: &str, start: &str, title: &str) -> Result<String> {
    if !is_interactive() {
        return Err(anyhow::Error::msg("No hay terminal interactiva para reproducir video"));
    }

    let _ = std::fs::remove_file(IPC_SOCKET);
    let socket_path = Path::new(IPC_SOCKET);

    let _ = std::io::stdout().flush();
    print!("\x1B[2J\x1B[H");
    let _ = std::io::stdout().flush();

    let mut cmd = Command::new("mpv");
    cmd.arg(format!("--input-ipc-server={}", IPC_SOCKET))
        .arg(format!("--start={}", start))
        .arg(format!("--force-media-title={}", title))
        .arg(url)
        .stdout(std::fs::File::create("/dev/null")?)
        .stderr(std::fs::File::create("/dev/null")?);

    let mut child = cmd.spawn()?;
    drop(cmd);

    let start_instant = Instant::now();
    let timeout = Duration::from_secs(2);

    let pos = loop {
        if start_instant.elapsed() > timeout {
            break None;
        }
        if socket_path.exists() {
            break query_position();
        }
        std::thread::sleep(Duration::from_millis(100));
    };

    let pos = match pos {
        Some(p) => p,
        None => start.to_string(),
    };

    let _ = child.wait();

    Ok(pos)
}

fn is_interactive() -> bool {
    let no_fzf = std::env::var("ANI_ES_NO_FZF").ok().as_deref() == Some("1");
    let has_tty = atty::is(atty::Stream::Stdin) && atty::is(atty::Stream::Stdout);
    !no_fzf && has_tty
}

fn query_position() -> Option<String> {
    let mut stream = UnixStream::connect(IPC_SOCKET).ok()?;
    let cmd = format!("{}\n", r#"{"command":["get_property","time-pos"],"request_id":1}"#);
    stream.write_all(cmd.as_bytes()).ok()?;

    let reader = BufReader::new(stream);
    for line in reader.lines().flatten() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&trimmed) {
            if json.get("error").and_then(|v| v.as_str()) == Some("success") {
                if let Some(data) = json.get("data") {
                    if let Some(secs) = data.as_f64() {
                        return Some(format_position(secs));
                    }
                }
            }
        }
    }

    None
}

fn format_position(secs: f64) -> String {
    let total = secs.max(0.0) as u64;
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    format!("{:02}:{:02}:{:02}", h, m, s)
}
