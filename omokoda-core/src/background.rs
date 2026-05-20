use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;

/// Status of a background task, readable without blocking
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackgroundStatus {
    Running,
    Completed(String),
    Failed(String),
    Cancelled,
}

impl BackgroundStatus {
    pub fn is_terminal(&self) -> bool {
        !matches!(self, Self::Running)
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Completed(_) => "completed",
            Self::Failed(_) => "failed",
            Self::Cancelled => "cancelled",
        }
    }
}

/// A handle to a running background task
pub struct BackgroundHandle {
    pub task_id: String,
    pub description: String,
    status: Arc<Mutex<BackgroundStatus>>,
    join: Option<JoinHandle<()>>,
}

impl BackgroundHandle {
    pub fn status(&self) -> BackgroundStatus {
        self.status.lock().unwrap().clone()
    }

    pub fn is_done(&self) -> bool {
        self.status().is_terminal()
    }

    pub fn cancel(&mut self) {
        if let Some(handle) = self.join.take() {
            handle.abort();
            let mut s = self.status.lock().unwrap();
            if !s.is_terminal() {
                *s = BackgroundStatus::Cancelled;
            }
        }
    }
}

/// Configuration for the background runner
#[derive(Debug, Clone)]
pub struct BackgroundConfig {
    /// Max concurrently running background tasks
    pub max_concurrent: usize,
    /// Auto-trigger compaction after this many completed tasks
    pub auto_compact_every: Option<usize>,
}

impl Default for BackgroundConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 4,
            auto_compact_every: Some(10),
        }
    }
}

/// Tracks and manages background `think`/`act` executions.
/// Mirrors `BackgroundTask` / `useTasksV2`.
pub struct BackgroundRegistry {
    handles: HashMap<String, BackgroundHandle>,
    config: BackgroundConfig,
    completed_count: usize,
    pub compact_requested: bool,
}

impl Default for BackgroundRegistry {
    fn default() -> Self {
        Self::new(BackgroundConfig::default())
    }
}

impl BackgroundRegistry {
    pub fn new(config: BackgroundConfig) -> Self {
        Self {
            handles: HashMap::new(),
            config,
            completed_count: 0,
            compact_requested: false,
        }
    }

    /// Spawn a background task; returns the task ID.
    /// `fut` is any async future that produces `Result<String, String>`.
    pub fn spawn<F>(
        &mut self,
        task_id: impl Into<String>,
        description: impl Into<String>,
        fut: F,
    ) -> Result<String, String>
    where
        F: std::future::Future<Output = Result<String, String>> + Send + 'static,
    {
        let running = self.handles.values().filter(|h| !h.is_done()).count();

        if running >= self.config.max_concurrent {
            return Err(format!(
                "Max concurrent background tasks ({}) reached",
                self.config.max_concurrent
            ));
        }

        let id = task_id.into();
        let desc = description.into();
        let status = Arc::new(Mutex::new(BackgroundStatus::Running));
        let status_clone = Arc::clone(&status);

        let join = tokio::spawn(async move {
            match fut.await {
                Ok(output) => {
                    *status_clone.lock().unwrap() = BackgroundStatus::Completed(output);
                }
                Err(e) => {
                    *status_clone.lock().unwrap() = BackgroundStatus::Failed(e);
                }
            }
        });

        self.handles.insert(
            id.clone(),
            BackgroundHandle {
                task_id: id.clone(),
                description: desc,
                status,
                join: Some(join),
            },
        );

        Ok(id)
    }

    pub fn status(&self, task_id: &str) -> Option<BackgroundStatus> {
        self.handles.get(task_id).map(|h| h.status())
    }

    /// Poll all handles; collect newly completed ones and maybe trigger compaction.
    pub fn poll(&mut self) -> Vec<(String, BackgroundStatus)> {
        let mut newly_done = Vec::new();
        for (id, handle) in self.handles.iter() {
            let status = handle.status();
            if status.is_terminal() {
                newly_done.push((id.clone(), status));
            }
        }

        self.completed_count += newly_done.len();

        if let Some(every) = self.config.auto_compact_every {
            if self.completed_count > 0 && self.completed_count % every == 0 {
                self.compact_requested = true;
            }
        }

        newly_done
    }

    pub fn cancel(&mut self, task_id: &str) -> bool {
        if let Some(handle) = self.handles.get_mut(task_id) {
            handle.cancel();
            true
        } else {
            false
        }
    }

    pub fn cancel_all(&mut self) {
        for handle in self.handles.values_mut() {
            handle.cancel();
        }
    }

    pub fn running_count(&self) -> usize {
        self.handles.values().filter(|h| !h.is_done()).count()
    }

    pub fn summary(&self) -> String {
        let running = self.running_count();
        let done = self.handles.values().filter(|h| h.is_done()).count();
        format!(
            "Background: {} running, {} done, {} total",
            running,
            done,
            self.handles.len()
        )
    }

    /// Remove terminal tasks from the registry
    pub fn prune(&mut self) {
        self.handles.retain(|_, h| !h.is_done());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_registry() -> BackgroundRegistry {
        BackgroundRegistry::new(BackgroundConfig {
            max_concurrent: 2,
            auto_compact_every: Some(2),
        })
    }

    #[tokio::test]
    async fn test_spawn_and_complete() {
        let mut registry = make_registry();
        let id = registry
            .spawn("t1", "test task", async { Ok("done".to_string()) })
            .unwrap();
        // Yield to let the spawned task complete
        tokio::task::yield_now().await;
        tokio::task::yield_now().await;
        let status = registry.status(&id).unwrap();
        assert!(matches!(status, BackgroundStatus::Completed(_)));
    }

    #[tokio::test]
    async fn test_max_concurrent_rejected() {
        let mut registry = make_registry();
        let never = async {
            tokio::time::sleep(std::time::Duration::from_secs(999)).await;
            Ok("done".to_string())
        };
        let never2 = async {
            tokio::time::sleep(std::time::Duration::from_secs(999)).await;
            Ok("done".to_string())
        };
        registry.spawn("t1", "a", never).unwrap();
        registry.spawn("t2", "b", never2).unwrap();
        let result = registry.spawn("t3", "c", async { Ok("x".to_string()) });
        assert!(result.is_err());
    }

    #[test]
    fn test_summary() {
        let registry = make_registry();
        let s = registry.summary();
        assert!(s.contains("Background"));
    }
}
