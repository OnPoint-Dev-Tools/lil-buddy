use super::{PromptMode, ProviderInvocation};

pub fn build(
    prompt: &str,
    command: &str,
    _output_format: &str,
    workspace_path: Option<String>,
    selected_model: Option<String>,
) -> Result<ProviderInvocation, String> {
    let command = if command.trim().is_empty() {
        "claude".to_string()
    } else {
        command.trim().to_string()
    };

    // Claude CLI interactive initial-prompt mode.
    // Do not use `-p`; that belongs to Claude's print/Agent SDK-style flow.
    // Passing the prompt as a positional argument launches `claude "query"`.
    let mut args = vec![prompt.to_string()];

    if let Some(model) = selected_model.clone().filter(|value| !value.trim().is_empty()) {
        args.push("--model".to_string());
        args.push(model);
    }

    let mut preview_parts = vec![command.clone(), shell_quote(prompt)];

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
        mode: PromptMode::Arg,
        workspace_path,
    })
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace("'", "'\\''"))
}
