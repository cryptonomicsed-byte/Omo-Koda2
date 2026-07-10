use std::collections::VecDeque;

use crate::background::{BackgroundConfig, BackgroundRegistry, BackgroundStatus};
use crate::dream::{DreamConfig, DreamEngine};
use crate::memory::OduDirectory;
use crate::query::{QueryConfig, QueryEngine, QueryState, StopCondition};
use crate::tasks::types::{TaskKind, TaskManager};
use crate::usage::TokenUsage;

/// Configuration for the integrated task scheduler.
#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    pub background: BackgroundConfig,
    pub query: QueryConfig,
    pub dream: DreamConfig,
    /// Trigger a Dream consolidation run after this many tasks complete.
    pub dream_after_tasks: usize,
    /// Starting synapse (energy) balance for the query engine.
    pub initial_synapse: f64,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            background: BackgroundConfig::default(),
            query: QueryConfig::default(),
            dream: DreamConfig::default(),
            dream_after_tasks: 10,
            initial_synapse: 100_000.0,
        }
    }
}

/// Result of polling the scheduler for completed background tasks.
#[derive(Debug)]
pub struct PollResult {
    pub completed: Vec<(String, BackgroundStatus)>,
    pub dream_triggered: bool,
    pub budget_stop: Option<StopCondition>,
}

/// Integrated task scheduler — ties together three patterns:
///
/// - **Pattern 52 (Query Engine)**: `QueryEngine` enforces token/energy budget across all Think tasks.
/// - **Pattern 53 (Task Heterogeneity)**: Dispatches `Think`, `Act`, `Dream`, `Delegate`, `Background`.
/// - **Pattern 54 (Background Execution)**: `BackgroundRegistry` runs async tasks; `poll()` reaps
///   completions and auto-triggers Dream consolidation after `dream_after_tasks` completions.
pub struct TaskScheduler {
    tasks: TaskManager,
    background: BackgroundRegistry,
    engine: QueryEngine,
    dream: DreamEngine,
    query_state: QueryState,
    config: SchedulerConfig,
    /// Tasks submitted but not yet dispatched to the background runner.
    pending_queue: VecDeque<String>,
    completed_since_dream: usize,
}

impl TaskScheduler {
    pub fn new(config: SchedulerConfig) -> Self {
        let engine = QueryEngine::new(config.query.clone());
        let query_state = QueryState::new(&config.query, config.initial_synapse);
        let dream = DreamEngine::new(config.dream.clone());
        let background = BackgroundRegistry::new(config.background.clone());

        Self {
            tasks: TaskManager::new(),
            background,
            engine,
            dream,
            query_state,
            config,
            pending_queue: VecDeque::new(),
            completed_since_dream: 0,
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(SchedulerConfig::default())
    }

    /// Submit a task. Returns the task ID.
    /// Async-capable kinds (`Dream`, `Delegate`, `Background`) are queued for background dispatch.
    /// `Think` and `Act` are submitted inline (caller drives execution).
    pub fn submit(&mut self, kind: TaskKind) -> String {
        let id = self.tasks.submit(kind.clone());
        if kind.is_async() {
            self.pending_queue.push_back(id.clone());
        }
        id
    }

    /// Mark a Think task as having started a new turn, recording budget usage.
    /// Returns the stop condition if the budget is now exceeded.
    pub fn record_think_turn(
        &mut self,
        task_id: &str,
        usage: &TokenUsage,
        synapse_cost: f64,
    ) -> Option<StopCondition> {
        if self.query_state.is_stopped() {
            return self.query_state.stop_reason.clone();
        }

        self.engine
            .advance_turn(&mut self.query_state, usage, synapse_cost);

        if self.query_state.is_stopped() {
            // Fail the task due to budget exhaustion
            if let Some(cond) = &self.query_state.stop_reason {
                self.tasks.fail(task_id, format!("budget stop: {:?}", cond));
            }
            return self.query_state.stop_reason.clone();
        }

        None
    }

    /// Dispatch pending async tasks into the background runner.
    /// Stops when the background runner is at capacity.
    pub fn dispatch_pending(&mut self) {
        while let Some(id) = self.pending_queue.front().cloned() {
            let kind = match self.tasks.get(&id) {
                Some(t) => t.kind.clone(),
                None => {
                    self.pending_queue.pop_front();
                    continue;
                }
            };

            let description = format!("[{}] {}", kind.label(), id);
            let id_clone = id.clone();

            let fut = async move {
                // Simulate async execution (real impls call providers/tools)
                tokio::task::yield_now().await;
                Ok(format!("{} completed", id_clone))
            };

            match self.background.spawn(id.clone(), description, fut) {
                Ok(_) => {
                    self.tasks.start(&id);
                    self.pending_queue.pop_front();
                }
                Err(_) => break, // capacity full; retry on next poll
            }
        }
    }

    /// Poll background tasks for completions, update TaskManager, and maybe trigger Dream.
    pub fn poll(&mut self, odu: Option<&mut OduDirectory>) -> PollResult {
        self.dispatch_pending();

        let newly_done = self.background.poll();

        for (bg_id, status) in &newly_done {
            match status {
                BackgroundStatus::Completed(output) => {
                    self.tasks.complete(bg_id, output.clone());
                }
                BackgroundStatus::Failed(err) => {
                    self.tasks.fail(bg_id, err.clone());
                }
                BackgroundStatus::Cancelled => {
                    self.tasks.cancel(bg_id);
                }
                BackgroundStatus::Running => {}
            }
        }

        let completions = newly_done.len();
        self.completed_since_dream += completions;

        let mut dream_triggered = false;
        let budget_stop = self.query_state.stop_reason.clone();

        if self.completed_since_dream >= self.config.dream_after_tasks {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            if let Some(dir) = odu {
                let mut ran = false;
                if self.dream.should_consolidate(now) {
                    ran |= self.dream.try_consolidate(dir, now).is_some();
                }
                // Weekly REM pass rides the same trigger; it gates itself on
                // its own (much longer) cadence.
                ran |= self.dream.try_rem_cycle(dir, now).is_some();
                if ran {
                    dream_triggered = true;
                    self.completed_since_dream = 0;
                }
            } else if self.dream.should_consolidate(now) {
                dream_triggered = true;
                self.completed_since_dream = 0;
            }
        }

        PollResult {
            completed: newly_done,
            dream_triggered,
            budget_stop,
        }
    }

    /// Current query state (budget, turn count, synapse).
    pub fn query_state(&self) -> &QueryState {
        &self.query_state
    }

    /// Whether the query budget is exhausted.
    pub fn is_budget_stopped(&self) -> bool {
        self.query_state.is_stopped()
    }

    /// Task manager view (all submitted tasks, any kind).
    pub fn tasks(&self) -> &TaskManager {
        &self.tasks
    }

    /// Summary of scheduler state.
    pub fn summary(&self) -> String {
        let qs = &self.query_state;
        format!(
            "{} | turns={} synapse={:.0} budget_in={}/{} pending_queue={}",
            self.tasks.summary(),
            qs.turn_count,
            qs.synapse_remaining,
            qs.budget.used_input,
            qs.budget.input_limit,
            self.pending_queue.len(),
        )
    }
}

#[cfg(test)]
mod scheduler_tests {
    use super::*;
    use crate::query::QueryConfig;
    use crate::tasks::types::TaskStatus;

    fn make_scheduler(dream_after: usize) -> TaskScheduler {
        let config = SchedulerConfig {
            background: BackgroundConfig {
                max_concurrent: 4,
                auto_compact_every: None,
            },
            query: QueryConfig {
                max_turns: 10,
                ..Default::default()
            },
            dream: DreamConfig {
                consolidation_interval_secs: 0, // always eligible
                ..Default::default()
            },
            dream_after_tasks: dream_after,
            initial_synapse: 50_000.0,
        };
        TaskScheduler::new(config)
    }

    #[test]
    fn test_submit_think_task() {
        let mut sched = make_scheduler(5);
        let id = sched.submit(TaskKind::Think {
            prompt: "What is Ifá?".to_string(),
            private: false,
        });
        assert_eq!(sched.tasks().status(&id), Some(&TaskStatus::Pending));
        assert_eq!(sched.pending_queue.len(), 0); // Think is not async
    }

    #[test]
    fn test_submit_dream_is_async() {
        let mut sched = make_scheduler(5);
        let id = sched.submit(TaskKind::Dream {
            consolidation_target: None,
        });
        assert_eq!(sched.pending_queue.len(), 1);
        assert!(sched.pending_queue.contains(&id));
    }

    #[test]
    fn test_submit_delegate_is_async() {
        let mut sched = make_scheduler(5);
        let id = sched.submit(TaskKind::Delegate {
            agent_id: "peer-1".to_string(),
            task_description: "analyse data".to_string(),
            swarm_node: None,
        });
        assert!(sched.pending_queue.contains(&id));
    }

    #[test]
    fn test_task_kind_labels() {
        assert_eq!(
            TaskKind::Dream {
                consolidation_target: None
            }
            .label(),
            "dream"
        );
        assert_eq!(
            TaskKind::Delegate {
                agent_id: "a".to_string(),
                task_description: "b".to_string(),
                swarm_node: None,
            }
            .label(),
            "delegate"
        );
    }

    #[test]
    fn test_task_kind_is_write() {
        assert!(!TaskKind::Think {
            prompt: "q".to_string(),
            private: false
        }
        .is_write());
        assert!(TaskKind::Act {
            tool: "bash".to_string(),
            params: "ls".to_string(),
            sandbox: false,
        }
        .is_write());
        assert!(TaskKind::Delegate {
            agent_id: "x".to_string(),
            task_description: "y".to_string(),
            swarm_node: None,
        }
        .is_write());
    }

    #[test]
    fn test_record_think_turn_consumes_budget() {
        let mut sched = make_scheduler(5);
        let id = sched.submit(TaskKind::Think {
            prompt: "test".to_string(),
            private: false,
        });
        sched.tasks.start(&id);

        let usage = TokenUsage {
            input_tokens: 1000,
            output_tokens: 500,
            cache_creation_input_tokens: 0,
            cache_read_input_tokens: 0,
        };

        let stop = sched.record_think_turn(&id, &usage, 100.0);
        assert!(stop.is_none());
        assert_eq!(sched.query_state().budget.used_input, 1000);
        assert_eq!(sched.query_state().budget.used_output, 500);
    }

    #[test]
    fn test_budget_exhaustion_stops_think() {
        let config = SchedulerConfig {
            query: QueryConfig {
                max_turns: 100,
                budget: crate::query::TokenBudget::new(100, 50),
                synapse_floor: 0.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut sched = TaskScheduler::new(config);
        let id = sched.submit(TaskKind::Think {
            prompt: "exhaust".to_string(),
            private: false,
        });
        sched.tasks.start(&id);

        // Consume entire input budget in one turn
        let usage = TokenUsage {
            input_tokens: 200,
            output_tokens: 0,
            cache_creation_input_tokens: 0,
            cache_read_input_tokens: 0,
        };

        let stop = sched.record_think_turn(&id, &usage, 0.0);
        assert!(matches!(stop, Some(StopCondition::BudgetExceeded)));
        assert!(sched.is_budget_stopped());
    }

    #[test]
    fn test_summary_contains_key_info() {
        let sched = make_scheduler(5);
        let s = sched.summary();
        assert!(s.contains("turns="));
        assert!(s.contains("synapse="));
        assert!(s.contains("pending_queue="));
    }
}
