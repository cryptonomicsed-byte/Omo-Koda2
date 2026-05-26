//! Python hook runtime — Pattern 69.
//!
//! Runs Python scripts as hook handlers by spawning a `python3` subprocess.
//! Hook context is passed as JSON on stdin; the script writes a JSON outcome to stdout.
//!
//! Expected stdout format:
//!   `{"outcome": "allow"}`
//!   `{"outcome": "block", "reason": "..."}`
//!   `{"outcome": "warn",  "message": "..."}`
//!
//! This approach avoids a PyO3 C-extension build dependency while still letting
//! plugin authors write hooks in idiomatic Python.

use std::io::Write;
use std::path::PathBuf;

use super::hook_registry::{HookContext, HookHandler, HookMetadata, HookOutcome};

/// Executes a Python script as a hook handler.
pub struct PythonHookHandler {
    pub meta: HookMetadata,
    pub script_path: PathBuf,
    /// Python interpreter to use (default: `python3`)
    pub interpreter: String,
}

impl PythonHookHandler {
    pub fn new(meta: HookMetadata, script_path: PathBuf) -> Self {
        Self {
            meta,
            script_path,
            interpreter: "python3".to_string(),
        }
    }

    pub fn with_interpreter(mut self, interpreter: impl Into<String>) -> Self {
        self.interpreter = interpreter.into();
        self
    }

    fn parse_outcome(output: &str) -> HookOutcome {
        if output.is_empty() {
            return HookOutcome::Allow;
        }
        let Ok(val) = serde_json::from_str::<serde_json::Value>(output) else {
            return HookOutcome::Allow;
        };
        match val
            .get("outcome")
            .and_then(|v| v.as_str())
            .unwrap_or("allow")
        {
            "block" => HookOutcome::Block(
                val.get("reason")
                    .and_then(|v| v.as_str())
                    .unwrap_or("blocked by python hook")
                    .to_string(),
            ),
            "warn" => HookOutcome::Warn(
                val.get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("python hook warning")
                    .to_string(),
            ),
            _ => HookOutcome::Allow,
        }
    }
}

impl HookHandler for PythonHookHandler {
    fn handle(&self, ctx: &HookContext) -> HookOutcome {
        let ctx_json = serde_json::to_string(ctx).unwrap_or_default();

        let mut child = match std::process::Command::new(&self.interpreter)
            .arg(&self.script_path)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                return HookOutcome::Block(format!(
                    "Failed to spawn python hook ({}): {}",
                    self.interpreter, e
                ))
            }
        };

        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(ctx_json.as_bytes());
        }

        match child.wait_with_output() {
            Ok(out) if out.status.success() => {
                Self::parse_outcome(String::from_utf8_lossy(&out.stdout).trim())
            }
            Ok(out) => HookOutcome::Warn(format!(
                "Python hook exited non-zero: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            )),
            Err(e) => HookOutcome::Block(format!("Python hook wait failed: {}", e)),
        }
    }

    fn metadata(&self) -> &HookMetadata {
        &self.meta
    }
}

/// Convenience: build a `PythonHookHandler` from a hook config entry.
pub fn python_hook_from_script(
    meta: HookMetadata,
    script_path: impl Into<PathBuf>,
) -> PythonHookHandler {
    PythonHookHandler::new(meta, script_path.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution::hook_registry::{HookEventType, HookSource};

    fn dummy_meta() -> HookMetadata {
        HookMetadata {
            id: "py-test".to_string(),
            event: HookEventType::PreThink,
            source: HookSource::Python("/tmp/hook.py".into()),
            priority: 0,
            blocking: false,
            description: "test python hook".to_string(),
        }
    }

    #[test]
    fn test_parse_allow() {
        assert_eq!(
            PythonHookHandler::parse_outcome(r#"{"outcome":"allow"}"#),
            HookOutcome::Allow
        );
    }

    #[test]
    fn test_parse_block() {
        let out = PythonHookHandler::parse_outcome(r#"{"outcome":"block","reason":"denied"}"#);
        assert!(matches!(out, HookOutcome::Block(r) if r == "denied"));
    }

    #[test]
    fn test_parse_warn() {
        let out = PythonHookHandler::parse_outcome(r#"{"outcome":"warn","message":"risky"}"#);
        assert!(matches!(out, HookOutcome::Warn(m) if m == "risky"));
    }

    #[test]
    fn test_parse_empty_is_allow() {
        assert_eq!(PythonHookHandler::parse_outcome(""), HookOutcome::Allow);
    }

    #[test]
    fn test_parse_invalid_json_is_allow() {
        assert_eq!(
            PythonHookHandler::parse_outcome("not json"),
            HookOutcome::Allow
        );
    }

    #[test]
    fn test_missing_python_returns_block() {
        let meta = dummy_meta();
        let handler = PythonHookHandler::new(meta, PathBuf::from("/nonexistent/hook.py"))
            .with_interpreter("python3_definitely_not_here_xyzzy");
        let ctx = crate::execution::hook_registry::HookContext {
            event: HookEventType::PreThink,
            tool_name: None,
            params: None,
            session_id: None,
            timestamp: 0,
        };
        let outcome = handler.handle(&ctx);
        assert!(matches!(outcome, HookOutcome::Block(_)));
    }
}
