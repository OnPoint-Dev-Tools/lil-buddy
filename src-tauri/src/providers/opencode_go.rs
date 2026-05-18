use super::{PromptMode, ProviderInvocation};

pub fn build(
    prompt: &str,
    mode: &str,
    opencode_command: &str,
    opencode_provider_key: &str,
    workspace: Option<String>,
    selected_model: Option<String>,
) -> Result<ProviderInvocation, String> {
    match mode {
        "run-formatted" => build_run_formatted(
            prompt,
            workspace,
            opencode_command,
            opencode_provider_key,
            selected_model,
        ),
        "run-stdin" => build_run_stdin(
            prompt,
            workspace,
            opencode_command,
            opencode_provider_key,
            selected_model,
        ),
        "raw-arg" => build_raw_arg(prompt, opencode_command, opencode_provider_key),
        _ => build_run_json(
            prompt,
            workspace,
            opencode_command,
            opencode_provider_key,
            selected_model,
        ),
    }
}

fn base_run_args(workspace: Option<String>, selected_model: Option<String>) -> Vec<String> {
    let mut args = vec!["run".to_string()];

    if let Some(dir) = workspace {
        if !dir.trim().is_empty() {
            args.push("--dir".to_string());
            args.push(dir);
        }
    }

    if let Some(model) = selected_model {
        if !model.trim().is_empty() {
            args.push("--model".to_string());
            args.push(model);
        }
    }

    args
}

fn preview_prefix(command: &str, provider_key: &str) -> String {
    format!("{} # provider-id: {}", command, provider_key)
}

fn build_run_json(
    prompt: &str,
    workspace: Option<String>,
    command: &str,
    provider_key: &str,
    selected_model: Option<String>,
) -> Result<ProviderInvocation, String> {
    let mut args = base_run_args(workspace.clone(), selected_model);
    args.push("--thinking".to_string());
    args.push("--format".to_string());
    args.push("json".to_string());
    args.push(prompt.to_string());

    Ok(ProviderInvocation {
        command: command.to_string(),
        fallback_command: None,
        args: args.clone(),
        stdin: None,
        preview: format!("{} {}", preview_prefix(command, provider_key), shell_words(&args)),
        mode: PromptMode::Arg,
        workspace_path: workspace,
    })
}

fn build_run_formatted(
    prompt: &str,
    workspace: Option<String>,
    command: &str,
    provider_key: &str,
    selected_model: Option<String>,
) -> Result<ProviderInvocation, String> {
    let mut args = base_run_args(workspace.clone(), selected_model);
    args.push(prompt.to_string());

    Ok(ProviderInvocation {
        command: command.to_string(),
        fallback_command: None,
        args: args.clone(),
        stdin: None,
        preview: format!("{} {}", preview_prefix(command, provider_key), shell_words(&args)),
        mode: PromptMode::Arg,
        workspace_path: workspace,
    })
}

fn build_run_stdin(
    prompt: &str,
    workspace: Option<String>,
    command: &str,
    provider_key: &str,
    selected_model: Option<String>,
) -> Result<ProviderInvocation, String> {
    let args = base_run_args(workspace.clone(), selected_model);

    Ok(ProviderInvocation {
        command: command.to_string(),
        fallback_command: None,
        args: args.clone(),
        stdin: Some(prompt.to_string()),
        preview: format!(
            "echo '<prompt>' | {} {} # provider-id: {}",
            command,
            shell_words(&args),
            provider_key
        ),
        mode: PromptMode::Stdin,
        workspace_path: workspace,
    })
}

fn build_raw_arg(
    prompt: &str,
    command: &str,
    provider_key: &str,
) -> Result<ProviderInvocation, String> {
    let args = vec![prompt.to_string()];

    Ok(ProviderInvocation {
        command: command.to_string(),
        fallback_command: None,
        args: args.clone(),
        stdin: None,
        preview: format!("{} {} # provider-id: {}", command, shell_words(&args), provider_key),
        mode: PromptMode::Arg,
        workspace_path: None,
    })
}

fn shell_words(args: &[String]) -> String {
    args.iter()
        .map(|arg| shell_quote(arg))
        .collect::<Vec<_>>()
        .join(" ")
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace("'", "'\\''"))
}
