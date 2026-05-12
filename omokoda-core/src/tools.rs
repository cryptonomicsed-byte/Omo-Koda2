use std::collections::HashMap;

pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn execute(&self, params: &str) -> Result<String, String>;
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
        registry.register(Box::new(GlobTool));
        registry.register(Box::new(GrepTool));
        registry
    }

    pub fn register(&mut self, tool: Box<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    pub fn execute(&self, name: &str, params: &str) -> Result<String, String> {
        self.tools
            .get(name)
            .ok_or_else(|| format!("tool not found: {}", name))?
            .execute(params)
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
impl Tool for ReadFileTool {
    fn name(&self) -> &str {
        "read_file"
    }
    fn description(&self) -> &str {
        "Read a file from the workspace"
    }
    fn execute(&self, params: &str) -> Result<String, String> {
        // Implementation stub
        Ok(format!("contents of {}", params))
    }
}

struct GlobTool;
impl Tool for GlobTool {
    fn name(&self) -> &str {
        "glob"
    }
    fn description(&self) -> &str {
        "Find files matching a pattern"
    }
    fn execute(&self, params: &str) -> Result<String, String> {
        // Implementation stub
        Ok(format!("files matching {}", params))
    }
}

struct GrepTool;
impl Tool for GrepTool {
    fn name(&self) -> &str {
        "grep"
    }
    fn description(&self) -> &str {
        "Search for a pattern in files"
    }
    fn execute(&self, params: &str) -> Result<String, String> {
        // Implementation stub
        Ok(format!("matches for {}", params))
    }
}
