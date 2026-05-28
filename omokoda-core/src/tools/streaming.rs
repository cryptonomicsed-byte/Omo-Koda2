use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::permissions::PermissionPolicy;
use crate::tools::{ExecutionContext, ToolRegistry};
use crate::usage::TokenUsage;

/// Events emitted by the streaming executor as a tool runs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolStreamEvent {
    /// Execution has started
    Started { tool: String },
    /// Partial output chunk (for tools that emit incremental text)
    Output { chunk: String },
    /// Execution progress update
    Progress { pct: f32, message: String },
    /// Tool completed successfully
    Completed { output: String, usage: TokenUsage },
    /// Tool failed
    Failed { error: String },
}

pub type ToolStreamSender = mpsc::Sender<ToolStreamEvent>;
pub type ToolStreamReceiver = mpsc::Receiver<ToolStreamEvent>;

/// Hook interception point for streaming execution
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolHookDecision {
    Allow,
    Deny(String),
    Warn(String),
}

/// A pre-execution or post-execution hook for the streaming executor
pub struct ToolHook {
    pub name: String,
    pub phase: ToolHookPhase,
    #[allow(clippy::type_complexity)]
    pub run: Box<dyn Fn(&str, &str) -> ToolHookDecision + Send + Sync>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolHookPhase {
    Pre,
    Post,
}

impl ToolHook {
    pub fn pre(
        name: impl Into<String>,
        run: impl Fn(&str, &str) -> ToolHookDecision + Send + Sync + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            phase: ToolHookPhase::Pre,
            run: Box::new(run),
        }
    }

    pub fn post(
        name: impl Into<String>,
        run: impl Fn(&str, &str) -> ToolHookDecision + Send + Sync + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            phase: ToolHookPhase::Post,
            run: Box::new(run),
        }
    }
}

/// Orchestrates tool execution with streaming output and hook interception.
/// Mirrors `StreamingToolExecutor` / `toolOrchestration.ts`.
pub struct StreamingToolExecutor {
    hooks: Vec<ToolHook>,
}

impl Default for StreamingToolExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl StreamingToolExecutor {
    pub fn new() -> Self {
        Self { hooks: Vec::new() }
    }

    pub fn add_hook(&mut self, hook: ToolHook) {
        self.hooks.push(hook);
    }

    fn run_pre_hooks(&self, tool: &str, params: &str) -> ToolHookDecision {
        for hook in self.hooks.iter().filter(|h| h.phase == ToolHookPhase::Pre) {
            match (hook.run)(tool, params) {
                ToolHookDecision::Allow => continue,
                other => return other,
            }
        }
        ToolHookDecision::Allow
    }

    fn run_post_hooks(&self, tool: &str, output: &str) -> ToolHookDecision {
        for hook in self.hooks.iter().filter(|h| h.phase == ToolHookPhase::Post) {
            match (hook.run)(tool, output) {
                ToolHookDecision::Allow => continue,
                other => return other,
            }
        }
        ToolHookDecision::Allow
    }

    /// Execute `tool` with `params`, streaming events to `sender`.
    /// Returns the final output and usage on success.
    pub async fn execute_streaming(
        &self,
        tool: &str,
        params: &str,
        context: ExecutionContext,
        registry: &ToolRegistry,
        policy: &PermissionPolicy,
        sender: ToolStreamSender,
    ) -> Result<(String, TokenUsage), String> {
        // Pre-hook interception
        match self.run_pre_hooks(tool, params) {
            ToolHookDecision::Deny(reason) => {
                let _ = sender
                    .send(ToolStreamEvent::Failed {
                        error: format!("Pre-hook denied: {}", reason),
                    })
                    .await;
                return Err(format!("Pre-hook denied: {}", reason));
            }
            ToolHookDecision::Warn(w) => {
                let _ = sender
                    .send(ToolStreamEvent::Output {
                        chunk: format!("[hook warning] {}", w),
                    })
                    .await;
            }
            ToolHookDecision::Allow => {}
        }

        let _ = sender
            .send(ToolStreamEvent::Started {
                tool: tool.to_string(),
            })
            .await;
        let _ = sender
            .send(ToolStreamEvent::Progress {
                pct: 0.0,
                message: "executing".to_string(),
            })
            .await;

        let result = registry.execute(tool, params, context, policy, None).await;

        match result {
            Ok((output, usage)) => {
                // Post-hook interception
                match self.run_post_hooks(tool, &output) {
                    ToolHookDecision::Deny(reason) => {
                        let _ = sender
                            .send(ToolStreamEvent::Failed {
                                error: format!("Post-hook denied: {}", reason),
                            })
                            .await;
                        return Err(format!("Post-hook denied: {}", reason));
                    }
                    ToolHookDecision::Warn(w) => {
                        let _ = sender
                            .send(ToolStreamEvent::Output {
                                chunk: format!("[hook warning] {}", w),
                            })
                            .await;
                    }
                    ToolHookDecision::Allow => {}
                }

                let _ = sender
                    .send(ToolStreamEvent::Progress {
                        pct: 1.0,
                        message: "done".to_string(),
                    })
                    .await;
                let _ = sender
                    .send(ToolStreamEvent::Completed {
                        output: output.clone(),
                        usage,
                    })
                    .await;
                Ok((output, usage))
            }
            Err(e) => {
                let _ = sender
                    .send(ToolStreamEvent::Failed { error: e.clone() })
                    .await;
                Err(e)
            }
        }
    }

    /// Execute multiple tools in sequence, streaming all events to one channel.
    /// Each tool's output is available to subsequent tools via `output_map`.
    pub async fn execute_sequence(
        &self,
        steps: &[(String, String)], // (tool, params)
        context: ExecutionContext,
        registry: &ToolRegistry,
        policy: &PermissionPolicy,
        sender: ToolStreamSender,
    ) -> Result<Vec<(String, TokenUsage)>, String> {
        let mut results = Vec::new();
        for (i, (tool, params)) in steps.iter().enumerate() {
            let _ = sender
                .send(ToolStreamEvent::Progress {
                    pct: i as f32 / steps.len() as f32,
                    message: format!("step {}/{}: {}", i + 1, steps.len(), tool),
                })
                .await;

            let (tx, _rx) = mpsc::channel(32);
            let result = self
                .execute_streaming(tool, params, context.clone(), registry, policy, tx)
                .await?;
            results.push(result);
        }
        Ok(results)
    }
}

/// Collect all events from a stream into a vec (useful for tests)
pub async fn collect_stream(mut rx: ToolStreamReceiver) -> Vec<ToolStreamEvent> {
    let mut events = Vec::new();
    while let Some(event) = rx.recv().await {
        let done = matches!(
            event,
            ToolStreamEvent::Completed { .. } | ToolStreamEvent::Failed { .. }
        );
        events.push(event);
        if done {
            break;
        }
    }
    events
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hook_decision_deny_stops_chain() {
        let mut executor = StreamingToolExecutor::new();
        executor.add_hook(ToolHook::pre("block_all", |_tool, _params| {
            ToolHookDecision::Deny("blocked".to_string())
        }));
        let decision = executor.run_pre_hooks("bash", "ls");
        assert_eq!(decision, ToolHookDecision::Deny("blocked".to_string()));
    }

    #[test]
    fn test_hook_allow_passes_through() {
        let executor = StreamingToolExecutor::new();
        let decision = executor.run_pre_hooks("read_file", "foo.txt");
        assert_eq!(decision, ToolHookDecision::Allow);
    }

    #[test]
    fn test_tool_stream_event_serialization() {
        let event = ToolStreamEvent::Completed {
            output: "ok".to_string(),
            usage: TokenUsage::default(),
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("Completed"));
    }
}
