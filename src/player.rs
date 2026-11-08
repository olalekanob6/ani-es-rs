use anyhow::Result;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

#[cfg(windows)]
use std::fs::{File, OpenOptions};
#[cfg(not(windows))]
use std::os::unix::net::UnixStream;

#[cfg(not(windows))]
type IpcStream = UnixStream;
#[cfg(windows)]
type IpcStream = File;

pub fn play_video(url: &str, start: &str, title: &str) -> Result<String> {
    if atty::is(atty::Stream::Stdout) {
        let _ = std::io::stdout().flush();
        print!("\x1B[2J\x1B[H");
        let _ = std::io::stdout().flush();
    }

    let ipc_addr = ipc_address()?;

    let start = if is_valid_start(start) {
        start.to_string()
    } else {
        "00:00:00".to_string()
    };

    let mut cmd = Command::new("mpv");
    cmd.arg(format!("--input-ipc-server={}", ipc_addr))
        .arg(format!("--start={}", start))
        .arg("--really-quiet")
        .arg("--no-terminal")
        .arg("--idle=no")
        .arg("--keep-open=no")
        .arg("--msg-level=ffmpeg=no")
        .arg(format!("--force-media-title={}", title))
        .arg(url)
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    let mut child = ChildGuard::new(cmd.spawn()?);

    let started = Instant::now();
    let mut ipc_stream: Option<IpcStream> = None;
    let mut ipc_reader: Option<BufReader<IpcStream>> = None;
    let mut position = start.clone();
    let mut got_position = false;
    let mut duration_secs: Option<f64> = None;
    let mut request_id = 1u64;

    loop {
        if ipc_stream.is_none() {
            if let Some(stream) = connect_ipc(&ipc_addr)? {
                prepare_ipc_stream(&stream)?;
                ipc_reader = Some(BufReader::new(stream.try_clone()?));
                ipc_stream = Some(stream);
            }
        }

        if let (Some(stream), Some(reader)) = (ipc_stream.as_mut(), ipc_reader.as_mut()) {
            match query_property_from_stream(stream, reader, "time-pos", request_id) {
                QueryOutcome::Value(value) => {
                    position = value;
                    got_position = true;
                }
                QueryOutcome::Pending => {}
                QueryOutcome::Dead => {
                    ipc_stream = None;
                    ipc_reader = None;
                }
            }
            request_id = request_id.wrapping_add(1);
            if let (Some(stream), Some(reader)) = (ipc_stream.as_mut(), ipc_reader.as_mut()) {
                if let QueryOutcome::Value(value) =
                    query_property_from_stream(stream, reader, "duration", request_id)
                {
                    if let Ok(secs) = value.parse::<f64>() {
                        duration_secs = Some(secs);
                    }
                }
                request_id = request_id.wrapping_add(1);
            }
        }

        if let Some(status) = child.try_wait()? {
            if !status.success() {
                // Si ya se obtuvo posición, se conserva (el usuario vio
                // parte del episodio); si no, el error se propaga.
                if !got_position {
                    return Err(anyhow::anyhow!("mpv terminó con estado {status}"));
                }
                break;
            }
            break;
        }
        if ipc_stream.is_none() && started.elapsed() > Duration::from_secs(30) {
            let _ = child.kill();
            let _ = child.wait();
            cleanup_ipc_socket(&ipc_addr);
            return Err(anyhow::anyhow!("mpv no abrió su canal IPC en 30 segundos"));
        }
        thread::sleep(Duration::from_millis(250));
    }

    // Episodio terminado: no reanudar al final la próxima vez.
    if let Some(duration) = duration_secs {
        if duration > 0.0 && position_to_secs(&position) >= duration - 2.0 {
            position = "00:00:00".to_string();
        }
    }

    cleanup_ipc_socket(&ipc_addr);
    Ok(position)
}

/// Mata al hijo mpv si sale de `play_video` por cualquier camino.
struct ChildGuard(std::process::Child);

impl ChildGuard {
    fn new(child: std::process::Child) -> Self {
        Self(child)
    }
}

impl std::ops::Deref for ChildGuard {
    type Target = std::process::Child;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for ChildGuard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        if self.0.try_wait().ok().flatten().is_none() {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
}

fn is_valid_start(value: &str) -> bool {
    let parts: Vec<&str> = value.split(':').collect();
    parts.len() == 3
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()))
}

fn position_to_secs(value: &str) -> f64 {
    let parts: Vec<f64> = value
        .split(':')
        .filter_map(|part| part.parse::<f64>().ok())
        .collect();
    match parts.as_slice() {
        [h, m, s] => h * 3600.0 + m * 60.0 + s,
        [m, s] => m * 60.0 + s,
        [s] => *s,
        _ => 0.0,
    }
}

#[cfg(not(windows))]
fn cleanup_ipc_socket(ipc_addr: &str) {
    let _ = std::fs::remove_file(ipc_addr);
}

#[cfg(windows)]
fn cleanup_ipc_socket(_ipc_addr: &str) {}

enum QueryOutcome {
    Value(String),
    Pending,
    Dead,
}

fn query_property_from_stream(
    stream: &mut impl Write,
    reader: &mut impl BufRead,
    property: &str,
    request_id: u64,
) -> QueryOutcome {
    let command = serde_json::json!({
        "command": ["get_property", property],
        "request_id": request_id,
    });
    if writeln!(stream, "{command}").is_err() {
        return QueryOutcome::Dead;
    }

    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => return QueryOutcome::Dead,
            Ok(_) => {
                if let Some(secs) = numeric_response(&line, request_id) {
                    let value = if property == "time-pos" {
                        format_position(secs)
                    } else {
                        secs.to_string()
                    };
                    return QueryOutcome::Value(value);
                }
            }
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) =>
            {
                return QueryOutcome::Pending;
            }
            Err(_) => return QueryOutcome::Dead,
        }
    }
}

fn numeric_response(response: &str, request_id: u64) -> Option<f64> {
    let json = serde_json::from_str::<serde_json::Value>(response).ok()?;
    if json.get("request_id").and_then(|value| value.as_u64()) != Some(request_id)
        || json.get("error").and_then(|value| value.as_str()) != Some("success")
    {
        return None;
    }
    json.get("data")?.as_f64()
}

/// Dirección IPC para mpv. mpv es el servidor: nosotros solo
/// pasamos la dirección y conectamos como clientes.
/// En Linux/macOS mpv solo acepta socket Unix (un "host:puerto"
/// se interpreta como ruta de socket literal y nunca conecta por TCP).
/// En Windows usa un named pipe.
#[cfg(not(windows))]
fn ipc_address() -> Result<String> {
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT_SOCKET_ID: AtomicU64 = AtomicU64::new(0);

    let path: PathBuf = std::env::temp_dir().join(format!(
        "ani-es-rs-{}-{}.sock",
        std::process::id(),
        NEXT_SOCKET_ID.fetch_add(1, Ordering::Relaxed)
    ));
    if path.exists() {
        std::fs::remove_file(&path)?;
    }
    Ok(path.to_string_lossy().into_owned())
}

#[cfg(windows)]
fn ipc_address() -> Result<String> {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT_PIPE_ID: AtomicU64 = AtomicU64::new(0);

    Ok(format!(
        r"\\.\pipe\ani-es-rs-{}-{}",
        std::process::id(),
        NEXT_PIPE_ID.fetch_add(1, Ordering::Relaxed)
    ))
}

#[cfg(not(windows))]
#[cfg(not(windows))]
fn connect_ipc(ipc_addr: &str) -> Result<Option<IpcStream>> {
    match UnixStream::connect(ipc_addr) {
        Ok(stream) => Ok(Some(stream)),
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::NotFound
                    | std::io::ErrorKind::ConnectionRefused
                    | std::io::ErrorKind::WouldBlock
            ) =>
        {
            Ok(None)
        }
        Err(error) => Err(error.into()),
    }
}

#[cfg(windows)]
fn connect_ipc(ipc_addr: &str) -> Result<Option<IpcStream>> {
    match OpenOptions::new().read(true).write(true).open(ipc_addr) {
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
            numeric_response(response, 1).map(format_position),
            Some("01:01:01".to_string())
        );
        assert_eq!(
            numeric_response(r#"{"request_id":1,"error":"success","data":null}"#, 1),
            None
        );
        assert_eq!(numeric_response(response, 2), None);
    }

    #[test]
    fn start_validation_rejects_garbage_progress() {
        assert!(is_valid_start("00:00:00"));
        assert!(is_valid_start("01:23:45"));
        assert!(!is_valid_start("garbage"));
        assert!(!is_valid_start("12:34"));
        assert!(!is_valid_start(""));
    }

    #[cfg(not(windows))]
    #[test]
    fn unix_ipc_address_is_a_socket_path() {
        let address = ipc_address().unwrap();
        assert!(address.ends_with(".sock"), "{address}");
        // Sin mpv en marcha, conectar devuelve None (reintentable), no error.
        assert!(connect_ipc(&address).unwrap().is_none());
        cleanup_ipc_socket(&address);
    }

    #[cfg(windows)]
    #[test]
    fn windows_ipc_address_uses_a_named_pipe() {
        let address = ipc_address().unwrap();
        assert!(address.contains("pipe"));
        assert!(address.contains("ani-es-rs-"));
    }
}
