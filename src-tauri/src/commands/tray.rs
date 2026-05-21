use tauri::{AppHandle, Manager};

use crate::{native_companion_manager, settings};

#[tauri::command]
pub fn tray_show_lil_buddy(app: AppHandle) -> Result<(), String> {
    let _ = native_companion_manager::start(&app);
    let _ = native_companion_manager::show();

    Ok(())
}

#[tauri::command]
pub fn tray_hide_lil_buddy(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("chat") {
        window.hide().map_err(|error| error.to_string())?;
    }

    let _ = native_companion_manager::hide();
    let _ = native_companion_manager::stop();
    Ok(())
}

#[tauri::command]
pub fn tray_open_chat(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("chat") {
        window.show().map_err(|error| error.to_string())?;
        window.set_focus().ok();
    }

    Ok(())
}

#[tauri::command]
pub fn tray_quit(app: AppHandle) {
    if let Some(window) = app.get_webview_window("chat") {
        let _ = window.hide();
    }

    let _ = native_companion_manager::hide();
    let _ = native_companion_manager::stop();
    app.exit(0);
}

#[tauri::command]
pub fn restore_monitor_aware_positions(app: AppHandle) -> Result<(), String> {
    let chat = app
        .get_webview_window("chat")
        .ok_or_else(|| "chat window not found".to_string())?;

    let monitor = chat
        .current_monitor()
        .map_err(|error| error.to_string())?
        .or_else(|| chat.primary_monitor().ok().flatten());

    if let Some(monitor) = monitor {
        let size = monitor.size();
        let pos = monitor.position();

        let companion_x = pos.x + size.width as i32 - 230;
        let companion_y = pos.y + size.height as i32 - 270;
        settings::save_companion_position_xy(&app, companion_x, companion_y)?;

        let _ = native_companion_manager::stop();
        let _ = native_companion_manager::start(&app);

        let chat_x = (companion_x - 510).max(pos.x + 12);
        let chat_y = (companion_y - 520).max(pos.y + 12);
        chat.set_position(tauri::PhysicalPosition::new(chat_x, chat_y)).ok();
    }

    Ok(())
}
