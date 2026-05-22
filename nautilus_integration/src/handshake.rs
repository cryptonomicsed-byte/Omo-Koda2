use std::time::{Duration, SystemTime, UNIX_EPOCH};
use thiserror::Error;

const SESSION_TIMEOUT_SECS: u64 = 300; // 5 minutes

#[derive(Debug, Error)]
pub enum SessionError {
    #[error("session expired")]
    Expired,
    #[error("nonce replay detected")]
    NonceReplay,
    #[error("attestation failed: {0}")]
    Attestation(String),
}

#[derive(Debug, Clone)]
pub struct HandshakeSession {
    pub session_id: [u8; 16],
    pub seal_key: [u8; 32],
    created_at: u64,
    used_nonces: Vec<[u8; 16]>,
}

impl HandshakeSession {
    pub fn new(seal_key: [u8; 32]) -> Self {
        let mut session_id = [0u8; 16];
        rand::RngCore::fill_bytes(&mut rand::thread_rng(), &mut session_id);
        Self {
            session_id,
            seal_key,
            created_at: unix_now(),
            used_nonces: Vec::new(),
        }
    }

    pub fn is_expired(&self) -> bool {
        unix_now().saturating_sub(self.created_at) >= SESSION_TIMEOUT_SECS
    }

    pub fn check_nonce(&mut self, nonce: &[u8; 16]) -> Result<(), SessionError> {
        if self.is_expired() {
            return Err(SessionError::Expired);
        }
        if self.used_nonces.contains(nonce) {
            return Err(SessionError::NonceReplay);
        }
        self.used_nonces.push(*nonce);
        Ok(())
    }
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs()
}
