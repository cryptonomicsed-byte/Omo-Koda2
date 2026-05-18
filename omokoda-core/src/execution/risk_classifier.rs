//! Layer 4: AutoModeClassifier
//! Dynamically assesses the risk of a tool request and suggests security posture.

use crate::permissions::PermissionMode;

pub enum RiskLevel {
    Safe,
    Moderate,
    High,
    Critical,
}

pub struct AutoModeClassifier;

impl AutoModeClassifier {
    /// Classifies the risk level of a tool request based on intent and metadata.
    pub fn classify_risk(tool_name: &str, input: &str) -> RiskLevel {
        if tool_name.contains("bash") || tool_name.contains("wasm") {
            if input.contains("sudo") || input.contains("rm") || input.contains("network") {
                return RiskLevel::Critical;
            }
            return RiskLevel::High;
        }
        
        if tool_name.contains("read") || tool_name.contains("search") {
            return RiskLevel::Safe;
        }

        RiskLevel::Moderate
    }

    /// Determines the required mode based on risk level.
    pub fn suggest_mode(risk: RiskLevel) -> PermissionMode {
        match risk {
            RiskLevel::Safe => PermissionMode::ReadOnly,
            RiskLevel::Moderate => PermissionMode::WorkspaceWrite,
            RiskLevel::High => PermissionMode::Prompt,
            RiskLevel::Critical => PermissionMode::DangerFullAccess,
        }
    }
}
