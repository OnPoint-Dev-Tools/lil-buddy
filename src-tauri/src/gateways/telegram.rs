use once_cell::sync::OnceCell;
use serde_json::Value;
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};
use tauri::{AppHandle, Emitter};

use crate::{
    providers::unified_cli::{build_invocation, AiCliContext},
    runtime::stream_event,
    settings::{self as app_settings, AppSettings},
    workspace,
};

struct GatewayState {
    running: Arc<AtomicBool>,
}

static TELEGRAM_GATEWAY: OnceCell<Mutex<Option<GatewayState>>> = OnceCell::new();


fn redact_telegram_secret(message: &str, settings: &AppSettings) -> String {
    let mut safe = message.to_string();

    for secret in [
        settings.telegram_bot_token.trim(),
        settings.telegram_webhook_secret.trim(),
        settings.telegram_webhook_path_secret.trim(),
    ] {
        if !secret.is_empty() {
            safe = safe.replace(secret, "[redacted]");
        }
    }

    safe
}

fn redact_telegram_token(message: &str, token: &str) -> String {
    if token.trim().is_empty() {
        return message.to_string();
    }

    message.replace(token.trim(), "[redacted-telegram-token]")
}


fn cell() -> &'static Mutex<Option<GatewayState>> {
    TELEGRAM_GATEWAY.get_or_init(|| Mutex::new(None))
}

pub fn is_running() -> bool {
    cell()
        .lock()
        .map(|guard| guard.as_ref().map(|state| state.running.load(Ordering::SeqCst)).unwrap_or(false))
        .unwrap_or(false)
}

pub fn start(app: AppHandle) -> Result<(), String> {
    let current_settings = app_settings::load_settings(&app);
    if current_settings.telegram_bot_token.trim().is_empty() {
        return Err("Telegram bot token is required before starting the gateway.".to_string());
    }

    let mut guard = cell().lock().map_err(|_| "telegram gateway lock poisoned".to_string())?;
    if guard.as_ref().map(|state| state.running.load(Ordering::SeqCst)).unwrap_or(false) {
        return Ok(());
    }

    let running = Arc::new(AtomicBool::new(true));
    let worker_running = running.clone();
    let worker_app = app.clone();
    let mode = current_settings.telegram_gateway_mode.trim().to_string();

    if mode == "webhook" {
        configure_webhook(&current_settings).map_err(|error| redact_telegram_secret(&error, &current_settings))?;
        thread::spawn(move || run_webhook_server(worker_app, worker_running));
    } else {
        let _ = delete_webhook(&current_settings.telegram_bot_token);
        thread::spawn(move || run_poll_loop(worker_app, worker_running));
    }

    *guard = Some(GatewayState { running });

    let mut next = current_settings;
    next.telegram_gateway_enabled = true;
    app_settings::save_settings(&app, &next)?;

    let label = if mode == "webhook" { "telegram webhook gateway started" } else { "telegram polling gateway started" };
    let _ = app.emit("runtime://stream", stream_event("gateway", label));
    Ok(())
}

pub fn stop(app: AppHandle) -> Result<(), String> {
    let current_settings = app_settings::load_settings(&app);

    if let Ok(mut guard) = cell().lock() {
        if let Some(state) = guard.take() {
            state.running.store(false, Ordering::SeqCst);
        }
    }

    if current_settings.telegram_gateway_mode == "webhook" && !current_settings.telegram_bot_token.trim().is_empty() {
        let _ = delete_webhook(&current_settings.telegram_bot_token);
    }

    let mut next = current_settings;
    next.telegram_gateway_enabled = false;
    app_settings::save_settings(&app, &next)?;
    let _ = app.emit("runtime://stream", stream_event("gateway", "telegram gateway stopped"));
    Ok(())
}

fn configure_webhook(settings: &AppSettings) -> Result<(), String> {
    let token = settings.telegram_bot_token.trim();
    let url = normalized_webhook_url(settings)?;

    let mut command = Command::new("curl");
    command
        .arg("-sS")
        .arg("-X")
        .arg("POST")
        .arg(format!("https://api.telegram.org/bot{}/setWebhook", token))
        .arg("--data-urlencode")
        .arg(format!("url={}", url))
        .arg("-d")
        .arg("drop_pending_updates=true");

    if !settings.telegram_webhook_secret.trim().is_empty() {
        command
            .arg("--data-urlencode")
            .arg(format!("secret_token={}", settings.telegram_webhook_secret.trim()));
    }

    let output = command.output().map_err(|error| format!("failed to run curl: {}", error))?;
    telegram_api_result("setWebhook", &output.stdout, &output.stderr)
}

fn delete_webhook(token: &str) -> Result<(), String> {
    if token.trim().is_empty() {
        return Ok(());
    }

    let output = Command::new("curl")
        .arg("-sS")
        .arg("-X")
        .arg("POST")
        .arg(format!("https://api.telegram.org/bot{}/deleteWebhook", token.trim()))
        .arg("-d")
        .arg("drop_pending_updates=false")
        .output()
        .map_err(|error| format!("failed to run curl: {}", error))?;

    telegram_api_result("deleteWebhook", &output.stdout, &output.stderr)
}


fn telegram_api_result(action: &str, stdout: &[u8], stderr: &[u8]) -> Result<(), String> {
    let stdout_text = String::from_utf8_lossy(stdout).trim().to_string();
    let stderr_text = String::from_utf8_lossy(stderr).trim().to_string();

    if !stdout_text.is_empty() {
        if let Ok(parsed) = serde_json::from_str::<Value>(&stdout_text) {
            if parsed.get("ok").and_then(Value::as_bool).unwrap_or(false) {
                return Ok(());
            }

            let description = parsed
                .get("description")
                .and_then(Value::as_str)
                .unwrap_or("Telegram returned an unknown error");

            return Err(format!("Telegram {} failed: {}", action, description));
        }
    }

    if !stderr_text.is_empty() {
        return Err(format!("Telegram {} failed: {}", action, stderr_text));
    }

    Err(format!("Telegram {} failed with an empty response", action))
}

fn normalized_webhook_url(settings: &AppSettings) -> Result<String, String> {
    let base = settings.telegram_webhook_public_url.trim().trim_end_matches('/');
    if base.is_empty() {
        return Err("Telegram webhook public HTTPS URL is required in webhook mode.".to_string());
    }

    if !base.starts_with("https://") {
        return Err("Telegram webhook public URL must start with https://. Telegram does not accept plain http webhook URLs.".to_string());
    }

    if base.contains("localhost") || base.contains("127.0.0.1") {
        return Err("Telegram webhook public URL must be publicly reachable. Use your Cloudflare Tunnel URL, not localhost.".to_string());
    }

    let path_secret = settings.telegram_webhook_path_secret.trim().trim_matches('/');
    if path_secret.is_empty() {
        return Err("Telegram webhook path secret is required in webhook mode.".to_string());
    }

    Ok(format!("{}/telegram/{}", base, path_secret))
}

fn run_webhook_server(app: AppHandle, running: Arc<AtomicBool>) {
    let initial_settings = app_settings::load_settings(&app);
    let port = initial_settings.telegram_webhook_local_port.max(1);
    let bind_addr = format!("127.0.0.1:{}", port);

    let listener = match TcpListener::bind(&bind_addr) {
        Ok(listener) => listener,
        Err(error) => {
            let _ = app.emit(
                "runtime://stream",
                stream_event("stderr", &format!("telegram webhook bind failed on {}: {}", bind_addr, error)),
            );
            return;
        }
    };

    let _ = listener.set_nonblocking(true);
    let _ = app.emit(
        "runtime://stream",
        stream_event("gateway", &format!("telegram webhook listening on http://{}", bind_addr)),
    );

    while running.load(Ordering::SeqCst) {
        match listener.accept() {
            Ok((mut stream, _addr)) => {
                let app = app.clone();
                thread::spawn(move || {
                    handle_webhook_stream(&app, &mut stream);
                });
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(120));
            }
            Err(error) => {
                let _ = app.emit(
                    "runtime://stream",
                    stream_event("stderr", &format!("telegram webhook accept failed: {}", error)),
                );
                thread::sleep(Duration::from_secs(1));
            }
        }
    }
}

fn handle_webhook_stream(app: &AppHandle, stream: &mut TcpStream) {
    let mut buffer = vec![0_u8; 1024 * 1024];
    let Ok(size) = stream.read(&mut buffer) else {
        let _ = write_http(stream, 400, "bad request");
        return;
    };

    let request = String::from_utf8_lossy(&buffer[..size]).to_string();
    let Some((headers, body)) = request.split_once("\r\n\r\n") else {
        let _ = write_http(stream, 400, "bad request");
        return;
    };

    let first_line = headers.lines().next().unwrap_or_default();
    if !first_line.starts_with("POST ") {
        let _ = write_http(stream, 405, "method not allowed");
        return;
    }

    let current_settings = app_settings::load_settings(app);
    let expected_path = format!("/telegram/{}", current_settings.telegram_webhook_path_secret.trim());
    if !first_line.contains(&format!(" {} ", expected_path)) {
        let _ = write_http(stream, 404, "not found");
        return;
    }

    let expected_secret = current_settings.telegram_webhook_secret.trim();
    if !expected_secret.is_empty() {
        let valid_secret = headers.lines().any(|line| {
            let lower = line.to_ascii_lowercase();
            lower.starts_with("x-telegram-bot-api-secret-token:")
                && line.split_once(':').map(|(_, value)| value.trim() == expected_secret).unwrap_or(false)
        });

        if !valid_secret {
            let _ = write_http(stream, 401, "unauthorized");
            return;
        }
    }

    let parsed: Value = match serde_json::from_str(body.trim()) {
        Ok(value) => value,
        Err(error) => {
            let _ = app.emit(
                "runtime://stream",
                stream_event("stderr", &format!("telegram webhook JSON parse failed: {}", error)),
            );
            let _ = write_http(stream, 200, "ok");
            return;
        }
    };

    let _ = write_http(stream, 200, "ok");

    if let Some((chat_id, text)) = extract_chat_message(&parsed) {
        let token = current_settings.telegram_bot_token.clone();
        let _ = app.emit(
            "runtime://stream",
            stream_event("gateway", &format!("telegram webhook inbound chat={} {}", chat_id, text)),
        );
        handle_telegram_message(app, &current_settings, &token, &chat_id, &text);
    }
}

fn write_http(stream: &mut TcpStream, status: u16, body: &str) -> std::io::Result<()> {
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        401 => "Unauthorized",
        404 => "Not Found",
        405 => "Method Not Allowed",
        _ => "OK",
    };

    let response = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        status,
        reason,
        body.len(),
        body,
    );

    stream.write_all(response.as_bytes())
}

fn run_poll_loop(app: AppHandle, running: Arc<AtomicBool>) {
    let mut offset: i64 = 0;

    while running.load(Ordering::SeqCst) {
        let current_settings = app_settings::load_settings(&app);
        let token = current_settings.telegram_bot_token.trim().to_string();

        if token.is_empty() {
            thread::sleep(Duration::from_secs(3));
            continue;
        }

        match telegram_get_updates(&token, offset) {
            Ok(updates) => {
                for update in updates {
                    if let Some(update_id) = update.get("update_id").and_then(Value::as_i64) {
                        offset = update_id + 1;
                    }

                    if let Some((chat_id, text)) = extract_chat_message(&update) {
                        let _ = app.emit(
                            "runtime://stream",
                            stream_event("gateway", &format!("telegram polling inbound chat={} {}", chat_id, text)),
                        );
                        handle_telegram_message(&app, &app_settings::load_settings(&app), &token, &chat_id, &text);
                    }
                }
            }
            Err(error) => {
                let _ = app.emit(
                    "runtime://stream",
                    stream_event("stderr", &format!("telegram gateway poll failed: {}", redact_telegram_token(&error, &token))),
                );
                thread::sleep(Duration::from_secs(5));
            }
        }

        thread::sleep(Duration::from_millis(900));
    }
}

fn telegram_get_updates(token: &str, offset: i64) -> Result<Vec<Value>, String> {
    let url = format!(
        "https://api.telegram.org/bot{}/getUpdates?timeout=20&offset={}",
        token, offset
    );

    let output = Command::new("curl")
        .arg("-fsSL")
        .arg(url)
        .output()
        .map_err(|error| format!("failed to run curl: {}", error))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    let parsed: Value = serde_json::from_slice(&output.stdout).map_err(|error| error.to_string())?;
    if !parsed.get("ok").and_then(Value::as_bool).unwrap_or(false) {
        return Err(parsed.to_string());
    }

    Ok(parsed
        .get("result")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default())
}

fn extract_chat_message(update: &Value) -> Option<(String, String)> {
    let message = update.get("message")?;
    let chat_id = message.get("chat")?.get("id")?.to_string();
    let text = message.get("text")?.as_str()?.trim().to_string();
    if text.is_empty() {
        return None;
    }

    Some((chat_id, text))
}

fn handle_telegram_message(app: &AppHandle, settings: &AppSettings, token: &str, chat_id: &str, text: &str) {
    if !is_authorized_or_pair(app, settings, chat_id, token) {
        return;
    }

    if text == "/start" {
        let _ = telegram_send(token, chat_id, "Lil Buddy Telegram gateway is connected. Send a message to chat with the active expert. Use /help for commands.");
        return;
    }

    if text == "/help" {
        let _ = telegram_send(token, chat_id, "Commands:\n/start - pair/check connection\n/expert - list experts\n/expert default - use Default Lil Buddy\n/expert <number or name> - switch expert\n/workspace - show active workspace\n/help - show this help");
        return;
    }

    if text == "/expert" || text.starts_with("/expert ") {
        handle_expert_command(app, settings, token, chat_id, text);
        return;
    }

    if text == "/workspace" {
        let workspace = settings.telegram_active_workspace_path.clone()
            .or_else(|| settings.workspace_path.clone())
            .or_else(|| settings.default_directory.clone())
            .unwrap_or_else(|| "OS home directory fallback".to_string());
        let _ = telegram_send(token, chat_id, &format!("Active workspace: {}", workspace));
        return;
    }

    let response = run_expert_prompt(app, settings, text)
        .unwrap_or_else(|error| format!("Lil Buddy gateway error: {}", error));
    let _ = telegram_send_chunked(token, chat_id, &response);
}


fn handle_expert_command(app: &AppHandle, settings: &AppSettings, token: &str, chat_id: &str, text: &str) {
    let query = text.trim_start_matches("/expert").trim();

    if query.is_empty() {
        let active = settings
            .telegram_active_expert_name
            .clone()
            .unwrap_or_else(|| "Lil Buddy".to_string());
        let mut lines = vec![
            format!("Active expert: {}", active),
            String::new(),
            "Available experts:".to_string(),
            "0. Default Lil Buddy".to_string(),
        ];

        for (index, expert) in telegram_expert_list(settings).iter().enumerate() {
            let name = expert.get("name").and_then(Value::as_str).unwrap_or("Unnamed expert");
            let role = expert.get("role").and_then(Value::as_str).unwrap_or("Expert");
            lines.push(format!("{}. {} — {}", index + 1, name, role));
        }

        lines.push(String::new());
        lines.push("Use /expert <number or name> to switch.".to_string());
        let _ = telegram_send(token, chat_id, &lines.join("\\n"));
        return;
    }

    if matches!(query.to_ascii_lowercase().as_str(), "0" | "default" | "lil buddy" | "default lil buddy") {
        let mut next = settings.clone();
        next.telegram_active_expert_id = Some("default-lil-buddy".to_string());
        next.telegram_active_expert_name = Some("Lil Buddy".to_string());
        next.telegram_active_expert_role = Some("Default Lil Buddy".to_string());
        next.telegram_active_expert_prompt = Some("You are Lil Buddy, the user's default helpful assistant. Be clear, concise, practical, and warm.".to_string());
        next.telegram_active_workspace_path = None;
        let _ = app_settings::save_settings(app, &next);
        let _ = telegram_send(token, chat_id, "Switched to Default Lil Buddy.");
        return;
    }

    let experts = telegram_expert_list(settings);
    let selected = query
        .parse::<usize>()
        .ok()
        .and_then(|number| number.checked_sub(1))
        .and_then(|index| experts.get(index).cloned())
        .or_else(|| {
            let needle = query.to_ascii_lowercase();
            experts
                .iter()
                .find(|expert| {
                    expert.get("id").and_then(Value::as_str).map(|id| id.eq_ignore_ascii_case(query)).unwrap_or(false)
                        || expert.get("name").and_then(Value::as_str).map(|name| name.to_ascii_lowercase().contains(&needle)).unwrap_or(false)
                })
                .cloned()
        });

    let Some(expert) = selected else {
        let _ = telegram_send(token, chat_id, "Expert not found. Send /expert to see available experts.");
        return;
    };

    let mut next = settings.clone();
    next.telegram_active_expert_id = expert.get("id").and_then(Value::as_str).map(str::to_string);
    next.telegram_active_expert_name = expert.get("name").and_then(Value::as_str).map(str::to_string);
    next.telegram_active_expert_role = expert.get("role").and_then(Value::as_str).map(str::to_string);
    next.telegram_active_expert_prompt = expert.get("systemPrompt").and_then(Value::as_str).map(str::to_string);
    next.telegram_active_workspace_path = expert
        .get("workspacePath")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string);

    let name = next.telegram_active_expert_name.clone().unwrap_or_else(|| "selected expert".to_string());
    let _ = app_settings::save_settings(app, &next);
    let _ = telegram_send(token, chat_id, &format!("Switched to {}.", name));
}

fn telegram_expert_list(settings: &AppSettings) -> Vec<Value> {
    serde_json::from_str::<Vec<Value>>(&settings.telegram_experts_json).unwrap_or_default()
}


fn is_authorized_or_pair(app: &AppHandle, settings: &AppSettings, chat_id: &str, token: &str) -> bool {
    if let Some(allowed) = settings.telegram_allowed_chat_id.as_ref().filter(|value| !value.trim().is_empty()) {
        if allowed.trim() == chat_id {
            return true;
        }

        let _ = telegram_send(token, chat_id, "This Lil Buddy gateway is already paired with another Telegram chat.");
        return false;
    }

    let mut next = settings.clone();
    next.telegram_allowed_chat_id = Some(chat_id.to_string());
    let _ = app_settings::save_settings(app, &next);
    let _ = app.emit("runtime://stream", stream_event("gateway", &format!("telegram paired chat={}", chat_id)));
    true
}

fn run_expert_prompt(app: &AppHandle, settings: &AppSettings, text: &str) -> Result<String, String> {
    let provider_id = settings.selected_provider.clone();
    let name = settings.telegram_active_expert_name.clone().unwrap_or_else(|| "Lil Buddy".to_string());
    let role = settings.telegram_active_expert_role.clone().unwrap_or_else(|| "Default Lil Buddy".to_string());
    let prompt = settings.telegram_active_expert_prompt.clone().unwrap_or_else(|| {
        "You are Lil Buddy, the user's default helpful assistant. Be clear, concise, practical, and warm.".to_string()
    });

    let explicit_workspace = settings
        .telegram_active_workspace_path
        .clone()
        .filter(|value| !value.trim().is_empty());

    let workspace_note = explicit_workspace
        .clone()
        .or_else(|| settings.default_directory.clone())
        .unwrap_or_else(|| "No project workspace is selected. Treat this as a general chat unless the user asks to work in a folder.".to_string());

    let full_prompt = format!(
        "You are responding to the user from Telegram as the selected expert only.

Active expert:
Name: {name}
Role: {role}
Expert instructions:
{prompt}

Active workspace:
{workspace_note}

Rules:
- Do not mention the Telegram gateway, routing pipeline, webhook, provider CLI, or Lil Buddy app internals unless the user asks about them.
- Do not inspect or describe the Lil Buddy application's source code unless the active workspace is explicitly set to that project and the user asks for it.
- For greetings or casual messages, respond naturally and briefly.
- Use the active workspace only as task context; do not assume the user is working on the Lil Buddy app.
- Return the final user-facing answer only.

User message:
{text}"
    );

    let mut run_settings = settings.clone();
    run_settings.workspace_path = explicit_workspace.clone().or_else(|| settings.default_directory.clone());

    let context = AiCliContext::from_settings(full_prompt, &run_settings);
    let invocation = build_invocation(&provider_id, &context)?;

    let mut command = Command::new(&invocation.command);
    command.args(&invocation.args);
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    if invocation.stdin.is_some() {
        command.stdin(Stdio::piped());
    }

    if let Some(path) = invocation.workspace_path.clone() {
        command.current_dir(path);
    } else if let Some(fallback) = workspace::fallback_workspace_path(settings.default_directory.clone()) {
        command.current_dir(fallback);
    }

    let mut child = command.spawn().map_err(|error| error.to_string())?;

    if let Some(stdin_payload) = invocation.stdin.clone() {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(stdin_payload.as_bytes());
            let _ = stdin.write_all(b"
");
        }
    }

    let output = child.wait_with_output().map_err(|error| error.to_string())?;
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    let _ = app.emit("runtime://stream", stream_event("gateway", "telegram expert response generated"));

    if !stdout.is_empty() {
        Ok(format_provider_output_for_telegram(&stdout))
    } else if !stderr.is_empty() {
        Ok(format_provider_output_for_telegram(&stderr))
    } else {
        Ok("The expert finished with no visible output.".to_string())
    }
}

fn format_provider_output_for_telegram(raw: &str) -> String {
    let mut thinking_parts: Vec<String> = Vec::new();
    let mut message_parts: Vec<String> = Vec::new();
    let mut saw_json_line = false;

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let Ok(value) = serde_json::from_str::<Value>(trimmed) else {
            continue;
        };

        saw_json_line = true;

        let event_type = value.get("type").and_then(Value::as_str).unwrap_or_default();
        let part = value.get("part").unwrap_or(&Value::Null);
        let part_type = part.get("type").and_then(Value::as_str).unwrap_or(event_type);

        let text = part
            .get("text")
            .and_then(Value::as_str)
            .or_else(|| value.get("text").and_then(Value::as_str))
            .unwrap_or_default()
            .trim();

        if text.is_empty() {
            continue;
        }

        match part_type {
            "reasoning" | "thinking" => thinking_parts.push(text.to_string()),
            "text" | "message" | "assistant" => message_parts.push(text.to_string()),
            _ => {}
        }
    }

    if !saw_json_line {
        return raw.trim().to_string();
    }

    let message = dedupe_join(message_parts);
    if !message.is_empty() {
        return message;
    }

    let thinking = dedupe_join(thinking_parts);
    if !thinking.is_empty() {
        return format!("Thinking:\n{}", trim_for_telegram(&thinking, 900));
    }

    "The expert finished with no visible output.".to_string()
}

fn trim_for_telegram(value: &str, max_chars: usize) -> String {
    let trimmed = value.trim();
    if trimmed.chars().count() <= max_chars {
        return trimmed.to_string();
    }

    let mut output: String = trimmed.chars().take(max_chars).collect();
    output.push_str("...");
    output
}

fn dedupe_join(parts: Vec<String>) -> String {
    let mut out: Vec<String> = Vec::new();

    for part in parts {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }

        if out.last().map(|last| last.trim() == trimmed).unwrap_or(false) {
            continue;
        }

        out.push(trimmed.to_string());
    }

    out.join("\n")
}

fn telegram_send_chunked(token: &str, chat_id: &str, text: &str) -> Result<(), String> {
    let mut current = String::new();

    for line in text.lines() {
        if current.len() + line.len() + 1 > 3600 {
            telegram_send(token, chat_id, &current)?;
            current.clear();
        }
        current.push_str(line);
        current.push('\n');
    }

    if !current.trim().is_empty() {
        telegram_send(token, chat_id, &current)?;
    }

    Ok(())
}

fn telegram_send(token: &str, chat_id: &str, text: &str) -> Result<(), String> {
    let url = format!("https://api.telegram.org/bot{}/sendMessage", token);

    let output = Command::new("curl")
        .arg("-fsSL")
        .arg("-X")
        .arg("POST")
        .arg(url)
        .arg("-d")
        .arg(format!("chat_id={}", chat_id))
        .arg("--data-urlencode")
        .arg(format!("text={}", text))
        .output()
        .map_err(|error| format!("failed to run curl: {}", error))?;

    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}
