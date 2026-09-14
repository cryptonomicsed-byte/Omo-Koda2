use serde::{Deserialize, Serialize};

/// Extended lifecycle stage for a sovereign agent.
/// Extends AgentStatus (sub-agent supervision) with long-horizon states.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentLifecycleStage {
    /// Born, never thought or acted.
    Nascent,
    /// Active — thinking and acting normally.
    Active,
    /// Moving to a new host or deployment.
    Migration { destination_hint: Option<String> },
    /// Dormant — not accepting new tasks but resumable.
    Hibernation,
    /// Permanently ceased — no new sessions, memory sealed.
    Retirement,
    /// Forcibly deactivated by governance or security event.
    Revoked { reason: String },
}

impl Default for AgentLifecycleStage {
    fn default() -> Self { AgentLifecycleStage::Nascent }
}

impl AgentLifecycleStage {
    pub fn is_active(&self) -> bool {
        matches!(self, AgentLifecycleStage::Active)
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, AgentLifecycleStage::Retirement | AgentLifecycleStage::Revoked { .. })
    }

    pub fn can_accept_tasks(&self) -> bool {
        matches!(self, AgentLifecycleStage::Nascent | AgentLifecycleStage::Active)
    }

    pub fn stage_name(&self) -> &'static str {
        match self {
            AgentLifecycleStage::Nascent           => "nascent",
            AgentLifecycleStage::Active            => "active",
            AgentLifecycleStage::Migration { .. }  => "migration",
            AgentLifecycleStage::Hibernation       => "hibernation",
            AgentLifecycleStage::Retirement        => "retirement",
            AgentLifecycleStage::Revoked { .. }    => "revoked",
        }
    }
}

/// Lifecycle transition request — checked before applying.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleTransition {
    pub agent_id:   String,
    pub from:       AgentLifecycleStage,
    pub to:         AgentLifecycleStage,
    pub authorized_by: Option<String>,
    pub timestamp:  u64,
}

/// Returns Ok(()) if the transition from → to is valid.
pub fn validate_transition(
    from: &AgentLifecycleStage,
    to:   &AgentLifecycleStage,
) -> Result<(), String> {
    use AgentLifecycleStage::*;
    match (from, to) {
        (Nascent, Active)           => Ok(()),
        (Active, Migration { .. })  => Ok(()),
        (Active, Hibernation)       => Ok(()),
        (Active, Retirement)        => Ok(()),
        (Active, Revoked { .. })    => Ok(()),
        (Migration { .. }, Active)  => Ok(()),
        (Hibernation, Active)       => Ok(()),
        (_, Revoked { .. })         => Ok(()), // any → revoked is always legal
        (Retirement, _)             => Err("retired agents cannot transition".to_string()),
        (Revoked { .. }, _)         => Err("revoked agents cannot transition".to_string()),
        (f, t) => Err(format!("invalid transition: {:?} → {:?}", f.stage_name(), t.stage_name())),
    }
}
