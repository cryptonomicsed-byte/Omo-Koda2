use async_trait::async_trait;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn required_tier(&self) -> u8;
    async fn execute(&self, params: &str, sandbox: bool) -> Result<String, String>;
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
        registry.register(Box::new(BashTool));
        registry.register(Box::new(WebSearchTool));
        registry.register(Box::new(AgentOrchestrationTool));
        // Keep these for backward compatibility in tests if needed, or remove if stubs
        registry.register(Box::new(GlobTool));
        registry.register(Box::new(GrepTool));
        registry
    }

    pub fn register(&mut self, tool: Box<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    pub fn is_allowed(&self, name: &str, tier: u8) -> bool {
        self.tools.get(name).map_or(false, |t| tier >= t.required_tier())
    }

    pub fn list_available(&self, tier: u8) -> Vec<String> {
        let mut list: Vec<String> = self.tools.values()
            .filter(|t| tier >= t.required_tier())
            .map(|t| t.name().to_string())
            .collect();
        list.sort();
        list
    }

    pub async fn execute(&self, name: &str, params: &str, sandbox: bool, current_tier: u8) -> Result<String, String> {

        let tool = self.tools
            .get(name)
            .ok_or_else(|| format!("tool not found: {}", name))?;
            
        if current_tier < tool.required_tier() {
            return Err(format!("tool '{}' requires tier {}, current tier is {}", name, tool.required_tier(), current_tier));
        }
        
        tool.execute(params, sandbox).await
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
        "Read a file from the workspace"
    }
    fn required_tier(&self) -> u8 {
        0
    }
    async fn execute(&self, params: &str, _sandbox: bool) -> Result<String, String> {
        let path = Path::new(params);
        if path.is_absolute() || params.contains("..") {
            return Err("path must be relative and within workspace (no .. allowed)".to_string());
        }
        
        fs::read_to_string(path).map_err(|e| format!("failed to read file: {}", e))
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
    async fn execute(&self, params: &str, sandbox: bool) -> Result<String, String> {
        let mut cmd = if sandbox {
            let mut c = Command::new("unshare");
            c.args(["--map-root-user", "--net", "--mount", "--pid", "--fork", "bash", "-c", params]);
            c
        } else {
            let mut c = Command::new("bash");
            c.args(["-c", params]);
            c
        };

        let output = cmd.output().map_err(|e| format!("failed to execute bash: {}", e))?;
        
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        
        if output.status.success() {
            Ok(stdout)
        } else {
            Err(format!("bash failed with status {}: {}", output.status, stderr))
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
    async fn execute(&self, params: &str, _sandbox: bool) -> Result<String, String> {
        let client = reqwest::Client::new();
        let url = format!("https://duckduckgo.com/lite/?q={}", urlencoding::encode(params));
        
        let resp = client.get(url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
            .send()
            .await
            .map_err(|e| format!("web search failed: {}", e))?;
            
        let body = resp.text().await.map_err(|e| format!("failed to read web search body: {}", e))?;
        
        // Return first 2000 chars for now
        Ok(body.chars().take(2000).collect())
    }
}

struct GlobTool;
#[async_trait]
impl Tool for GlobTool {
    fn name(&self) -> &str { "glob" }
    fn description(&self) -> &str { "Find files matching a pattern" }
    fn required_tier(&self) -> u8 { 0 }
    async fn execute(&self, params: &str, _sandbox: bool) -> Result<String, String> {
        if params.contains("..") {
            return Err("path must be within workspace (no .. allowed)".to_string());
        }
        
        let mut results = Vec::new();
        for entry in glob::glob(params).map_err(|e| format!("invalid glob pattern: {}", e))? {
            match entry {
                Ok(path) => results.push(path.display().to_string()),
                Err(e) => results.push(format!("error: {}", e)),
            }
        }
        Ok(results.join("\n"))
    }
}

struct GrepTool;
#[async_trait]
impl Tool for GrepTool {
    fn name(&self) -> &str { "grep" }
    fn description(&self) -> &str { "Search for a pattern in files" }
    fn required_tier(&self) -> u8 { 0 }
    async fn execute(&self, params: &str, _sandbox: bool) -> Result<String, String> {
        // Simple parser for "pattern path"
        let parts: Vec<&str> = params.splitn(2, ' ').collect();
        if parts.len() != 2 {
            return Err("grep requires 'pattern path'".to_string());
        }
        let pattern = parts[0];
        let path_str = parts[1];
        
        if path_str.contains("..") {
            return Err("path must be within workspace (no .. allowed)".to_string());
        }

        let re = regex::Regex::new(pattern).map_err(|e| format!("invalid regex: {}", e))?;
        let content = fs::read_to_string(path_str).map_err(|e| format!("failed to read file: {}", e))?;
        
        let mut results = Vec::new();
        for (i, line) in content.lines().enumerate() {
            if re.is_match(line) {
                results.push(format!("{}: {}", i + 1, line));
            }
        }
        Ok(results.join("\n"))
    }
}

struct AgentOrchestrationTool;
#[async_trait]
impl Tool for AgentOrchestrationTool {
    fn name(&self) -> &str { "agent_orchestration" }
    fn description(&self) -> &str { "Orchestrate other agents" }
    fn required_tier(&self) -> u8 { 4 }
    async fn execute(&self, _params: &str, _sandbox: bool) -> Result<String, String> {
        Ok("orchestration complete".to_string())
    }
}
