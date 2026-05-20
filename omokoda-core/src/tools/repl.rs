use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

use crate::tools::{ExecutionContext, Tool};
use crate::usage::TokenUsage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplRequest {
    pub language: String,
    pub code: String,
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
}

fn default_timeout() -> u64 {
    10
}

pub struct ReplTool;

#[async_trait]
impl Tool for ReplTool {
    fn name(&self) -> &str {
        "repl"
    }
    fn description(&self) -> &str {
        "Execute code in a sandboxed REPL. Params: JSON {language, code, timeout_secs?}. \
         Languages: python, node, bash"
    }
    fn required_tier(&self) -> u8 {
        2
    }
    fn is_write_operation(&self) -> bool {
        true
    }

    fn params_schema(&self) -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "type": "object",
            "properties": {
                "language": {
                    "type": "string",
                    "enum": ["python", "node", "bash"],
                    "description": "Programming language to execute"
                },
                "code": { "type": "string", "description": "Code to execute" },
                "timeout_secs": { "type": "number", "description": "Timeout in seconds" }
            },
            "required": ["language", "code"]
        }))
    }

    async fn execute(
        &self,
        params: &str,
        context: &ExecutionContext,
    ) -> Result<(String, TokenUsage), String> {
        let req: ReplRequest =
            serde_json::from_str(params).map_err(|e| format!("repl requires JSON: {}", e))?;

        let timeout_secs = req.timeout_secs.clamp(1, 60);

        let (program, args): (&str, Vec<&str>) = match req.language.as_str() {
            "python" => ("python3", vec!["-c", &req.code]),
            "node" => ("node", vec!["-e", &req.code]),
            "bash" => ("bash", vec!["-c", &req.code]),
            other => {
                return Err(format!(
                    "Unsupported REPL language: '{}'. Use: python, node, bash",
                    other
                ))
            }
        };

        let mut cmd = Command::new(program);
        cmd.args(&args)
            .current_dir(&context.workspace_root)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .env_clear()
            .env("PATH", std::env::var("PATH").unwrap_or_default())
            .env("HOME", std::env::var("HOME").unwrap_or_default());

        let result = timeout(Duration::from_secs(timeout_secs), cmd.output())
            .await
            .map_err(|_| format!("REPL timed out after {}s", timeout_secs))?
            .map_err(|e| format!("Failed to spawn {}: {}", program, e))?;

        let stdout = String::from_utf8_lossy(&result.stdout);
        let stderr = String::from_utf8_lossy(&result.stderr);

        let max_chars = 4000;
        let output = if result.status.success() {
            let out = stdout.trim();
            if out.len() > max_chars {
                format!("{}... [truncated]", &out[..max_chars])
            } else {
                out.to_string()
            }
        } else {
            let err = stderr.trim();
            return Err(format!(
                "REPL exited with {}: {}",
                result.status,
                if err.len() > max_chars {
                    &err[..max_chars]
                } else {
                    err
                }
            ));
        };

        Ok((output, TokenUsage::default()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repl_request_parse() {
        let req: ReplRequest =
            serde_json::from_str(r#"{"language":"python","code":"print(1+1)"}"#).unwrap();
        assert_eq!(req.language, "python");
        assert_eq!(req.timeout_secs, 10);
    }

    #[test]
    fn test_default_timeout() {
        assert_eq!(default_timeout(), 10);
    }
}
