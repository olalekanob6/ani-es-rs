use anyhow::Result;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const IPC_HOST: &str = "127.0.0.1";

pub fn play_video(url: &str, start: &str, title: &str) -> Result<String> {
    if !is_interactive() {
        return Err(anyhow::Error::msg(
            "No hay terminal interactiva para reproducir video",
        ));
    }

    let _ = std::io::stdout().flush();
    print!("\x1B[2J\x1B[H");
    let _ = std::io::stdout().flush();

    let listener = TcpListener::bind((IPC_HOST, 0)).expect("bind IPC port");
    let ipc_port = listener.local_addr().expect("local addr").port();
    let ipc_addr = format!("{}:{}", IPC_HOST, ipc_port);

    let mut cmd = Command::new("mpv");
    cmd.arg(format!("--input-ipc-server={}", ipc_addr))
        .arg(format!("--start={}", start))
        .arg("--really-quiet")
        .arg("--no-terminal")
        .arg("--msg-level=ffmpeg=no")
        .arg(format!("--force-media-title={}", title))
        .arg(url)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn()?;
    drop(cmd);

    let _start_instant = Instant::now();
    let _timeout = Duration::from_secs(2);

    let _ = listener.set_nonblocking(true);

    let pos = match listener.accept() {
        Ok((stream, _)) => query_position_from_stream(stream).unwrap_or_else(|| start.to_string()),
        Err(_) => start.to_string(),
    };

    let _ = child.wait();

    Ok(pos)
}

fn is_interactive() -> bool {
    let no_fzf = std::env::var("ANI_ES_NO_FZF").ok().as_deref() == Some("1");
    let has_tty = atty::is(atty::Stream::Stdin) && atty::is(atty::Stream::Stdout);
    !no_fzf && has_tty
}

fn query_position_from_stream(mut stream: TcpStream) -> Option<String> {
    let cmd = format!(
        "{}\n",
        r#"{"command":["get_property","time-pos"],"request_id":1}"#
    );
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
