use std::{path::PathBuf, process::Command};

use tauri::AppHandle;

use crate::{native_companion_manager, settings::{load_settings as load_app_settings, save_settings}};

fn normalize_workspace_path(path: String) -> Option<String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return None;
    }

    let raw = PathBuf::from(trimmed);
    let canonical = raw.canonicalize().unwrap_or(raw);

    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--show-toplevel")
        .current_dir(&canonical)
        .output()
        .ok();

    if let Some(output) = output {
        if output.status.success() {
            let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !root.is_empty() {
                return Some(root);
            }
        }
    }

    Some(canonical.to_string_lossy().to_string())
}

#[tauri::command]
pub fn load_settings(app: AppHandle) -> crate::settings::AppSettings {
    load_app_settings(&app)
}

#[tauri::command]
pub fn save_selected_provider(app: AppHandle, provider_id: String) -> Result<(), String> {
    let mut settings = load_app_settings(&app);
    let value = provider_id.trim();
    settings.selected_provider = match value {
        "claude" | "opencode-go" => value.to_string(),
        _ => "opencode-go".to_string(),
    };
    save_settings(&app, &settings)
}

#[tauri::command]
pub fn save_opencode_go_mode(app: AppHandle, mode: String) -> Result<(), String> {
    let mut settings = load_app_settings(&app);
    let value = mode.trim();
    settings.opencode_go_mode = match value {
        "run-json" | "run-formatted" | "run-stdin" | "raw-arg" => value.to_string(),
        _ => "run-json".to_string(),
    };
    save_settings(&app, &settings)
}

#[tauri::command]
pub fn save_opencode_command(app: AppHandle, command: String) -> Result<(), String> {
    let mut settings = load_app_settings(&app);
    settings.opencode_command = command;
    save_settings(&app, &settings)
}

#[tauri::command]
pub fn save_opencode_provider_key(app: AppHandle, provider_key: String) -> Result<(), String> {
    let mut settings = load_app_settings(&app);
    settings.opencode_provider_key = provider_key;
    save_settings(&app, &settings)
}

#[tauri::command]
pub fn save_claude_command(app: AppHandle, command: String) -> Result<(), String> {
    let mut settings = load_app_settings(&app);
    let trimmed = command.trim();
    settings.claude_command = if trimmed.is_empty() {
        "claude".to_string()
    } else {
        trimmed.to_string()
    };
    save_settings(&app, &settings)
}

#[tauri::command]
pub fn save_claude_output_format(app: AppHandle, output_format: String) -> Result<(), String> {
    let mut settings = load_app_settings(&app);
    let value = output_format.trim();
    settings.claude_output_format = match value {
        "stream-json" | "json" | "text" => value.to_string(),
        _ => "stream-json".to_string(),
    };
    save_settings(&app, &settings)
}


#[tauri::command]
pub fn save_default_directory(app: AppHandle, path: String) -> Result<(), String> {
    let mut settings = load_app_settings(&app);
    settings.default_directory = normalize_workspace_path(path);
    save_settings(&app, &settings)
}

#[tauri::command]
pub fn save_workspace_path(app: AppHandle, path: String) -> Result<(), String> {
    let mut settings = load_app_settings(&app);
    settings.workspace_path = normalize_workspace_path(path);
    save_settings(&app, &settings)
}

#[tauri::command]
pub fn save_selected_model(app: AppHandle, model: Option<String>) -> Result<(), String> {
    let mut settings = load_app_settings(&app);
    settings.selected_model = model.filter(|value| !value.trim().is_empty());
    save_settings(&app, &settings)
}

#[tauri::command]
pub fn save_favorite_models(app: AppHandle, models: Vec<String>) -> Result<(), String> {
    let mut settings = load_app_settings(&app);
    settings.favorite_models = models;
    save_settings(&app, &settings)
}

#[tauri::command]
pub fn save_safety_mode(app: AppHandle, safety_mode: String) -> Result<(), String> {
    let mut settings = load_app_settings(&app);
    settings.safety_mode = safety_mode;
    save_settings(&app, &settings)
}

#[tauri::command]
pub fn save_theme_accent(app: AppHandle, theme_accent: String) -> Result<(), String> {
    let mut settings = load_app_settings(&app);
    settings.theme_accent = theme_accent;
    save_settings(&app, &settings)
}

#[tauri::command]
pub fn save_launch_at_startup(app: AppHandle, launch_at_startup: bool) -> Result<(), String> {
    let mut settings = load_app_settings(&app);
    settings.launch_at_startup = launch_at_startup;
    save_settings(&app, &settings)
}

#[tauri::command]
pub fn save_user_name(app: AppHandle, user_name: String) -> Result<(), String> {
    let mut settings = load_app_settings(&app);
    let trimmed = user_name.trim().to_string();
    settings.user_name = if trimmed.is_empty() { None } else { Some(trimmed) };
    settings.onboarding_complete = settings.user_name.is_some();
    save_settings(&app, &settings)
}

#[tauri::command]
pub fn save_onboarding_complete(app: AppHandle, complete: bool) -> Result<(), String> {
    let mut settings = load_app_settings(&app);
    settings.onboarding_complete = complete;
    save_settings(&app, &settings)
}

fn safe_companion_character(value: &str) -> String {
    match value.trim() {
        "tan-explorer" | "pink-hood" | "autumn-vest" | "green-scout" | "blue-hoodie" | "lavender-bear" | "yellow-rain" | "red-beanie" => value.trim().to_string(),
        _ => "tan-explorer".to_string(),
    }
}


fn safe_companion_size(value: &str) -> String {
    match value.trim() {
        "small" | "medium" | "large" => value.trim().to_string(),
        _ => "medium".to_string(),
    }
}

#[tauri::command]
pub fn save_companion_size(app: AppHandle, companion_size: String) -> Result<(), String> {
    let mut settings = load_app_settings(&app);
    settings.companion_size = safe_companion_size(&companion_size);
    save_settings(&app, &settings)?;
    let _ = native_companion_manager::stop();
    let _ = native_companion_manager::start(&app);
    Ok(())
}

#[tauri::command]
pub fn save_companion_character(app: AppHandle, character_id: String) -> Result<(), String> {
    let mut settings = load_app_settings(&app);
    settings.companion_character_id = safe_companion_character(&character_id);
    save_settings(&app, &settings)?;
    let _ = native_companion_manager::stop();
    let _ = native_companion_manager::start(&app);
    Ok(())
}

fn safe_companion_animation(value: &str, fallback: &str) -> String {
    match value.trim() {
        "hello" | "idle" | "working" | "walk-left" | "walk-right" | "thinking" | "command" | "done" | "error" => value.trim().to_string(),
        _ => fallback.to_string(),
    }
}

#[tauri::command]
pub fn save_companion_animation_settings(
    app: AppHandle,
    work_animation: String,
    thinking_animation: String,
    command_animation: String,
    done_animation: String,
    error_animation: String,
    idle_wave_animation: String,
) -> Result<(), String> {
    let mut settings = load_app_settings(&app);
    settings.companion_work_animation = safe_companion_animation(&work_animation, "working");
    settings.companion_thinking_animation = safe_companion_animation(&thinking_animation, "thinking");
    settings.companion_command_animation = safe_companion_animation(&command_animation, "command");
    settings.companion_done_animation = safe_companion_animation(&done_animation, "done");
    settings.companion_error_animation = safe_companion_animation(&error_animation, "error");
    settings.companion_idle_wave_animation = safe_companion_animation(&idle_wave_animation, "walk-right");
    save_settings(&app, &settings)?;
    let _ = native_companion_manager::stop();
    let _ = native_companion_manager::start(&app);
    Ok(())
}
