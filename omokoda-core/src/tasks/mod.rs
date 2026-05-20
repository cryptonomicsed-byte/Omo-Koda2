pub mod types;

pub use types::{Task, TaskKind, TaskManager, TaskStatus};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_lifecycle() {
        let mut manager = TaskManager::new();
        let id = manager.submit(TaskKind::Think {
            prompt: "What is Ifá?".to_string(),
            private: false,
        });
        assert_eq!(manager.status(&id), Some(&TaskStatus::Pending));
        manager.start(&id);
        assert_eq!(manager.status(&id), Some(&TaskStatus::Running));
        manager.complete(&id, "Ifá is a divination system.".to_string());
        assert!(matches!(
            manager.status(&id),
            Some(TaskStatus::Completed(_))
        ));
    }

    #[test]
    fn test_task_cancel() {
        let mut manager = TaskManager::new();
        let id = manager.submit(TaskKind::Act {
            tool: "bash".to_string(),
            params: "ls".to_string(),
            sandbox: false,
        });
        manager.start(&id);
        manager.cancel(&id);
        assert_eq!(manager.status(&id), Some(&TaskStatus::Cancelled));
    }

    #[test]
    fn test_list_by_status() {
        let mut manager = TaskManager::new();
        manager.submit(TaskKind::Think {
            prompt: "a".to_string(),
            private: false,
        });
        let running_id = manager.submit(TaskKind::Think {
            prompt: "b".to_string(),
            private: false,
        });
        manager.start(&running_id);

        let pending = manager.list_by_status(&TaskStatus::Pending);
        let running = manager.list_by_status(&TaskStatus::Running);
        assert_eq!(pending.len(), 1);
        assert_eq!(running.len(), 1);
    }
}
