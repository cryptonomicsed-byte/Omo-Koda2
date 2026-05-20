pub mod bus;
pub mod config;
pub mod economics;
pub mod execution;
pub mod identity;
pub mod intent;
pub mod interpreter;
pub mod justice;
pub mod memory;
pub mod parser;
pub mod permissions;
pub mod providers;
pub mod receipt;
pub mod reputation;
pub mod rhythm;
pub mod sandbox;
pub mod session;
pub mod tools;
pub mod usage;

pub use identity::AgentId;
pub use intent::{
    IntentClass, IntentCompilation, IntentCompileContext, IntentCompiler, IntentPlan,
    SubAgentSuggestion,
};
pub use interpreter::{AgentState, ExecutionResult, Steward};
pub use parser::{parse, Statement};
pub use receipt::{Receipt, ReceiptStore};
pub use session::{EncryptedSession, SensitiveKey};

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
