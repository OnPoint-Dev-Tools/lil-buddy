use serde::Deserialize;
use tauri::AppHandle;

use crate::{gateways, settings};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TelegramGatewaySettingsPayload {
    gateway_mode: String,
    bot_token: String,
    bot_token_from_env: bool,
    allowed_chat_id: String,
    allowed_chat_id_from_env: bool,
    webhook_public_url: String,
    webhook_public_url_from_env: bool,
    webhook_local_port: u16,
    webhook_path_secret: String,
    webhook_path_secret_from_env: bool,
    webhook_secret: String,
    webhook_secret_from_env: bool,
}

#[tauri::command]
pub fn save_telegram_gateway_settings(
    app: AppHandle,
    payload: TelegramGatewaySettingsPayload,
) -> Result<(), String> {
    let mut settings = settings::load_settings(&app);
    settings.telegram_gateway_mode = match payload.gateway_mode.trim() {
        "polling" | "webhook" => payload.gateway_mode.trim().to_string(),
        _ => "webhook".to_string(),
    };
    settings.telegram_bot_token = payload.bot_token.trim().to_string();
    settings.telegram_bot_token_from_env = payload.bot_token_from_env;
    settings.telegram_allowed_chat_id = if payload.allowed_chat_id.trim().is_empty() {
        None
    } else {
        Some(payload.allowed_chat_id.trim().to_string())
    };
    settings.telegram_allowed_chat_id_from_env = payload.allowed_chat_id_from_env;
    settings.telegram_webhook_public_url = payload
        .webhook_public_url
        .trim()
        .trim_end_matches('/')
        .to_string();
    settings.telegram_webhook_public_url_from_env = payload.webhook_public_url_from_env;
    settings.telegram_webhook_local_port = if payload.webhook_local_port == 0 {
        8787
    } else {
        payload.webhook_local_port
    };
    settings.telegram_webhook_path_secret = payload.webhook_path_secret.trim().to_string();
    settings.telegram_webhook_path_secret_from_env = payload.webhook_path_secret_from_env;
    settings.telegram_webhook_secret = payload.webhook_secret.trim().to_string();
    settings.telegram_webhook_secret_from_env = payload.webhook_secret_from_env;
    settings::save_settings(&app, &settings)
}

#[tauri::command]
pub fn save_telegram_active_expert(
    app: AppHandle,
    expert_id: String,
    name: String,
    role: String,
    system_prompt: String,
    workspace_path: String,
) -> Result<(), String> {
    let mut settings = settings::load_settings(&app);
    settings.telegram_active_expert_id = if expert_id.trim().is_empty() { None } else { Some(expert_id.trim().to_string()) };
    settings.telegram_active_expert_name = if name.trim().is_empty() { None } else { Some(name.trim().to_string()) };
    settings.telegram_active_expert_role = if role.trim().is_empty() { None } else { Some(role.trim().to_string()) };
    settings.telegram_active_expert_prompt = if system_prompt.trim().is_empty() { None } else { Some(system_prompt.trim().to_string()) };
    settings.telegram_active_workspace_path = if workspace_path.trim().is_empty() { None } else { Some(workspace_path.trim().to_string()) };
    settings::save_settings(&app, &settings)
}

#[tauri::command]
pub fn save_telegram_experts(app: AppHandle, experts_json: String) -> Result<(), String> {
    let mut settings = settings::load_settings(&app);
    settings.telegram_experts_json = experts_json;
    settings::save_settings(&app, &settings)
}

#[tauri::command]
pub fn start_telegram_gateway(app: AppHandle) -> Result<(), String> {
    gateways::telegram::start(app)
}

#[tauri::command]
pub fn stop_telegram_gateway(app: AppHandle) -> Result<(), String> {
    gateways::telegram::stop(app)
}

#[tauri::command]
pub fn telegram_gateway_running() -> bool {
    gateways::telegram::is_running()
}
