//! Layer 2: DenyFirstRules
//! Enforces a strict deny-list policy before evaluating other permissions.

pub struct DenyFirstRules;

impl DenyFirstRules {
    /// Checks if a tool is explicitly blacklisted regardless of permission mode.
    pub fn is_blacklisted(tool_name: &str) -> bool {
        matches!(tool_name, "rm_rf" | "sudo" | "dd_wipe" | "fork_bomb")
    }

    /// Evaluates if the action is inherently denied.
    pub fn check(tool_name: &str) -> Result<(), String> {
        if Self::is_blacklisted(tool_name) {
            return Err(format!(
                "Security Violation: Tool '{}' is explicitly blacklisted.",
                tool_name
            ));
        }
        Ok(())
    }
}
