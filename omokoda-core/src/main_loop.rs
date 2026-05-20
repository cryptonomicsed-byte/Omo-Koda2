use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use tokio::sync::mpsc;

use crate::query::{QueryConfig, QueryEngine, QueryState};
use crate::usage::TokenUsage;

/// Events that drive the main loop — inputs from user, tools, timers, or the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoopEvent {
    /// User submitted a new prompt
    UserInput { prompt: String, private: bool },
    /// A tool returned output (from an agentic inner loop)
    ToolResult { tool: String, output: String },
    /// A background task completed
    BackgroundDone { task_id: String, output: String },
    /// Switch the active provider/model
    SwitchModel { provider: String },
    /// Trigger context compaction
    Compact,
    /// Pause the loop (e.g. awaiting confirmation)
    Pause,
    /// Resume after a pause
    Resume,
    /// Shutdown the loop
    Shutdown,
}

/// Observable state of the main loop
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoopState {
    /// Waiting for user input
    Idle,
    /// Executing an LLM think turn
    Thinking,
    /// Executing a tool act
    Acting { tool: String },
    /// Compacting the context window
    Compacting,
    /// Switching provider/model
    SwitchingModel,
    /// Paused, awaiting user confirmation
    Paused,
    /// Loop has stopped
    Stopped,
}

impl LoopState {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Thinking => "thinking",
            Self::Acting { .. } => "acting",
            Self::Compacting => "compacting",
            Self::SwitchingModel => "switching_model",
            Self::Paused => "paused",
            Self::Stopped => "stopped",
        }
    }

    pub fn is_terminal(&self) -> bool {
        *self == Self::Stopped
    }

    pub fn can_accept_input(&self) -> bool {
        matches!(self, Self::Idle | Self::Paused)
    }
}

/// Configuration for the main loop
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MainLoopConfig {
    pub queue_capacity: usize,
    pub query: QueryConfig,
    pub auto_compact_threshold: usize,
    /// Automatically switch to a cheaper model after N turns
    pub model_switch_after_turns: Option<u32>,
    pub fallback_provider: Option<String>,
}

impl Default for MainLoopConfig {
    fn default() -> Self {
        Self {
            queue_capacity: 64,
            query: QueryConfig::default(),
            auto_compact_threshold: 50,
            model_switch_after_turns: None,
            fallback_provider: None,
        }
    }
}

/// Outcome of processing one event
#[derive(Debug, Clone)]
pub enum TurnOutcome {
    /// Loop advanced normally; new state and optional output
    Continue {
        new_state: LoopState,
        output: Option<String>,
        usage: TokenUsage,
    },
    /// Loop is switching models; processing continues after switch
    ModelSwitch { to_provider: String },
    /// Loop should compact before continuing
    CompactNeeded,
    /// Loop stopped
    Stopped,
}

/// The main event-driven loop — queue processor + state machine.
/// Mirrors `useMainLoopModel` / `queueProcessor`.
pub struct MainLoop {
    pub state: LoopState,
    pub config: MainLoopConfig,
    queue: VecDeque<LoopEvent>,
    pub query_engine: QueryEngine,
    pub query_state: QueryState,
    pub active_provider: String,
    pub turn_history: Vec<(LoopEvent, TurnOutcome)>,
}

impl MainLoop {
    pub fn new(
        config: MainLoopConfig,
        initial_provider: impl Into<String>,
        initial_synapse: f64,
    ) -> Self {
        let query_engine = QueryEngine::new(config.query.clone());
        let query_state = QueryState::new(&config.query, initial_synapse);
        Self {
            state: LoopState::Idle,
            queue: VecDeque::with_capacity(config.queue_capacity),
            active_provider: initial_provider.into(),
            turn_history: Vec::new(),
            query_engine,
            query_state,
            config,
        }
    }

    pub fn with_defaults(provider: impl Into<String>, initial_synapse: f64) -> Self {
        Self::new(MainLoopConfig::default(), provider, initial_synapse)
    }

    /// Enqueue an event for processing
    pub fn push(&mut self, event: LoopEvent) -> Result<(), String> {
        if self.state.is_terminal() {
            return Err("Loop is stopped".to_string());
        }
        if self.queue.len() >= self.config.queue_capacity {
            return Err("Event queue full".to_string());
        }
        self.queue.push_back(event);
        Ok(())
    }

    /// Drain up to `max` queued events, returning their outcomes
    pub fn drain(&mut self, max: usize) -> Vec<TurnOutcome> {
        let mut outcomes = Vec::new();
        let n = max.min(self.queue.len());
        for _ in 0..n {
            if let Some(event) = self.queue.pop_front() {
                let outcome = self.process(event);
                if matches!(outcome, TurnOutcome::Stopped) {
                    self.state = LoopState::Stopped;
                    outcomes.push(outcome);
                    break;
                }
                outcomes.push(outcome);
            }
        }
        outcomes
    }

    /// Process a single event, transitioning state and returning the outcome
    pub fn process(&mut self, event: LoopEvent) -> TurnOutcome {
        if self.state.is_terminal() {
            return TurnOutcome::Stopped;
        }

        // Auto-compact check
        if self.query_state.turn_count > 0
            && self.query_state.turn_count as usize % self.config.auto_compact_threshold == 0
            && !matches!(self.state, LoopState::Compacting)
        {
            self.state = LoopState::Compacting;
            return TurnOutcome::CompactNeeded;
        }

        // Auto model-switch
        if let Some(after) = self.config.model_switch_after_turns {
            if self.query_state.turn_count > 0 && self.query_state.turn_count % after == 0 {
                if let Some(fallback) = self.config.fallback_provider.clone() {
                    if fallback != self.active_provider {
                        let to = fallback.clone();
                        self.active_provider = fallback;
                        self.state = LoopState::SwitchingModel;
                        return TurnOutcome::ModelSwitch { to_provider: to };
                    }
                }
            }
        }

        match &event {
            LoopEvent::UserInput {
                prompt: _,
                private: _,
            } => {
                if !self.state.can_accept_input() {
                    self.queue.push_front(event.clone());
                    return TurnOutcome::Continue {
                        new_state: self.state.clone(),
                        output: Some("queued: loop is busy".to_string()),
                        usage: TokenUsage::default(),
                    };
                }
                self.state = LoopState::Thinking;
                let usage = TokenUsage::default();
                self.query_engine
                    .advance_turn(&mut self.query_state, &usage, 1000.0);
                let outcome = TurnOutcome::Continue {
                    new_state: LoopState::Idle,
                    output: Some(format!(
                        "[turn {}] thinking...",
                        self.query_state.turn_count
                    )),
                    usage,
                };
                self.state = LoopState::Idle;
                self.turn_history.push((event, outcome.clone()));
                outcome
            }

            LoopEvent::ToolResult { tool, output } => {
                let outcome = TurnOutcome::Continue {
                    new_state: LoopState::Idle,
                    output: Some(format!("[tool:{}] {}", tool, output)),
                    usage: TokenUsage::default(),
                };
                self.state = LoopState::Idle;
                self.turn_history.push((event, outcome.clone()));
                outcome
            }

            LoopEvent::BackgroundDone { task_id, output } => {
                let outcome = TurnOutcome::Continue {
                    new_state: self.state.clone(),
                    output: Some(format!("[bg:{}] {}", task_id, output)),
                    usage: TokenUsage::default(),
                };
                self.turn_history.push((event, outcome.clone()));
                outcome
            }

            LoopEvent::SwitchModel { provider } => {
                let to = provider.clone();
                self.active_provider = provider.clone();
                self.state = LoopState::Idle;
                let outcome = TurnOutcome::ModelSwitch { to_provider: to };
                self.turn_history.push((event, outcome.clone()));
                outcome
            }

            LoopEvent::Compact => {
                self.state = LoopState::Compacting;
                let outcome = TurnOutcome::CompactNeeded;
                self.state = LoopState::Idle;
                self.turn_history.push((event, outcome.clone()));
                outcome
            }

            LoopEvent::Pause => {
                self.state = LoopState::Paused;
                TurnOutcome::Continue {
                    new_state: LoopState::Paused,
                    output: None,
                    usage: TokenUsage::default(),
                }
            }

            LoopEvent::Resume => {
                if self.state == LoopState::Paused {
                    self.state = LoopState::Idle;
                }
                TurnOutcome::Continue {
                    new_state: self.state.clone(),
                    output: None,
                    usage: TokenUsage::default(),
                }
            }

            LoopEvent::Shutdown => {
                self.state = LoopState::Stopped;
                TurnOutcome::Stopped
            }
        }
    }

    pub fn is_stopped(&self) -> bool {
        self.state.is_terminal()
    }

    pub fn queue_len(&self) -> usize {
        self.queue.len()
    }

    pub fn turn_count(&self) -> u32 {
        self.query_state.turn_count
    }
}

/// Channel-based interface to drive the loop from async code.
/// Spawn this with `tokio::spawn` to run the loop in the background.
pub struct LoopHandle {
    pub sender: mpsc::Sender<LoopEvent>,
}

impl LoopHandle {
    pub fn spawn(
        provider: impl Into<String>,
        initial_synapse: f64,
    ) -> (Self, mpsc::Receiver<TurnOutcome>) {
        let (event_tx, mut event_rx) = mpsc::channel::<LoopEvent>(64);
        let (outcome_tx, outcome_rx) = mpsc::channel::<TurnOutcome>(64);
        let mut loop_ = MainLoop::with_defaults(provider, initial_synapse);

        tokio::spawn(async move {
            while let Some(event) = event_rx.recv().await {
                let outcome = loop_.process(event);
                let stopped = matches!(outcome, TurnOutcome::Stopped);
                let _ = outcome_tx.send(outcome).await;
                if stopped {
                    break;
                }
            }
        });

        (Self { sender: event_tx }, outcome_rx)
    }

    pub async fn send(&self, event: LoopEvent) -> Result<(), String> {
        self.sender
            .send(event)
            .await
            .map_err(|_| "loop has stopped".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_loop() -> MainLoop {
        MainLoop::with_defaults("mock", 100_000.0)
    }

    #[test]
    fn test_loop_starts_idle() {
        let loop_ = make_loop();
        assert_eq!(loop_.state, LoopState::Idle);
    }

    #[test]
    fn test_user_input_advances_turn() {
        let mut loop_ = make_loop();
        loop_
            .push(LoopEvent::UserInput {
                prompt: "hello".to_string(),
                private: false,
            })
            .unwrap();
        let outcomes = loop_.drain(1);
        assert_eq!(outcomes.len(), 1);
        assert!(matches!(outcomes[0], TurnOutcome::Continue { .. }));
        assert_eq!(loop_.turn_count(), 1);
    }

    #[test]
    fn test_shutdown_stops_loop() {
        let mut loop_ = make_loop();
        loop_.push(LoopEvent::Shutdown).unwrap();
        let outcomes = loop_.drain(1);
        assert!(matches!(outcomes[0], TurnOutcome::Stopped));
        assert!(loop_.is_stopped());
    }

    #[test]
    fn test_model_switch_event() {
        let mut loop_ = make_loop();
        loop_
            .push(LoopEvent::SwitchModel {
                provider: "ollama".to_string(),
            })
            .unwrap();
        loop_.drain(1);
        assert_eq!(loop_.active_provider, "ollama");
    }

    #[test]
    fn test_pause_and_resume() {
        let mut loop_ = make_loop();
        loop_.push(LoopEvent::Pause).unwrap();
        loop_.drain(1);
        assert_eq!(loop_.state, LoopState::Paused);

        loop_.push(LoopEvent::Resume).unwrap();
        loop_.drain(1);
        assert_eq!(loop_.state, LoopState::Idle);
    }

    #[test]
    fn test_queue_overflow_rejected() {
        let config = MainLoopConfig {
            queue_capacity: 2,
            ..Default::default()
        };
        let mut loop_ = MainLoop::new(config, "mock", 100_000.0);
        assert!(loop_.push(LoopEvent::Compact).is_ok());
        assert!(loop_.push(LoopEvent::Compact).is_ok());
        assert!(loop_.push(LoopEvent::Compact).is_err());
    }
}
