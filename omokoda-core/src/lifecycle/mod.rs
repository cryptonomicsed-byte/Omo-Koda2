pub mod agent_lifecycle;
pub mod heartbeat;
pub mod job_daemon;
pub mod migration;
pub mod nostr_publisher;
pub mod runtime;
pub mod scheduler;
pub mod sensor;
pub mod skill_daemon;
pub mod soma_lifecycle;
pub mod supervisor;

pub use heartbeat::{AgentHeartbeat, HeartbeatState, SomaVector};
pub use sensor::SensorReading;
pub use soma_lifecycle::{SomaLifecycle, SessionSummary};
pub use runtime::{AgentRuntime, DaemonRegistry, DaemonEntry, DaemonStatus};
pub use scheduler::{SchedulerConfig, spawn_scheduler};
pub use job_daemon::spawn_job_daemon;
pub use skill_daemon::spawn_skill_daemon;
pub use supervisor::DaemonSupervisor;
pub use agent_lifecycle::{
    AgentLifecycleStage, LifecycleTransition, validate_transition,
    TransitionKind, SignedLifecycleTransition,
};
pub use migration::{AgentCapsule, MigrationState};
pub use nostr_publisher::{NostrPublisherConfig, spawn_nostr_publisher};
