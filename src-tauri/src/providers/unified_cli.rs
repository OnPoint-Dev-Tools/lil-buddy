use std::{
    env,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Output, Stdio},
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
    pub session_id: Option<String>,
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
            session_id: None,
        }
    }
}

pub trait AiCliAdapter {
    fn build_invocation(&self, context: &AiCliContext) -> Result<ProviderInvocation, String>;
}

pub struct OpenCodeGoAdapter;
pub struct ClaudeAdapter;

impl AiCliAdapter for OpenCodeGoAdapter {
    fn build_invocation(&self, context: &AiCliContext) -> Result<ProviderInvocation, String> {
        opencode_go::build(
            &context.prompt,
            &context.opencode_mode,
            &context.opencode_command,
            &context.opencode_provider_key,
            context.workspace_path.clone(),
            context.selected_model.clone(),
            context.session_id.clone(),
        )
    }
}

impl AiCliAdapter for ClaudeAdapter {
    fn build_invocation(&self, context: &AiCliContext) -> Result<ProviderInvocation, String> {
        claude::build(
            &context.prompt,
            &context.claude_command,
            &context.claude_output_format,
            context.workspace_path.clone(),
            context.selected_model.clone(),
            context.session_id.clone(),
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

        let mut command = build_process_command(&resolved_command, &self.invocation.args);
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
        if let Some(resolved) = resolve_command_path(&self.invocation.command) {
            return Ok(resolved);
        }

        if let Some(fallback) = self.invocation.fallback_command.clone() {
            if let Some(resolved) = resolve_command_path(&fallback) {
                self.emit("provider", &format!("Using fallback command: {}", fallback));
                return Ok(resolved);
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

#[cfg(target_os = "windows")]
fn build_process_command(command: &str, args: &[String]) -> Command {
    let extension = Path::new(command)
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase());

    if matches!(extension.as_deref(), Some("cmd" | "bat")) {
        let mut process = Command::new("cmd.exe");
        process.arg("/C").arg(command);
        process.args(args);
        return process;
    }

    let mut process = Command::new(command);
    process.args(args);
    process
}

#[cfg(not(target_os = "windows"))]
fn build_process_command(command: &str, args: &[String]) -> Command {
    let mut process = Command::new(command);
    process.args(args);
    process
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
    resolve_command_path(command).is_some()
}

pub fn command_output(command: &str, args: &[String]) -> std::io::Result<Output> {
    let resolved = resolve_command_path(command).unwrap_or_else(|| command.trim().to_string());
    build_process_command(&resolved, args).output()
}

pub fn resolve_command_path(command: &str) -> Option<String> {
    let trimmed = command.trim();

    if trimmed.is_empty() {
        return None;
    }

    if let Some(path) = resolve_literal_command_path(trimmed) {
        return Some(path);
    }

    if let Some(path) = resolve_from_process_lookup(trimmed) {
        return Some(path);
    }

    resolve_from_common_bin_dirs(trimmed)
}

fn resolve_literal_command_path(command: &str) -> Option<String> {
    if command.contains(std::path::MAIN_SEPARATOR)
        || command.contains('/')
        || command.contains('\\')
        || Path::new(command).is_absolute()
        || cfg!(target_os = "windows") && command.contains(':')
    {
        return canonical_or_original_if_exists(Path::new(command));
    }

    None
}

#[cfg(target_os = "windows")]
fn resolve_from_process_lookup(command: &str) -> Option<String> {
    let output = Command::new("where.exe").arg(command).output().ok()?;
    if !output.status.success() {
        return None;
    }

    let mut matches = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| line.to_string())
        .collect::<Vec<_>>();

    matches.sort_by_key(|path| windows_command_priority(path));
    matches.into_iter().next()
}

#[cfg(not(target_os = "windows"))]
fn resolve_from_process_lookup(command: &str) -> Option<String> {
    Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {}", shell_escape_for_sh(command)))
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout
                .lines()
                .map(str::trim)
                .find(|line| !line.is_empty())
                .map(|line| line.to_string())
        })
}

fn resolve_from_common_bin_dirs(command: &str) -> Option<String> {
    common_bin_dirs()
        .into_iter()
        .find_map(|dir| resolve_in_dir(&dir, command))
}

fn resolve_in_dir(dir: &Path, command: &str) -> Option<String> {
    candidate_command_paths(dir, command)
        .into_iter()
        .find_map(|candidate| canonical_or_original_if_exists(&candidate))
}

fn canonical_or_original_if_exists(path: &Path) -> Option<String> {
    if !path.exists() {
        return None;
    }

    std::fs::canonicalize(path)
        .ok()
        .or_else(|| Some(path.to_path_buf()))
        .map(|value| value.to_string_lossy().to_string())
}

fn common_bin_dirs() -> Vec<PathBuf> {
    let mut dirs = env::var_os("PATH")
        .map(|value| env::split_paths(&value).collect::<Vec<_>>())
        .unwrap_or_default();

    dirs.extend(platform_fallback_bin_dirs());
    uniq_paths(dirs)
}

fn uniq_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut unique = Vec::new();

    for path in paths {
        if path.as_os_str().is_empty() {
            continue;
        }

        if !unique.iter().any(|existing| existing == &path) {
            unique.push(path);
        }
    }

    unique
}

#[cfg(target_os = "windows")]
fn candidate_command_paths(dir: &Path, command: &str) -> Vec<PathBuf> {
    let base = dir.join(command);
    let extension = Path::new(command)
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase());

    if extension.is_some() {
        return vec![base];
    }

    let pathext = env::var("PATHEXT")
        .unwrap_or(".COM;.EXE;.BAT;.CMD".to_string())
        .split(';')
        .filter_map(|value| {
            let trimmed = value.trim().trim_start_matches('.');
            (!trimmed.is_empty()).then(|| trimmed.to_ascii_lowercase())
        })
        .collect::<Vec<_>>();

    let mut candidates = vec![base.clone()];
    candidates.extend(pathext.into_iter().map(|ext| dir.join(format!("{}.{}", command, ext))));
    candidates
}

#[cfg(not(target_os = "windows"))]
fn candidate_command_paths(dir: &Path, command: &str) -> Vec<PathBuf> {
    vec![dir.join(command)]
}

#[cfg(target_os = "windows")]
fn platform_fallback_bin_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if let Some(user_profile) = env::var_os("USERPROFILE") {
        let home = PathBuf::from(&user_profile);
        dirs.push(home.join(".bun/bin"));
        dirs.push(home.join(".cargo/bin"));
        dirs.push(home.join("scoop/shims"));
    }

    if let Some(app_data) = env::var_os("APPDATA") {
        dirs.push(PathBuf::from(app_data).join("npm"));
    }

    if let Some(local_app_data) = env::var_os("LOCALAPPDATA") {
        let local = PathBuf::from(local_app_data);
        dirs.push(local.join("pnpm"));
        dirs.push(local.join("Volta/bin"));
        dirs.push(local.join("mise/shims"));
        dirs.push(local.join("Yarn/bin"));
    }

    if let Some(nvm_symlink) = env::var_os("NVM_SYMLINK") {
        dirs.push(PathBuf::from(nvm_symlink));
    }

    dirs
}

#[cfg(not(target_os = "windows"))]
fn platform_fallback_bin_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if let Some(home) = env::var_os("HOME") {
        let home = PathBuf::from(home);
        dirs.push(home.join(".cache/.bun/bin"));
        dirs.push(home.join(".bun/bin"));
        dirs.push(home.join(".local/bin"));
        dirs.push(home.join(".local/share/pnpm"));
        dirs.push(home.join(".volta/bin"));
        dirs.push(home.join(".asdf/shims"));
        dirs.push(home.join(".fnm/aliases/default/bin"));
        dirs.push(home.join(".yarn/bin"));
        dirs.push(home.join(".config/yarn/global/node_modules/.bin"));
        dirs.push(home.join(".npm-global/bin"));
        dirs.push(home.join(".cargo/bin"));
        dirs.push(home.join(".local/share/mise/shims"));
    }

    dirs.push(PathBuf::from("/opt/homebrew/bin"));
    dirs.push(PathBuf::from("/opt/homebrew/sbin"));
    dirs.push(PathBuf::from("/usr/local/bin"));
    dirs.push(PathBuf::from("/usr/local/sbin"));
    dirs.push(PathBuf::from("/home/linuxbrew/.linuxbrew/bin"));

    dirs
}

#[cfg(target_os = "windows")]
fn windows_command_priority(path: &str) -> u8 {
    let extension = Path::new(path)
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase());

    match extension.as_deref() {
        Some("cmd") => 0,
        Some("bat") => 1,
        Some("exe") => 2,
        _ => 3,
    }
}

pub fn command_version(command: &str) -> Option<String> {
    let resolved = resolve_command_path(command).unwrap_or_else(|| command.to_string());
    let candidates = [vec!["--version"], vec!["version"], vec!["-V"]];

    for args in candidates {
        let output = Command::new(&resolved).args(args).output().ok()?;
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
    let resolved = resolve_command_path(command).unwrap_or_else(|| command.to_string());

    if command_basename_matches(command, &["opencode", "opencode-go"]) {
        let output = Command::new(&resolved).args(["auth", "list"]).output();
        if let Ok(output) = output {
            return output.status.success();
        }
    }

    if command_basename_matches(command, &["claude"]) {
        let output = Command::new(&resolved).arg("--version").output();
        if let Ok(output) = output {
            return output.status.success();
        }
    }

    command_exists(command)
}

fn command_basename_matches(command: &str, expected: &[&str]) -> bool {
    let lowered = Path::new(command)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or(command)
        .to_ascii_lowercase();

    expected.iter().any(|candidate| {
        let candidate = candidate.to_ascii_lowercase();
        lowered == candidate
            || lowered == format!("{}.exe", candidate)
            || lowered == format!("{}.cmd", candidate)
            || lowered == format!("{}.bat", candidate)
    })
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

#[cfg(not(target_os = "windows"))]
pub fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace("'", "'\\''"))
}

#[cfg(not(target_os = "windows"))]
fn shell_escape_for_sh(value: &str) -> String {
    shell_quote(value)
}
