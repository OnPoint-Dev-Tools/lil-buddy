use super::{claude_events, opencode_events};

#[derive(Clone, Debug)]
pub struct NormalizedProviderEvent {
    pub kind: String,
    pub text: String,
}

pub trait ProviderOutputParser {
    fn parse_line(&self, line: &str, is_stderr: bool) -> Option<NormalizedProviderEvent>;
}

pub struct OpenCodeParser;
pub struct ClaudeParser;
pub struct FallbackParser;

impl ProviderOutputParser for OpenCodeParser {
    fn parse_line(&self, line: &str, is_stderr: bool) -> Option<NormalizedProviderEvent> {
        if let Some(parsed) = opencode_events::parse_line(line) {
            return Some(NormalizedProviderEvent {
                kind: parsed.kind,
                text: parsed.text,
            });
        }

        if is_stderr {
            return Some(NormalizedProviderEvent {
                kind: "stderr".to_string(),
                text: line.to_string(),
            });
        }

        None
    }
}

impl ProviderOutputParser for ClaudeParser {
    fn parse_line(&self, line: &str, is_stderr: bool) -> Option<NormalizedProviderEvent> {
        claude_events::parse_line(line, is_stderr)
    }
}

impl ProviderOutputParser for FallbackParser {
    fn parse_line(&self, line: &str, is_stderr: bool) -> Option<NormalizedProviderEvent> {
        if line.trim().is_empty() {
            return None;
        }

        Some(NormalizedProviderEvent {
            kind: if is_stderr { "stderr" } else { "stdout" }.to_string(),
            text: line.to_string(),
        })
    }
}

pub enum ParserKind {
    OpenCode(OpenCodeParser),
    Claude(ClaudeParser),
    Fallback(FallbackParser),
}

impl ParserKind {
    pub fn parse_line(&self, line: &str, is_stderr: bool) -> Option<NormalizedProviderEvent> {
        match self {
            Self::OpenCode(parser) => parser.parse_line(line, is_stderr),
            Self::Claude(parser) => parser.parse_line(line, is_stderr),
            Self::Fallback(parser) => parser.parse_line(line, is_stderr),
        }
    }
}

pub fn parser_for_provider(provider_id: &str) -> ParserKind {
    match provider_id {
        "opencode-go" => ParserKind::OpenCode(OpenCodeParser),
        "claude" => ParserKind::Claude(ClaudeParser),
        _ => ParserKind::Fallback(FallbackParser),
    }
}
