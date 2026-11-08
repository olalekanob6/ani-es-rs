use anyhow::Result;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

#[cfg(windows)]
use std::fs::{File, OpenOptions};
#[cfg(not(windows))]
use std::net::{TcpListener, TcpStream};

const IPC_HOST: &str = "127.0.0.1";

#[cfg(not(windows))]
type IpcListener = TcpListener;
#[cfg(not(windows))]
type IpcStream = TcpStream;
#[cfg(windows)]
type IpcListener = String;
#[cfg(windows)]
type IpcStream = File;

pub fn play_video(url: &str, start: &str, title: &str) -> Result<String> {
    if atty::is(atty::Stream::Stdout) {
        let _ = std::io::stdout().flush();
        print!("\x1B[2J\x1B[H");
        let _ = std::io::stdout().flush();
    }

    let (ipc_addr, listener) = bind_ipc()?;

    let mut cmd = Command::new("mpv");
    cmd.arg(format!("--input-ipc-server={}", ipc_addr))
        .arg(format!("--start={}", start))
        .arg("--really-quiet")
        .arg("--no-terminal")
        .arg("--msg-level=ffmpeg=no")
        .arg(format!("--force-media-title={}", title))
        .arg(url)
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    let mut child = cmd.spawn()?;
    drop(cmd);

    let started = Instant::now();
    let mut ipc_stream: Option<IpcStream> = None;
    let mut ipc_reader: Option<BufReader<IpcStream>> = None;
    let mut position = start.to_string();
    let mut request_id = 1u64;

    loop {
        if ipc_stream.is_none() {
            if let Some(stream) = accept_ipc(&listener)? {
                prepare_ipc_stream(&stream)?;
                ipc_reader = Some(BufReader::new(stream.try_clone()?));
                ipc_stream = Some(stream);
            }
        }

        if let (Some(stream), Some(reader)) = (ipc_stream.as_mut(), ipc_reader.as_mut()) {
            if let Some(value) = query_position_from_stream(stream, reader, request_id) {
                position = value;
            }
            request_id = request_id.wrapping_add(1);
        }

        if let Some(status) = child.try_wait()? {
            if !status.success() {
                return Err(anyhow::anyhow!("mpv terminó con estado {status}"));
            }
            break;
        }
        if ipc_stream.is_none() && started.elapsed() > Duration::from_secs(30) {
            let _ = child.kill();
            let _ = child.wait();
            return Err(anyhow::anyhow!("mpv no abrió su canal IPC en 30 segundos"));
        }
        thread::sleep(Duration::from_millis(250));
    }

    Ok(position)
}

fn query_position_from_stream(
    stream: &mut impl Write,
    reader: &mut impl BufRead,
    request_id: u64,
) -> Option<String> {
    let command = serde_json::json!({
        "command": ["get_property", "time-pos"],
        "request_id": request_id,
    });
    writeln!(stream, "{command}").ok()?;

    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => return None,
            Ok(_) => {
                if let Some(position) = position_from_response(&line, request_id) {
                    return Some(position);
                }
            }
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) =>
            {
                return None;
            }
            Err(_) => return None,
        }
    }
}

#[cfg(not(windows))]
fn bind_ipc() -> Result<(String, IpcListener)> {
    let listener = TcpListener::bind((IPC_HOST, 0))?;
    listener.set_nonblocking(true)?;
    let address = format!("{}:{}", IPC_HOST, listener.local_addr()?.port());
    Ok((address, listener))
}

#[cfg(windows)]
fn bind_ipc() -> Result<(String, IpcListener)> {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT_PIPE_ID: AtomicU64 = AtomicU64::new(0);

    let pipe = format!(
        r"\\.\pipe\ani-es-rs-{}-{}",
        std::process::id(),
        NEXT_PIPE_ID.fetch_add(1, Ordering::Relaxed)
    );
    Ok((pipe.clone(), pipe))
}

#[cfg(not(windows))]
fn accept_ipc(listener: &IpcListener) -> Result<Option<IpcStream>> {
    match listener.accept() {
        Ok((stream, _)) => Ok(Some(stream)),
        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => Ok(None),
        Err(error) => Err(error.into()),
    }
}

#[cfg(windows)]
fn accept_ipc(pipe: &IpcListener) -> Result<Option<IpcStream>> {
    match OpenOptions::new().read(true).write(true).open(pipe) {
        Ok(stream) => Ok(Some(stream)),
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::NotFound
                    | std::io::ErrorKind::WouldBlock
                    | std::io::ErrorKind::ConnectionRefused
            ) =>
        {
            Ok(None)
        }
        Err(error) => Err(error.into()),
    }
}

#[cfg(not(windows))]
fn prepare_ipc_stream(stream: &IpcStream) -> std::io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_millis(250)))?;
    stream.set_write_timeout(Some(Duration::from_secs(1)))
}

#[cfg(windows)]
fn prepare_ipc_stream(_: &IpcStream) -> std::io::Result<()> {
    Ok(())
}

fn position_from_response(response: &str, request_id: u64) -> Option<String> {
    let json = serde_json::from_str::<serde_json::Value>(response).ok()?;
    if json.get("request_id").and_then(|value| value.as_u64()) != Some(request_id)
        || json.get("error").and_then(|value| value.as_str()) != Some("success")
    {
        return None;
    }
    json.get("data")?.as_f64().map(format_position)
}

fn format_position(secs: f64) -> String {
    let total = secs.max(0.0) as u64;
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    format!("{:02}:{:02}:{:02}", h, m, s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ipc_response_returns_numeric_position_only_on_success() {
        let response = r#"{"request_id":1,"error":"success","data":3661.9}"#;
        assert_eq!(
            position_from_response(response, 1),
            Some("01:01:01".to_string())
        );
        assert_eq!(
            position_from_response(r#"{"request_id":1,"error":"success","data":null}"#, 1),
            None
        );
        assert_eq!(position_from_response(response, 2), None);
    }

    #[cfg(windows)]
    #[test]
    fn windows_ipc_address_uses_a_named_pipe() {
        let (address, pipe) = bind_ipc().unwrap();
        assert!(address.starts_with(r"\\.\pipe\ani-es-rs-"));
        assert_eq!(address, pipe);
    }
}
