use serde::Serialize;
use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Clone, Debug, Serialize)]
pub struct WorkspaceInfo {
    pub path: Option<String>,
    pub is_git_repo: bool,
    pub git_root: Option<String>,
    pub branch: Option<String>,
    pub dirty: bool,
    pub status_summary: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct FileDiff {
    pub path: String,
    pub diff: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct WorkspaceDiff {
    pub git_root: Option<String>,
    pub status_short: String,
    pub diff_stat: String,
    pub changed_files: Vec<String>,
    pub diffs: Vec<FileDiff>,
}


pub fn fallback_workspace_path(configured: Option<String>) -> Option<PathBuf> {
    configured
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(default_workspace_path)
}

pub fn detect_workspace_at(path: Option<String>) -> WorkspaceInfo {
    // No explicit workspace should be cheap and safe.
    // We still show the OS home fallback path, but we do NOT run git detection/diff there.
    if path.as_ref().map(|value| value.trim().is_empty()).unwrap_or(true) {
        let cwd = fallback_workspace_path(None);
        return WorkspaceInfo {
            path: cwd.map(|path| path.to_string_lossy().to_string()),
            is_git_repo: false,
            git_root: None,
            branch: None,
            dirty: false,
            status_summary: "workspace not selected".to_string(),
        };
    }

    let cwd = workspace_path(path);

    let git_root = run_git_in(cwd.as_deref(), ["rev-parse", "--show-toplevel"]);
    let branch = run_git_in(cwd.as_deref(), ["branch", "--show-current"]);
    let status = run_git_in(cwd.as_deref(), ["status", "--short", "--untracked-files=no"]).unwrap_or_default();

    WorkspaceInfo {
        path: cwd.map(|path| path.to_string_lossy().to_string()),
        is_git_repo: git_root.is_some(),
        git_root,
        branch,
        dirty: !status.trim().is_empty(),
        status_summary: if status.trim().is_empty() {
            "clean".to_string()
        } else {
            status
        },
    }
}

pub fn detect_workspace_with_fallback(path: Option<String>, fallback: Option<String>) -> WorkspaceInfo {
    if path.as_ref().map(|value| value.trim().is_empty()).unwrap_or(true) {
        let cwd = fallback_workspace_path(fallback);
        return WorkspaceInfo {
            path: cwd.map(|path| path.to_string_lossy().to_string()),
            is_git_repo: false,
            git_root: None,
            branch: None,
            dirty: false,
            status_summary: "workspace not selected".to_string(),
        };
    }

    detect_workspace_at(path)
}

pub fn get_workspace_diff_at(path: Option<String>) -> WorkspaceDiff {
    // Never diff the OS home fallback. Diffs only run after the user explicitly selects a workspace.
    if path.as_ref().map(|value| value.trim().is_empty()).unwrap_or(true) {
        return WorkspaceDiff {
            git_root: None,
            status_short: "workspace not selected".to_string(),
            diff_stat: String::new(),
            changed_files: Vec::new(),
            diffs: Vec::new(),
        };
    }

    let cwd = workspace_path(path);

    let git_root = run_git_in(cwd.as_deref(), ["rev-parse", "--show-toplevel"]);
    let status_short = run_git_in(cwd.as_deref(), ["status", "--short", "--untracked-files=no"]).unwrap_or_default();
    let diff_stat = run_git_in(cwd.as_deref(), ["diff", "--stat"]).unwrap_or_default();

    let changed_files = parse_changed_files(&status_short);
    let mut diffs = Vec::new();

    for file in changed_files.iter().take(8) {
        if let Some(diff) = run_git_dynamic_in(cwd.as_deref(), &["diff", "--", file]) {
            diffs.push(FileDiff {
                path: file.clone(),
                diff: trim_large_diff(&diff),
            });
        }
    }

    WorkspaceDiff {
        git_root,
        status_short,
        diff_stat,
        changed_files,
        diffs,
    }
}

pub fn get_workspace_diff() -> WorkspaceDiff {
    get_workspace_diff_at(None)
}

pub fn restore_file_at(path: Option<String>, file: &str) -> Result<String, String> {
    let cwd = workspace_path(path);

    if file.trim().is_empty() {
        return Err("file path is empty".to_string());
    }

    let is_repo = run_git_in(cwd.as_deref(), ["rev-parse", "--is-inside-work-tree"])
        .map(|value| value == "true")
        .unwrap_or(false);

    if !is_repo {
        return Err("workspace is not inside a Git repo".to_string());
    }

    // Prefer modern git restore.
    let restore = Command::new("git")
        .args(["restore", "--", file])
        .current_dir_opt(cwd.as_deref())
        .output();

    match restore {
        Ok(output) if output.status.success() => {
            return Ok(format!("restored {}", file));
        }
        _ => {}
    }

    // Fallback for older Git versions.
    let checkout = Command::new("git")
        .args(["checkout", "--", file])
        .current_dir_opt(cwd.as_deref())
        .output()
        .map_err(|error| error.to_string())?;

    if checkout.status.success() {
        Ok(format!("restored {}", file))
    } else {
        let stderr = String::from_utf8_lossy(&checkout.stderr).trim().to_string();
        Err(if stderr.is_empty() {
            format!("failed to restore {}", file)
        } else {
            stderr
        })
    }
}

pub fn restore_all_at(path: Option<String>) -> Result<String, String> {
    let diff = get_workspace_diff_at(path.clone());

    if diff.changed_files.is_empty() {
        return Ok("no changed files to restore".to_string());
    }

    let mut restored = Vec::new();
    let mut failed = Vec::new();

    for file in diff.changed_files {
        match restore_file_at(path.clone(), &file) {
            Ok(_) => restored.push(file),
            Err(error) => failed.push(format!("{}: {}", file, error)),
        }
    }

    if failed.is_empty() {
        Ok(format!("restored {} file(s)", restored.len()))
    } else {
        Err(format!(
            "restored {} file(s), failed {}: {}",
            restored.len(),
            failed.len(),
            failed.join("; ")
        ))
    }
}

pub fn parse_changed_files(status_short: &str) -> Vec<String> {
    status_short
        .lines()
        .filter_map(|line| {
            if line.len() < 4 {
                return None;
            }

            let raw = line.get(3..).unwrap_or("").trim();

            if raw.contains(" -> ") {
                raw.split(" -> ").last().map(|s| s.trim().to_string())
            } else if raw.is_empty() {
                None
            } else {
                Some(raw.to_string())
            }
        })
        .collect()
}

pub fn default_workspace_path() -> Option<PathBuf> {
    env::var_os("HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("USERPROFILE").map(PathBuf::from))
        .or_else(|| {
            let drive = env::var_os("HOMEDRIVE")?;
            let path = env::var_os("HOMEPATH")?;
            let mut home = PathBuf::from(drive);
            home.push(path);
            Some(home)
        })
        .or_else(|| env::current_dir().ok())
}

fn workspace_path(path: Option<String>) -> Option<PathBuf> {
    if let Some(path) = path {
        if !path.trim().is_empty() {
            return Some(PathBuf::from(path));
        }
    }

    default_workspace_path()
}

fn trim_large_diff(diff: &str) -> String {
    const LIMIT: usize = 12000;

    if diff.len() <= LIMIT {
        return diff.to_string();
    }

    let mut trimmed = diff.chars().take(LIMIT).collect::<String>();
    trimmed.push_str("\n\n...diff trimmed by Lil Buddy...");
    trimmed
}

fn run_git_in<const N: usize>(cwd: Option<&Path>, args: [&str; N]) -> Option<String> {
    let mut command = Command::new("git");
    command.args(args);
    command.current_dir_opt(cwd);

    let output = command.output().ok()?;

    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

fn run_git_dynamic_in(cwd: Option<&Path>, args: &[&str]) -> Option<String> {
    let mut command = Command::new("git");
    command.args(args);
    command.current_dir_opt(cwd);

    let output = command.output().ok()?;

    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();

    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

trait CommandCurrentDirOpt {
    fn current_dir_opt(&mut self, cwd: Option<&Path>) -> &mut Self;
}

impl CommandCurrentDirOpt for Command {
    fn current_dir_opt(&mut self, cwd: Option<&Path>) -> &mut Self {
        if let Some(cwd) = cwd {
            self.current_dir(cwd);
        }

        self
    }
}
