use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProbationLevel {
    Observe,
    Restricted,
    Quarantined,
}

impl ProbationLevel {
    pub fn escalate(&self) -> Self {
        match self {
            Self::Observe => Self::Restricted,
            Self::Restricted => Self::Quarantined,
            Self::Quarantined => Self::Quarantined,
        }
    }

    pub fn de_escalate(&self) -> Option<Self> {
        match self {
            Self::Quarantined => Some(Self::Restricted),
            Self::Restricted => Some(Self::Observe),
            Self::Observe => None,
        }
    }
}

pub struct ProbationEntry {
    pub level: ProbationLevel,
    pub reason: String,
    pub since: u64,
    pub escalations: u32,
}

pub struct ProbationManager {
    entries: HashMap<String, ProbationEntry>,
}

impl ProbationManager {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn escalate(&mut self, agent_id: &str, reason: &str) -> &ProbationLevel {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let entry = self
            .entries
            .entry(agent_id.to_string())
            .or_insert_with(|| ProbationEntry {
                level: ProbationLevel::Observe,
                reason: reason.to_string(),
                since: now,
                escalations: 0,
            });
        entry.level = entry.level.escalate();
        entry.reason = reason.to_string();
        entry.escalations += 1;
        &entry.level
    }

    pub fn clear(&mut self, agent_id: &str) {
        self.entries.remove(agent_id);
    }

    pub fn de_escalate(&mut self, agent_id: &str) {
        if let Some(entry) = self.entries.get_mut(agent_id) {
            match entry.level.de_escalate() {
                Some(next) => entry.level = next,
                None => {
                    self.entries.remove(agent_id);
                }
            }
        }
    }

    pub fn is_quarantined(&self, agent_id: &str) -> bool {
        self.entries
            .get(agent_id)
            .map(|e| e.level == ProbationLevel::Quarantined)
            .unwrap_or(false)
    }

    pub fn level(&self, agent_id: &str) -> Option<&ProbationLevel> {
        self.entries.get(agent_id).map(|e| &e.level)
    }

    pub fn all_on_probation(&self) -> Vec<(&str, &ProbationLevel)> {
        self.entries
            .iter()
            .map(|(id, e)| (id.as_str(), &e.level))
            .collect()
    }
}

impl Default for ProbationManager {
    fn default() -> Self {
        Self::new()
    }
}
