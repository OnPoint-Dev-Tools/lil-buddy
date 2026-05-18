use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Serialize)]
pub struct RuntimeStreamEvent {
    pub kind: String,
    pub text: String,
    pub ts: String,
}

pub fn now_iso() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();

    millis.to_string()
}

pub fn stream_event(kind: &str, text: &str) -> RuntimeStreamEvent {
    RuntimeStreamEvent {
        kind: kind.to_string(),
        text: text.to_string(),
        ts: now_iso(),
    }
}

pub fn classify_runtime_line(line: &str) -> (&'static str, String) {
    let lower = line.to_lowercase();

    if lower.contains("tool") || lower.contains("function") {
        return ("tool-call", line.to_string());
    }

    if lower.contains("read") && (lower.contains("file") || lower.contains("path")) {
        return ("file-read", line.to_string());
    }

    if lower.contains("write") || lower.contains("modified") || lower.contains("created") {
        return ("file-write", line.to_string());
    }

    if lower.starts_with("diff ") || lower.contains("--- ") || lower.contains("+++ ") {
        return ("diff", line.to_string());
    }

    if lower.contains("bash") || lower.contains("shell") || lower.contains("command") || lower.starts_with("$ ") {
        return ("shell-command", line.to_string());
    }

    if lower.contains("error") || lower.contains("failed") || lower.contains("panic") {
        return ("stderr", line.to_string());
    }

    ("stdout", line.to_string())
}
