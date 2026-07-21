pub mod clients;
pub mod sango;
pub mod zangbeto;
pub mod events {
    include!(concat!(env!("OUT_DIR"), "/omokoda.v1.rs"));
}

pub use self::events::SovereignEvent;
pub use clients::{
    AgentPresence, AgentStatus, HermeticResult, HttpOsunClient, HttpOyaClient, HttpYemojaClient,
    LocalObatalaStub, LocalOsunStub, LocalOyaStub, LocalSangoStub, LocalYemojaStub, ObatalaClient,
    OsunClient, OyaClient, SangoClient, YemojaClient,
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
