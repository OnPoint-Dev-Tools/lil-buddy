use tauri::{
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent},
    App, AppHandle, Manager,
    image::Image as TauriImage,
};

use crate::{native_companion_manager, settings};

const TRAY_ID: &str = "lil-buddy-tray";
const SHOW_LIL_BUDDY_ID: &str = "show-lil-buddy";
const HIDE_LIL_BUDDY_ID: &str = "hide-lil-buddy";
const OPEN_CHAT_ID: &str = "open-chat";
const HIDE_CHAT_ID: &str = "hide-chat";
const RESTORE_POSITION_ID: &str = "restore-position";
const QUIT_ID: &str = "quit";

fn quit_app(app: &AppHandle) {
    hide_chat(app);
    let _ = native_companion_manager::hide();
    let _ = native_companion_manager::stop();
    app.exit(0);
}

pub fn setup_tray(app: &mut App) -> tauri::Result<()> {
    let show_lil_buddy = MenuItem::with_id(
        app,
        SHOW_LIL_BUDDY_ID,
        "Show Lil Buddy",
        true,
        None::<&str>,
    )?;

    let hide_lil_buddy = MenuItem::with_id(
        app,
        HIDE_LIL_BUDDY_ID,
        "Hide Lil Buddy",
        true,
        None::<&str>,
    )?;

    let open_chat = MenuItem::with_id(
        app,
        OPEN_CHAT_ID,
        "Open Chat",
        true,
        None::<&str>,
    )?;

    let hide_chat = MenuItem::with_id(
        app,
        HIDE_CHAT_ID,
        "Hide Chat",
        true,
        None::<&str>,
    )?;

    let restore_position = MenuItem::with_id(
        app,
        RESTORE_POSITION_ID,
        "Restore Position",
        true,
        None::<&str>,
    )?;

    let quit = MenuItem::with_id(
        app,
        QUIT_ID,
        "Quit",
        true,
        None::<&str>,
    )?;

    let separator_1 = PredefinedMenuItem::separator(app)?;
    let separator_2 = PredefinedMenuItem::separator(app)?;

    let menu = Menu::with_items(
        app,
        &[
            &show_lil_buddy,
            &hide_lil_buddy,
            &separator_1,
            &open_chat,
            &hide_chat,
            &restore_position,
            &separator_2,
            &quit,
        ],
    )?;

    let tray_icon = ::image::load_from_memory(include_bytes!("../icons/128x128@2x.png"))
        .map_err(|_| tauri::Error::AssetNotFound("128x128@2x.png".into()))?
        .to_rgba8();
    let (tray_width, tray_height) = tray_icon.dimensions();
    let icon = TauriImage::new_owned(tray_icon.into_raw(), tray_width, tray_height);

    TrayIconBuilder::with_id(TRAY_ID)
        .tooltip("Lil Buddy")
        .icon(icon)
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app: &AppHandle, event: MenuEvent| {
            handle_menu_event(app, event.id().as_ref());
        })
        .on_tray_icon_event(|tray: &TrayIcon, event: TrayIconEvent| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                show_companion(app);
            }
        })
        .build(app)?;

    Ok(())
}

fn handle_menu_event(app: &AppHandle, id: &str) {
    match id {
        SHOW_LIL_BUDDY_ID => show_companion(app),
        HIDE_LIL_BUDDY_ID => hide_companion(app),
        OPEN_CHAT_ID => show_chat(app),
        HIDE_CHAT_ID => hide_chat(app),
        RESTORE_POSITION_ID => {
            restore_positions(app);
        }
        QUIT_ID => quit_app(app),
        _ => {}
    }
}

fn show_companion(app: &AppHandle) {
    // The companion may have been fully stopped by Hide Lil Buddy.
    // Start first, then send show as a no-op/safety command if it is already running.
    let _ = native_companion_manager::start(app);
    let _ = native_companion_manager::show();
}

fn hide_companion(app: &AppHandle) {
    hide_chat(app);

    // Hard-hide the native companion by stopping the native process.
    // This avoids Linux/Wayland compositors ignoring set_visible(false).
    let _ = native_companion_manager::hide();
    let _ = native_companion_manager::stop();
}


fn show_chat(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("chat") {
        window.show().ok();
        window.set_focus().ok();
    }
}

fn hide_chat(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("chat") {
        window.hide().ok();
    }
}

fn restore_positions(app: &AppHandle) {
    let Some(chat) = app.get_webview_window("chat") else {
        return;
    };

    let monitor = chat
        .current_monitor()
        .ok()
        .flatten()
        .or_else(|| chat.primary_monitor().ok().flatten());

    if let Some(monitor) = monitor {
        let pos = monitor.position();
        let size = monitor.size();

        let companion_x = pos.x + size.width as i32 - 230;
        let companion_y = pos.y + size.height as i32 - 270;

        let _ = settings::save_companion_position_xy(app, companion_x, companion_y);
        let _ = native_companion_manager::stop();
        let _ = native_companion_manager::start(app);

        let chat_x = (companion_x - 510).max(pos.x + 12);
        let chat_y = (companion_y - 520).max(pos.y + 12);
        let _ = chat.set_position(tauri::PhysicalPosition::new(chat_x, chat_y));
    }
}
