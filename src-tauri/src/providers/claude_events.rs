use serde_json::Value;

use super::parser_registry::NormalizedProviderEvent;

pub fn parse_line(line: &str, is_stderr: bool) -> Option<NormalizedProviderEvent> {
    let trimmed = line.trim();

    if trimmed.is_empty() {
        return None;
    }

    if is_stderr {
        return Some(event("stderr", trimmed.to_string()));
    }

    if trimmed.starts_with('{') {
        if let Ok(value) = serde_json::from_str::<Value>(trimmed) {
            return parse_json_event(&value);
        }

        return Some(event("stderr", format!("Could not parse Claude JSON stream event: {}", truncate(trimmed, 240))));
    }

    parse_plain_text(trimmed)
}

fn parse_json_event(value: &Value) -> Option<NormalizedProviderEvent> {
    let lower_type = string_at(value, &["type"]).unwrap_or_default().to_lowercase();
    let lower_subtype = string_at(value, &["subtype"]).unwrap_or_default().to_lowercase();

    // These are Claude Code protocol/session events, not chat content.
    if lower_type == "system"
        || lower_type == "user"
        || lower_type == "rate_limit_event"
        || lower_type == "message_start"
        || lower_type == "message_stop"
        || lower_type == "content_block_stop"
        || lower_subtype.starts_with("hook_")
        || string_at(value, &["hook_name"]).is_some()
    {
        return None;
    }

    if lower_type == "assistant" || lower_type == "result" || lower_type == "message" {
        if let Some(text) = collect_content_text(value, "text") {
            return Some(event("assistant", text));
        }

        if let Some(thinking) = collect_content_text(value, "thinking") {
            return Some(event("reasoning", thinking));
        }

        if let Some(tool) = collect_tool_use(value) {
            return Some(event("tool-call", tool));
        }

        if let Some(result) = first_string(value, &[&["result"], &["text"]]) {
            return Some(event("assistant", result));
        }

        return None;
    }

    if lower_type.contains("content_block_start") {
        let block_type = first_string(
            value,
            &[
                &["content_block", "type"],
                &["message", "content_block", "type"],
                &["block", "type"],
            ],
        )
        .unwrap_or_default()
        .to_lowercase();

        if block_type == "thinking" {
            if let Some(text) = first_string(
                value,
                &[
                    &["content_block", "thinking"],
                    &["content_block", "text"],
                    &["block", "thinking"],
                    &["block", "text"],
                ],
            ) {
                return Some(event("reasoning", text));
            }

            return Some(event("reasoning", "Claude started thinking."));
        }

        if block_type == "tool_use" {
            let tool_name = first_string(value, &[&["content_block", "name"], &["block", "name"]])
                .unwrap_or_else(|| "tool".to_string());
            return Some(event("tool-call", format!("tool={} · status=started", normalize_tool_name(&tool_name))));
        }

        return None;
    }

    if lower_type.contains("content_block_delta") {
        let delta_type = first_string(value, &[&["delta", "type"], &["message", "delta", "type"]])
            .unwrap_or_default()
            .to_lowercase();

        if delta_type == "text_delta" {
            if let Some(text) = first_string(value, &[&["delta", "text"], &["message", "delta", "text"]]) {
                return Some(event("assistant", text));
            }
        }

        if delta_type == "thinking_delta" {
            if let Some(text) = first_string(
                value,
                &[
                    &["delta", "thinking"],
                    &["message", "delta", "thinking"],
                    &["delta", "text"],
                    &["message", "delta", "text"],
                ],
            ) {
                return Some(event("reasoning", text));
            }
        }

        if delta_type == "input_json_delta" {
            if let Some(text) = first_string(
                value,
                &[
                    &["delta", "partial_json"],
                    &["message", "delta", "partial_json"],
                ],
            ) {
                return Some(event("tool-call", format!("tool=input · {}", truncate(&text, 360))));
            }
        }

        return None;
    }

    if lower_type.contains("tool_use") || lower_type.contains("tool") {
        if let Some(tool) = collect_tool_use(value) {
            return Some(event("tool-call", tool));
        }

        let tool_name = first_string(
            value,
            &[
                &["name"],
                &["tool", "name"],
                &["content_block", "name"],
                &["message", "content_block", "name"],
            ],
        )
        .unwrap_or_else(|| "tool".to_string());

        return Some(event("tool-call", format!("tool={} · status=running", normalize_tool_name(&tool_name))));
    }

    if lower_type.contains("thinking") || lower_type.contains("reasoning") {
        if let Some(text) = first_string(
            value,
            &[
                &["thinking"],
                &["reasoning"],
                &["delta", "thinking"],
                &["delta", "text"],
                &["text"],
            ],
        ) {
            return Some(event("reasoning", text));
        }
    }

    if lower_type == "error" {
        if let Some(text) = first_string(value, &[&["error", "message"], &["message"]]) {
            return Some(event("stderr", text));
        }
    }

    None
}

fn collect_content_text(value: &Value, wanted_type: &str) -> Option<String> {
    let arrays = [
        value.pointer("/message/content"),
        value.pointer("/content"),
        value.pointer("/response/content"),
    ];

    let mut parts = Vec::new();

    for array in arrays.into_iter().flatten() {
        let Some(items) = array.as_array() else {
            continue;
        };

        for item in items {
            let block_type = item.get("type").and_then(Value::as_str).unwrap_or("");

            if block_type == wanted_type {
                let text = match wanted_type {
                    "thinking" => item
                        .get("thinking")
                        .or_else(|| item.get("text"))
                        .and_then(Value::as_str),
                    _ => item.get("text").and_then(Value::as_str),
                };

                if let Some(text) = text {
                    if !text.trim().is_empty() {
                        parts.push(text.trim().to_string());
                    }
                }
            }
        }
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join("\n"))
    }
}

fn collect_tool_use(value: &Value) -> Option<String> {
    let arrays = [
        value.pointer("/message/content"),
        value.pointer("/content"),
        value.pointer("/response/content"),
    ];

    for array in arrays.into_iter().flatten() {
        let Some(items) = array.as_array() else {
            continue;
        };

        for item in items {
            let block_type = item.get("type").and_then(Value::as_str).unwrap_or("");

            if block_type == "tool_use" {
                let name = item.get("name").and_then(Value::as_str).unwrap_or("tool");
                let id = item.get("id").and_then(Value::as_str).unwrap_or("");
                let input = item.get("input");

                let file_path = input
                    .and_then(|input| {
                        input
                            .get("file_path")
                            .or_else(|| input.get("filePath"))
                            .or_else(|| input.get("path"))
                            .or_else(|| input.get("uri"))
                    })
                    .and_then(Value::as_str)
                    .unwrap_or("");

                let command = input
                    .and_then(|input| input.get("command"))
                    .and_then(Value::as_str)
                    .unwrap_or("");

                let pattern = input
                    .and_then(|input| input.get("pattern").or_else(|| input.get("query")))
                    .and_then(Value::as_str)
                    .unwrap_or("");

                let mut parts = vec![format!("tool={}", normalize_tool_name(name)), "status=started".to_string()];

                if !id.is_empty() {
                    parts.push(format!("id={}", id));
                }

                if !file_path.is_empty() {
                    parts.push(format!("filePath={}", file_path));
                    parts.push(format!("title=Read {}", short_path(file_path)));
                } else if !command.is_empty() {
                    parts.push(format!("command={}", command));
                    parts.push("title=Run shell command".to_string());
                } else if !pattern.is_empty() {
                    parts.push(format!("pattern={}", pattern));
                    parts.push("title=Search workspace".to_string());
                } else if let Some(input) = input {
                    let input = truncate(&input.to_string(), 360);
                    if !input.is_empty() && input != "null" {
                        parts.push(format!("input={}", input));
                    }
                }

                return Some(parts.join(" · "));
            }
        }
    }

    None
}

fn normalize_tool_name(name: &str) -> String {
    let lower = name.to_lowercase();

    if lower.contains("read") {
        "read".to_string()
    } else if lower.contains("write") || lower.contains("edit") {
        "edit".to_string()
    } else if lower.contains("grep") || lower.contains("search") {
        "search".to_string()
    } else if lower.contains("bash") || lower.contains("shell") {
        "bash".to_string()
    } else {
        name.to_string()
    }
}

fn short_path(path: &str) -> String {
    let parts = path.split('/').filter(|part| !part.is_empty()).collect::<Vec<_>>();

    if parts.len() <= 3 {
        return path.to_string();
    }

    format!("…/{}", parts[parts.len().saturating_sub(3)..].join("/"))
}

fn parse_plain_text(trimmed: &str) -> Option<NormalizedProviderEvent> {
    let lower = trimmed.to_lowercase();

    if lower.contains("not logged in")
        || lower.contains("login")
        || lower.contains("authentication")
        || lower.contains("oauth")
    {
        return Some(event("stderr", trimmed.to_string()));
    }

    if lower.contains("tool") || lower.contains("function") {
        return Some(event("tool-call", trimmed.to_string()));
    }

    if lower.starts_with("$ ") || lower.contains("bash") || lower.contains("shell") {
        return Some(event("shell-command", trimmed.to_string()));
    }

    if lower.contains("error") || lower.contains("failed") {
        return Some(event("stderr", trimmed.to_string()));
    }

    Some(event("assistant", trimmed.to_string()))
}

fn event(kind: &str, text: impl Into<String>) -> NormalizedProviderEvent {
    NormalizedProviderEvent {
        kind: kind.to_string(),
        text: clean_display(&text.into()),
    }
}

fn first_string(value: &Value, paths: &[&[&str]]) -> Option<String> {
    for path in paths {
        if let Some(text) = string_at(value, path) {
            if !text.trim().is_empty() {
                return Some(text);
            }
        }
    }

    None
}

fn string_at(value: &Value, path: &[&str]) -> Option<String> {
    let mut current = value;

    for segment in path {
        if let Ok(index) = segment.parse::<usize>() {
            current = current.as_array()?.get(index)?;
        } else {
            current = current.get(*segment)?;
        }
    }

    current.as_str().map(ToString::to_string)
}

fn clean_display(value: &str) -> String {
    strip_ansi(value).trim().to_string()
}

fn strip_ansi(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' && matches!(chars.peek(), Some('[')) {
            chars.next();

            for next in chars.by_ref() {
                if next.is_ascii_alphabetic() {
                    break;
                }
            }

            continue;
        }

        out.push(ch);
    }

    out
}

fn truncate(value: &str, limit: usize) -> String {
    if value.chars().count() <= limit {
        return value.to_string();
    }

    format!("{}…", value.chars().take(limit).collect::<String>())
}
