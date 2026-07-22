use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReflectionEntry {
    pub timestamp: u64,
    pub primitive: String,
    pub content: String,
    pub emotion: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ReflectionLedger {
    pub entries: Vec<ReflectionEntry>,
}

impl ReflectionLedger {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a primitive + content with no emotional tag. Prefer
    /// [`Self::record_with_emotion`] when an `EmotionState` is already in
    /// scope (the interpreter's public-think path always has one, computed
    /// for SOMA storage) -- this is the fallback for call sites that don't.
    pub fn record(&mut self, primitive: &str, content: &str, timestamp: u64) {
        self.entries.push(ReflectionEntry {
            timestamp,
            primitive: primitive.to_string(),
            content: content.to_string(),
            emotion: None,
        });
    }

    /// Record with the agent's `EmotionState` at this moment, rendered as a
    /// compact string. This is what makes the ledger genuinely distinct
    /// from `odu_dir`/the receipt chain -- neither carries emotional state
    /// alongside memory content.
    pub fn record_with_emotion(
        &mut self,
        primitive: &str,
        content: &str,
        timestamp: u64,
        emotion: &crate::emotion::EmotionState,
    ) {
        self.entries.push(ReflectionEntry {
            timestamp,
            primitive: primitive.to_string(),
            content: content.to_string(),
            emotion: Some(format!(
                "energy={:.2} tension={:.2} connection={:.2} focus={:.2}",
                emotion.energy, emotion.tension, emotion.connection, emotion.focus
            )),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_with_emotion_renders_all_four_fields() {
        let mut ledger = ReflectionLedger::new();
        let emotion = crate::emotion::EmotionState {
            energy: 0.5,
            tension: 0.25,
            connection: 0.75,
            focus: 1.0,
        };
        ledger.record_with_emotion("think", "hello", 100, &emotion);
        assert_eq!(ledger.entries.len(), 1);
        let e = &ledger.entries[0];
        assert_eq!(e.primitive, "think");
        assert_eq!(e.content, "hello");
        assert_eq!(e.timestamp, 100);
        let tag = e.emotion.as_deref().unwrap();
        assert!(tag.contains("energy=0.50"));
        assert!(tag.contains("tension=0.25"));
        assert!(tag.contains("connection=0.75"));
        assert!(tag.contains("focus=1.00"));
    }

    #[test]
    fn record_without_emotion_leaves_it_none() {
        let mut ledger = ReflectionLedger::new();
        ledger.record("act", "did a thing", 50);
        assert_eq!(ledger.entries[0].emotion, None);
    }
}
