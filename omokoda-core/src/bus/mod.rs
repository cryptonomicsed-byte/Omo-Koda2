pub mod clients;
pub mod events {
    include!(concat!(env!("OUT_DIR"), "/omokoda.v1.rs"));
}

pub use self::events::SovereignEvent;
pub use clients::{
    HermeticResult, LocalObatalaStub, LocalOsunStub, LocalOyaStub, LocalSangoStub,
    ObatalaClient, OsunClient, OyaClient, SangoClient,
};
use tokio::sync::broadcast;

#[derive(Debug, Clone)]
pub struct SovereignEventBus {
    sender: broadcast::Sender<SovereignEvent>,
}

impl SovereignEventBus {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    pub fn publish(
        &self,
        event: SovereignEvent,
    ) -> Result<usize, broadcast::error::SendError<SovereignEvent>> {
        self.sender.send(event)
    }

    pub fn subscribe(&self) -> broadcast::Receiver<SovereignEvent> {
        self.sender.subscribe()
    }
}

impl Default for SovereignEventBus {
    fn default() -> Self {
        Self::new(1024)
    }
}
