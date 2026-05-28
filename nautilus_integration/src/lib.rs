/// Nautilus TEE integration for private memory sealing.
/// Wraps the Nautilus SDK (Mysten Labs TEE for Sui) for AES-GCM seal/unseal
/// of agent private memory inside a trusted execution environment.
///
/// In production: replace the stub crypto with a real Nautilus SDK call.
/// The session handshake and attestation verification remain as-is.
pub mod attestation;
pub mod handshake;
pub mod sealed_memory;

pub use attestation::{AttestationResult, TeeQuote};
pub use handshake::{HandshakeSession, SessionError};
pub use sealed_memory::{SealError, SealedMemory};
