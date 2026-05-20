//! Permission Enforcement Layer
//! Enforces workspace boundaries and read-only mode constraints.

use crate::permissions::PermissionMode;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum EnforcementError {
    #[error("Boundary Violation: Attempted to access {0}")]
    BoundaryViolation(PathBuf),
    #[error("Read-Only Violation: Attempted {0} in read-only mode")]
    ReadOnlyViolation(String),
}

/// Validates that a path is within the workspace root.
pub fn validate_path_boundary(root: &Path, target: &Path) -> Result<PathBuf, EnforcementError> {
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
