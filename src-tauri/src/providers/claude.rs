use super::ProviderInvocation;

pub fn build(
    prompt: &str,
    command: &str,
    _output_format: &str,
    workspace_path: Option<String>,
    selected_model: Option<String>,
    session_id: Option<String>,
) -> Result<ProviderInvocation, String> {
    let command = if command.trim().is_empty() {
        "claude".to_string()
    } else {
        command.trim().to_string()
    };

    let has_session = session_id.as_ref().is_some_and(|id| !id.trim().is_empty());
    let mut args = vec![];

    if has_session {
        args.push("--continue".to_string());
    }

    args.push(prompt.to_string());

    if let Some(model) = selected_model.clone().filter(|value| !value.trim().is_empty()) {
        args.push("--model".to_string());
        args.push(model);
    }

    let mut preview_parts = vec![command.clone()];

    if has_session {
        preview_parts.push("--continue".to_string());
    }

    preview_parts.push(shell_quote(prompt));

    if let Some(model) = selected_model.filter(|value| !value.trim().is_empty()) {
        preview_parts.push("--model".to_string());
        preview_parts.push(shell_quote(&model));
    }

    Ok(ProviderInvocation {
        command,
        fallback_command: Some("claude".to_string()),
        args,
        stdin: None,
        preview: preview_parts.join(" "),
        workspace_path,
    })
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace("'", "'\\''"))
}
