use super::providers::{GenesisError, KooduProvider};
use super::receipt::KooduTimeProof;
use async_trait::async_trait;

const KOODU_GENESIS_BLOCK: u64 = 780_000;
const BLOCKS_PER_DAY: u64 = 144;
const BLOCKS_PER_CYCLE: u64 = BLOCKS_PER_DAY * 7; // ~1 week
const BLOCKS_PER_EPOCH: u64 = BLOCKS_PER_CYCLE * 52; // ~1 year

/// Derives Koodu temporal coordinates from a Bitcoin block height.
pub fn koodu_from_height(height: u64) -> (u64, u64, u8) {
    let blocks_since_genesis = height.saturating_sub(KOODU_GENESIS_BLOCK);
    let epoch = blocks_since_genesis / BLOCKS_PER_EPOCH;
    let cycle = (blocks_since_genesis % BLOCKS_PER_EPOCH) / BLOCKS_PER_CYCLE;
    let phase = ((blocks_since_genesis % BLOCKS_PER_CYCLE) / BLOCKS_PER_DAY) as u8;
    (epoch, cycle, phase)
}

/// Koodu from Gregorian fallback: maps Unix timestamp to equivalent coordinates.
pub fn koodu_from_unix(ts_ms: u64) -> (u64, u64, u8) {
    // Approximate: 1 block = 600s = 10 min
    let seconds = ts_ms / 1000;
    let synthetic_height = KOODU_GENESIS_BLOCK + (seconds / 600);
    koodu_from_height(synthetic_height)
}

/// Try to fetch the current Bitcoin block height from a public API.
/// Returns None if unavailable (offline, rate-limited, etc.).
async fn fetch_btc_height() -> Option<(u64, String)> {
    let url = std::env::var("KOODU_BTC_API")
        .unwrap_or_else(|_| "https://blockstream.info/api/blocks/tip/height".to_string());
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .ok()?;
    let text = client.get(&url).send().await.ok()?.text().await.ok()?;
    let height: u64 = text.trim().parse().ok()?;
    // Also fetch hash for anchor
    let hash_url = format!("https://blockstream.info/api/block-height/{}", height);
    let hash = client
        .get(&hash_url)
        .send()
        .await
        .ok()?
        .text()
        .await
        .unwrap_or_else(|_| hex::encode([0u8; 32]));
    Some((height, hash.trim().to_string()))
}

pub struct DefaultKooduProvider;

#[async_trait]
impl KooduProvider for DefaultKooduProvider {
    async fn birth_time(&self) -> Result<KooduTimeProof, GenesisError> {
        let born_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| GenesisError::KooduTime(e.to_string()))?
            .as_millis() as u64;

        match fetch_btc_height().await {
            Some((height, anchor)) => {
                let (epoch, cycle, phase) = koodu_from_height(height);
                Ok(KooduTimeProof {
                    born_at,
                    koodu_epoch: epoch,
                    koodu_cycle: cycle,
                    koodu_phase: phase,
                    bitcoin_height: Some(height),
                    bitcoin_anchor: Some(anchor),
                    gregorian_fallback: false,
                })
            }
            None => {
                let (epoch, cycle, phase) = koodu_from_unix(born_at);
                Ok(KooduTimeProof {
                    born_at,
                    koodu_epoch: epoch,
                    koodu_cycle: cycle,
                    koodu_phase: phase,
                    bitcoin_height: None,
                    bitcoin_anchor: None,
                    gregorian_fallback: true,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_koodu_from_height_genesis() {
        let (epoch, cycle, phase) = koodu_from_height(KOODU_GENESIS_BLOCK);
        assert_eq!(epoch, 0);
        assert_eq!(cycle, 0);
        assert_eq!(phase, 0);
    }

    #[test]
    fn test_koodu_phase_advances() {
        let (_, _, phase0) = koodu_from_height(KOODU_GENESIS_BLOCK);
        let (_, _, phase1) = koodu_from_height(KOODU_GENESIS_BLOCK + BLOCKS_PER_DAY);
        assert_eq!(phase0, 0);
        assert_eq!(phase1, 1);
    }

    #[test]
    fn test_koodu_cycle_advances() {
        let (_, c0, _) = koodu_from_height(KOODU_GENESIS_BLOCK);
        let (_, c1, _) = koodu_from_height(KOODU_GENESIS_BLOCK + BLOCKS_PER_CYCLE);
        assert_eq!(c0, 0);
        assert_eq!(c1, 1);
    }
}
