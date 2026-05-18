use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct CommandRisk {
    pub level: String,
    pub reason: String,
    pub command: String,
}

pub fn classify_command(command: &str) -> CommandRisk {
    let normalized = command.trim().to_lowercase();

    let dangerous_patterns = [
        "rm -rf /",
        "sudo rm",
        "mkfs",
        "dd if=",
        ":(){",
        "chmod -r 777",
        "curl ",
        "wget ",
        "| sh",
        "| bash",
        "git reset --hard",
        "git clean -fd",
    ];

    let approval_patterns = [
        "rm -rf",
        "rm ",
        "mv ",
        "chmod ",
        "chown ",
        "sudo ",
        "npm install",
        "pnpm install",
        "bun install",
        "cargo update",
        "git checkout",
        "git reset",
        "git clean",
        "docker system prune",
    ];

    for pattern in dangerous_patterns {
        if normalized.contains(pattern) {
            return CommandRisk {
                level: "block-or-confirm".to_string(),
                reason: format!("Dangerous command pattern detected: {}", pattern),
                command: command.to_string(),
            };
        }
    }

    for pattern in approval_patterns {
        if normalized.contains(pattern) {
            return CommandRisk {
                level: "confirm".to_string(),
                reason: format!("Command may modify files, dependencies, permissions, or git state: {}", pattern),
                command: command.to_string(),
            };
        }
    }

    CommandRisk {
        level: "allow".to_string(),
        reason: "No risky pattern detected.".to_string(),
        command: command.to_string(),
    }
}

pub fn maybe_extract_shell_command(line: &str) -> Option<String> {
    let trimmed = line.trim();

    if let Some(rest) = trimmed.strip_prefix("$ ") {
        return Some(rest.trim().to_string());
    }

    let lower = trimmed.to_lowercase();
    if lower.contains("running command") {
        if let Some((_, command)) = trimmed.split_once(':') {
            return Some(command.trim().to_string());
        }
    }

    if lower.contains("shell") && lower.contains("command") {
        if let Some((_, command)) = trimmed.split_once(':') {
            return Some(command.trim().to_string());
        }
    }

    None
}
