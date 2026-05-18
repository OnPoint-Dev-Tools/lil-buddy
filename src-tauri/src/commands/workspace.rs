use std::process::Command;

use tauri::{AppHandle, Emitter};
use tauri_plugin_dialog::DialogExt;

use crate::{runtime::stream_event, settings, workspace};

#[tauri::command]
pub fn detect_workspace(app: AppHandle) -> crate::workspace::WorkspaceInfo {
    let settings = settings::load_settings(&app);
    workspace::detect_workspace_with_fallback(settings.workspace_path, settings.default_directory)
}

#[tauri::command]
pub fn get_workspace_diff(app: AppHandle) -> crate::workspace::WorkspaceDiff {
    let settings = settings::load_settings(&app);
    workspace::get_workspace_diff_at(settings.workspace_path)
}

#[tauri::command]
pub async fn choose_workspace_folder(app: AppHandle) -> Result<Option<String>, String> {
    let folder = app
        .dialog()
        .file()
        .blocking_pick_folder()
        .map(|path| path.to_string());

    if let Some(path) = folder.clone() {
        let mut app_settings = settings::load_settings(&app);
        app_settings.workspace_path = Some(path.clone());
        settings::save_settings(&app, &app_settings)?;

        app.emit(
            "runtime://stream",
            stream_event("workspace", "workspace selected"),
        )
        .ok();
    }

    Ok(folder)
}

#[tauri::command]
pub fn open_workspace_terminal(app: AppHandle) -> Result<(), String> {
    let app_settings = settings::load_settings(&app);
    let path = app_settings
        .workspace_path
        .or(app_settings.default_directory)
        .or_else(|| workspace::default_workspace_path().map(|path| path.to_string_lossy().to_string()))
        .ok_or_else(|| "could not resolve home directory".to_string())?;

    // Linux-first terminal launcher scaffold. Best effort across common setups.
    let candidates = [
        ("kitty", vec!["--working-directory", &path]),
        ("alacritty", vec!["--working-directory", &path]),
        ("wezterm", vec!["start", "--cwd", &path]),
        ("gnome-terminal", vec!["--working-directory", &path]),
        ("konsole", vec!["--workdir", &path]),
        ("foot", vec!["--working-directory", &path]),
    ];

    for (binary, args) in candidates {
        let exists = Command::new("sh")
            .arg("-c")
            .arg(format!("command -v '{}'", binary))
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false);

        if exists {
            Command::new(binary)
                .args(args)
                .spawn()
                .map_err(|error| error.to_string())?;

            app.emit(
                "runtime://stream",
                stream_event("workspace", &format!("opened terminal in {}", path)),
            )
            .ok();

            return Ok(());
        }
    }

    Err("no supported terminal found: tried kitty, alacritty, wezterm, gnome-terminal, konsole, foot".to_string())
}

#[tauri::command]
pub fn restore_workspace_file(app: AppHandle, path: String) -> Result<String, String> {
    let app_settings = settings::load_settings(&app);

    let result = workspace::restore_file_at(app_settings.workspace_path, &path)?;

    app.emit(
        "runtime://stream",
        stream_event("file-change", &format!("restore: {}", result)),
    )
    .ok();

    Ok(result)
}

#[tauri::command]
pub fn restore_all_workspace_files(app: AppHandle) -> Result<String, String> {
    let app_settings = settings::load_settings(&app);

    let result = workspace::restore_all_at(app_settings.workspace_path)?;

    app.emit(
        "runtime://stream",
        stream_event("file-change", &format!("restore all: {}", result)),
    )
    .ok();

    Ok(result)
}

#[tauri::command]
pub fn emit_workspace_diff_events(app: AppHandle) -> Result<(), String> {
    let app_settings = settings::load_settings(&app);
    let diff = workspace::get_workspace_diff_at(app_settings.workspace_path);

    if diff.changed_files.is_empty() {
        app.emit(
            "runtime://stream",
            stream_event("file-change", "No git-tracked file changes detected."),
        )
        .ok();
        return Ok(());
    }

    app.emit(
        "runtime://stream",
        stream_event(
            "file-change",
            &format!("{} changed file(s): {}", diff.changed_files.len(), diff.changed_files.join(", ")),
        ),
    )
    .ok();

    if !diff.diff_stat.trim().is_empty() {
        app.emit(
            "runtime://stream",
            stream_event("diff", &format!("diff stat:\n{}", diff.diff_stat)),
        )
        .ok();
    }

    for file_diff in diff.diffs.iter().take(3) {
        app.emit(
            "runtime://stream",
            stream_event("diff", &format!("{}\n{}", file_diff.path, file_diff.diff)),
        )
        .ok();
    }

    Ok(())
}


#[tauri::command]
pub fn start_workspace_watch(app: AppHandle) -> Result<(), String> {
    app.emit(
        "runtime://stream",
        stream_event("watch", "workspace watch is managed by the unified runner during provider runs"),
    )
    .ok();

    Ok(())
}

#[tauri::command]
pub fn stop_workspace_watch(app: AppHandle) -> Result<(), String> {
    app.emit(
        "runtime://stream",
        stream_event("watch", "workspace watch stopped"),
    )
    .ok();

    Ok(())
}
