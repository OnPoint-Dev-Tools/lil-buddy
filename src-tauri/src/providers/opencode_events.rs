use serde_json::Value;

#[derive(Clone, Debug)]
pub struct ParsedOpenCodeEvent {
    pub kind: String,
    pub text: String,
}

pub fn parse_line(line: &str) -> Option<ParsedOpenCodeEvent> {
    let trimmed = line.trim();

    if !trimmed.starts_with('{') {
        return None;
    }

    let value: Value = serde_json::from_str(trimmed).ok()?;
    let event_type = string_field(&value, &["type"]).unwrap_or_default();
    let lower_type = event_type.to_lowercase();

    let part_type = nested_string(
        &value,
        &[
            &["part", "type"],
            &["properties", "part", "type"],
            &["message", "part", "type"],
            &["properties", "message", "part", "type"],
        ],
    )
    .unwrap_or_default()
    .to_lowercase();

    if !part_type.is_empty() {
        match part_type.as_str() {
            "step-start" | "step_start" => {
                return Some(ParsedOpenCodeEvent {
                    kind: "timeline".to_string(),
                    text: summarize_step_start(&value),
                });
            }
            "step-finish" | "step_finish" => {
                return Some(ParsedOpenCodeEvent {
                    kind: "session-stats".to_string(),
                    text: summarize_step_finish(&value),
                });
            }
            "text" | "assistant" => {
                return Some(ParsedOpenCodeEvent {
                    kind: "assistant".to_string(),
                    text: summarize_text(&value),
                });
            }
            "reasoning" | "thinking" | "reasoning-delta" | "thinking-delta" | "thought" | "summary" => {
                return Some(ParsedOpenCodeEvent {
                    kind: "reasoning".to_string(),
                    text: summarize_reasoning(&value),
                });
            }
            "tool" | "tool-use" | "tool_use" => {
                return Some(ParsedOpenCodeEvent {
                    kind: "tool-call".to_string(),
                    text: summarize_tool_use(&value),
                });
            }
            _ => {}
        }
    }

    // OpenCode JSON mode often uses event families such as:
    // message.part.updated + part.type=thinking/text/tool
    // message.part.added, message.updated, session.error, etc.
    if lower_type.contains("message.part") || lower_type.contains("part.updated") || lower_type.contains("part.added") {
        if lower_type.contains("reason") || lower_type.contains("think") {
            return Some(ParsedOpenCodeEvent {
                kind: "reasoning".to_string(),
                text: summarize_reasoning(&value),
            });
        }

        // If there is no part.type but the event is a part update, inspect fields.
        if has_any_reasoning_field(&value) {
            return Some(ParsedOpenCodeEvent {
                kind: "reasoning".to_string(),
                text: summarize_reasoning(&value),
            });
        }

        if has_any_text_field(&value) {
            return Some(ParsedOpenCodeEvent {
                kind: "assistant".to_string(),
                text: summarize_text(&value),
            });
        }

        if has_any_tool_field(&value) {
            return Some(ParsedOpenCodeEvent {
                kind: "tool-call".to_string(),
                text: summarize_tool_use(&value),
            });
        }
    }

    match lower_type.as_str() {
        "step_start" | "step-start" => Some(ParsedOpenCodeEvent {
            kind: "timeline".to_string(),
            text: summarize_step_start(&value),
        }),
        "text" | "assistant" => Some(ParsedOpenCodeEvent {
            kind: "assistant".to_string(),
            text: summarize_text(&value),
        }),
        "reasoning" | "thinking" | "reasoning_delta" | "thinking_delta" | "summary" | "progress" => {
            Some(ParsedOpenCodeEvent {
                kind: "reasoning".to_string(),
                text: summarize_reasoning(&value),
            })
        }
        "tool_use" | "tool-use" | "tool" => Some(ParsedOpenCodeEvent {
            kind: "tool-call".to_string(),
            text: summarize_tool_use(&value),
        }),
        "step_finish" | "step-finish" => Some(ParsedOpenCodeEvent {
            kind: "session-stats".to_string(),
            text: summarize_step_finish(&value),
        }),
        "error" | "session.error" => Some(ParsedOpenCodeEvent {
            kind: "stderr".to_string(),
            text: summarize_error(&value),
        }),
        _ => {
            if lower_type.contains("reason") || lower_type.contains("think") || lower_type.contains("summary") || lower_type.contains("progress") {
                return Some(ParsedOpenCodeEvent {
                    kind: "reasoning".to_string(),
                    text: summarize_reasoning(&value),
                });
            }

            if lower_type.contains("tool") {
                return Some(ParsedOpenCodeEvent {
                    kind: "tool-call".to_string(),
                    text: summarize_tool_use(&value),
                });
            }

            if lower_type.contains("command") || lower_type.contains("shell") || lower_type.contains("bash") {
                return Some(ParsedOpenCodeEvent {
                    kind: "shell-command".to_string(),
                    text: summarize_command(&value),
                });
            }

            if lower_type.contains("file") && (lower_type.contains("read") || lower_type.contains("open")) {
                return Some(ParsedOpenCodeEvent {
                    kind: "file-read".to_string(),
                    text: summarize_path(&value, "file read"),
                });
            }

            if lower_type.contains("file") && (lower_type.contains("write") || lower_type.contains("edit") || lower_type.contains("patch")) {
                return Some(ParsedOpenCodeEvent {
                    kind: "file-write".to_string(),
                    text: summarize_path(&value, "file write"),
                });
            }

            None
        }
    }
}


fn has_any_reasoning_field(value: &Value) -> bool {
    nested_string(
        value,
        &[
            &["part", "thinking"],
            &["part", "reasoning"],
            &["part", "thought"],
            &["part", "summary"],
            &["properties", "part", "thinking"],
            &["properties", "part", "reasoning"],
            &["properties", "part", "thought"],
            &["properties", "part", "summary"],
            &["thinking"],
            &["reasoning"],
            &["thought"],
            &["summary"],
        ],
    )
    .is_some()
}

fn has_any_text_field(value: &Value) -> bool {
    nested_string(
        value,
        &[
            &["part", "text"],
            &["part", "delta"],
            &["properties", "part", "text"],
            &["properties", "part", "delta"],
            &["text"],
            &["delta"],
            &["content"],
        ],
    )
    .is_some()
}

fn has_any_tool_field(value: &Value) -> bool {
    nested_string(
        value,
        &[
            &["part", "tool"],
            &["properties", "part", "tool"],
            &["tool"],
            &["name"],
            &["part", "state", "title"],
            &["properties", "part", "state", "title"],
        ],
    )
    .is_some()
}

fn summarize_step_start(value: &Value) -> String {
    let session = nested_string(value, &[&["sessionID"], &["part", "sessionID"]]).unwrap_or_default();
    let message = nested_string(value, &[&["part", "messageID"], &["messageID"]]).unwrap_or_default();

    if session.is_empty() && message.is_empty() {
        return "Started working.".to_string();
    }

    truncate(&format!("Started working. session={} · message={}", session, message), 240)
}

fn summarize_text(value: &Value) -> String {
    let text = nested_string(
        value,
        &[
            &["part", "text"],
            &["properties", "part", "text"],
            &["part", "delta"],
            &["properties", "part", "delta"],
            &["text"],
            &["delta"],
            &["message"],
            &["content"],
            &["summary"],
            &["progress"],
        ],
    )
    .unwrap_or_else(|| compact_json_or_content(value));

    strip_ansi(&truncate(&text, 12000))
}

fn summarize_reasoning(value: &Value) -> String {
    let text = nested_string(
        value,
        &[
            &["part", "text"],
            &["properties", "part", "text"],
            &["part", "delta"],
            &["properties", "part", "delta"],
            &["part", "content"],
            &["properties", "part", "content"],
            &["part", "thinking"],
            &["properties", "part", "thinking"],
            &["part", "reasoning"],
            &["properties", "part", "reasoning"],
            &["part", "thought"],
            &["properties", "part", "thought"],
            &["part", "summary"],
            &["properties", "part", "summary"],
            &["part", "reasoning"],
            &["properties", "part", "reasoning"],
            &["part", "thinking"],
            &["properties", "part", "thinking"],
            &["part", "thought"],
            &["properties", "part", "thought"],
            &["part", "summary"],
            &["properties", "part", "summary"],
            &["text"],
            &["delta"],
            &["message"],
            &["content"],
            &["summary"],
            &["progress"],
        ],
    )
    .unwrap_or_else(|| compact_json_or_content(value));

    format!("Thinking: {}", strip_ansi(&truncate(&text, 4000)))
}

fn summarize_tool_use(value: &Value) -> String {
    let tool = nested_string(
        value,
        &[
            &["part", "tool"],
            &["properties", "part", "tool"],
            &["tool"],
            &["name"],
        ],
    )
    .unwrap_or_else(|| "tool".to_string());

    let title = nested_string(
        value,
        &[
            &["part", "state", "title"],
            &["properties", "part", "state", "title"],
            &["state", "title"],
        ],
    )
    .unwrap_or_default();

    let status = nested_string(
        value,
        &[
            &["part", "state", "status"],
            &["properties", "part", "state", "status"],
            &["state", "status"],
        ],
    )
    .unwrap_or_default();

    let input_preview = nested_string(
        value,
        &[
            &["part", "state", "input", "filePath"],
            &["properties", "part", "state", "input", "filePath"],
            &["part", "state", "input", "pattern"],
            &["properties", "part", "state", "input", "pattern"],
            &["part", "state", "input", "query"],
            &["properties", "part", "state", "input", "query"],
        ],
    )
    .unwrap_or_default();

    let mut bits = vec![format!("tool={}", tool)];
    if !title.is_empty() { bits.push(format!("title={}", title)); }
    if !input_preview.is_empty() { bits.push(format!("target={}", input_preview)); }
    if !status.is_empty() { bits.push(format!("status={}", status)); }

    bits.join(" · ")
}

fn summarize_command(value: &Value) -> String {
    let command = nested_string(
        value,
        &[
            &["part", "command"],
            &["properties", "part", "command"],
            &["command"],
            &["part", "state", "input", "command"],
            &["properties", "part", "state", "input", "command"],
            &["message"],
        ],
    )
    .unwrap_or_else(|| compact_json_or_content(value));

    strip_ansi(&truncate(&command, 1200))
}

fn summarize_step_finish(value: &Value) -> String {
    let total = nested_u64(value, &[&["part", "tokens", "total"], &["properties", "part", "tokens", "total"], &["tokens", "total"]]);
    let input = nested_u64(value, &[&["part", "tokens", "input"], &["properties", "part", "tokens", "input"], &["tokens", "input"]]);
    let output = nested_u64(value, &[&["part", "tokens", "output"], &["properties", "part", "tokens", "output"], &["tokens", "output"]]);
    let reasoning = nested_u64(value, &[&["part", "tokens", "reasoning"], &["properties", "part", "tokens", "reasoning"], &["tokens", "reasoning"]]);
    let cost = nested_f64(value, &[&["part", "cost"], &["properties", "part", "cost"], &["cost"]]);

    let mut pieces = Vec::new();
    if let Some(total) = total { pieces.push(format!("tokens.total={}", total)); }
    if let Some(input) = input { pieces.push(format!("input={}", input)); }
    if let Some(output) = output { pieces.push(format!("output={}", output)); }
    if let Some(reasoning) = reasoning { pieces.push(format!("reasoning={}", reasoning)); }
    if let Some(cost) = cost { pieces.push(format!("cost=${:.6}", cost)); }
    pieces.join(" · ")
}

fn summarize_error(value: &Value) -> String {
    let raw = nested_string(
        value,
        &[
            &["error", "data", "message"],
            &["properties", "error", "data", "message"],
            &["error", "message"],
            &["properties", "error", "message"],
            &["message"],
        ],
    )
    .unwrap_or_else(|| compact_json_or_content(value));

    let cleaned = strip_ansi(&raw);
    if cleaned.to_lowercase().contains("permission requested") {
        return cleaned.replace("; auto-rejecting", ". Lil Buddy blocked it because the current workspace scope does not cover that path.");
    }

    cleaned
}

fn summarize_path(value: &Value, fallback: &str) -> String {
    let path = nested_string(
        value,
        &[
            &["path"],
            &["file"],
            &["filename"],
            &["tool", "input", "path"],
            &["input", "path"],
            &["input", "filePath"],
            &["part", "state", "input", "filePath"],
            &["properties", "part", "state", "input", "filePath"],
        ],
    )
    .unwrap_or_else(|| fallback.to_string());

    strip_ansi(&truncate(&path, 1400))
}

fn truncate(text: &str, limit: usize) -> String {
    let compact = text.trim();
    if compact.chars().count() <= limit {
        return compact.to_string();
    }

    let truncated = compact.chars().take(limit).collect::<String>();
    format!("{}…", truncated)
}

fn compact_json_or_content(value: &Value) -> String {
    strip_ansi(&truncate(&value.to_string(), 2400))
}

fn strip_ansi(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' {
            if matches!(chars.peek(), Some('[')) {
                chars.next();

                for next in chars.by_ref() {
                    if next.is_ascii_alphabetic() {
                        break;
                    }
                }

                continue;
            }
        }

        out.push(ch);
    }

    out
}

fn string_field(value: &Value, keys: &[&str]) -> Option<String> {
    for key in keys {
        if let Some(value) = value.get(*key).and_then(Value::as_str) {
            return Some(value.to_string());
        }
    }
    None
}

fn nested_string(value: &Value, paths: &[&[&str]]) -> Option<String> {
    for path in paths {
        if let Some(current) = nested_value(value, path) {
            if let Some(text) = current.as_str() {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    return Some(trimmed.to_string());
                }
            }
        }
    }
    None
}

fn nested_u64(value: &Value, paths: &[&[&str]]) -> Option<u64> {
    for path in paths {
        if let Some(current) = nested_value(value, path) {
            if let Some(number) = current.as_u64() {
                return Some(number);
            }
        }
    }
    None
}

fn nested_f64(value: &Value, paths: &[&[&str]]) -> Option<f64> {
    for path in paths {
        if let Some(current) = nested_value(value, path) {
            if let Some(number) = current.as_f64() {
                return Some(number);
            }
        }
    }
    None
}

fn nested_value<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut current = value;
    for part in path {
        current = current.get(*part)?;
    }
    Some(current)
}
