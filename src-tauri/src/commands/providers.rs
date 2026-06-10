use serde::Serialize;
use tauri::AppHandle;

use crate::{
    native_companion_manager,
    providers::unified_cli::{
        self, build_invocation, command_exists, command_output, command_version,
        provider_auth_status, AiCliContext, UnifiedCliRunner,
    },
    settings,
};


#[derive(Clone, Serialize)]
pub struct ProviderModel {
    pub id: String,
    pub provider_id: String,
    pub name: String,
    pub description: String,
    pub command_model: String,
    pub source: String,
    pub caps: Vec<String>,
    pub starred: bool,
}

#[derive(Clone, Serialize)]
pub struct ProviderStatus {
    pub id: String,
    pub name: String,
    pub command: String,
    pub native: bool,
    pub description: String,
    pub installed: bool,
    pub authenticated: bool,
    pub version: Option<String>,
    pub command_preview: Option<String>,
}

#[tauri::command]
pub fn detect_providers(app: AppHandle) -> Vec<ProviderStatus> {
    let settings = settings::load_settings(&app);

    let defs = vec![
        (
            "opencode-go",
            "OpenCode",
            settings.opencode_command.as_str(),
            true,
            "Primary Lil Buddy runtime. OpenCode handles OpenAI, Gemini, OpenRouter, and other model providers.",
        ),
        (
            "claude",
            "Claude Code",
            settings.claude_command.as_str(),
            false,
            "Standalone Claude Code CLI wrapper using the user's Claude login/session.",
        ),
    ];

    defs.into_iter()
        .map(|(id, name, command, native, description)| {
            let installed = command_exists(command);
            ProviderStatus {
                id: id.to_string(),
                name: name.to_string(),
                command: command.to_string(),
                native,
                description: description.to_string(),
                installed,
                authenticated: installed && provider_auth_status(command),
                version: if installed { command_version(command) } else { None },
                command_preview: None,
            }
        })
        .collect()
}

fn apply_workspace_override(context: &mut AiCliContext, workspace_path: Option<String>) {
    if let Some(path) = workspace_path {
        let trimmed = path.trim();
        context.workspace_path = if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        };
    }
}

fn apply_session_override(context: &mut AiCliContext, session_id: Option<String>) {
    if let Some(id) = session_id.filter(|value| !value.trim().is_empty()) {
        context.session_id = Some(id.trim().to_string());
    }
}

#[tauri::command]
pub fn preview_provider_command(
    app: AppHandle,
    provider_id: String,
    prompt: String,
    workspace_path: Option<String>,
    session_id: Option<String>,
) -> Result<String, String> {
    let settings = settings::load_settings(&app);
    let mut context = AiCliContext::from_settings(prompt, &settings);
    apply_workspace_override(&mut context, workspace_path);
    apply_session_override(&mut context, session_id);
    let invocation = build_invocation(&provider_id, &context)?;
    Ok(invocation.preview)
}

#[tauri::command]
pub fn run_provider_command(
    app: AppHandle,
    provider_id: String,
    prompt: String,
    workspace_path: Option<String>,
    session_id: Option<String>,
) -> Result<(), String> {
    let _ = native_companion_manager::set_category(&app, "work");
    let mut settings = settings::load_settings(&app);
    let mut context = AiCliContext::from_settings(prompt, &settings);
    apply_workspace_override(&mut context, workspace_path);
    apply_session_override(&mut context, session_id);
    if context.workspace_path != settings.workspace_path {
        settings.workspace_path = context.workspace_path.clone();
        let _ = settings::save_settings(&app, &settings);
    }
    let safe_provider = match provider_id.as_str() {
        "claude" | "opencode-go" => provider_id,
        _ => "opencode-go".to_string(),
    };
    let invocation = build_invocation(&safe_provider, &context)?;

    UnifiedCliRunner::new(app, safe_provider, invocation).run()
}

#[tauri::command]
pub fn stop_provider_command(app: AppHandle) -> Result<(), String> {
    unified_cli::stop_active(app)
}

#[tauri::command]
pub fn provider_is_running() -> bool {
    unified_cli::has_active_child()
}


#[tauri::command]
pub fn list_provider_models(app: AppHandle, provider_id: String, refresh: bool) -> Vec<ProviderModel> {
    let settings = settings::load_settings(&app);
    let favorites: std::collections::HashSet<String> = settings.favorite_models.into_iter().collect();

    let mut models = match provider_id.as_str() {
        "opencode-go" => list_opencode_models(&settings.opencode_command, refresh),
        "claude" => fallback_claude_models(),
        _ => Vec::new(),
    };

    for model in models.iter_mut() {
        model.starred = favorites.contains(&model.id);
    }

    models
}

fn list_opencode_models(command: &str, refresh: bool) -> Vec<ProviderModel> {
    let mut args = vec!["models".to_string()];

    if refresh {
        args.push("--refresh".to_string());
    }

    let output = opencode_models_output(command, &args);

    if let Ok(output) = output {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let parsed = parse_opencode_models(&stdout);
            if !parsed.is_empty() {
                return parsed;
            }
        }
    }

    fallback_opencode_models()
}

fn opencode_models_output(command: &str, args: &[String]) -> std::io::Result<std::process::Output> {
    command_output(command, args)
}

fn parse_opencode_models(output: &str) -> Vec<ProviderModel> {
    output
        .lines()
        .filter_map(|line| {
            let clean = strip_ansi(line);
            let trimmed = clean.trim();
            if trimmed.is_empty() || !trimmed.contains('/') {
                return None;
            }

            let command_model = trimmed.split_whitespace().next().unwrap_or(trimmed).trim().to_string();
            let mut parts = command_model.splitn(2, '/');
            let provider = parts.next().unwrap_or("opencode").to_string();
            let model_name = parts.next().unwrap_or(&command_model).to_string();

            Some(ProviderModel {
                id: command_model.clone(),
                provider_id: provider,
                name: prettify_model_name(&model_name),
                description: format!("OpenCode model: {}", command_model),
                command_model,
                source: "opencode models".to_string(),
                caps: vec!["tools".to_string(), "repo".to_string()],
                starred: false,
            })
        })
        .collect()
}

fn prettify_model_name(value: &str) -> String {
    value
        .replace(['-', '_'], " ")
        .split_whitespace()
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn fallback_opencode_models() -> Vec<ProviderModel> {
    vec![
        model("opencode-go/minimax-m2.5", "opencode-go", "Minimax M2.5", "OpenCode fallback model.", "fallback", vec!["tools", "repo"]),
        model("opencode-go/qwen3.5-plus", "opencode-go", "Qwen3.5 Plus", "OpenCode fallback model.", "fallback", vec!["tools", "repo"]),
    ]
}

fn strip_ansi(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    let mut chars = value.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' {
            if matches!(chars.peek(), Some('[')) {
                chars.next();
                while let Some(next) = chars.next() {
                    if ('@'..='~').contains(&next) {
                        break;
                    }
                }
                continue;
            }
        }

        result.push(ch);
    }

    result
}

fn fallback_claude_models() -> Vec<ProviderModel> {
    vec![
        model("claude-sonnet-4-6", "claude", "Claude Sonnet 4.6", "Claude Code default-style Sonnet model.", "fallback", vec!["tools", "code"]),
        model("claude-opus-4-5", "claude", "Claude Opus 4.5", "Higher-capability Claude model.", "fallback", vec!["tools", "code", "reasoning"]),
        model("claude-haiku-4-5", "claude", "Claude Haiku 4.5", "Fast lightweight Claude model.", "fallback", vec!["chat", "code"]),
    ]
}


fn model(id: &str, provider_id: &str, name: &str, description: &str, source: &str, caps: Vec<&str>) -> ProviderModel {
    ProviderModel {
        id: id.to_string(),
        provider_id: provider_id.to_string(),
        name: name.to_string(),
        description: description.to_string(),
        command_model: id.to_string(),
        source: source.to_string(),
        caps: caps.into_iter().map(|cap| cap.to_string()).collect(),
        starred: false,
    }
}
