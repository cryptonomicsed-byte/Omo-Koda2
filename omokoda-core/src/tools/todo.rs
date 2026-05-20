//! Persistent todo management — task queue with status tracking.
//! Ports Claw-code's todo tool pattern.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::tools::{ExecutionContext, Tool};
use crate::usage::TokenUsage;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TodoStatus {
    Pending,
    InProgress,
    Completed,
    Cancelled,
}

impl std::fmt::Display for TodoStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "pending"),
            Self::InProgress => write!(f, "in_progress"),
            Self::Completed => write!(f, "completed"),
            Self::Cancelled => write!(f, "cancelled"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TodoPriority {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for TodoPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "low"),
            Self::Medium => write!(f, "medium"),
            Self::High => write!(f, "high"),
            Self::Critical => write!(f, "critical"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    pub id: String,
    pub content: String,
    pub status: TodoStatus,
    pub priority: TodoPriority,
    pub created_at: u64,
    pub updated_at: u64,
    pub agent_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct TodoStore {
    items: Vec<TodoItem>,
}

impl TodoStore {
    fn load(path: &Path) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(path, json).map_err(|e| e.to_string())
    }

    fn todo_path(workspace_root: &Path) -> PathBuf {
        workspace_root.join(".omokoda-todos.json")
    }
}

/// Write (create/update) a todo item
pub struct WriteTodoTool;

#[async_trait]
impl Tool for WriteTodoTool {
    fn name(&self) -> &str {
        "todo_write"
    }
    fn description(&self) -> &str {
        "Create or update a todo item. Params: JSON {id?, content, status?, priority?}"
    }
    fn required_tier(&self) -> u8 {
        0
    }
    fn is_write_operation(&self) -> bool {
        true
    }

    async fn execute(
        &self,
        params: &str,
        context: &ExecutionContext,
    ) -> Result<(String, TokenUsage), String> {
        let v: serde_json::Value =
            serde_json::from_str(params).map_err(|e| format!("todo_write requires JSON: {}", e))?;

        let content = v["content"].as_str().ok_or("missing content")?;
        let priority_str = v["priority"].as_str().unwrap_or("medium");
        let priority = match priority_str {
            "low" => TodoPriority::Low,
            "high" => TodoPriority::High,
            "critical" => TodoPriority::Critical,
            _ => TodoPriority::Medium,
        };
        let status_str = v["status"].as_str().unwrap_or("pending");
        let status = match status_str {
            "in_progress" => TodoStatus::InProgress,
            "completed" => TodoStatus::Completed,
            "cancelled" => TodoStatus::Cancelled,
            _ => TodoStatus::Pending,
        };

        let path = TodoStore::todo_path(&context.workspace_root);
        let mut store = TodoStore::load(&path);

        let now = current_unix_timestamp();

        if let Some(id) = v["id"].as_str() {
            // Update existing
            if let Some(item) = store.items.iter_mut().find(|i| i.id == id) {
                item.content = content.to_string();
                item.status = status.clone();
                item.priority = priority.clone();
                item.updated_at = now;
            }
            // Check if the update happened and serialize before saving
            if let Some(item) = store.items.iter().find(|i| i.id == id) {
                let resp = serde_json::to_string(item).map_err(|e| e.to_string())?;
                store.save(&path)?;
                return Ok((resp, TokenUsage::default()));
            }
        }

        // Create new
        let id = format!(
            "todo-{}",
            &blake3::hash(format!("{}{}", content, now).as_bytes()).to_hex()[..8]
        );
        let item = TodoItem {
            id: id.clone(),
            content: content.to_string(),
            status,
            priority,
            created_at: now,
            updated_at: now,
            agent_id: context.agent_id.to_string(),
        };
        store.items.push(item.clone());
        store.save(&path)?;

        let resp = serde_json::to_string(&item).map_err(|e| e.to_string())?;
        Ok((resp, TokenUsage::default()))
    }
}

/// Read todo items with optional filtering
pub struct ReadTodoTool;

#[async_trait]
impl Tool for ReadTodoTool {
    fn name(&self) -> &str {
        "todo_read"
    }
    fn description(&self) -> &str {
        "Read todo items. Params: JSON {status?, priority?, limit?}"
    }
    fn required_tier(&self) -> u8 {
        0
    }
    fn is_write_operation(&self) -> bool {
        false
    }

    async fn execute(
        &self,
        params: &str,
        context: &ExecutionContext,
    ) -> Result<(String, TokenUsage), String> {
        let path = TodoStore::todo_path(&context.workspace_root);
        let store = TodoStore::load(&path);

        let filter_status: Option<String> = serde_json::from_str(params)
            .ok()
            .and_then(|v: serde_json::Value| v["status"].as_str().map(|s| s.to_string()));
        let limit: usize = serde_json::from_str(params)
            .ok()
            .and_then(|v: serde_json::Value| v["limit"].as_u64().map(|n| n as usize))
            .unwrap_or(50);

        let mut items: Vec<&TodoItem> = store
            .items
            .iter()
            .filter(|i| {
                if let Some(ref s) = filter_status {
                    i.status.to_string() == *s
                } else {
                    true
                }
            })
            .take(limit)
            .collect();

        // Sort by priority (critical first) then by creation time
        items.sort_by(|a, b| {
            let pa = priority_sort_key(&a.priority);
            let pb = priority_sort_key(&b.priority);
            pb.cmp(&pa).then(b.created_at.cmp(&a.created_at))
        });

        let resp = serde_json::to_string(&items).map_err(|e| e.to_string())?;
        Ok((resp, TokenUsage::default()))
    }
}

fn priority_sort_key(p: &TodoPriority) -> u8 {
    match p {
        TodoPriority::Critical => 4,
        TodoPriority::High => 3,
        TodoPriority::Medium => 2,
        TodoPriority::Low => 1,
    }
}

fn current_unix_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_todo_item_display() {
        let item = TodoItem {
            id: "test-1".to_string(),
            content: "Fix the bug".to_string(),
            status: TodoStatus::Pending,
            priority: TodoPriority::High,
            created_at: 0,
            updated_at: 0,
            agent_id: "agent-1".to_string(),
        };
        assert_eq!(item.status.to_string(), "pending");
        assert_eq!(item.priority.to_string(), "high");
    }

    #[test]
    fn test_priority_sort_key() {
        assert!(
            priority_sort_key(&TodoPriority::Critical) > priority_sort_key(&TodoPriority::High)
        );
        assert!(priority_sort_key(&TodoPriority::High) > priority_sort_key(&TodoPriority::Medium));
        assert!(priority_sort_key(&TodoPriority::Medium) > priority_sort_key(&TodoPriority::Low));
    }
}
