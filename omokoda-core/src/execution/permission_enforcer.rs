//! Permission Enforcement Layer
//! Enforces workspace boundaries, read-only mode, and agent tier constraints.

use crate::config::AgentTier;
use crate::permissions::PermissionMode;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum EnforcementError {
    #[error("Boundary Violation: Attempted to access {0}")]
    BoundaryViolation(PathBuf),
    #[error("Read-Only Violation: Attempted {0} in read-only mode")]
    ReadOnlyViolation(String),
    #[error("Private Access Violation: Attempted to access {0}")]
    PrivateAccessViolation(PathBuf),
    #[error("Tier Violation: {0} requires tier {1} but agent is {2}")]
    TierViolation(String, String, String),
}

/// Validates that a path is within the workspace root and not within /private.
pub fn validate_path_boundary(root: &Path, target: &Path) -> Result<PathBuf, EnforcementError> {
    // 1. Enforce /private boundary in runtime
    let target_str = target.to_string_lossy();
    if target_str.starts_with("/private")
        || target_str.starts_with("private/")
        || target_str == "private"
    {
        return Err(EnforcementError::PrivateAccessViolation(
            target.to_path_buf(),
        ));
    }

    let canonical_root = root
        .canonicalize()
        .map_err(|_| EnforcementError::BoundaryViolation(root.to_path_buf()))?;

    // Join handles relative paths; if absolute, target replaces base.
    let full_target = if target.is_absolute() {
        target.to_path_buf()
    } else {
        root.join(target)
    };

    // Normalize path components to prevent path traversal (e.g., ../../)
    let mut normalized_target = PathBuf::new();
    for component in full_target.components() {
        match component {
            std::path::Component::Normal(c) => normalized_target.push(c),
            std::path::Component::ParentDir => {
                normalized_target.pop();
            }
            std::path::Component::RootDir => {
                normalized_target = PathBuf::from(std::path::Component::RootDir.as_os_str());
            }
            _ => {}
        }
    }

    // Canonicalize normalized target for final boundary check
    let canonical_target = match std::fs::canonicalize(&normalized_target) {
        Ok(path) => path,
        Err(_) => normalized_target, // Might not exist yet
    };

    if canonical_target.starts_with(&canonical_root) {
        Ok(canonical_target)
    } else {
        Err(EnforcementError::BoundaryViolation(target.to_path_buf()))
    }
}

/// Enforces mode constraints based on the tool and requested action.
pub fn enforce_mode(
    mode: PermissionMode,
    action: &str,
    is_write: bool,
) -> Result<(), EnforcementError> {
    if mode == PermissionMode::ReadOnly && is_write {
        return Err(EnforcementError::ReadOnlyViolation(action.to_string()));
    }
    Ok(())
}

/// Enforces tier-based capability constraints.
/// - Tool execution requires Resident (tier >= 2) or higher.
/// - Self-modification (skill_patch) requires Sovereign (tier 3).
pub fn enforce_tier(
    tier: AgentTier,
    action: &str,
    is_self_modify: bool,
) -> Result<(), EnforcementError> {
    if is_self_modify && !tier.can_self_modify() {
        return Err(EnforcementError::TierViolation(
            action.to_string(),
            AgentTier::Sovereign.label().to_string(),
            tier.label().to_string(),
        ));
    }
    if !is_self_modify && !tier.can_execute_tools() {
        return Err(EnforcementError::TierViolation(
            action.to_string(),
            AgentTier::Resident.label().to_string(),
            tier.label().to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tier_tests {
    use super::*;

    #[test]
    fn observer_cannot_execute_tools() {
        let err = enforce_tier(AgentTier::Observer, "bash", false);
        assert!(matches!(err, Err(EnforcementError::TierViolation(..))));
    }

    #[test]
    fn resident_can_execute_tools() {
        assert!(enforce_tier(AgentTier::Resident, "bash", false).is_ok());
    }

    #[test]
    fn resident_cannot_self_modify() {
        let err = enforce_tier(AgentTier::Resident, "skill_patch", true);
        assert!(matches!(err, Err(EnforcementError::TierViolation(..))));
    }

    #[test]
    fn sovereign_can_self_modify() {
        assert!(enforce_tier(AgentTier::Sovereign, "skill_patch", true).is_ok());
    }
}
