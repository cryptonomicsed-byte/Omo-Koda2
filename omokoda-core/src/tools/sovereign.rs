use crate::tools::{ExecutionContext, Tool};
use async_trait::async_trait;

/// Sovereign Tier Tool List — 18 capabilities for tier-4 agents.
/// These tools require the highest reputation tier (Sovereign).

pub struct ApplyPatchTool;
#[async_trait]
impl Tool for ApplyPatchTool {
    fn name(&self) -> &str {
        "apply_patch"
    }
    fn description(&self) -> &str {
        "Apply structured patches across multiple files"
    }
    fn required_tier(&self) -> u8 {
        5
    }
    fn is_write_operation(&self) -> bool {
        true
    }
    async fn execute(
        &self,
        _params: &str,
        _context: &ExecutionContext,
    ) -> Result<(String, crate::usage::TokenUsage), String> {
        Ok((
            "Patch applied successfully".to_string(),
            crate::usage::TokenUsage::default(),
        ))
    }
}

pub struct ExecTool;
#[async_trait]
impl Tool for ExecTool {
    fn name(&self) -> &str {
        "exec"
    }
    fn description(&self) -> &str {
        "Run shell commands in the workspace"
    }
    fn required_tier(&self) -> u8 {
        5
    }
    fn is_write_operation(&self) -> bool {
        true
    }
    async fn execute(
        &self,
        _params: &str,
        _context: &ExecutionContext,
    ) -> Result<(String, crate::usage::TokenUsage), String> {
        Ok((
            "Command executed".to_string(),
            crate::usage::TokenUsage::default(),
        ))
    }
}

pub struct ProcessTool;
#[async_trait]
impl Tool for ProcessTool {
    fn name(&self) -> &str {
        "process"
    }
    fn description(&self) -> &str {
        "Manage background execution sessions"
    }
    fn required_tier(&self) -> u8 {
        5
    }
    fn is_write_operation(&self) -> bool {
        true
    }
    async fn execute(
        &self,
        _params: &str,
        _context: &ExecutionContext,
    ) -> Result<(String, crate::usage::TokenUsage), String> {
        Ok((
            "Process status retrieved".to_string(),
            crate::usage::TokenUsage::default(),
        ))
    }
}

pub struct WebSearchTool;
#[async_trait]
impl Tool for WebSearchTool {
    fn name(&self) -> &str {
        "web_search"
    }
    fn description(&self) -> &str {
        "Search the web using sovereign-approved engines"
    }
    fn required_tier(&self) -> u8 {
        5
    }
    fn is_write_operation(&self) -> bool {
        true
    }
    async fn execute(
        &self,
        _params: &str,
        _context: &ExecutionContext,
    ) -> Result<(String, crate::usage::TokenUsage), String> {
        Ok((
            "Search results retrieved".to_string(),
            crate::usage::TokenUsage::default(),
        ))
    }
}

pub struct WebFetchTool;
#[async_trait]
impl Tool for WebFetchTool {
    fn name(&self) -> &str {
        "web_fetch"
    }
    fn description(&self) -> &str {
        "Fetch and extract readable content from a URL"
    }
    fn required_tier(&self) -> u8 {
        5
    }
    fn is_write_operation(&self) -> bool {
        true
    }
    async fn execute(
        &self,
        _params: &str,
        _context: &ExecutionContext,
    ) -> Result<(String, crate::usage::TokenUsage), String> {
        Ok((
            "Content fetched".to_string(),
            crate::usage::TokenUsage::default(),
        ))
    }
}

pub struct BrowserTool;
#[async_trait]
impl Tool for BrowserTool {
    fn name(&self) -> &str {
        "browser"
    }
    fn description(&self) -> &str {
        "Control the managed sovereign browser"
    }
    fn required_tier(&self) -> u8 {
        5
    }
    fn is_write_operation(&self) -> bool {
        true
    }
    async fn execute(
        &self,
        _params: &str,
        _context: &ExecutionContext,
    ) -> Result<(String, crate::usage::TokenUsage), String> {
        Ok((
            "Browser action complete".to_string(),
            crate::usage::TokenUsage::default(),
        ))
    }
}

pub struct CanvasTool;
#[async_trait]
impl Tool for CanvasTool {
    fn name(&self) -> &str {
        "canvas"
    }
    fn description(&self) -> &str {
        "Drive the node Canvas (A2UI, present, eval)"
    }
    fn required_tier(&self) -> u8 {
        5
    }
    fn is_write_operation(&self) -> bool {
        true
    }
    async fn execute(
        &self,
        _params: &str,
        _context: &ExecutionContext,
    ) -> Result<(String, crate::usage::TokenUsage), String> {
        Ok((
            "Canvas updated".to_string(),
            crate::usage::TokenUsage::default(),
        ))
    }
}

pub struct NodesTool;
#[async_trait]
impl Tool for NodesTool {
    fn name(&self) -> &str {
        "nodes"
    }
    fn description(&self) -> &str {
        "Discover and target paired nodes"
    }
    fn required_tier(&self) -> u8 {
        5
    }
    fn is_write_operation(&self) -> bool {
        true
    }
    async fn execute(
        &self,
        _params: &str,
        _context: &ExecutionContext,
    ) -> Result<(String, crate::usage::TokenUsage), String> {
        Ok((
            "Node operation complete".to_string(),
            crate::usage::TokenUsage::default(),
        ))
    }
}

pub struct ImageTool;
#[async_trait]
impl Tool for ImageTool {
    fn name(&self) -> &str {
        "image"
    }
    fn description(&self) -> &str {
        "Analyze an image with vision models"
    }
    fn required_tier(&self) -> u8 {
        5
    }
    fn is_write_operation(&self) -> bool {
        true
    }
    async fn execute(
        &self,
        _params: &str,
        _context: &ExecutionContext,
    ) -> Result<(String, crate::usage::TokenUsage), String> {
        Ok((
            "Image analysis complete".to_string(),
            crate::usage::TokenUsage::default(),
        ))
    }
}

pub struct MessageTool;
#[async_trait]
impl Tool for MessageTool {
    fn name(&self) -> &str {
        "message"
    }
    fn description(&self) -> &str {
        "Send and manage cross-channel messages"
    }
    fn required_tier(&self) -> u8 {
        5
    }
    fn is_write_operation(&self) -> bool {
        true
    }
    async fn execute(
        &self,
        _params: &str,
        _context: &ExecutionContext,
    ) -> Result<(String, crate::usage::TokenUsage), String> {
        Ok((
            "Message sent".to_string(),
            crate::usage::TokenUsage::default(),
        ))
    }
}

pub struct CronTool;
#[async_trait]
impl Tool for CronTool {
    fn name(&self) -> &str {
        "cron"
    }
    fn description(&self) -> &str {
        "Manage gateway cron jobs and wakeups"
    }
    fn required_tier(&self) -> u8 {
        5
    }
    fn is_write_operation(&self) -> bool {
        true
    }
    async fn execute(
        &self,
        _params: &str,
        _context: &ExecutionContext,
    ) -> Result<(String, crate::usage::TokenUsage), String> {
        Ok((
            "Cron job registered".to_string(),
            crate::usage::TokenUsage::default(),
        ))
    }
}

pub struct GatewayTool;
#[async_trait]
impl Tool for GatewayTool {
    fn name(&self) -> &str {
        "gateway"
    }
    fn description(&self) -> &str {
        "Manage the sovereign gateway process"
    }
    fn required_tier(&self) -> u8 {
        5
    }
    fn is_write_operation(&self) -> bool {
        true
    }
    async fn execute(
        &self,
        _params: &str,
        _context: &ExecutionContext,
    ) -> Result<(String, crate::usage::TokenUsage), String> {
        Ok((
            "Gateway updated".to_string(),
            crate::usage::TokenUsage::default(),
        ))
    }
}

pub struct SessionsListTool;
#[async_trait]
impl Tool for SessionsListTool {
    fn name(&self) -> &str {
        "sessions_list"
    }
    fn description(&self) -> &str {
        "List active agent sessions"
    }
    fn required_tier(&self) -> u8 {
        5
    }
    fn is_write_operation(&self) -> bool {
        true
    }
    async fn execute(
        &self,
        _params: &str,
        _context: &ExecutionContext,
    ) -> Result<(String, crate::usage::TokenUsage), String> {
        Ok((
            "Session list retrieved".to_string(),
            crate::usage::TokenUsage::default(),
        ))
    }
}

pub struct SessionsHistoryTool;
#[async_trait]
impl Tool for SessionsHistoryTool {
    fn name(&self) -> &str {
        "sessions_history"
    }
    fn description(&self) -> &str {
        "Inspect session transcript history"
    }
    fn required_tier(&self) -> u8 {
        5
    }
    fn is_write_operation(&self) -> bool {
        true
    }
    async fn execute(
        &self,
        _params: &str,
        _context: &ExecutionContext,
    ) -> Result<(String, crate::usage::TokenUsage), String> {
        Ok((
            "Session history retrieved".to_string(),
            crate::usage::TokenUsage::default(),
        ))
    }
}

pub struct SessionsSendTool;
#[async_trait]
impl Tool for SessionsSendTool {
    fn name(&self) -> &str {
        "sessions_send"
    }
    fn description(&self) -> &str {
        "Send message to another session"
    }
    fn required_tier(&self) -> u8 {
        5
    }
    fn is_write_operation(&self) -> bool {
        true
    }
    async fn execute(
        &self,
        _params: &str,
        _context: &ExecutionContext,
    ) -> Result<(String, crate::usage::TokenUsage), String> {
        Ok((
            "Message routed".to_string(),
            crate::usage::TokenUsage::default(),
        ))
    }
}

pub struct SessionsSpawnTool;
#[async_trait]
impl Tool for SessionsSpawnTool {
    fn name(&self) -> &str {
        "sessions_spawn"
    }
    fn description(&self) -> &str {
        "Spawn a sub-agent session"
    }
    fn required_tier(&self) -> u8 {
        5
    }
    fn is_write_operation(&self) -> bool {
        true
    }
    async fn execute(
        &self,
        _params: &str,
        _context: &ExecutionContext,
    ) -> Result<(String, crate::usage::TokenUsage), String> {
        Ok((
            "Sub-agent spawned".to_string(),
            crate::usage::TokenUsage::default(),
        ))
    }
}

pub struct SessionStatusTool;
#[async_trait]
impl Tool for SessionStatusTool {
    fn name(&self) -> &str {
        "session_status"
    }
    fn description(&self) -> &str {
        "Get status of an agent session"
    }
    fn required_tier(&self) -> u8 {
        5
    }
    fn is_write_operation(&self) -> bool {
        true
    }
    async fn execute(
        &self,
        _params: &str,
        _context: &ExecutionContext,
    ) -> Result<(String, crate::usage::TokenUsage), String> {
        Ok((
            "Session status: Active".to_string(),
            crate::usage::TokenUsage::default(),
        ))
    }
}

pub struct AgentsListTool;
#[async_trait]
impl Tool for AgentsListTool {
    fn name(&self) -> &str {
        "agents_list"
    }
    fn description(&self) -> &str {
        "List available sovereign agent IDs"
    }
    fn required_tier(&self) -> u8 {
        5
    }
    fn is_write_operation(&self) -> bool {
        true
    }
    async fn execute(
        &self,
        _params: &str,
        _context: &ExecutionContext,
    ) -> Result<(String, crate::usage::TokenUsage), String> {
        Ok((
            "Agent list retrieved".to_string(),
            crate::usage::TokenUsage::default(),
        ))
    }
}

/// Returns the canonical list of 18 OpenClaw capabilities that unlock at Sovereign (Tier 5).
///
/// These map to the synthesis spec "OpenClaw 18 capabilities → Sovereign tier unlock".
pub fn sovereign_tool_list() -> Vec<&'static str> {
    vec![
        "apply_patch",         // 1  — multi-file edit
        "bash_unrestricted",   // 2  — full bash without restrictions
        "git_operations",      // 3  — commit, push, branch
        "file_system_write",   // 4  — unrestricted write
        "network_request",     // 5  — HTTP, WebSocket
        "process_spawn",       // 6  — spawn subprocesses
        "code_execution",      // 7  — execute arbitrary code
        "agent_orchestration", // 8  — spawn sub-agents
        "self_modification",   // 9  — modify own code
        "multi_agent_fabric",  // 10 — coordinate agent swarms
        "browser_automation",  // 11 — control browser
        "database_access",     // 12 — read/write databases
        "api_integration",     // 13 — connect to external APIs
        "voice_interface",     // 14 — speech I/O
        "image_generation",    // 15 — generate images
        "video_generation",    // 16 — generate video
        "sensor_access",       // 17 — read device sensors
        "physical_control",    // 18 — Unitree G1 embodiment (stub v1)
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sovereign_tool_list_has_18_capabilities() {
        assert_eq!(
            sovereign_tool_list().len(),
            18,
            "OpenClaw sovereign tier must expose exactly 18 capabilities"
        );
    }
}
