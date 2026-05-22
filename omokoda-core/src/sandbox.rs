use std::path::Path;
use wasmtime::{Engine, Linker, Module, Store};
use wasmtime_wasi::WasiCtxBuilder;

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
