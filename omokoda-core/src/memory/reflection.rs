use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectionEntry {
    pub timestamp: u64,
    pub primitive: String,
    pub content: String,
    pub emotion: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReflectionLedger {
    pub entries: Vec<ReflectionEntry>,
}

impl ReflectionLedger {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(&mut self, primitive: &str, content: &str, timestamp: u64) {
        self.entries.push(ReflectionEntry {
            timestamp,
            primitive: primitive.to_string(),
            content: content.to_string(),
            emotion: None,
        });
    }
}
