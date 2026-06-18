pub mod agents;
pub mod background;
pub mod behavioral;
pub mod bootstrap;
pub mod bus;
pub mod clients;
pub mod compact;
pub mod config;
pub mod dream;
pub mod economics;
pub mod emotion;
pub mod error;
pub mod execution;
pub mod gates;
pub mod identity;
pub mod intent;
pub mod interpreter;
pub mod justice;
pub mod lsp;
pub mod main_loop;
pub mod memory;
pub mod memory_vault;
pub mod parser;
pub mod permissions;
pub mod plugins;
pub mod policy;
pub mod prompt;
pub mod providers;
pub mod query;
pub mod receipt;
pub mod reputation;
pub mod rhythm;
pub mod sandbox;
pub mod server;
pub mod session;
pub mod skills;
pub mod steward;
pub mod tasks;
pub mod tools;
pub mod usage;

pub use identity::user::{IdentityError, PrivacyMode, UserIdentity};
pub use identity::AgentId;
pub use intent::{
    IntentClass, IntentCompilation, IntentCompileContext, IntentCompiler, IntentPlan,
    SubAgentSuggestion,
};
pub use interpreter::{AgentCore, AgentSnapshot, ExecutionResult, Steward};
pub use parser::{parse, Statement};
pub use plugins::{PluginManifest, PluginRegistry, PluginState};
pub use receipt::{Receipt, ReceiptStore};
pub use session::{EncryptedSession, SensitiveKey};
pub use skills::{OduModule, OduRegistry, OduSource};
pub use steward::dispatch::{DispatchError, PrimitiveDispatcher};
pub use steward::privacy::PrivacyEnforcer;

#[derive(Debug, Clone)]
pub enum Primitive {
    Birth {
        name: String,
        metadata: Vec<(String, String)>,
    },
    Think {
        prompt: String,
        private: bool,
    },
    Act {
        tool: String,
        params: String,
        sandbox: bool,
    },
}
