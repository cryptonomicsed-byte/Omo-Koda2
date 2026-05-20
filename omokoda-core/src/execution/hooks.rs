use async_trait::async_trait;
use crate::interpreter::ExecutionResult;
use crate::receipt::Receipt;

/// SwibePlugin Hook Contract
/// Ports the onBirth, onThink, onReceipt, and onSettle hooks from Swibe.
#[async_trait]
pub trait SwibePlugin: Send + Sync {
    /// Called when an agent is born.
    async fn on_birth(&self, name: &str, entropy: &[u8; 32]);

    /// Called before a 'think' statement is executed.
    async fn on_think(&self, prompt: &str);

    /// Called after a receipt is generated for an action.
    async fn on_receipt(&self, receipt: &Receipt);

    /// Called when a value settlement (e.g. escrow release) occurs.
    async fn on_settle(&self, result: &ExecutionResult);
}

#[derive(Default)]
pub struct HookManager {
    plugins: Vec<Box<dyn SwibePlugin>>,
}

impl HookManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, plugin: Box<dyn SwibePlugin>) {
        self.plugins.push(plugin);
    }

    pub async fn fire_birth(&self, name: &str, entropy: &[u8; 32]) {
        for plugin in &self.plugins {
            plugin.on_birth(name, entropy).await;
        }
    }

    pub async fn fire_think(&self, prompt: &str) {
        for plugin in &self.plugins {
            plugin.on_think(prompt).await;
        }
    }

    pub async fn fire_receipt(&self, receipt: &Receipt) {
        for plugin in &self.plugins {
            plugin.on_receipt(receipt).await;
        }
    }

    pub async fn fire_settle(&self, result: &ExecutionResult) {
        for plugin in &self.plugins {
            plugin.on_settle(result).await;
        }
    }
}
