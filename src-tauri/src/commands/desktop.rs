#[tauri::command]
pub fn detect_desktop_environment() -> crate::desktop::DesktopEnvironment {
    crate::desktop::detect_desktop_environment()
}
