use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{fs, path::PathBuf};
use tauri::{AppHandle, Manager};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkspaceSession {
    pub id: String,
    pub name: String,
    pub path: String,
    pub messages: Value,
    pub updated_at: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
struct WorkspaceSessionStore {
    sessions: Vec<WorkspaceSession>,
}

fn sessions_file(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;

    fs::create_dir_all(&dir).map_err(|error| error.to_string())?;

    Ok(dir.join("workspace_sessions.json"))
}

fn load_store(app: &AppHandle) -> Result<WorkspaceSessionStore, String> {
    let path = sessions_file(app)?;

    if !path.exists() {
        return Ok(WorkspaceSessionStore::default());
    }

    let text = fs::read_to_string(path).map_err(|error| error.to_string())?;

    if text.trim().is_empty() {
        return Ok(WorkspaceSessionStore::default());
    }

    serde_json::from_str(&text).map_err(|error| error.to_string())
}

fn save_store(app: &AppHandle, store: &WorkspaceSessionStore) -> Result<(), String> {
    let path = sessions_file(app)?;
    let text = serde_json::to_string_pretty(store).map_err(|error| error.to_string())?;
    fs::write(path, text).map_err(|error| error.to_string())
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

#[tauri::command]
pub fn list_workspace_sessions(app: AppHandle) -> Result<Vec<WorkspaceSession>, String> {
    let mut store = load_store(&app)?;
    store
        .sessions
        .sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(store.sessions)
}

#[tauri::command]
pub fn save_workspace_session(
    app: AppHandle,
    name: String,
    path: String,
    messages: Value,
) -> Result<WorkspaceSession, String> {
    if path.trim().is_empty() {
        return Err("workspace path is empty".to_string());
    }

    let mut store = load_store(&app)?;
    let session = WorkspaceSession {
        id: path.clone(),
        name: if name.trim().is_empty() {
            workspace_name_from_path(&path)
        } else {
            name
        },
        path: path.clone(),
        messages,
        updated_at: now_ms(),
    };

    store.sessions.retain(|item| item.id != session.id);
    store.sessions.insert(0, session.clone());
    store.sessions.truncate(30);

    save_store(&app, &store)?;

    Ok(session)
}

#[tauri::command]
pub fn load_workspace_session(app: AppHandle, id: String) -> Result<Option<WorkspaceSession>, String> {
    let store = load_store(&app)?;
    Ok(store.sessions.into_iter().find(|session| session.id == id))
}

#[tauri::command]
pub fn get_workspace_session(
    app: AppHandle,
    workspace_path: String,
    expert_name: String,
) -> Result<Option<WorkspaceSession>, String> {
    let _ = expert_name;
    let store = load_store(&app)?;
    Ok(store.sessions.into_iter().find(|session| session.id == workspace_path || session.path == workspace_path))
}

#[tauri::command]
pub fn sync_workspace_session(
    app: AppHandle,
    workspace_path: String,
    expert_name: String,
    messages: Vec<Value>,
) -> Result<WorkspaceSession, String> {
    if workspace_path.trim().is_empty() {
        return Err("workspace path is empty".to_string());
    }

    let mut store = load_store(&app)?;
    let path = workspace_path.trim().to_string();
    let existing_name = store
        .sessions
        .iter()
        .find(|session| session.id == path || session.path == path)
        .map(|session| session.name.clone());
    let fallback_name = if expert_name.trim().is_empty() {
        workspace_name_from_path(&path)
    } else {
        format!("{} · {}", workspace_name_from_path(&path), expert_name.trim())
    };
    let session = WorkspaceSession {
        id: path.clone(),
        name: existing_name.unwrap_or(fallback_name),
        path: path.clone(),
        messages: Value::Array(messages),
        updated_at: now_ms(),
    };

    store.sessions.retain(|item| item.id != session.id && item.path != session.path);
    store.sessions.insert(0, session.clone());
    store.sessions.truncate(30);

    save_store(&app, &store)?;

    Ok(session)
}

#[tauri::command]
pub fn delete_workspace_session(app: AppHandle, id: String) -> Result<(), String> {
    let mut store = load_store(&app)?;
    store.sessions.retain(|session| session.id != id);
    save_store(&app, &store)
}

fn workspace_name_from_path(path: &str) -> String {
    std::path::Path::new(path)
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "Workspace".to_string())
}
