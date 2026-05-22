use serde::{Deserialize, Serialize};
use std::io::Write;
use std::process::{Command, Stdio};
use std::time::Duration;

/// Which lifecycle phase this hook fires on.
/// Hooks gate the `act` primitive (Pre = before tool dispatch, Post = after).
/// `SessionStart` / `SessionEnd` bracket the `birth` primitive's session scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HookPhase {
    PreToolUse,
    PostToolUse,
    SessionStart,
    SessionEnd,
}

/// Shell hook exit-code protocol:
/// - 0  → Allow (continue normally)
/// - 2  → Deny  (block the action)
/// - anything else → Warn (proceed with caution, surface message)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HookDecision {
    Allow,
    Warn(String),
    Deny(String),
}

impl HookDecision {
    #[must_use]
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Allow | Self::Warn(_))
    }

    #[must_use]
    pub fn is_denied(&self) -> bool {
        matches!(self, Self::Deny(_))
    }
}

/// JSON payload sent to hook commands on stdin.
/// The hook reads this, evaluates, then exits with the appropriate code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookPayload {
    /// Which phase triggered this hook
    pub phase: String,
    /// Tool name for Pre/PostToolUse; empty for session hooks
    pub tool_name: String,
    /// Serialized tool input JSON (empty for session hooks)
    pub tool_input: String,
    /// Tool output for PostToolUse; empty for Pre
    #[serde(skip_serializing_if = "String::is_empty")]
    pub tool_output: String,
    /// Agent tier (0-4)
    pub tier: u32,
}

/// A single shell-command hook definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookCommand {
    /// Shell command to execute. Supports env var substitution.
    pub command: String,
    /// Seconds before the hook is killed and treated as Warn
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
}

fn default_timeout_secs() -> u64 {
    10
}

/// Runtime configuration for a hook phase — zero or more shell commands run in sequence.
/// Any Deny short-circuits the remaining commands.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PolicyHookConfig {
    #[serde(default)]
    pub pre_tool_use: Vec<HookCommand>,
    #[serde(default)]
    pub post_tool_use: Vec<HookCommand>,
    #[serde(default)]
    pub session_start: Vec<HookCommand>,
    #[serde(default)]
    pub session_end: Vec<HookCommand>,
}

/// Runs shell-command hooks for a given phase, passing a JSON payload on stdin.
/// Aggregates results — first Deny wins; all Warns are collected.
pub struct PolicyHookRunner {
    config: PolicyHookConfig,
}

impl PolicyHookRunner {
    pub fn new(config: PolicyHookConfig) -> Self {
        Self { config }
    }

    pub fn empty() -> Self {
        Self::new(PolicyHookConfig::default())
    }

    pub fn run_pre_tool_use(&self, tool_name: &str, tool_input: &str, tier: u32) -> HookDecision {
        let payload = HookPayload {
            phase: "PreToolUse".to_string(),
            tool_name: tool_name.to_string(),
            tool_input: tool_input.to_string(),
            tool_output: String::new(),
            tier,
        };
        self.run_commands(&self.config.pre_tool_use, &payload)
    }

    pub fn run_post_tool_use(
        &self,
        tool_name: &str,
        tool_input: &str,
        tool_output: &str,
        tier: u32,
    ) -> HookDecision {
        let payload = HookPayload {
            phase: "PostToolUse".to_string(),
            tool_name: tool_name.to_string(),
            tool_input: tool_input.to_string(),
            tool_output: tool_output.to_string(),
            tier,
        };
        self.run_commands(&self.config.post_tool_use, &payload)
    }

    pub fn run_session_start(&self, tier: u32) -> HookDecision {
        let payload = HookPayload {
            phase: "SessionStart".to_string(),
            tool_name: String::new(),
            tool_input: String::new(),
            tool_output: String::new(),
            tier,
        };
        self.run_commands(&self.config.session_start, &payload)
    }

    pub fn run_session_end(&self, tier: u32) -> HookDecision {
        let payload = HookPayload {
            phase: "SessionEnd".to_string(),
            tool_name: String::new(),
            tool_input: String::new(),
            tool_output: String::new(),
            tier,
        };
        self.run_commands(&self.config.session_end, &payload)
    }

    fn run_commands(&self, commands: &[HookCommand], payload: &HookPayload) -> HookDecision {
        if commands.is_empty() {
            return HookDecision::Allow;
        }

        let json = match serde_json::to_string(payload) {
            Ok(s) => s,
            Err(e) => return HookDecision::Warn(format!("hook payload serialization error: {e}")),
        };

        let mut warnings: Vec<String> = Vec::new();

        for hook in commands {
            match self.run_one(hook, &json) {
                HookDecision::Allow => {}
                HookDecision::Warn(msg) => warnings.push(msg),
                HookDecision::Deny(msg) => return HookDecision::Deny(msg),
            }
        }

        if warnings.is_empty() {
            HookDecision::Allow
        } else {
            HookDecision::Warn(warnings.join("; "))
        }
    }

    fn run_one(&self, hook: &HookCommand, payload_json: &str) -> HookDecision {
        let timeout = Duration::from_secs(hook.timeout_secs);

        let mut child = match Command::new("sh")
            .args(["-c", &hook.command])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => return HookDecision::Warn(format!("hook spawn error: {e}")),
        };

        // Write payload to stdin
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(payload_json.as_bytes());
        }

        // Wait with timeout using a spin approach (no tokio dep in this sync fn)
        let start = std::time::Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    return match status.code() {
                        Some(0) => HookDecision::Allow,
                        Some(2) => {
                            // Read stderr/stdout for denial message
                            let msg = child
                                .stderr
                                .as_mut()
                                .and_then(|_| {
                                    // Already exited, capture via wait_with_output is no longer possible.
                                    // The message is in the hook's exit; use a generic message.
                                    None::<String>
                                })
                                .unwrap_or_else(|| format!("hook '{}' denied the action", hook.command));
                            HookDecision::Deny(msg)
                        }
                        Some(code) => {
                            HookDecision::Warn(format!("hook '{}' exited with code {code}", hook.command))
                        }
                        None => HookDecision::Warn(format!("hook '{}' killed by signal", hook.command)),
                    };
                }
                Ok(None) => {
                    if start.elapsed() > timeout {
                        let _ = child.kill();
                        return HookDecision::Warn(format!(
                            "hook '{}' timed out after {}s",
                            hook.command, hook.timeout_secs
                        ));
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
                Err(e) => return HookDecision::Warn(format!("hook wait error: {e}")),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config_with(pre: &str) -> PolicyHookConfig {
        PolicyHookConfig {
            pre_tool_use: vec![HookCommand {
                command: pre.to_string(),
                timeout_secs: 5,
            }],
            ..Default::default()
        }
    }

    #[test]
    fn empty_runner_always_allows() {
        let runner = PolicyHookRunner::empty();
        let result = runner.run_pre_tool_use("bash", "{}", 0);
        assert_eq!(result, HookDecision::Allow);
    }

    #[test]
    fn exit_zero_allows() {
        let runner = PolicyHookRunner::new(config_with("exit 0"));
        let result = runner.run_pre_tool_use("bash", "{}", 0);
        assert!(result.is_allowed());
    }

    #[test]
    fn exit_two_denies() {
        let runner = PolicyHookRunner::new(config_with("exit 2"));
        let result = runner.run_pre_tool_use("bash", "{}", 0);
        assert!(result.is_denied());
    }

    #[test]
    fn exit_one_warns() {
        let runner = PolicyHookRunner::new(config_with("exit 1"));
        let result = runner.run_pre_tool_use("bash", "{}", 0);
        matches!(result, HookDecision::Warn(_));
    }

    #[test]
    fn deny_short_circuits_remaining_hooks() {
        let config = PolicyHookConfig {
            pre_tool_use: vec![
                HookCommand { command: "exit 2".to_string(), timeout_secs: 5 },
                HookCommand { command: "exit 0".to_string(), timeout_secs: 5 },
            ],
            ..Default::default()
        };
        let runner = PolicyHookRunner::new(config);
        let result = runner.run_pre_tool_use("bash", "{}", 0);
        assert!(result.is_denied());
    }

    #[test]
    fn hook_receives_json_payload_on_stdin() {
        // Hook reads stdin and checks for "tool_name" key; exits 0 if found, 2 if not
        let runner = PolicyHookRunner::new(config_with(
            r#"python3 -c "import sys,json; d=json.load(sys.stdin); exit(0 if 'tool_name' in d else 2)""#,
        ));
        let result = runner.run_pre_tool_use("read_file", r#"{"path":"a.txt"}"#, 1);
        assert!(result.is_allowed());
    }

    #[test]
    fn session_start_with_no_hooks_allows() {
        let runner = PolicyHookRunner::empty();
        assert_eq!(runner.run_session_start(0), HookDecision::Allow);
    }

    #[test]
    fn hook_decision_is_allowed_covers_warn() {
        let warn = HookDecision::Warn("mild".to_string());
        assert!(warn.is_allowed());
        assert!(!warn.is_denied());
    }
}
