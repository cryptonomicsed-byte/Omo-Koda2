//! Async hook registry: multiple handlers per event type with priority ordering,
//! source tracking, and automatic registration from skill/Odu manifests.

use crate::plugins::manifest::HookConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// All event types the hook registry can handle
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HookEventType {
    // Tool lifecycle
    PreToolUse,
    PostToolUse,
    // Sampling/LLM lifecycle
    PreSampling,
    PostSampling,
    // Session lifecycle
    SessionStart,
    SessionEnd,
    // Think lifecycle (Pattern 67)
    PreThink,
    PostThink,
    OnThink,
    // Act lifecycle (Pattern 67)
    PreAct,
    PostAct,
    // System events (Pattern 67)
    OnReceipt,
    OnError,
    OnCompact,
    OnDream,
    OnSettle,
}

impl HookEventType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PreToolUse => "pre_tool_use",
            Self::PostToolUse => "post_tool_use",
            Self::PreSampling => "pre_sampling",
            Self::PostSampling => "post_sampling",
            Self::SessionStart => "session_start",
            Self::SessionEnd => "session_end",
            Self::PreThink => "pre_think",
            Self::PostThink => "post_think",
            Self::OnThink => "on_think",
            Self::PreAct => "pre_act",
            Self::PostAct => "post_act",
            Self::OnReceipt => "on_receipt",
            Self::OnError => "on_error",
            Self::OnCompact => "on_compact",
            Self::OnDream => "on_dream",
            Self::OnSettle => "on_settle",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pre_tool_use" => Some(Self::PreToolUse),
            "post_tool_use" => Some(Self::PostToolUse),
            "pre_sampling" => Some(Self::PreSampling),
            "post_sampling" => Some(Self::PostSampling),
            "session_start" => Some(Self::SessionStart),
            "session_end" => Some(Self::SessionEnd),
            "pre_think" => Some(Self::PreThink),
            "post_think" => Some(Self::PostThink),
            "on_think" => Some(Self::OnThink),
            "pre_act" => Some(Self::PreAct),
            "post_act" => Some(Self::PostAct),
            "on_receipt" => Some(Self::OnReceipt),
            "on_error" => Some(Self::OnError),
            "on_compact" => Some(Self::OnCompact),
            "on_dream" => Some(Self::OnDream),
            "on_settle" => Some(Self::OnSettle),
            _ => None,
        }
    }

    /// All standardized hook points in canonical order.
    pub fn all() -> &'static [HookEventType] {
        &[
            Self::PreThink,
            Self::PostThink,
            Self::OnThink,
            Self::PreAct,
            Self::PostAct,
            Self::PreToolUse,
            Self::PostToolUse,
            Self::PreSampling,
            Self::PostSampling,
            Self::SessionStart,
            Self::SessionEnd,
            Self::OnReceipt,
            Self::OnError,
            Self::OnCompact,
            Self::OnDream,
            Self::OnSettle,
        ]
    }
}

/// Where a hook came from
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HookSource {
    BuiltIn,
    Plugin(String),
    OduSkill(String),
    Wasm(PathBuf),
    /// Shell script hook — Pattern 68
    Shell(PathBuf),
    /// Python script hook — Pattern 69
    Python(PathBuf),
}

/// Metadata describing a registered hook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookMetadata {
    pub id: String,
    pub event: HookEventType,
    pub source: HookSource,
    /// Lower numbers run first
    pub priority: i32,
    /// If true a Block outcome stops further hook processing and returns an error
    pub blocking: bool,
    pub description: String,
}

/// Context passed to each hook on fire
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookContext {
    pub event: HookEventType,
    pub tool_name: Option<String>,
    pub params: Option<String>,
    pub session_id: Option<String>,
    pub timestamp: u64,
}

/// Result returned by a single hook handler
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HookOutcome {
    Allow,
    Block(String),
    Warn(String),
}

impl HookOutcome {
    pub fn is_blocking(&self) -> bool {
        matches!(self, Self::Block(_))
    }
}

/// Synchronous hook handler — async wrappers can be added at the call site
pub trait HookHandler: Send + Sync {
    fn handle(&self, ctx: &HookContext) -> HookOutcome;
    fn metadata(&self) -> &HookMetadata;
}

/// A hook that executes an external command (stub — real execution via sandbox)
pub struct CommandHook {
    pub meta: HookMetadata,
    pub command: String,
}

impl HookHandler for CommandHook {
    fn handle(&self, _ctx: &HookContext) -> HookOutcome {
        HookOutcome::Allow
    }
    fn metadata(&self) -> &HookMetadata {
        &self.meta
    }
}

/// Shell-script hook handler — Pattern 68.
/// Runs a `.sh` script; passes `HookContext` as JSON on stdin.
/// Script stdout must be JSON: `{"outcome":"allow"}` / `{"outcome":"block","reason":"..."}` / `{"outcome":"warn","message":"..."}`.
/// Empty stdout is treated as Allow. Non-zero exit without JSON is treated as Warn.
pub struct ShellHookHandler {
    pub meta: HookMetadata,
    pub script_path: PathBuf,
}

impl ShellHookHandler {
    pub fn new(meta: HookMetadata, script_path: PathBuf) -> Self {
        Self { meta, script_path }
    }

    fn parse_outcome(output: &str) -> HookOutcome {
        if output.is_empty() {
            return HookOutcome::Allow;
        }
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(output) {
            match val.get("outcome").and_then(|v| v.as_str()).unwrap_or("allow") {
                "block" => HookOutcome::Block(
                    val.get("reason")
                        .and_then(|v| v.as_str())
                        .unwrap_or("blocked by shell hook")
                        .to_string(),
                ),
                "warn" => HookOutcome::Warn(
                    val.get("message")
                        .and_then(|v| v.as_str())
                        .unwrap_or("shell hook warning")
                        .to_string(),
                ),
                _ => HookOutcome::Allow,
            }
        } else {
            HookOutcome::Allow
        }
    }
}

impl HookHandler for ShellHookHandler {
    fn handle(&self, ctx: &HookContext) -> HookOutcome {
        use std::io::Write;
        let ctx_json = serde_json::to_string(ctx).unwrap_or_default();

        let mut child = match std::process::Command::new("sh")
            .arg(&self.script_path)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => return HookOutcome::Block(format!("Failed to spawn shell hook: {}", e)),
        };

        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(ctx_json.as_bytes());
        }

        match child.wait_with_output() {
            Ok(out) if out.status.success() => {
                Self::parse_outcome(String::from_utf8_lossy(&out.stdout).trim())
            }
            Ok(out) => HookOutcome::Warn(format!(
                "Shell hook exited non-zero: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            )),
            Err(e) => HookOutcome::Block(format!("Shell hook wait failed: {}", e)),
        }
    }

    fn metadata(&self) -> &HookMetadata {
        &self.meta
    }
}

/// No-op passthrough — used for Odu skill hooks whose action happens in the skill runtime
pub struct PassthroughHook {
    pub meta: HookMetadata,
}

impl HookHandler for PassthroughHook {
    fn handle(&self, _ctx: &HookContext) -> HookOutcome {
        HookOutcome::Allow
    }
    fn metadata(&self) -> &HookMetadata {
        &self.meta
    }
}

// ── ShellHookRunner ───────────────────────────────────────────────────────────
// Shell hook execution: exit 0=Allow, exit 2=Deny, other=Warn
//   exit 0  → Allow (optional message from stdout)
//   exit 2  → Deny  (optional reason from stdout)
//   other   → Warn  (non-blocking, warning message)
// JSON payload is sent via stdin; env vars carry additional context.

/// Result returned by `ShellHookRunner`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HookRunResult {
    denied: bool,
    messages: Vec<String>,
}

impl HookRunResult {
    /// Construct an allow result with optional messages
    #[must_use]
    pub fn allow(messages: Vec<String>) -> Self {
        Self {
            denied: false,
            messages,
        }
    }

    /// Returns true if the hook denied the operation
    #[must_use]
    pub fn is_denied(&self) -> bool {
        self.denied
    }

    /// Returns the messages produced by hook commands
    #[must_use]
    pub fn messages(&self) -> &[String] {
        &self.messages
    }
}

/// Runs a list of shell commands using the sovereign hook protocol.
/// Commands receive a JSON payload on stdin and communicate via exit code + stdout.
pub struct ShellHookRunner;

impl ShellHookRunner {
    /// Run pre-tool-use hooks.
    /// `commands` — list of shell command strings to execute in order.
    #[must_use]
    pub fn run_pre_tool_use(
        commands: &[String],
        tool_name: &str,
        tool_input: &str,
    ) -> HookRunResult {
        Self::run_commands(
            commands,
            "PreToolUse",
            tool_name,
            tool_input,
            None,
            false,
        )
    }

    /// Run post-tool-use hooks.
    #[must_use]
    pub fn run_post_tool_use(
        commands: &[String],
        tool_name: &str,
        tool_input: &str,
        tool_output: &str,
        is_error: bool,
    ) -> HookRunResult {
        Self::run_commands(
            commands,
            "PostToolUse",
            tool_name,
            tool_input,
            Some(tool_output),
            is_error,
        )
    }

    fn run_commands(
        commands: &[String],
        event_name: &str,
        tool_name: &str,
        tool_input: &str,
        tool_output: Option<&str>,
        is_error: bool,
    ) -> HookRunResult {
        if commands.is_empty() {
            return HookRunResult::allow(Vec::new());
        }

        let payload = serde_json::json!({
            "hook_event_name": event_name,
            "tool_name": tool_name,
            "tool_input": Self::parse_tool_input(tool_input),
            "tool_input_json": tool_input,
            "tool_output": tool_output,
            "tool_result_is_error": is_error,
        })
        .to_string();

        let mut messages = Vec::new();

        for command in commands {
            match Self::run_single_command(
                command,
                event_name,
                tool_name,
                tool_input,
                tool_output,
                is_error,
                &payload,
            ) {
                ShellCommandOutcome::Allow { message } => {
                    if let Some(msg) = message {
                        messages.push(msg);
                    }
                }
                ShellCommandOutcome::Deny { message } => {
                    let msg = message.unwrap_or_else(|| {
                        format!("{event_name} hook denied tool `{tool_name}`")
                    });
                    messages.push(msg);
                    return HookRunResult {
                        denied: true,
                        messages,
                    };
                }
                ShellCommandOutcome::Warn { message } => messages.push(message),
            }
        }

        HookRunResult::allow(messages)
    }

    fn run_single_command(
        command: &str,
        event_name: &str,
        tool_name: &str,
        tool_input: &str,
        tool_output: Option<&str>,
        is_error: bool,
        payload: &str,
    ) -> ShellCommandOutcome {
        use std::io::Write;

        // Use `sh <path>` if the command is a path that exists; otherwise `sh -lc <cmd>`
        let mut child_cmd = if std::path::Path::new(command).exists() {
            let mut c = std::process::Command::new("sh");
            c.arg(command);
            c
        } else {
            let mut c = std::process::Command::new("sh");
            c.arg("-c").arg(command);
            c
        };

        child_cmd
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .env("HOOK_EVENT", event_name)
            .env("HOOK_TOOL_NAME", tool_name)
            .env("HOOK_TOOL_INPUT", tool_input)
            .env("HOOK_TOOL_IS_ERROR", if is_error { "1" } else { "0" });

        if let Some(output) = tool_output {
            child_cmd.env("HOOK_TOOL_OUTPUT", output);
        }

        let mut child = match child_cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                return ShellCommandOutcome::Warn {
                    message: format!(
                        "{event_name} hook `{command}` failed to start for `{tool_name}`: {e}"
                    ),
                }
            }
        };

        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(payload.as_bytes());
        }

        match child.wait_with_output() {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
                let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
                let message = (!stdout.is_empty()).then_some(stdout);
                match out.status.code() {
                    Some(0) => ShellCommandOutcome::Allow { message },
                    Some(2) => ShellCommandOutcome::Deny { message },
                    Some(code) => ShellCommandOutcome::Warn {
                        message: Self::format_warning(command, code, message.as_deref(), &stderr),
                    },
                    None => ShellCommandOutcome::Warn {
                        message: format!(
                            "{event_name} hook `{command}` terminated by signal while handling `{tool_name}`"
                        ),
                    },
                }
            }
            Err(e) => ShellCommandOutcome::Warn {
                message: format!(
                    "{event_name} hook `{command}` failed to start for `{tool_name}`: {e}"
                ),
            },
        }
    }

    fn parse_tool_input(tool_input: &str) -> serde_json::Value {
        serde_json::from_str(tool_input)
            .unwrap_or_else(|_| serde_json::json!({ "raw": tool_input }))
    }

    fn format_warning(command: &str, code: i32, stdout: Option<&str>, stderr: &str) -> String {
        let mut msg = format!(
            "Hook `{command}` exited with status {code}; allowing tool execution to continue"
        );
        if let Some(s) = stdout.filter(|s| !s.is_empty()) {
            msg.push_str(": ");
            msg.push_str(s);
        } else if !stderr.is_empty() {
            msg.push_str(": ");
            msg.push_str(stderr);
        }
        msg
    }
}

enum ShellCommandOutcome {
    Allow { message: Option<String> },
    Deny { message: Option<String> },
    Warn { message: String },
}

/// Async hook registry — maps event types to ordered handler lists
pub struct AsyncHookRegistry {
    handlers: HashMap<HookEventType, Vec<Box<dyn HookHandler>>>,
    registered: Vec<HookMetadata>,
}

impl Default for AsyncHookRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl AsyncHookRegistry {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            registered: Vec::new(),
        }
    }

    /// Register a handler; handlers are kept sorted by priority (ascending)
    pub fn register(&mut self, handler: Box<dyn HookHandler>) {
        let meta = handler.metadata().clone();
        let event = meta.event;
        self.registered.push(meta);
        let vec = self.handlers.entry(event).or_default();
        vec.push(handler);
        vec.sort_by_key(|h| h.metadata().priority);
    }

    /// Fire all handlers for the event, returning outcomes.
    /// Stops after the first *blocking* handler whose outcome is Block.
    pub fn fire(&self, ctx: &HookContext) -> Vec<HookOutcome> {
        let Some(handlers) = self.handlers.get(&ctx.event) else {
            return Vec::new();
        };
        let mut outcomes = Vec::new();
        for handler in handlers {
            let outcome = handler.handle(ctx);
            let stop = outcome.is_blocking() && handler.metadata().blocking;
            outcomes.push(outcome);
            if stop {
                break;
            }
        }
        outcomes
    }

    /// Fire and return Err if any blocking handler blocked
    pub fn fire_and_check(&self, ctx: &HookContext) -> Result<Vec<HookOutcome>, String> {
        let outcomes = self.fire(ctx);
        for outcome in &outcomes {
            if let HookOutcome::Block(reason) = outcome {
                return Err(reason.clone());
            }
        }
        Ok(outcomes)
    }

    pub fn handler_count(&self, event: HookEventType) -> usize {
        self.handlers.get(&event).map(|v| v.len()).unwrap_or(0)
    }

    pub fn registered_metadata(&self) -> &[HookMetadata] {
        &self.registered
    }

    pub fn hooks_for_event(&self, event: HookEventType) -> Vec<&HookMetadata> {
        let mut v: Vec<&HookMetadata> = self
            .registered
            .iter()
            .filter(|m| m.event == event)
            .collect();
        v.sort_by_key(|m| m.priority);
        v
    }
}

/// Auto-register hooks declared in a plugin manifest's hook_configs
pub fn register_skill_hooks(
    registry: &mut AsyncHookRegistry,
    plugin_name: &str,
    hook_configs: &[HookConfig],
) {
    for config in hook_configs {
        let Some(event) = HookEventType::from_str(&config.event) else {
            continue;
        };
        let id = format!("{}::{}", plugin_name, config.command);
        let meta = HookMetadata {
            id,
            event,
            source: HookSource::Plugin(plugin_name.to_string()),
            priority: 0,
            blocking: config.blocking,
            description: format!("Plugin hook: {}", config.command),
        };
        registry.register(Box::new(CommandHook {
            command: config.command.clone(),
            meta,
        }));
    }
}

/// Auto-register passthrough hooks for an Odu skill across a set of event types
pub fn register_odu_hooks(
    registry: &mut AsyncHookRegistry,
    skill_name: &str,
    events: &[HookEventType],
) {
    for &event in events {
        let id = format!("odu::{}::{}", skill_name, event.as_str());
        let meta = HookMetadata {
            id,
            event,
            source: HookSource::OduSkill(skill_name.to_string()),
            priority: 10,
            blocking: false,
            description: format!("Odu skill hook: {} on {}", skill_name, event.as_str()),
        };
        registry.register(Box::new(PassthroughHook { meta }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(event: HookEventType) -> HookContext {
        HookContext {
            event,
            tool_name: None,
            params: None,
            session_id: None,
            timestamp: 0,
        }
    }

    fn meta(event: HookEventType, priority: i32, blocking: bool) -> HookMetadata {
        HookMetadata {
            id: format!("test-{}-{}", event.as_str(), priority),
            event,
            source: HookSource::BuiltIn,
            priority,
            blocking,
            description: "test".to_string(),
        }
    }

    struct BlockingHook {
        meta: HookMetadata,
    }
    impl HookHandler for BlockingHook {
        fn handle(&self, _: &HookContext) -> HookOutcome {
            HookOutcome::Block("blocked by test".to_string())
        }
        fn metadata(&self) -> &HookMetadata {
            &self.meta
        }
    }

    #[test]
    fn test_empty_registry_returns_no_outcomes() {
        let r = AsyncHookRegistry::new();
        assert!(r.fire(&ctx(HookEventType::PreToolUse)).is_empty());
    }

    #[test]
    fn test_register_and_fire_allow() {
        let mut r = AsyncHookRegistry::new();
        r.register(Box::new(PassthroughHook {
            meta: meta(HookEventType::PreToolUse, 0, false),
        }));
        let outcomes = r.fire(&ctx(HookEventType::PreToolUse));
        assert_eq!(outcomes, vec![HookOutcome::Allow]);
    }

    #[test]
    fn test_blocking_hook_stops_chain() {
        let mut r = AsyncHookRegistry::new();
        r.register(Box::new(BlockingHook {
            meta: meta(HookEventType::PreToolUse, 0, true),
        }));
        r.register(Box::new(PassthroughHook {
            meta: meta(HookEventType::PreToolUse, 10, false),
        }));
        let outcomes = r.fire(&ctx(HookEventType::PreToolUse));
        assert_eq!(outcomes.len(), 1);
        assert!(matches!(outcomes[0], HookOutcome::Block(_)));
    }

    #[test]
    fn test_non_blocking_block_does_not_stop_chain() {
        let mut r = AsyncHookRegistry::new();
        r.register(Box::new(BlockingHook {
            meta: meta(HookEventType::OnThink, 0, false), // blocking=false
        }));
        r.register(Box::new(PassthroughHook {
            meta: meta(HookEventType::OnThink, 10, false),
        }));
        let outcomes = r.fire(&ctx(HookEventType::OnThink));
        assert_eq!(outcomes.len(), 2);
    }

    #[test]
    fn test_fire_and_check_returns_err_on_block() {
        let mut r = AsyncHookRegistry::new();
        r.register(Box::new(BlockingHook {
            meta: meta(HookEventType::PostToolUse, 0, true),
        }));
        assert!(r.fire_and_check(&ctx(HookEventType::PostToolUse)).is_err());
    }

    #[test]
    fn test_priority_ordering() {
        let mut r = AsyncHookRegistry::new();
        r.register(Box::new(PassthroughHook {
            meta: meta(HookEventType::OnThink, 5, false),
        }));
        r.register(Box::new(PassthroughHook {
            meta: meta(HookEventType::OnThink, 1, false),
        }));
        r.register(Box::new(PassthroughHook {
            meta: meta(HookEventType::OnThink, 3, false),
        }));
        let hooks = r.hooks_for_event(HookEventType::OnThink);
        assert_eq!(hooks[0].priority, 1);
        assert_eq!(hooks[1].priority, 3);
        assert_eq!(hooks[2].priority, 5);
    }

    #[test]
    fn test_register_skill_hooks() {
        let mut r = AsyncHookRegistry::new();
        let configs = vec![
            HookConfig {
                event: "pre_tool_use".to_string(),
                command: "check.sh".to_string(),
                blocking: true,
            },
            HookConfig {
                event: "unknown_event".to_string(),
                command: "skip.sh".to_string(),
                blocking: false,
            },
        ];
        register_skill_hooks(&mut r, "my-plugin", &configs);
        assert_eq!(r.handler_count(HookEventType::PreToolUse), 1);
        assert_eq!(r.handler_count(HookEventType::PostToolUse), 0);
    }

    #[test]
    fn test_register_odu_hooks() {
        let mut r = AsyncHookRegistry::new();
        register_odu_hooks(
            &mut r,
            "odu-oracle",
            &[HookEventType::OnReceipt, HookEventType::OnSettle],
        );
        assert_eq!(r.handler_count(HookEventType::OnReceipt), 1);
        assert_eq!(r.handler_count(HookEventType::OnSettle), 1);
    }

    #[test]
    fn test_event_type_round_trip() {
        for &event in HookEventType::all() {
            assert_eq!(
                HookEventType::from_str(event.as_str()),
                Some(event),
                "round-trip failed for {:?}",
                event
            );
        }
    }

    #[test]
    fn test_all_returns_all_variants() {
        // all() must cover every variant — update this count when adding variants
        assert_eq!(HookEventType::all().len(), 16);
    }

    #[test]
    fn test_new_hook_points_exist() {
        for name in ["pre_think", "post_think", "pre_act", "post_act", "on_error", "on_compact", "on_dream"] {
            assert!(HookEventType::from_str(name).is_some(), "{} not found", name);
        }
    }

    #[test]
    fn test_shell_hook_parse_allow() {
        let out = ShellHookHandler::parse_outcome(r#"{"outcome":"allow"}"#);
        assert_eq!(out, HookOutcome::Allow);
    }

    #[test]
    fn test_shell_hook_parse_block() {
        let out = ShellHookHandler::parse_outcome(r#"{"outcome":"block","reason":"not allowed"}"#);
        assert!(matches!(out, HookOutcome::Block(r) if r == "not allowed"));
    }

    #[test]
    fn test_shell_hook_parse_warn() {
        let out = ShellHookHandler::parse_outcome(r#"{"outcome":"warn","message":"check this"}"#);
        assert!(matches!(out, HookOutcome::Warn(m) if m == "check this"));
    }

    #[test]
    fn test_shell_hook_parse_empty() {
        assert_eq!(ShellHookHandler::parse_outcome(""), HookOutcome::Allow);
    }

    // ── ShellHookRunner tests ─────────────────────────────────────────────────

    #[test]
    fn shell_runner_allows_exit_zero_captures_stdout() {
        let cmds = vec!["printf 'hook ok'".to_string()];
        let result = ShellHookRunner::run_pre_tool_use(
            &cmds,
            "Read",
            r#"{"path":"README.md"}"#,
        );
        assert!(!result.is_denied());
        // Login shells may emit profile noise (e.g. nvm); check that the payload is present
        assert!(
            result.messages().iter().any(|m| m.contains("hook ok")),
            "expected stdout to contain 'hook ok', got: {:?}",
            result.messages()
        );
    }

    #[test]
    fn shell_runner_denies_exit_two() {
        let cmds = vec!["printf 'blocked'; exit 2".to_string()];
        let result = ShellHookRunner::run_pre_tool_use(
            &cmds,
            "Bash",
            r#"{"command":"rm -rf /"}"#,
        );
        assert!(result.is_denied());
        // Login shells may prepend profile noise; assert the denial message is present
        assert!(
            result.messages().iter().any(|m| m.contains("blocked")),
            "expected denial message to contain 'blocked', got: {:?}",
            result.messages()
        );
    }

    #[test]
    fn shell_runner_warns_for_other_non_zero() {
        let cmds = vec!["printf 'warn msg'; exit 1".to_string()];
        let result = ShellHookRunner::run_pre_tool_use(
            &cmds,
            "Edit",
            r#"{"file":"src/lib.rs"}"#,
        );
        assert!(!result.is_denied());
        assert!(
            result.messages().iter().any(|m| m.contains("allowing tool execution to continue")),
            "expected warning message, got: {:?}",
            result.messages()
        );
    }
}
