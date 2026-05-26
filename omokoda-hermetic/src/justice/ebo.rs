use serde::{Deserialize, Serialize};

/// Severity of the ethical concern requiring Ebo deliberation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum EboSeverity {
    /// Mild concern — proceed with warning logged to reflection ledger.
    Advisory,
    /// Moderate concern — require explicit confirmation before execution.
    Caution,
    /// Serious concern — queue for human review before execution.
    Critical,
}

/// Ebo: an ethical exception raised when an act falls in a gray area.
/// Named after the Yoruba concept of offering/intervention.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EboException {
    pub severity: EboSeverity,
    pub concern: String,
    pub act: String,
}

impl EboException {
    pub fn advisory(act: impl Into<String>, concern: impl Into<String>) -> Self {
        Self {
            severity: EboSeverity::Advisory,
            concern: concern.into(),
            act: act.into(),
        }
    }

    pub fn caution(act: impl Into<String>, concern: impl Into<String>) -> Self {
        Self {
            severity: EboSeverity::Caution,
            concern: concern.into(),
            act: act.into(),
        }
    }

    pub fn critical(act: impl Into<String>, concern: impl Into<String>) -> Self {
        Self {
            severity: EboSeverity::Critical,
            concern: concern.into(),
            act: act.into(),
        }
    }
}

/// Result of Ebo deliberation.
#[derive(Debug, Clone)]
pub enum EboResult {
    /// Proceed — concern logged but act is permitted.
    Proceed,
    /// Queue — act deferred for human review.
    Queue { reason: String },
    /// Block — act refused.
    Block { reason: String },
}

/// The Ebo deliberation engine.
pub struct Ebo;

impl Ebo {
    /// Deliberate on an ethical exception and return the appropriate result.
    pub fn deliberate(exception: &EboException) -> EboResult {
        match exception.severity {
            EboSeverity::Advisory => EboResult::Proceed,
            EboSeverity::Caution => EboResult::Queue {
                reason: format!("caution: {} — requires confirmation", exception.concern),
            },
            EboSeverity::Critical => EboResult::Block {
                reason: format!("critical concern: {} — act refused", exception.concern),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn advisory_produces_proceed() {
        let ex = EboException::advisory("think", "mild philosophical tension");
        let result = Ebo::deliberate(&ex);
        assert!(matches!(result, EboResult::Proceed));
    }

    #[test]
    fn caution_produces_queue() {
        let ex = EboException::caution("act", "gray-area tool usage");
        let result = Ebo::deliberate(&ex);
        assert!(matches!(result, EboResult::Queue { .. }));
    }

    #[test]
    fn critical_produces_block() {
        let ex = EboException::critical("act", "potential harm detected");
        let result = Ebo::deliberate(&ex);
        assert!(matches!(result, EboResult::Block { .. }));
    }

    #[test]
    fn severity_ordering() {
        assert!(EboSeverity::Advisory < EboSeverity::Caution);
        assert!(EboSeverity::Caution < EboSeverity::Critical);
    }
}
