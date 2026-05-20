use serde::Serialize;
use tauri::AppHandle;

use crate::{
    native_companion_manager,
    providers::unified_cli::{
        self, build_invocation, command_exists, command_version, provider_auth_status,
        AiCliContext, UnifiedCliRunner,
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

#[tauri::command]
pub fn preview_provider_command(
    app: AppHandle,
    provider_id: String,
    prompt: String,
) -> Result<String, String> {
    let settings = settings::load_settings(&app);
    let context = AiCliContext::from_settings(prompt, &settings);
    let invocation = build_invocation(&provider_id, &context)?;
    Ok(invocation.preview)
}

#[tauri::command]
pub fn run_provider_command(
    app: AppHandle,
    provider_id: String,
    prompt: String,
) -> Result<(), String> {
    let _ = native_companion_manager::set_category(&app, "work");
    let settings = settings::load_settings(&app);
    let context = AiCliContext::from_settings(prompt, &settings);
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

    let output = std::process::Command::new(command).args(args).output();

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

fn parse_opencode_models(output: &str) -> Vec<ProviderModel> {
    output
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
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
        model("openai/gpt-5.4", "openai", "GPT-5.4", "Fast everyday coding and chat through OpenCode.", "fallback", vec!["tools", "repo", "search"]),
        model("anthropic/claude-sonnet-4-6", "anthropic", "Claude Sonnet 4.6", "Strong real-world coding and reasoning model through OpenCode.", "fallback", vec!["tools", "repo"]),
        model("google/gemini-3-flash", "google", "Gemini 3 Flash", "Fast Gemini model for lower-latency coding help.", "fallback", vec!["tools", "repo"]),
        model("google/gemini-ultra-2", "google", "Gemini Ultra 2", "Higher-capability Gemini reasoning model.", "fallback", vec!["tools", "repo"]),
        model("opencode/gpt-5.1-codex", "opencode", "GPT-5.1 Codex", "OpenCode recommended coding model.", "fallback", vec!["tools", "repo", "diffs"]),
    ]
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
