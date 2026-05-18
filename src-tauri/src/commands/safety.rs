#[tauri::command]
pub fn classify_command_risk(command: String) -> crate::safety::CommandRisk {
    crate::safety::classify_command(&command)
}
