/// Python bridge — forwards act calls to the Ògún tool runner (port 7779).
///
/// Each `PythonTool` instance wraps one Python-backed tool with its own tier
/// requirement. Rust-side tier checks in `ToolRegistry::execute` still apply;
/// the Python service re-validates so defence-in-depth holds even if called
/// directly.
use crate::tools::{ExecutionContext, Tool};
use async_trait::async_trait;
use serde::Deserialize;

fn python_url() -> String {
    std::env::var("PYTHON_TOOL_URL").unwrap_or_else(|_| "http://localhost:7779".into())
}

pub struct PythonTool {
    pub name: &'static str,
    pub description: &'static str,
    pub tier: u8,
    pub write: bool,
}

#[derive(Deserialize)]
struct PythonResponse {
    output: String,
}

#[async_trait]
impl Tool for PythonTool {
    fn name(&self) -> &str {
        self.name
    }

    fn description(&self) -> &str {
        self.description
    }

    fn required_tier(&self) -> u8 {
        self.tier
    }

    fn is_write_operation(&self) -> bool {
        self.write
    }

    async fn execute(&self, params: &str, context: &ExecutionContext) -> Result<String, String> {
        let url = format!("{}/execute", python_url());

        let body = serde_json::json!({
            "tool": self.name,
            "params": params,
            "agent_id": context.agent_id.as_str(),
            "tier": context.tier,
        });

        let client = reqwest::Client::new();
        let resp = client
            .post(&url)
            .json(&body)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| format!("python bridge unavailable: {}", e))?;

        if resp.status().is_success() {
            let py_resp: PythonResponse = resp
                .json()
                .await
                .map_err(|e| format!("python bridge parse error: {}", e))?;
            Ok(py_resp.output)
        } else {
            let status = resp.status().as_u16();
            let body: serde_json::Value = resp
                .json()
                .await
                .unwrap_or(serde_json::json!({"error": "unknown"}));
            Err(format!(
                "python tool '{}' error {}: {}",
                self.name,
                status,
                body["error"].as_str().unwrap_or("unknown")
            ))
        }
    }
}

// ---------------------------------------------------------------------------
// One constructor per Python-backed tool — keeps mod.rs clean.
// ---------------------------------------------------------------------------

pub fn web_search_py() -> PythonTool {
    PythonTool {
        name: "py_web_search",
        description: "Rich web search via Python (DuckDuckGo, structured results)",
        tier: 0,
        write: false,
    }
}

pub fn code_runner() -> PythonTool {
    PythonTool {
        name: "code_runner",
        description: "Execute Python code in a sandboxed subprocess (Tier 2+)",
        tier: 2,
        write: true,
    }
}

pub fn data_analysis() -> PythonTool {
    PythonTool {
        name: "data_analysis",
        description:
            "Statistical analysis on JSON/CSV data (mean, median, stdev, histogram, correlation)",
        tier: 1,
        write: false,
    }
}

pub fn cosmos() -> PythonTool {
    PythonTool {
        name: "cosmos",
        description: "NVIDIA Cosmos world/app generation (Tier 3+)",
        tier: 3,
        write: false,
    }
}
