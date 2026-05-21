
use once_cell::sync::OnceCell;
use serde::Serialize;
use std::{
    io::{BufRead, BufReader, Write},
    net::TcpListener,
    process::{Child, ChildStdin, Command, Stdio},
    sync::Mutex,
    thread,
    time::Duration,
};
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition};

use crate::{commands, settings, ui_geometry};

struct NativeCompanionHandle {
    child: Child,
    stdin: ChildStdin,
}

static NATIVE_COMPANION: OnceCell<Mutex<Option<NativeCompanionHandle>>> = OnceCell::new();

fn handle_cell() -> &'static Mutex<Option<NativeCompanionHandle>> {
    NATIVE_COMPANION.get_or_init(|| Mutex::new(None))
}

#[derive(Serialize)]
struct MoodCommand<'a> {
    #[serde(rename = "type")]
    kind: &'static str,
    mood: &'a str,
}

#[derive(Serialize)]
struct PoseCommand<'a> {
    #[serde(rename = "type")]
    kind: &'static str,
    pose: &'a str,
}

#[derive(Serialize)]
struct CategoryCommand<'a> {
    #[serde(rename = "type")]
    kind: &'static str,
    category: &'a str,
    pose: &'a str,
}

#[derive(Serialize)]
struct WalkCommand<'a> {
    #[serde(rename = "type")]
    kind: &'static str,
    direction: &'a str,
    distance: i32,
    duration_ms: u64,
}

#[derive(Serialize)]
struct SimpleCommand {
    #[serde(rename = "type")]
    kind: &'static str,
}

fn write_command<T: Serialize>(value: &T) -> Result<(), String> {
    let raw = serde_json::to_string(value).map_err(|error| error.to_string())?;
    let mut guard = handle_cell().lock().map_err(|_| "native companion lock poisoned".to_string())?;
    let Some(handle) = guard.as_mut() else {
        return Err("native companion is not running".to_string());
    };

    writeln!(handle.stdin, "{raw}").map_err(|error| error.to_string())?;
    handle.stdin.flush().map_err(|error| error.to_string())
}

fn apply_chat_position_reliably(chat: &tauri::WebviewWindow, x: i32, y: i32) {
    // Some Linux compositors, especially Wayland/Hyprland, may center a window
    // when it is first shown and ignore the pre-show position. Apply the
    // position before show, immediately after show, and a few delayed times.
    let position = PhysicalPosition::new(x, y);
    let _ = chat.set_position(position);

    for delay_ms in [25_u64, 90, 180, 360] {
        let chat = chat.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(delay_ms));
            let _ = chat.set_position(position);
        });
    }
}

fn clamp_to_monitor(window: &tauri::WebviewWindow, x: i32, y: i32) -> (i32, i32) {
    let monitor = window
        .current_monitor()
        .ok()
        .flatten()
        .or_else(|| window.primary_monitor().ok().flatten());

    let Some(monitor) = monitor else {
        return (x.max(8), y.max(8));
    };

    let pos = monitor.position();
    let size = monitor.size();
    let max_x = pos.x + size.width as i32 - 120;
    let max_y = pos.y + size.height as i32 - 120;

    (x.clamp(pos.x + 8, max_x.max(pos.x + 8)), y.clamp(pos.y + 8, max_y.max(pos.y + 8)))
}

fn chat_position_above_companion(chat: &tauri::WebviewWindow, companion_center_x: i32, companion_top_y: i32) -> (i32, i32) {
    let (x, y) = ui_geometry::chat_position_above_companion(companion_center_x, companion_top_y);
    clamp_to_monitor(chat, x, y)
}

fn companion_launch_position(app: &AppHandle, x: i32, y: i32) -> (i32, i32) {
    let Some(chat) = app.get_webview_window("chat") else {
        return (x.max(8), y.max(8));
    };

    let monitor = chat
        .current_monitor()
        .ok()
        .flatten()
        .or_else(|| chat.primary_monitor().ok().flatten());

    let Some(monitor) = monitor else {
        return (x.max(8), y.max(8));
    };

    let pos = monitor.position();
    let size = monitor.size();
    let max_x = pos.x + size.width as i32 - 140;
    let max_y = pos.y + size.height as i32 - 140;

    (
        x.clamp(pos.x + 8, max_x.max(pos.x + 8)),
        y.clamp(pos.y + 8, max_y.max(pos.y + 8)),
    )
}

fn centered_companion_position(app: &AppHandle) -> (i32, i32) {
    let Some(chat) = app.get_webview_window("chat") else {
        return (80, 80);
    };

    let monitor = chat
        .current_monitor()
        .ok()
        .flatten()
        .or_else(|| chat.primary_monitor().ok().flatten());

    let Some(monitor) = monitor else {
        return (80, 80);
    };

    let pos = monitor.position();
    let size = monitor.size();
    let centered_x = pos.x + ((size.width as i32 - ui_geometry::COMPANION_WIDTH) / 2);
    let centered_y = pos.y + ((size.height as i32 - ui_geometry::COMPANION_HEIGHT) / 2);
    companion_launch_position(app, centered_x, centered_y)
}

fn toggle_chat_above_head(app: &AppHandle, anchor_x: i32, anchor_y: i32) {
    let Some(chat) = app.get_webview_window("chat") else {
        return;
    };

    let visible = chat.is_visible().unwrap_or(false);
    let minimized = chat.is_minimized().unwrap_or(false);
    if visible && !minimized {
        let _ = chat.hide();
        return;
    }

    if minimized {
        let _ = chat.unminimize();
    }

    let (chat_x, chat_y) = chat_position_above_companion(&chat, anchor_x, anchor_y);

    apply_chat_position_reliably(&chat, chat_x, chat_y);
    let _ = chat.show();
    apply_chat_position_reliably(&chat, chat_x, chat_y);
    let _ = chat.set_focus();
}

fn start_ipc_listener(app: AppHandle) -> Result<u16, String> {
    let listener = TcpListener::bind(("127.0.0.1", 0)).map_err(|error| error.to_string())?;
    let port = listener.local_addr().map_err(|error| error.to_string())?.port();

    thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let mut reader = BufReader::new(stream);
            let mut line = String::new();

            if reader.read_line(&mut line).is_err() {
                continue;
            }

            let trimmed = line.trim();

            if let Some(rest) = trimmed.strip_prefix("toggle_chat_at ") {
                let parts: Vec<_> = rest.split_whitespace().collect();
                if parts.len() == 2 {
                    if let (Ok(x), Ok(y)) = (parts[0].parse::<i32>(), parts[1].parse::<i32>()) {
                        toggle_chat_above_head(&app, x, y);
                    }
                }
            } else if trimmed == "toggle_chat" {
                let _ = commands::window::toggle_chat_window(app.clone());
            } else if trimmed == "open_menu" {
                let _ = app.emit("native-companion://menu", ());
            } else if let Some(rest) = trimmed.strip_prefix("position ") {
                let parts: Vec<_> = rest.split_whitespace().collect();
                if parts.len() == 2 {
                    if let (Ok(x), Ok(y)) = (parts[0].parse::<i32>(), parts[1].parse::<i32>()) {
                        let _ = settings::save_companion_position_xy(&app, x, y);
                    }
                }
            }
        }
    });

    Ok(port)
}

fn forward_child_stream<R: std::io::Read + Send + 'static>(app: AppHandle, reader: R, kind: &'static str) {
    thread::spawn(move || {
        for line in BufReader::new(reader).lines().map_while(Result::ok) {
            let trimmed = line.trim();

            if trimmed.is_empty() {
                continue;
            }

            let _ = app.emit(
                "runtime://stream",
                crate::runtime::stream_event(kind, &format!("native companion: {trimmed}")),
            );
        }
    });
}

pub fn start(app: &AppHandle) -> Result<(), String> {
    let mut guard = handle_cell().lock().map_err(|_| "native companion lock poisoned".to_string())?;

    if guard.is_some() {
        return Ok(());
    }

    let settings = settings::load_settings(app);
    let (x, y) = centered_companion_position(app);
    if settings.companion_x != Some(x) || settings.companion_y != Some(y) {
        let _ = settings::save_companion_position_xy(app, x, y);
    }
    let port = start_ipc_listener(app.clone())?;
    let current_exe = std::env::current_exe().map_err(|error| error.to_string())?;

    let mut child = Command::new(current_exe)
        .arg("--native-companion")
        .arg("--x")
        .arg(x.to_string())
        .arg("--y")
        .arg(y.to_string())
        .arg("--ipc-port")
        .arg(port.to_string())
        .arg("--idle-wave-mood")
        .arg(settings.companion_idle_wave_animation.clone())
        .arg("--character-id")
        .arg(settings.companion_character_id.clone())
        .arg("--companion-size")
        .arg(settings.companion_size.clone())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| error.to_string())?;

    if let Some(stdout) = child.stdout.take() {
        forward_child_stream(app.clone(), stdout, "status");
    }

    if let Some(stderr) = child.stderr.take() {
        forward_child_stream(app.clone(), stderr, "stderr");
    }

    let Some(stdin) = child.stdin.take() else {
        return Err("failed to open native companion stdin".to_string());
    };

    *guard = Some(NativeCompanionHandle { child, stdin });
    Ok(())
}

pub fn stop() -> Result<(), String> {
    let mut guard = handle_cell().lock().map_err(|_| "native companion lock poisoned".to_string())?;

    if let Some(mut handle) = guard.take() {
        let _ = writeln!(handle.stdin, r#"{{"type":"quit"}}"#);
        let _ = handle.stdin.flush();
        let _ = handle.child.kill();
    }

    Ok(())
}

pub fn set_mood(mood: &str) -> Result<(), String> {
    write_command(&MoodCommand { kind: "mood", mood })
}

pub fn set_pose(pose: &str) -> Result<(), String> {
    write_command(&PoseCommand { kind: "pose", pose })
}

pub fn walk(direction: &str, distance: i32, duration_ms: u64) -> Result<(), String> {
    write_command(&WalkCommand {
        kind: "walk",
        direction,
        distance,
        duration_ms,
    })
}

pub fn show() -> Result<(), String> {
    write_command(&SimpleCommand { kind: "show" })
}

pub fn hide() -> Result<(), String> {
    let raw = r#"{"type":"hide"}"#;
    let mut guard = handle_cell().lock().map_err(|_| "native companion lock poisoned".to_string())?;
    let Some(handle) = guard.as_mut() else {
        return Err("native companion is not running".to_string());
    };

    writeln!(handle.stdin, "{raw}").map_err(|error| error.to_string())?;
    handle.stdin.flush().map_err(|error| error.to_string())
}


fn normalize_category_pose(pose: String) -> String {
    match pose.as_str() {
        // Walk is now automatic companion behavior, not a user-selectable category pose.
        "walk-left" | "walk-right" => "working".to_string(),
        _ => pose,
    }
}

fn native_pose_for_category(settings: &settings::AppSettings, category: &str) -> String {
    let pose = match category {
        "work" => settings.companion_work_animation.clone(),
        "thinking" => settings.companion_thinking_animation.clone(),
        "command" => settings.companion_command_animation.clone(),
        "done" => settings.companion_done_animation.clone(),
        "error" => settings.companion_error_animation.clone(),
        "idle-wave" => settings.companion_idle_wave_animation.clone(),
        _ => "idle".to_string(),
    };

    normalize_category_pose(pose)
}

pub fn set_category(app: &AppHandle, category: &str) -> Result<(), String> {
    let app_settings = settings::load_settings(app);
    let pose = native_pose_for_category(&app_settings, category);

    match write_command(&CategoryCommand { kind: "category", category, pose: &pose }) {
        Ok(()) => {
            let _ = app.emit(
                "runtime://stream",
                crate::runtime::stream_event(
                    "companion",
                    &format!("native category={} pose={}", category, pose),
                ),
            );
            Ok(())
        }
        Err(error) => {
            let _ = app.emit(
                "runtime://stream",
                crate::runtime::stream_event(
                    "stderr",
                    &format!("native companion category failed: {} -> {} ({})", category, pose, error),
                ),
            );
            Err(error)
        }
    }
}
