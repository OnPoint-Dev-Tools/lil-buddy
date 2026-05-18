use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use tauri::{AppHandle, Manager};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub companion_x: Option<i32>,
    pub companion_y: Option<i32>,
    pub chat_offset_x: i32,
    pub chat_offset_y: i32,
    pub selected_provider: String,
    pub opencode_go_mode: String,
    pub opencode_command: String,
    pub opencode_provider_key: String,
    pub claude_command: String,
    pub claude_output_format: String,
    pub workspace_path: Option<String>,
    pub default_directory: Option<String>,
    pub selected_model: Option<String>,
    pub favorite_models: Vec<String>,
    pub safety_mode: String,
    pub theme_accent: String,
    pub launch_at_startup: bool,
    pub user_name: Option<String>,
    pub onboarding_complete: bool,
    pub companion_work_animation: String,
    pub companion_thinking_animation: String,
    pub companion_command_animation: String,
    pub companion_done_animation: String,
    pub companion_error_animation: String,
    pub companion_idle_wave_animation: String,
    pub companion_character_id: String,
    pub companion_size: String,
    pub telegram_gateway_enabled: bool,
    pub telegram_gateway_mode: String,
    pub telegram_bot_token: String,
    pub telegram_webhook_public_url: String,
    pub telegram_webhook_local_port: u16,
    pub telegram_webhook_path_secret: String,
    pub telegram_webhook_secret: String,
    pub telegram_allowed_chat_id: Option<String>,
    pub telegram_active_expert_id: Option<String>,
    pub telegram_active_expert_name: Option<String>,
    pub telegram_active_expert_role: Option<String>,
    pub telegram_active_expert_prompt: Option<String>,
    pub telegram_active_workspace_path: Option<String>,
    pub telegram_experts_json: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            companion_x: None,
            companion_y: None,
            chat_offset_x: -240,
            chat_offset_y: -780,
            selected_provider: "opencode-go".to_string(),
            opencode_go_mode: "run-json".to_string(),
            opencode_command: "opencode".to_string(),
            opencode_provider_key: "opencode-go".to_string(),
            claude_command: "claude".to_string(),
            claude_output_format: "stream-json".to_string(),
            workspace_path: None,
            default_directory: None,
            selected_model: None,
            favorite_models: Vec::new(),
            safety_mode: "confirm".to_string(),
            theme_accent: "honey".to_string(),
            launch_at_startup: false,
            user_name: None,
            onboarding_complete: false,
            companion_work_animation: "working".to_string(),
            companion_thinking_animation: "thinking".to_string(),
            companion_command_animation: "command".to_string(),
            companion_done_animation: "done".to_string(),
            companion_error_animation: "error".to_string(),
            companion_idle_wave_animation: "walk-right".to_string(),
            companion_character_id: "tan-explorer".to_string(),
            companion_size: "medium".to_string(),
            telegram_gateway_enabled: false,
            telegram_gateway_mode: "webhook".to_string(),
            telegram_bot_token: String::new(),
            telegram_webhook_public_url: "https://lil-buddy.cortex-ai.icu".to_string(),
            telegram_webhook_local_port: 8787,
            telegram_webhook_path_secret: "lil-buddy-telegram".to_string(),
            telegram_webhook_secret: "lil-buddy-secret".to_string(),
            telegram_allowed_chat_id: None,
            telegram_active_expert_id: Some("default-lil-buddy".to_string()),
            telegram_active_expert_name: Some("Lil Buddy".to_string()),
            telegram_active_expert_role: Some("Default Lil Buddy".to_string()),
            telegram_active_expert_prompt: Some("You are Lil Buddy, the default helpful coding companion. Help the user clearly, stay practical, and keep them updated while working.".to_string()),
            telegram_active_workspace_path: None,
            telegram_experts_json: String::new(),
        }
    }
}

pub fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_config_dir()
        .map_err(|error| error.to_string())?;

    fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    Ok(dir.join("settings.json"))
}

pub fn load_settings(app: &AppHandle) -> AppSettings {
    let Ok(path) = settings_path(app) else {
        return AppSettings::default();
    };

    let Ok(raw) = fs::read_to_string(path) else {
        return AppSettings::default();
    };

    serde_json::from_str(&raw).unwrap_or_default()
}

pub fn save_settings(app: &AppHandle, settings: &AppSettings) -> Result<(), String> {
    let path = settings_path(app)?;
    let raw = serde_json::to_string_pretty(settings).map_err(|error| error.to_string())?;
    fs::write(path, raw).map_err(|error| error.to_string())
}

pub fn save_companion_position(_app: &AppHandle) -> Result<(), String> {
    Ok(())
}

pub fn restore_companion_position(_app: &AppHandle) -> Result<(), String> {
    Ok(())
}

pub fn reset_companion_position(app: &AppHandle) -> Result<(), String> {
    let mut settings = load_settings(app);
    settings.companion_x = None;
    settings.companion_y = None;
    save_settings(app, &settings)
}


pub fn save_companion_position_xy(app: &AppHandle, x: i32, y: i32) -> Result<(), String> {
    let mut settings = load_settings(app);
    settings.companion_x = Some(x);
    settings.companion_y = Some(y);
    save_settings(app, &settings)
}
