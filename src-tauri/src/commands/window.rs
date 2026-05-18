use std::{thread, time::Duration};
use tauri::{AppHandle, Manager, PhysicalPosition};

use crate::settings;

fn apply_chat_position_reliably(chat: &tauri::WebviewWindow, x: i32, y: i32) {
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

fn companion_anchor_position(app: &AppHandle, chat: &tauri::WebviewWindow) -> (i32, i32) {
    let settings = settings::load_settings(app);
    let companion_x = settings.companion_x.unwrap_or(80);
    let companion_y = settings.companion_y.unwrap_or(80);
    clamp_to_monitor(chat, companion_x + settings.chat_offset_x, companion_y + settings.chat_offset_y)
}

#[tauri::command]
pub fn show_chat_window(app: AppHandle) -> Result<(), String> {
    let chat = app
        .get_webview_window("chat")
        .ok_or_else(|| "chat window not found".to_string())?;

    let (x, y) = companion_anchor_position(&app, &chat);
    apply_chat_position_reliably(&chat, x, y);
    chat.show().map_err(|error| error.to_string())?;
    apply_chat_position_reliably(&chat, x, y);
    chat.set_focus().map_err(|error| error.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn hide_chat_window(app: AppHandle) -> Result<(), String> {
    // Hide only. Do not stop active provider runs; they should continue while
    // chat is hidden and the native companion should notify on completion.
    let chat = app
        .get_webview_window("chat")
        .ok_or_else(|| "chat window not found".to_string())?;

    chat.hide().map_err(|error| error.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn toggle_chat_window(app: AppHandle) -> Result<(), String> {
    let chat = app
        .get_webview_window("chat")
        .ok_or_else(|| "chat window not found".to_string())?;

    let visible = chat.is_visible().map_err(|error| error.to_string())?;

    if visible {
        // Hide only. Background provider runs stay alive.
        chat.hide().map_err(|error| error.to_string())?;
        return Ok(());
    }

    let (x, y) = companion_anchor_position(&app, &chat);
    apply_chat_position_reliably(&chat, x, y);
    chat.show().map_err(|error| error.to_string())?;
    apply_chat_position_reliably(&chat, x, y);
    chat.set_focus().map_err(|error| error.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn anchor_chat_to_companion(app: AppHandle) -> Result<(), String> {
    let chat = app
        .get_webview_window("chat")
        .ok_or_else(|| "chat window not found".to_string())?;

    let settings = settings::load_settings(&app);
    let companion_x = settings.companion_x.unwrap_or(80);
    let companion_y = settings.companion_y.unwrap_or(80);

    let x = companion_x + settings.chat_offset_x;
    let y = companion_y + settings.chat_offset_y;

    chat.set_position(PhysicalPosition::new(x.max(0), y.max(0)))
        .map_err(|error| error.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn save_companion_position(_app: AppHandle) -> Result<(), String> {
    // Native companion saves its position through native_companion_manager IPC.
    Ok(())
}

#[tauri::command]
pub fn restore_companion_position(_app: AppHandle) -> Result<(), String> {
    // Native companion reads its saved position on startup.
    Ok(())
}

#[tauri::command]
pub fn reset_companion_position(app: AppHandle) -> Result<(), String> {
    settings::reset_companion_position(&app)
}

#[tauri::command]
pub fn set_companion_menu_mode(_app: AppHandle, _open: bool) -> Result<(), String> {
    // Old WebView companion menu is removed.
    Ok(())
}

#[tauri::command]
pub fn nudge_companion(_app: AppHandle, dx: i32, dy: i32) -> Result<(), String> {
    // Preserve API compatibility; native nudge can be added later through native manager.
    if dx == 0 && dy == 0 {
        return Ok(());
    }
    Ok(())
}

#[tauri::command]
pub fn set_companion_walk_mode(_app: AppHandle, _open: bool, _direction: String, _distance: i32) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub fn animate_companion_walk(_app: AppHandle, direction: String, distance: i32, duration_ms: u64) -> Result<(), String> {
    crate::native_companion_manager::walk(&direction, distance, duration_ms)
}

#[tauri::command]
pub fn force_companion_repaint(_app: AppHandle) -> Result<(), String> {
    // No WebView repaint needed; native WGPU renderer owns pixels.
    Ok(())
}

#[tauri::command]
pub fn commit_companion_walk(_app: AppHandle, direction: String, distance: i32, duration_ms: u64) -> Result<(), String> {
    crate::native_companion_manager::walk(&direction, distance, duration_ms)
}

#[tauri::command]
pub fn start_native_companion(app: AppHandle) -> Result<(), String> {
    crate::native_companion_manager::start(&app)
}

#[tauri::command]
pub fn stop_native_companion() -> Result<(), String> {
    crate::native_companion_manager::stop()
}

#[tauri::command]
pub fn show_native_companion(app: AppHandle) -> Result<(), String> {
    let _ = crate::native_companion_manager::start(&app);
    crate::native_companion_manager::show()
}

#[tauri::command]
pub fn hide_native_companion() -> Result<(), String> {
    let _ = crate::native_companion_manager::hide();
    crate::native_companion_manager::stop()
}

#[tauri::command]
pub fn set_native_companion_mood(mood: String) -> Result<(), String> {
    crate::native_companion_manager::set_mood(&mood)
}

#[tauri::command]
pub fn set_native_companion_pose(pose: String) -> Result<(), String> {
    crate::native_companion_manager::set_pose(&pose)
}

#[tauri::command]
pub fn set_native_companion_category(app: AppHandle, category: String) -> Result<(), String> {
    crate::native_companion_manager::set_category(&app, &category)
}

#[tauri::command]
pub fn walk_native_companion(direction: String, distance: i32, duration_ms: u64) -> Result<(), String> {
    crate::native_companion_manager::walk(&direction, distance, duration_ms)
}
