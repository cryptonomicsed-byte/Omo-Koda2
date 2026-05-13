pub mod identity;
pub mod interpreter;
pub mod parser;
pub mod providers;
pub mod receipt;
pub mod reputation;
pub mod justice;
pub mod session;
pub mod tools;

pub use identity::AgentId;
pub use interpreter::{Steward, AgentState, ExecutionResult};
pub use parser::{parse, Statement};
pub use receipt::{Receipt, ReceiptStore};

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
