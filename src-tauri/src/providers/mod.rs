pub mod claude;
pub mod opencode_go;
pub mod opencode_events;
pub mod unified_cli;

#[derive(Clone, Debug)]
pub enum PromptMode {
    Arg,
    Stdin,
}

#[derive(Clone, Debug)]
pub struct ProviderInvocation {
    pub command: String,
    pub fallback_command: Option<String>,
    pub args: Vec<String>,
    pub stdin: Option<String>,
    pub preview: String,
    pub mode: PromptMode,
    pub workspace_path: Option<String>,
}

pub fn build_invocation(
    provider_id: &str,
    prompt: &str,
    opencode_go_mode: &str,
    opencode_command: &str,
    opencode_provider_key: &str,
    workspace_path: Option<String>,
    selected_model: Option<String>,
) -> Result<ProviderInvocation, String> {
    match provider_id {
        "opencode-go" => opencode_go::build(
            prompt,
            opencode_go_mode,
            opencode_command,
            opencode_provider_key,
            workspace_path,
            selected_model,
        ),
        "claude" => claude::build(
            prompt,
            "claude",
            "stream-json",
            workspace_path,
            selected_model,
        ),
        other => Err(format!("unsupported provider: {}", other)),
    }
}

pub mod parser_registry;
pub mod claude_events;
