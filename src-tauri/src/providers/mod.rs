pub mod claude;
pub mod opencode_go;
pub mod opencode_events;
pub mod unified_cli;

#[derive(Clone, Debug)]
pub struct ProviderInvocation {
    pub command: String,
    pub fallback_command: Option<String>,
    pub args: Vec<String>,
    pub stdin: Option<String>,
    pub preview: String,
    pub workspace_path: Option<String>,
}

pub mod parser_registry;
pub mod claude_events;
