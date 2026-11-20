use anyhow::Result;
use serde_json::Value;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::ui::select_from_list;

#[cfg(windows)]
use std::fs::{File, OpenOptions};
#[cfg(not(windows))]
use std::os::unix::net::UnixStream;

#[cfg(not(windows))]
type IpcStream = UnixStream;
#[cfg(windows)]
type IpcStream = File;

pub fn play_video(url: &str, start: &str, title: &str) -> Result<String> {
    play_video_inner(url, start, title, false)
}

pub fn play_video_with_track_menu(url: &str, start: &str, title: &str) -> Result<String> {
    play_video_inner(url, start, title, true)
}

fn play_video_inner(url: &str, start: &str, title: &str, show_track_menu: bool) -> Result<String> {
    play_video_with_ipc_policy(url, start, title, show_track_menu, false)
}

fn play_video_with_ipc_policy(
    url: &str,
    start: &str,
    title: &str,
    show_track_menu: bool,
    tolerate_missing_ipc: bool,
) -> Result<String> {
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
    if show_track_menu {
        cmd.arg("--pause=yes");
    }
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
    let mut track_menu_pending = show_track_menu;
    let track_menu_started = Instant::now();
    let mut first_track_list_at: Option<Instant> = None;
    let mut track_request_id: Option<u64> = None;

    loop {
        if ipc_stream.is_none() {
            if let Some(stream) = connect_ipc(&ipc_addr)? {
                prepare_ipc_stream(&stream)?;
                ipc_reader = Some(BufReader::new(stream.try_clone()?));
                ipc_stream = Some(stream);
            }
        }

        if track_menu_pending {
            match track_menu_timeout_action(track_menu_started.elapsed(), ipc_stream.is_some()) {
                TrackMenuTimeoutAction::Wait => {}
                TrackMenuTimeoutAction::Resume => {
                    if let Some(stream) = ipc_stream.as_mut() {
                        set_property_from_stream(
                            stream,
                            "pause",
                            serde_json::Value::Bool(false),
                            request_id,
                        )?;
                        request_id = request_id.wrapping_add(1);
                        track_request_id = None;
                        track_menu_pending = false;
                    }
                }
                TrackMenuTimeoutAction::RestartWithoutMenu => {
                    let _ = child.kill();
                    let _ = child.wait();
                    cleanup_ipc_socket(&ipc_addr);
                    return play_video_with_ipc_policy(url, &start, title, false, true);
                }
            }
            if track_menu_pending {
                if let (Some(stream), Some(reader)) = (ipc_stream.as_mut(), ipc_reader.as_mut()) {
                    if track_request_id.is_none() {
                        let next_id = request_id;
                        request_id = request_id.wrapping_add(1);
                        if send_property_query(stream, "track-list", next_id).is_ok() {
                            track_request_id = Some(next_id);
                        }
                    }
                    if let Some(track_id) = track_request_id {
                        match read_value_response(reader, track_id) {
                            ValueQueryOutcome::Data(value) => {
                                track_request_id = None;
                                let tracks = parse_media_tracks(&value);
                                if value
                                    .as_array()
                                    .is_some_and(|track_list| !track_list.is_empty())
                                {
                                    let first_seen =
                                        *first_track_list_at.get_or_insert_with(Instant::now);
                                    if has_language_choices(&tracks)
                                        || first_seen.elapsed() >= Duration::from_millis(700)
                                    {
                                        if has_language_choices(&tracks) {
                                            select_track_menu(
                                                &tracks,
                                                stream,
                                                reader,
                                                &mut request_id,
                                            )?;
                                        }
                                        set_property_from_stream(
                                            stream,
                                            "pause",
                                            serde_json::Value::Bool(false),
                                            request_id,
                                        )?;
                                        request_id = request_id.wrapping_add(1);
                                        track_menu_pending = false;
                                    }
                                }
                            }
                            ValueQueryOutcome::Failed => {
                                track_request_id = None;
                                set_property_from_stream(
                                    stream,
                                    "pause",
                                    serde_json::Value::Bool(false),
                                    request_id,
                                )?;
                                request_id = request_id.wrapping_add(1);
                                track_menu_pending = false;
                            }
                            ValueQueryOutcome::Pending => {}
                            ValueQueryOutcome::Dead => {
                                track_request_id = None;
                                ipc_stream = None;
                                ipc_reader = None;
                            }
                        }
                    }
                }
            }
        }

        if !track_menu_pending {
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
        if ipc_stream.is_none()
            && should_abort_for_missing_ipc(started.elapsed(), tolerate_missing_ipc)
        {
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

const TRACK_MENU_TIMEOUT: Duration = Duration::from_secs(8);
const MISSING_IPC_TIMEOUT: Duration = Duration::from_secs(30);

fn should_abort_for_missing_ipc(elapsed: Duration, tolerate_missing_ipc: bool) -> bool {
    elapsed > MISSING_IPC_TIMEOUT && !tolerate_missing_ipc
}

#[derive(Debug, PartialEq, Eq)]
enum TrackMenuTimeoutAction {
    Wait,
    Resume,
    RestartWithoutMenu,
}

fn track_menu_timeout_action(elapsed: Duration, ipc_connected: bool) -> TrackMenuTimeoutAction {
    if elapsed < TRACK_MENU_TIMEOUT {
        TrackMenuTimeoutAction::Wait
    } else if ipc_connected {
        TrackMenuTimeoutAction::Resume
    } else {
        TrackMenuTimeoutAction::RestartWithoutMenu
    }
}

#[derive(Clone, Debug)]
struct MediaTrack {
    id: i64,
    track_type: String,
    language: Option<String>,
    title: Option<String>,
    selected: bool,
}

fn parse_media_tracks(value: &Value) -> Vec<MediaTrack> {
    value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|track| {
            let id = track.get("id")?.as_i64()?;
            let track_type = track.get("type")?.as_str()?.to_string();
            if !matches!(track_type.as_str(), "audio" | "sub" | "subtitle") {
                return None;
            }
            Some(MediaTrack {
                id,
                track_type,
                language: track
                    .get("lang")
                    .and_then(Value::as_str)
                    .filter(|value| !value.is_empty())
                    .map(str::to_string),
                title: track
                    .get("title")
                    .and_then(Value::as_str)
                    .filter(|value| !value.is_empty())
                    .map(str::to_string),
                selected: track
                    .get("selected")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
            })
        })
        .collect()
}

fn has_language_choices(tracks: &[MediaTrack]) -> bool {
    let audio_count = tracks
        .iter()
        .filter(|track| track.track_type == "audio")
        .count();
    audio_count > 1
        || tracks
            .iter()
            .any(|track| matches!(track.track_type.as_str(), "sub" | "subtitle"))
}

fn track_label(track: &MediaTrack) -> String {
    let language = track.language.as_deref().unwrap_or("idioma desconocido");
    let title = track.title.as_deref().unwrap_or("");
    let current = if track.selected { " — actual" } else { "" };
    if title.is_empty() || title == language {
        format!("{language} (id {}){current}", track.id)
    } else {
        format!("{language} — {title} (id {}){current}", track.id)
    }
}

fn select_track_menu(
    tracks: &[MediaTrack],
    stream: &mut impl Write,
    reader: &mut impl BufRead,
    request_id: &mut u64,
) -> Result<()> {
    let audio_tracks: Vec<&MediaTrack> = tracks
        .iter()
        .filter(|track| track.track_type == "audio")
        .collect();
    let subtitle_tracks: Vec<&MediaTrack> = tracks
        .iter()
        .filter(|track| matches!(track.track_type.as_str(), "sub" | "subtitle"))
        .collect();

    loop {
        let mut options = Vec::new();
        if audio_tracks.len() > 1 {
            options.push("Cambiar audio".to_string());
        }
        if !subtitle_tracks.is_empty() {
            options.push("Cambiar subtítulos".to_string());
        }
        options.push("Iniciar capítulo".to_string());

        let Some(selection) = select_from_list(&options, "Idioma del capítulo:", "~8")? else {
            return Ok(());
        };
        match selection.as_str() {
            "Cambiar audio" if audio_tracks.len() > 1 => {
                let mut choices = vec![
                    (
                        "Automático (predeterminado)".to_string(),
                        Value::String("auto".into()),
                    ),
                    ("Sin audio".to_string(), Value::String("no".into())),
                ];
                choices.extend(
                    audio_tracks
                        .iter()
                        .map(|track| (track_label(track), serde_json::json!(track.id))),
                );
                choose_track_value("aid", choices, stream, reader, request_id)?;
            }
            "Cambiar subtítulos" if !subtitle_tracks.is_empty() => {
                let mut choices = vec![
                    ("Automático (mpv)".to_string(), Value::String("auto".into())),
                    ("Desactivados".to_string(), Value::String("no".into())),
                ];
                choices.extend(
                    subtitle_tracks
                        .iter()
                        .map(|track| (track_label(track), serde_json::json!(track.id))),
                );
                choose_track_value("sid", choices, stream, reader, request_id)?;
            }
            _ => return Ok(()),
        }
    }
}

fn choose_track_value(
    property: &str,
    choices: Vec<(String, Value)>,
    stream: &mut impl Write,
    _reader: &mut impl BufRead,
    request_id: &mut u64,
) -> Result<()> {
    let labels: Vec<String> = choices.iter().map(|(label, _)| label.clone()).collect();
    if let Some(selected) = select_from_list(&labels, "Selecciona pista:", "~10")? {
        if let Some((_, value)) = choices.iter().find(|(label, _)| label == &selected) {
            set_property_from_stream(stream, property, value.clone(), *request_id)?;
            *request_id = request_id.wrapping_add(1);
        }
    }
    Ok(())
}

fn set_property_from_stream(
    stream: &mut impl Write,
    property: &str,
    value: Value,
    request_id: u64,
) -> std::io::Result<()> {
    let command = serde_json::json!({
        "command": ["set_property", property, value],
        "request_id": request_id,
    });
    writeln!(stream, "{command}")
}

enum ValueQueryOutcome {
    Data(Value),
    Failed,
    Pending,
    Dead,
}

fn send_property_query(
    stream: &mut impl Write,
    property: &str,
    request_id: u64,
) -> std::io::Result<()> {
    let command = serde_json::json!({
        "command": ["get_property", property],
        "request_id": request_id,
    });
    writeln!(stream, "{command}")
}

fn read_value_response(reader: &mut impl BufRead, request_id: u64) -> ValueQueryOutcome {
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line) {
            Ok(0) => return ValueQueryOutcome::Dead,
            Ok(_) => match ipc_value_response(&line, request_id) {
                Some(Ok(value)) => return ValueQueryOutcome::Data(value),
                Some(Err(())) => return ValueQueryOutcome::Failed,
                None => {}
            },
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) =>
            {
                return ValueQueryOutcome::Pending;
            }
            Err(_) => return ValueQueryOutcome::Dead,
        }
    }
}

fn ipc_value_response(response: &str, request_id: u64) -> Option<std::result::Result<Value, ()>> {
    let json = serde_json::from_str::<Value>(response).ok()?;
    if json.get("request_id").and_then(Value::as_u64) != Some(request_id) {
        return None;
    }
    if json.get("error").and_then(Value::as_str) != Some("success") {
        return Some(Err(()));
    }
    Some(Ok(json.get("data").cloned().unwrap_or(Value::Null)))
}

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
    fn missing_ipc_at_track_menu_timeout_restarts_without_pausing_playback() {
        assert_eq!(
            track_menu_timeout_action(Duration::from_secs(7), false),
            TrackMenuTimeoutAction::Wait
        );
        assert_eq!(
            track_menu_timeout_action(Duration::from_secs(8), true),
            TrackMenuTimeoutAction::Resume
        );
        assert_eq!(
            track_menu_timeout_action(Duration::from_secs(8), false),
            TrackMenuTimeoutAction::RestartWithoutMenu
        );
    }

    #[test]
    fn missing_ipc_fallback_allows_playback_after_ipc_timeout() {
        assert!(should_abort_for_missing_ipc(Duration::from_secs(31), false));
        assert!(!should_abort_for_missing_ipc(Duration::from_secs(31), true));
        assert!(!should_abort_for_missing_ipc(
            Duration::from_secs(29),
            false
        ));
    }

    #[test]
    fn track_selection_writes_the_mpv_property_command() {
        let mut output = Vec::new();
        set_property_from_stream(&mut output, "aid", serde_json::json!(2), 9).unwrap();
        let command: Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(
            command["command"],
            serde_json::json!(["set_property", "aid", 2])
        );
        assert_eq!(command["request_id"], 9);
    }

    #[test]
    fn ipc_track_list_response_reads_data_only_for_its_request_id() {
        let reply = r#"{"request_id":7,"error":"success","data":[{"id":1,"type":"audio"}]}"#;
        assert!(ipc_value_response(reply, 7).unwrap().is_ok());
        assert!(ipc_value_response(reply, 8).is_none());
    }

    #[test]
    fn parses_audio_and_subtitle_tracks_for_language_menu() {
        let value = serde_json::json!([
            {"id": 1, "type": "audio", "lang": "jpn", "title": "Japanese", "selected": true},
            {"id": 2, "type": "audio", "lang": "spa", "title": "Español"},
            {"id": 3, "type": "sub", "lang": "spa", "title": "Español"}
        ]);

        let tracks = parse_media_tracks(&value);
        assert_eq!(tracks.len(), 3);
        assert_eq!(tracks[0].language.as_deref(), Some("jpn"));
        assert!(tracks[0].selected);
        assert_eq!(tracks[2].track_type, "sub");
    }

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
