use std::{
    io::{BufRead, BufReader, Write},
    process::{Child, Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, OnceLock,
    },
    thread,
};

use tauri::{AppHandle, Emitter};

use crate::{
    native_companion_manager,
    runtime::{classify_runtime_line, stream_event},
    safety,
    settings::AppSettings,
    workspace,
};

use super::{claude, opencode_go, parser_registry, ProviderInvocation};

static ACTIVE_CHILD: OnceLock<Mutex<Option<Child>>> = OnceLock::new();

fn active_child() -> &'static Mutex<Option<Child>> {
    ACTIVE_CHILD.get_or_init(|| Mutex::new(None))
}

fn clear_active_child() {
    if let Ok(mut active) = active_child().lock() {
        *active = None;
    }
}

#[derive(Clone, Debug)]
pub enum AiCliProvider {
    OpenCodeGo,
    Claude,
}

impl AiCliProvider {
    pub fn from_id(provider_id: &str) -> Result<Self, String> {
        match provider_id {
            "opencode-go" => Ok(Self::OpenCodeGo),
            "claude" => Ok(Self::Claude),
            other => Err(format!("unsupported provider: {}", other)),
        }
    }

    pub fn id(&self) -> &'static str {
        match self {
            Self::OpenCodeGo => "opencode-go",
            Self::Claude => "claude",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::OpenCodeGo => "OpenCode",
            Self::Claude => "Claude Code",
        }
    }

    pub fn binary(&self, settings: &AppSettings) -> String {
        match self {
            Self::OpenCodeGo => settings.opencode_command.clone(),
            Self::Claude => "claude".to_string(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct AiCliContext {
    pub prompt: String,
    pub workspace_path: Option<String>,
    pub opencode_mode: String,
    pub opencode_command: String,
    pub opencode_provider_key: String,
    pub claude_command: String,
    pub claude_output_format: String,
    pub selected_model: Option<String>,
}

impl AiCliContext {
    pub fn from_settings(prompt: String, settings: &AppSettings) -> Self {
        Self {
            prompt,
            workspace_path: settings.workspace_path.clone(),
            opencode_mode: settings.opencode_go_mode.clone(),
            opencode_command: settings.opencode_command.clone(),
            opencode_provider_key: settings.opencode_provider_key.clone(),
            claude_command: settings.claude_command.clone(),
            claude_output_format: settings.claude_output_format.clone(),
            selected_model: settings.selected_model.clone(),
        }
    }
}

pub trait AiCliAdapter {
    fn provider(&self) -> AiCliProvider;
    fn build_invocation(&self, context: &AiCliContext) -> Result<ProviderInvocation, String>;
}

pub struct OpenCodeGoAdapter;
pub struct ClaudeAdapter;

impl AiCliAdapter for OpenCodeGoAdapter {
    fn provider(&self) -> AiCliProvider {
        AiCliProvider::OpenCodeGo
    }

    fn build_invocation(&self, context: &AiCliContext) -> Result<ProviderInvocation, String> {
        opencode_go::build(
            &context.prompt,
            &context.opencode_mode,
            &context.opencode_command,
            &context.opencode_provider_key,
            context.workspace_path.clone(),
            context.selected_model.clone(),
        )
    }
}

impl AiCliAdapter for ClaudeAdapter {
    fn provider(&self) -> AiCliProvider {
        AiCliProvider::Claude
    }

    fn build_invocation(&self, context: &AiCliContext) -> Result<ProviderInvocation, String> {
        claude::build(
            &context.prompt,
            &context.claude_command,
            &context.claude_output_format,
            context.workspace_path.clone(),
            context.selected_model.clone(),
        )
    }
}


pub fn build_invocation(
    provider_id: &str,
    context: &AiCliContext,
) -> Result<ProviderInvocation, String> {
    match AiCliProvider::from_id(provider_id)? {
        AiCliProvider::OpenCodeGo => OpenCodeGoAdapter.build_invocation(context),
        AiCliProvider::Claude => ClaudeAdapter.build_invocation(context),
    }
}

fn drive_native_companion_from_event(app: &AppHandle, kind: &str) {
    let category = match kind {
        "reasoning" | "timeline" => Some("thinking"),
        "tool-call" | "file-read" | "file-write" => Some("work"),
        "shell-command" | "command-approval" => Some("command"),
        "stderr" => Some("error"),
        "exit" => Some("done"),
        _ => None,
    };

    if let Some(category) = category {
        let _ = native_companion_manager::set_category(app, category);
    }
}

fn emit_runtime_event(app: &AppHandle, kind: &str, text: &str) {
    drive_native_companion_from_event(app, kind);
    app.emit("runtime://stream", stream_event(kind, text)).ok();
}

pub struct UnifiedCliRunner {
    app: AppHandle,
    provider_id: String,
    invocation: ProviderInvocation,
}

impl UnifiedCliRunner {
    pub fn new(app: AppHandle, provider_id: String, invocation: ProviderInvocation) -> Self {
        Self {
            app,
            provider_id,
            invocation,
        }
    }

    pub fn run(self) -> Result<(), String> {
        let before = workspace::get_workspace_diff();
        self.emit(
            "workspace",
            &format!("pre-run snapshot: {} changed file(s)", before.changed_files.len()),
        );

        self.emit(
            "status",
            &format!(
                "provider={} command={}",
                self.provider_id, self.invocation.command
            ),
        );

        self.emit("command-preview", &self.invocation.preview);
        let _ = native_companion_manager::set_category(&self.app, "work");

        let resolved_command = self.resolve_command()?;

        self.emit(
            "status",
            &format!("spawn {} via unified runner", resolved_command),
        );

        let mut command = Command::new(&resolved_command);
        command.args(&self.invocation.args);
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        if self.invocation.stdin.is_some() {
            command.stdin(Stdio::piped());
        }

        if let Some(workspace_path) = workspace_dir_from_invocation(&self.invocation) {
            command.current_dir(workspace_path);
        }

        let mut child = command.spawn().map_err(|error| error.to_string())?;

        if let Some(stdin_payload) = self.invocation.stdin.clone() {
            if let Some(mut stdin) = child.stdin.take() {
                thread::spawn(move || {
                    let _ = stdin.write_all(stdin_payload.as_bytes());
                    let _ = stdin.write_all(b"\n");
                });
            }
        }

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        if let Some(stdout) = stdout {
            spawn_stdout_reader(self.app.clone(), self.provider_id.clone(), stdout);
        }

        if let Some(stderr) = stderr {
            spawn_stderr_reader(self.app.clone(), self.provider_id.clone(), stderr);
        }

        {
            let mut active = active_child()
                .lock()
                .map_err(|_| "failed to lock active child".to_string())?;
            *active = Some(child);
        }

        let watcher_running = spawn_live_workspace_watch(self.app.clone());
        spawn_waiter(self.app.clone(), watcher_running);

        Ok(())
    }

    fn resolve_command(&self) -> Result<String, String> {
        if command_exists(&self.invocation.command) {
            return Ok(self.invocation.command.clone());
        }

        if let Some(fallback) = self.invocation.fallback_command.clone() {
            if command_exists(&fallback) {
                self.emit("provider", &format!("Using fallback command: {}", fallback));
                return Ok(fallback);
            }
        }

        self.emit(
            "stderr",
            &format!("Command not found: {}", self.invocation.command),
        );
        self.emit("exit", "provider missing");

        Err(format!("Command not found: {}", self.invocation.command))
    }

    fn emit(&self, kind: &str, text: &str) {
        emit_runtime_event(&self.app, kind, text);
    }
}

pub fn has_active_child() -> bool {
    active_child()
        .lock()
        .map(|active| active.is_some())
        .unwrap_or(false)
}

pub fn stop_active(app: AppHandle) -> Result<(), String> {
    let maybe_child = {
        let mut active = active_child()
            .lock()
            .map_err(|_| "failed to lock active child".to_string())?;

        active.take()
    };

    if let Some(mut child) = maybe_child {
        child.kill().map_err(|error| error.to_string())?;
        emit_runtime_event(&app, "exit", "process stopped");
    } else {
        emit_runtime_event(&app, "status", "no active process");
    }

    Ok(())
}

pub fn command_exists(command: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {}", shell_escape_for_sh(command)))
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

pub fn command_version(command: &str) -> Option<String> {
    let candidates = [vec!["--version"], vec!["version"], vec!["-V"]];

    for args in candidates {
        let output = Command::new(command).args(args).output().ok()?;
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let value = if !stdout.is_empty() { stdout } else { stderr };
            if !value.is_empty() {
                return Some(value.lines().next().unwrap_or("").to_string());
            }
        }
    }

    None
}

pub fn provider_auth_status(command: &str) -> bool {
    if command == "opencode" || command == "opencode-go" {
        let output = Command::new(command).args(["auth", "list"]).output();
        if let Ok(output) = output {
            return output.status.success();
        }
    }

    if command == "claude" || command.ends_with("/claude") {
        let output = Command::new(command).arg("--version").output();
        if let Ok(output) = output {
            return output.status.success();
        }
    }

    command_exists(command)
}

fn spawn_stdout_reader(app: AppHandle, provider_id: String, stdout: impl std::io::Read + Send + 'static) {
    thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines().map_while(Result::ok) {
            emit_normalized_line(&app, &provider_id, &line, false);
        }
    });
}

fn spawn_stderr_reader(app: AppHandle, provider_id: String, stderr: impl std::io::Read + Send + 'static) {
    thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines().map_while(Result::ok) {
            emit_normalized_line(&app, &provider_id, &line, true);
        }
    });
}

fn emit_normalized_line(app: &AppHandle, provider_id: &str, line: &str, force_stderr: bool) {
    let parser = parser_registry::parser_for_provider(provider_id);

    if let Some(parsed) = parser.parse_line(line, force_stderr) {
        scan_safety(app, &parsed.text);
        emit_runtime_event(app, &parsed.kind, &parsed.text);
        return;
    }

    if provider_id == "claude" && line.trim_start().starts_with('{') {
        // Claude Code stream-json contains system/hooks/rate-limit/user protocol objects.
        // The Claude parser returns None for ignored protocol events on purpose.
        // Never fall back to raw stdout for those lines.
        emit_runtime_event(app, "status", "Claude internal stream event ignored");
        return;
    }

    let (kind, text) = classify_runtime_line(line);
    scan_safety(app, line);

    let event_kind = if force_stderr && kind == "stdout" {
        "stderr"
    } else {
        kind
    };

    emit_runtime_event(app, event_kind, &text);
}

fn scan_safety(app: &AppHandle, text: &str) {
    if let Some(command) = safety::maybe_extract_shell_command(text) {
        let risk = safety::classify_command(&command);
        if risk.level != "allow" {
            app.emit(
                "runtime://stream",
                stream_event(
                    "command-approval",
                    &format!("{}: {}", risk.level, risk.reason),
                ),
            )
            .ok();
        }
    }
}

fn spawn_live_workspace_watch(app: AppHandle) -> Arc<AtomicBool> {
    let watcher_running = Arc::new(AtomicBool::new(true));
    let watcher_flag = watcher_running.clone();

    thread::spawn(move || {
        let mut last_status = String::new();

        while watcher_flag.load(Ordering::SeqCst) {
            let snapshot = workspace::get_workspace_diff();

            if snapshot.status_short != last_status {
                last_status = snapshot.status_short.clone();

                if snapshot.changed_files.is_empty() {
                    app.emit(
                        "runtime://stream",
                        stream_event("file-change", "Live watch: workspace clean"),
                    )
                    .ok();
                } else {
                    app.emit(
                        "runtime://stream",
                        stream_event(
                            "file-change",
                            &format!(
                                "Live watch: {} changed file(s): {}",
                                snapshot.changed_files.len(),
                                snapshot.changed_files.join(", ")
                            ),
                        ),
                    )
                    .ok();

                    if !snapshot.diff_stat.trim().is_empty() {
                        app.emit(
                            "runtime://stream",
                            stream_event("diff", &format!("Live diff stat:\n{}", snapshot.diff_stat)),
                        )
                        .ok();
                    }
                }
            }

            thread::sleep(std::time::Duration::from_millis(1200));
        }
    });

    watcher_running
}

fn spawn_waiter(app: AppHandle, watcher_running: Arc<AtomicBool>) {
    thread::spawn(move || {
        let maybe_child = {
            let mut active = match active_child().lock() {
                Ok(lock) => lock,
                Err(_) => {
                    emit_runtime_event(&app, "stderr", "failed to lock process state");
                    return;
                }
            };

            active.take()
        };

        if let Some(mut child) = maybe_child {
            match child.wait() {
                Ok(status) => {
                    watcher_running.store(false, Ordering::SeqCst);
                    emit_post_run_diff(&app);
                    emit_runtime_event(&app, "exit", &format!("process exited: {}", status));
                }
                Err(error) => {
                    watcher_running.store(false, Ordering::SeqCst);

                    if let Ok(mut active) = active_child().lock() {
                        *active = None;
                    }

                    emit_runtime_event(&app, "stderr", &format!("process wait failed: {}", error));
                }
            }
        }
    });
}

fn emit_post_run_diff(app: &AppHandle) {
    let after = workspace::get_workspace_diff();

    if after.changed_files.is_empty() {
        app.emit(
            "runtime://stream",
            stream_event("file-change", "No git-tracked file changes detected after run."),
        )
        .ok();
        return;
    }

    app.emit(
        "runtime://stream",
        stream_event(
            "file-change",
            &format!(
                "{} changed file(s): {}",
                after.changed_files.len(),
                after.changed_files.join(", ")
            ),
        ),
    )
    .ok();

    if !after.diff_stat.trim().is_empty() {
        app.emit(
            "runtime://stream",
            stream_event("diff", &format!("diff stat:\n{}", after.diff_stat)),
        )
        .ok();
    }
}

fn workspace_dir_from_invocation(invocation: &ProviderInvocation) -> Option<String> {
    if let Some(path) = invocation.workspace_path.clone() {
        if !path.trim().is_empty() {
            return Some(path);
        }
    }

    let args = &invocation.args;

    for index in 0..args.len() {
        if args[index] == "--dir" {
            return args.get(index + 1).cloned();
        }
    }

    workspace::default_workspace_path().map(|path| path.to_string_lossy().to_string())
}

pub fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace("'", "'\\''"))
}

pub fn preview_command(command: &str, args: &[String]) -> String {
    if args.is_empty() {
        return command.to_string();
    }

    format!(
        "{} {}",
        command,
        args.iter()
            .map(|arg| shell_quote_if_needed(arg))
            .collect::<Vec<_>>()
            .join(" ")
    )
}

fn shell_quote_if_needed(value: &str) -> String {
    if value.chars().any(|c| c.is_whitespace() || c == '\'' || c == '"' || c == '$') {
        shell_quote(value)
    } else {
        value.to_string()
    }
}

fn shell_escape_for_sh(value: &str) -> String {
    shell_quote(value)
}
