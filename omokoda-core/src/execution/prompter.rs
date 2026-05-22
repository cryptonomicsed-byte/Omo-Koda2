use crate::permissions::{PermissionPromptDecision, PermissionPrompter, PermissionRequest};
use std::io::{self, BufRead, Write};

/// Interactive permission prompter for tool escalation at the act gate.
/// Presents the tool name, input, and required permission mode via stdout/stdin
/// and waits for an allow/deny decision.
#[derive(Debug)]
pub struct TerminalPrompter {
    pub agent_name: String,
}

impl TerminalPrompter {
    pub fn new(agent_name: impl Into<String>) -> Self {
        Self {
            agent_name: agent_name.into(),
        }
    }
}

impl PermissionPrompter for TerminalPrompter {
    fn decide(&mut self, request: &PermissionRequest) -> PermissionPromptDecision {
        let prompt = format!(
            "\n[{}] Permission Required\n  Tool    : {}\n  Input   : {}\n  Current : {}\n  Required: {}\nAllow? [y/N] ",
            self.agent_name,
            request.tool_name,
            request.input,
            request.current_mode.as_str(),
            request.required_mode.as_str(),
        );

        let stdout = io::stdout();
        let mut out = stdout.lock();
        let _ = write!(out, "{}", prompt);
        let _ = out.flush();

        let stdin = io::stdin();
        let mut line = String::new();
        if stdin.lock().read_line(&mut line).is_err() {
            return PermissionPromptDecision::Deny {
                reason: "stdin read failed".to_string(),
            };
        }

        match line.trim().to_lowercase().as_str() {
            "y" | "yes" => PermissionPromptDecision::Allow,
            _ => PermissionPromptDecision::Deny {
                reason: "user denied".to_string(),
            },
        }
    }
}

/// Deterministic prompter for tests — always allows or always denies based on construction.
#[derive(Debug)]
pub struct AutoPrompter {
    allow: bool,
    pub calls: Vec<String>,
}

impl AutoPrompter {
    pub fn allow() -> Self {
        Self {
            allow: true,
            calls: Vec::new(),
        }
    }

    pub fn deny() -> Self {
        Self {
            allow: false,
            calls: Vec::new(),
        }
    }
}

impl PermissionPrompter for AutoPrompter {
    fn decide(&mut self, request: &PermissionRequest) -> PermissionPromptDecision {
        self.calls.push(request.tool_name.clone());
        if self.allow {
            PermissionPromptDecision::Allow
        } else {
            PermissionPromptDecision::Deny {
                reason: "auto-deny".to_string(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permissions::{PermissionMode, PermissionRequest};

    fn req(tool: &str) -> PermissionRequest {
        PermissionRequest {
            tool_name: tool.to_string(),
            input: r#"{"path":"/tmp/x"}"#.to_string(),
            current_mode: PermissionMode::ReadOnly,
            required_mode: PermissionMode::WorkspaceWrite,
        }
    }

    #[test]
    fn auto_prompter_allow_records_calls() {
        let mut p = AutoPrompter::allow();
        let r = p.decide(&req("bash"));
        assert_eq!(r, PermissionPromptDecision::Allow);
        assert_eq!(p.calls, vec!["bash"]);
    }

    #[test]
    fn auto_prompter_deny_returns_deny() {
        let mut p = AutoPrompter::deny();
        let r = p.decide(&req("write_file"));
        assert!(matches!(r, PermissionPromptDecision::Deny { .. }));
        assert_eq!(p.calls, vec!["write_file"]);
    }

    #[test]
    fn auto_prompter_multi_call_accumulates() {
        let mut p = AutoPrompter::allow();
        p.decide(&req("tool_a"));
        p.decide(&req("tool_b"));
        p.decide(&req("tool_c"));
        assert_eq!(p.calls.len(), 3);
        assert_eq!(p.calls[1], "tool_b");
    }

    #[test]
    fn terminal_prompter_constructs() {
        let p = TerminalPrompter::new("Luna");
        assert_eq!(p.agent_name, "Luna");
    }
}
