use serde::Serialize;
use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Emitter};

use crate::runtime::stream_event;

#[derive(Clone, Debug, Serialize)]
pub struct PendingCommandApproval {
    pub id: String,
    pub command: String,
    pub risk_level: String,
    pub reason: String,
}

static APPROVALS: OnceLock<Mutex<HashMap<String, PendingCommandApproval>>> = OnceLock::new();

fn approvals() -> &'static Mutex<HashMap<String, PendingCommandApproval>> {
    APPROVALS.get_or_init(|| Mutex::new(HashMap::new()))
}

#[tauri::command]
pub fn request_command_approval(
    app: AppHandle,
    command: String,
    reason: String,
    risk_level: String,
) -> Result<PendingCommandApproval, String> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_millis();

    let approval = PendingCommandApproval {
        id: format!("approval-{}", now),
        command,
        risk_level,
        reason,
    };

    approvals()
        .lock()
        .map_err(|_| "failed to lock approvals".to_string())?
        .insert(approval.id.clone(), approval.clone());

    app.emit(
        "runtime://stream",
        stream_event(
            "command-approval",
            &format!(
                "pending={} · risk={} · command={} · reason={}",
                approval.id, approval.risk_level, approval.command, approval.reason
            ),
        ),
    )
    .ok();

    Ok(approval)
}

#[tauri::command]
pub fn approve_command_approval(app: AppHandle, id: String) -> Result<(), String> {
    let approval = approvals()
        .lock()
        .map_err(|_| "failed to lock approvals".to_string())?
        .remove(&id);

    if let Some(approval) = approval {
        app.emit(
            "runtime://stream",
            stream_event(
                "command-approval",
                &format!("approved={} · command={}", approval.id, approval.command),
            ),
        )
        .ok();
    }

    Ok(())
}

#[tauri::command]
pub fn deny_command_approval(app: AppHandle, id: String) -> Result<(), String> {
    let approval = approvals()
        .lock()
        .map_err(|_| "failed to lock approvals".to_string())?
        .remove(&id);

    if let Some(approval) = approval {
        app.emit(
            "runtime://stream",
            stream_event(
                "command-approval",
                &format!("denied={} · command={}", approval.id, approval.command),
            ),
        )
        .ok();
    }

    Ok(())
}


fn permission_pattern_for_path(path: &str) -> String {
    let trimmed = path.trim().trim_end_matches(['/', '*']);
    format!("{}/**", trimmed)
}

#[tauri::command]
pub fn allow_external_directory(app: AppHandle, path: String) -> Result<String, String> {
    let cleaned = path
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim_end_matches("/*")
        .trim_end_matches("/**")
        .to_string();

    if cleaned.is_empty() {
        return Err("missing external directory path".to_string());
    }

    let settings = crate::settings::load_settings(&app);
    let workspace_path = settings
        .workspace_path
        .clone()
        .ok_or_else(|| "choose a workspace before approving external directory access".to_string())?;

    let workspace = std::path::PathBuf::from(workspace_path)
        .canonicalize()
        .map_err(|error| error.to_string())?;

    let config_path = workspace.join("opencode.json");
    let mut root_value = if config_path.exists() {
        let raw = std::fs::read_to_string(&config_path).map_err(|error| error.to_string())?;
        serde_json::from_str::<serde_json::Value>(&raw).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({
            "$schema": "https://opencode.ai/config.json"
        })
    };

    if !root_value.is_object() {
        root_value = serde_json::json!({
            "$schema": "https://opencode.ai/config.json"
        });
    }

    let object = root_value
        .as_object_mut()
        .ok_or_else(|| "failed to open opencode config object".to_string())?;

    object
        .entry("$schema".to_string())
        .or_insert_with(|| serde_json::json!("https://opencode.ai/config.json"));

    let permission_value = object
        .entry("permission".to_string())
        .or_insert_with(|| serde_json::json!({}));

    if !permission_value.is_object() {
        *permission_value = serde_json::json!({});
    }

    let permission_object = permission_value
        .as_object_mut()
        .ok_or_else(|| "failed to open permission config".to_string())?;

    let external_value = permission_object
        .entry("external_directory".to_string())
        .or_insert_with(|| serde_json::json!({}));

    if !external_value.is_object() {
        *external_value = serde_json::json!({});
    }

    let external_object = external_value
        .as_object_mut()
        .ok_or_else(|| "failed to open external_directory config".to_string())?;

    let pattern = permission_pattern_for_path(&cleaned);
    external_object.insert(pattern.clone(), serde_json::json!("allow"));

    let formatted = serde_json::to_string_pretty(&root_value).map_err(|error| error.to_string())?;
    std::fs::write(&config_path, formatted).map_err(|error| error.to_string())?;

    app.emit(
        "runtime://stream",
        stream_event(
            "status",
            &format!("allowed external_directory {} in {}", pattern, config_path.display()),
        ),
    )
    .ok();

    Ok(pattern)
}
