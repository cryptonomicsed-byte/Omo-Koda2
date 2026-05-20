use std::path::Path;
use wasmtime::{Engine, Linker, Module, Store};
use wasmtime_wasi::WasiCtxBuilder;

/// Filesystem isolation mode for tool execution
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FilesystemMode {
    Off,
    WorkspaceOnly,
    AllowList(Vec<std::path::PathBuf>),
    ReadOnly,
}

/// Network isolation mode
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum NetworkMode {
    Isolated,
    Full,
    AllowList(Vec<String>),
}

/// Per-tool sandbox profile — determines isolation level for tool execution
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SandboxProfile {
    pub name: String,
    pub filesystem: FilesystemMode,
    pub network: NetworkMode,
    pub use_namespaces: bool,
    pub timeout_secs: u64,
    pub max_output_bytes: usize,
}

impl SandboxProfile {
    pub fn strict() -> Self {
        Self {
            name: "strict".to_string(),
            filesystem: FilesystemMode::WorkspaceOnly,
            network: NetworkMode::Isolated,
            use_namespaces: true,
            timeout_secs: 30,
            max_output_bytes: 1024 * 1024,
        }
    }

    pub fn read_only() -> Self {
        Self {
            name: "read_only".to_string(),
            filesystem: FilesystemMode::ReadOnly,
            network: NetworkMode::Isolated,
            use_namespaces: false,
            timeout_secs: 10,
            max_output_bytes: 512 * 1024,
        }
    }

    pub fn networked() -> Self {
        Self {
            name: "networked".to_string(),
            filesystem: FilesystemMode::WorkspaceOnly,
            network: NetworkMode::Full,
            use_namespaces: false,
            timeout_secs: 60,
            max_output_bytes: 2 * 1024 * 1024,
        }
    }

    pub fn unrestricted() -> Self {
        Self {
            name: "unrestricted".to_string(),
            filesystem: FilesystemMode::Off,
            network: NetworkMode::Full,
            use_namespaces: false,
            timeout_secs: 0,
            max_output_bytes: 10 * 1024 * 1024,
        }
    }

    pub fn for_tool(tool_name: &str) -> Self {
        match tool_name {
            "bash" | "wasm" | "exec" => Self::strict(),
            "write_file" | "edit_file" | "apply_patch" => Self::strict(),
            "read_file" | "glob" | "grep" => Self::read_only(),
            "web_search" | "web_fetch" => Self::networked(),
            _ => Self::strict(),
        }
    }

    pub fn is_container_environment() -> bool {
        std::path::Path::new("/.dockerenv").exists()
            || std::env::var("KUBERNETES_SERVICE_HOST").is_ok()
            || std::env::var("CI").is_ok()
    }
}

pub struct WasmSandbox {
    engine: Engine,
}

impl WasmSandbox {
    pub fn new() -> Result<Self, String> {
        let engine = Engine::default();
        Ok(Self { engine })
    }

    pub fn execute_module(
        &self,
        module_path: &Path,
        _args: &[String],
        _sandboxed: bool,
    ) -> Result<String, String> {
        let wasi = WasiCtxBuilder::new()
            .inherit_stdout()
            .inherit_stderr()
            .build();

        let mut store = Store::new(&self.engine, wasi);
        let module = Module::from_file(&self.engine, module_path)
            .map_err(|e| format!("failed to load module: {}", e))?;

        let mut linker = Linker::new(&self.engine);
        wasmtime_wasi::add_to_linker(&mut linker, |s| s)
            .map_err(|e| format!("failed to add wasi: {}", e))?;

        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| format!("failed to instantiate: {}", e))?;

        let func = instance
            .get_typed_func::<(), ()>(&mut store, "_start")
            .or_else(|_| instance.get_typed_func::<(), ()>(&mut store, "main"))
            .map_err(|_| "module lacks _start or main".to_string())?;

        func.call(&mut store, ())
            .map_err(|e| format!("execution error: {}", e))?;

        Ok("WASM execution completed".to_string())
    }
}
