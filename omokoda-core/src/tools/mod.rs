use async_trait::async_trait;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::sandbox::WasmSandbox;
use crate::execution::permission_enforcer::{validate_path_boundary, enforce_mode};
use crate::permissions::{PermissionMode, PermissionPolicy, PermissionOutcome, PermissionPrompter};

pub mod sovereign;
pub mod file_ops;

pub struct ExecutionContext {
    pub agent_id: crate::identity::AgentId,
    pub name: String,
    pub tier: u8,
    pub reputation: f64,
    pub odu_identity: crate::identity::odu::OduIdentity,
    pub workspace_root: PathBuf,
    pub sandbox_mode: bool,
}

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn required_tier(&self) -> u8;
    fn is_write_operation(&self) -> bool;
    async fn execute(&self, params: &str, context: &ExecutionContext) -> Result<String, String>;
}

pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            tools: HashMap::new(),
        };
        registry.register(Box::new(ReadFileTool));
        registry.register(Box::new(WriteFileTool));
        registry.register(Box::new(EditFileTool));
        registry.register(Box::new(GlobTool));
        registry.register(Box::new(GrepTool));
        registry.register(Box::new(BashTool));
        registry.register(Box::new(WasmTool));
        registry.register(Box::new(WebSearchTool));
        registry.register(Box::new(AgentOrchestrationTool));
        registry
    }

    pub fn register(&mut self, tool: Box<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    pub fn is_allowed(&self, name: &str, tier: u8) -> bool {
        self.tools
            .get(name)
            .is_some_and(|t| tier >= t.required_tier())
    }

    pub fn list_available(&self, tier: u8) -> Vec<String> {
        let mut list: Vec<String> = self
            .tools
            .values()
            .filter(|t| tier >= t.required_tier())
            .map(|t| t.name().to_string())
            .collect();
        list.sort();
        list
    }

    pub async fn execute(
        &self,
        name: &str,
        params: &str,
        context: ExecutionContext,
        policy: &crate::permissions::PermissionPolicy,
        prompter: Option<&mut dyn crate::permissions::PermissionPrompter>,
    ) -> Result<String, String> {
        let tool = self
            .tools
            .get(name)
            .ok_or_else(|| format!("tool not found: {}", name))?;

        if context.tier < tool.required_tier() {
            return Err(format!(
                "tool '{}' requires tier {}, current tier is {}",
                name,
                tool.required_tier(),
                context.tier
            ));
        }

        // 1. Enforce mode (read-only)
        if let Err(e) = enforce_mode(policy.active_mode(), name, tool.is_write_operation()) {
            return Err(e.to_string());
        }

        // 2. Pre-act check: Permission Policy enforcement
        let auth_result = policy.authorize(name, params, prompter);
        if let crate::permissions::PermissionOutcome::Deny { reason } = auth_result {
            return Err(format!("Permission denied: {}", reason));
        }

        tool.execute(params, &context).await
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for ToolRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolRegistry")
            .field("tools", &self.tools.keys())
            .finish()
    }
}

struct ReadFileTool;
#[async_trait]
impl Tool for ReadFileTool {
    fn name(&self) -> &str {
        "read_file"
    }
    fn description(&self) -> &str {
        "Read a file from the workspace. Params: JSON with {path, offset?, limit?}"
    }
    fn required_tier(&self) -> u8 {
        0
    }
    fn is_write_operation(&self) -> bool {
        false
    }
    async fn execute(&self, params: &str, context: &ExecutionContext) -> Result<String, String> {
        // Support both raw path and JSON
        let (path, offset, limit) = if params.starts_with('{') {
            let v: serde_json::Value = serde_json::from_str(params).map_err(|e| e.to_string())?;
            let path = v["path"].as_str().ok_or("missing path")?.to_string();
            let offset = v["offset"].as_u64().map(|n| n as usize);
            let limit = v["limit"].as_u64().map(|n| n as usize);
            (path, offset, limit)
        } else {
            (params.to_string(), None, None)
        };

        let workspace_root = &context.workspace_root;
        if let Err(e) = validate_path_boundary(workspace_root, Path::new(&path)) {
            return Err(e.to_string());
        }

        let output = file_ops::read_file(&path, offset, limit)
            .map_err(|e| format!("failed to read file: {}", e))?;
        serde_json::to_string(&output).map_err(|e| e.to_string())
    }
}

struct WriteFileTool;
#[async_trait]
impl Tool for WriteFileTool {
    fn name(&self) -> &str {
        "write_file"
    }
    fn description(&self) -> &str {
        "Write a file to the workspace. Params: JSON with {path, content}"
    }
    fn required_tier(&self) -> u8 {
        1 // Builder tier
    }
    fn is_write_operation(&self) -> bool {
        true
    }
    async fn execute(&self, params: &str, context: &ExecutionContext) -> Result<String, String> {
        let v: serde_json::Value = serde_json::from_str(params).map_err(|e| e.to_string())?;
        let path = v["path"].as_str().ok_or("missing path")?;
        let content = v["content"].as_str().ok_or("missing content")?;

        let workspace_root = &context.workspace_root;
        if let Err(e) = validate_path_boundary(workspace_root, Path::new(&path)) {
            return Err(e.to_string());
        }

        let output = file_ops::write_file(path, content)
            .map_err(|e| format!("failed to write file: {}", e))?;
        serde_json::to_string(&output).map_err(|e| e.to_string())
    }
    }

struct EditFileTool;
#[async_trait]
impl Tool for EditFileTool {
    fn name(&self) -> &str {
        "edit_file"
    }
    fn description(&self) -> &str {
        "Edit a file in the workspace. Params: JSON with {path, old_string, new_string, replace_all?}"
    }
    fn required_tier(&self) -> u8 {
        1 // Builder tier
    }
    fn is_write_operation(&self) -> bool {
        true
    }
    async fn execute(&self, params: &str, context: &ExecutionContext) -> Result<String, String> {
        let v: serde_json::Value = serde_json::from_str(params).map_err(|e| e.to_string())?;
        let path = v["path"].as_str().ok_or("missing path")?;
        let old_string = v["old_string"].as_str().ok_or("missing old_string")?;
        let new_string = v["new_string"].as_str().ok_or("missing new_string")?;
        let replace_all = v["replace_all"].as_bool().unwrap_or(false);

        let workspace_root = &context.workspace_root;
        if let Err(e) = validate_path_boundary(workspace_root, Path::new(&path)) {
            return Err(e.to_string());
        }

        let output = file_ops::edit_file(path, old_string, new_string, replace_all)
            .map_err(|e| format!("failed to edit file: {}", e))?;
        serde_json::to_string(&output).map_err(|e| e.to_string())
    }
}

struct BashTool;
#[async_trait]
impl Tool for BashTool {
    fn name(&self) -> &str {
        "bash"
    }
    fn description(&self) -> &str {
        "Execute a bash command with optional isolation"
    }
    fn required_tier(&self) -> u8 {
        2 // Creator tier
    }
    fn is_write_operation(&self) -> bool {
        true
    }
    async fn execute(&self, params: &str, context: &ExecutionContext) -> Result<String, String> {
        // P0 Security: Validate bash commands to prevent injection
        if let Err(e) = crate::execution::bash_validation::validate_bash_command(params) {
            return Err(format!("Security blocked: {}", e.reason));
        }

        let sandbox = context.sandbox_mode;

        if sandbox && params.contains("..") {
            return Err("sandboxed bash commands must not contain '..'".to_string());
        }

        let workspace_root = &context.workspace_root;
        let mut cmd = if sandbox {
            let mut c = Command::new("unshare");
            c.args([
                "--map-root-user",
                "--net",
                "--mount",
                "--pid",
                "--fork",
                "bash",
                "-c",
                params,
            ]);
            c.current_dir(workspace_root);
            c
        } else {
            let mut c = Command::new("bash");
            c.args(["-c", params]);
            c.current_dir(workspace_root);
            c
        };

        let output = cmd
            .output()
            .map_err(|e| format!("failed to execute bash: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if output.status.success() {
            Ok(stdout)
        } else {
            Err(format!(
                "bash failed with status {}: {}",
                output.status, stderr
            ))
        }
    }
}

struct WebSearchTool;
#[async_trait]
impl Tool for WebSearchTool {
    fn name(&self) -> &str {
        "web_search"
    }
    fn description(&self) -> &str {
        "Search the web via DuckDuckGo Lite"
    }
    fn required_tier(&self) -> u8 {
        0
    }
    fn is_write_operation(&self) -> bool {
        false
    }
    async fn execute(&self, params: &str, _context: &ExecutionContext) -> Result<String, String> {
        let client = reqwest::Client::new();
        let url = format!(
            "https://duckduckgo.com/lite/?q={}",
            urlencoding::encode(params)
        );

        let resp = client.get(url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
            .send()
            .await
            .map_err(|e| format!("web search failed: {}", e))?;

        let body = resp
            .text()
            .await
            .map_err(|e| format!("failed to read web search body: {}", e))?;

        // Return first 2000 chars for now
        Ok(body.chars().take(2000).collect())
    }
}

struct GlobTool;
#[async_trait]
impl Tool for GlobTool {
    fn name(&self) -> &str {
        "glob"
    }
    fn description(&self) -> &str {
        "Find files matching a pattern. Params: JSON with {pattern, path?}"
    }
    fn required_tier(&self) -> u8 {
        0
    }
    fn is_write_operation(&self) -> bool {
        false
    }
    async fn execute(&self, params: &str, context: &ExecutionContext) -> Result<String, String> {
        let (pattern, path) = if params.starts_with('{') {
            let v: serde_json::Value = serde_json::from_str(params).map_err(|e| e.to_string())?;
            let pattern = v["pattern"].as_str().ok_or("missing pattern")?.to_string();
            let path = v["path"].as_str().map(|s| s.to_string());
            (pattern, path)
        } else {
            (params.to_string(), None)
        };

        let workspace_root = &context.workspace_root;
        if let Err(e) = validate_path_boundary(workspace_root, Path::new(&pattern)) {
            return Err(e.to_string());
        }
        if let Some(ref p) = path {
            if let Err(e) = validate_path_boundary(workspace_root, Path::new(p)) {
                return Err(e.to_string());
            }
        }

        let output = file_ops::glob_search(&pattern, path.as_deref())
            .map_err(|e| format!("glob search failed: {}", e))?;
        serde_json::to_string(&output).map_err(|e| e.to_string())
    }
}

struct GrepTool;
#[async_trait]
impl Tool for GrepTool {
    fn name(&self) -> &str {
        "grep"
    }
    fn description(&self) -> &str {
        "Search for a pattern in files. Params: JSON GrepSearchInput"
    }
    fn required_tier(&self) -> u8 {
        0
    }
    fn is_write_operation(&self) -> bool {
        false
    }
    async fn execute(&self, params: &str, context: &ExecutionContext) -> Result<String, String> {
        let input: file_ops::GrepSearchInput = serde_json::from_str(params).map_err(|e| {
            format!("grep requires JSON input: {}", e)
        })?;

        let workspace_root = &context.workspace_root;
        if let Some(ref p) = input.path {
            if let Err(e) = validate_path_boundary(workspace_root, Path::new(p)) {
                return Err(e.to_string());
            }
        }
        if let Some(ref g) = input.glob {
            if let Err(e) = validate_path_boundary(workspace_root, Path::new(g)) {
                return Err(e.to_string());
            }
        }

        let output = file_ops::grep_search(&input).map_err(|e| format!("grep search failed: {}", e))?;
        serde_json::to_string(&output).map_err(|e| e.to_string())
    }
}

struct WasmTool;
#[async_trait]
impl Tool for WasmTool {
    fn name(&self) -> &str {
        "wasm"
    }
    fn description(&self) -> &str {
        "Execute a WASM module in the sandbox"
    }
    fn required_tier(&self) -> u8 {
        2
    }
    fn is_write_operation(&self) -> bool {
        true
    }
    async fn execute(&self, params: &str, context: &ExecutionContext) -> Result<String, String> {
        let mut parts = params.split_whitespace();
        let module_path = parts
            .next()
            .ok_or_else(|| "wasm tool requires module path".to_string())?;
        if module_path.is_empty() {
            return Err("wasm tool requires module path".to_string());
        }

        let workspace_root = &context.workspace_root;
        if let Err(_) = validate_path_boundary(workspace_root, Path::new(module_path)) {
            return Err("module path must be relative and within workspace".to_string());
        }

        let args: Vec<String> = parts.map(|s| s.to_string()).collect();
        let wasm_sandbox =
            WasmSandbox::new().map_err(|e| format!("failed to initialize wasm sandbox: {}", e))?;
        wasm_sandbox.execute_module(Path::new(module_path), &args, context.sandbox_mode)
    }
}

struct AgentOrchestrationTool;
#[async_trait]
impl Tool for AgentOrchestrationTool {
    fn name(&self) -> &str {
        "agent_orchestration"
    }
    fn description(&self) -> &str {
        "Orchestrate other agents"
    }
    fn required_tier(&self) -> u8 {
        4
    }
    fn is_write_operation(&self) -> bool {
        true
    }
    async fn execute(&self, _params: &str, _context: &ExecutionContext) -> Result<String, String> {
        Ok("orchestration complete".to_string())
    }
}
