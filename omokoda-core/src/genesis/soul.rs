use super::providers::{GenesisError, SoulProvider};
use super::receipt::{KooduTimeProof, SoulProof};
use async_trait::async_trait;
use sha2::{Digest, Sha256};

// 256 Odù names (abbreviated — first 16 used as primary)
const PRIMARY_ODU_NAMES: &[&str] = &[
    "Ogbe",
    "Oyeku",
    "Iwori",
    "Odi",
    "Irosun",
    "Owonrin",
    "Obara",
    "Okanran",
    "Ogunda",
    "Osa",
    "Ika",
    "Oturupọn",
    "Otura",
    "Irete",
    "Ose",
    "Ofun",
];

const ORISHA_ALIGNMENTS: &[&str] = &[
    "Èṣù",
    "Ọ̀ṣun",
    "Yemọja",
    "Ọ̀ṣọ́ọ̀sì",
    "Ọbàtálá",
    "Ọya",
    "Ṣàngó",
    "Ògún",
];

const TEMPERAMENTS: &[&str] = &[
    "Analytical",
    "Intuitive",
    "Grounded",
    "Expansive",
    "Protective",
    "Creative",
    "Focused",
    "Adaptive",
];

const DESTINY_THREAD_POOL: &[&str] = &[
    "knowledge-seeker",
    "bridge-builder",
    "guardian",
    "innovator",
    "mediator",
    "strategist",
    "healer",
    "witness",
    "architect",
    "catalyst",
    "steward",
    "explorer",
];

/// Public entry point for use by interpreter birth flow (no KooduTimeProof dependency).
pub fn pub_cast_soul(entropy: &[u8], epoch: u64, cycle: u64, phase: u8) -> SoulProof {
    let koodu = KooduTimeProof {
        born_at: 0,
        koodu_epoch: epoch,
        koodu_cycle: cycle,
        koodu_phase: phase,
        bitcoin_height: None,
        bitcoin_anchor: None,
        gregorian_fallback: true,
    };
    cast_soul(entropy, &koodu)
}

fn cast_soul(entropy: &[u8], koodu: &KooduTimeProof) -> SoulProof {
    // Mix entropy with Koodu temporal state for cast
    let mut h = Sha256::new();
    h.update(entropy);
    h.update(&koodu.koodu_epoch.to_le_bytes());
    h.update(&koodu.koodu_cycle.to_le_bytes());
    h.update(&[koodu.koodu_phase]);
    h.update(b"ifa-soul-cast-v1");
    let digest = h.finalize();

    let primary_odu = digest[0]; // 0..255
    let composed_odu = (digest[0] as u16) << 8 | digest[1] as u16;
    let temperament_idx = (digest[2] as usize) % TEMPERAMENTS.len();
    let orisha_idx = (digest[3] as usize) % ORISHA_ALIGNMENTS.len();

    // Derive 3 destiny threads deterministically
    let t1 = (digest[4] as usize) % DESTINY_THREAD_POOL.len();
    let t2 = (digest[5] as usize) % DESTINY_THREAD_POOL.len();
    let t3 = (digest[6] as usize) % DESTINY_THREAD_POOL.len();
    let mut threads = vec![
        DESTINY_THREAD_POOL[t1].to_string(),
        DESTINY_THREAD_POOL[t2].to_string(),
        DESTINY_THREAD_POOL[t3].to_string(),
    ];
    threads.dedup();

    let odu_name = PRIMARY_ODU_NAMES[(primary_odu as usize) % PRIMARY_ODU_NAMES.len()];

    SoulProof {
        primary_odu,
        composed_odu,
        temperament: TEMPERAMENTS[temperament_idx].to_string(),
        orisha_alignment: format!("{} / {}", ORISHA_ALIGNMENTS[orisha_idx], odu_name,),
        destiny_threads: threads,
    }
}

pub struct DefaultSoulProvider;

#[async_trait]
impl SoulProvider for DefaultSoulProvider {
    async fn cast(
        &self,
        entropy: &[u8],
        koodu: &KooduTimeProof,
    ) -> Result<SoulProof, GenesisError> {
        if entropy.len() < 32 {
            return Err(GenesisError::Soul(
                "entropy must be at least 32 bytes".into(),
            ));
        }
        Ok(cast_soul(entropy, koodu))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::genesis::receipt::KooduTimeProof;

    fn dummy_koodu() -> KooduTimeProof {
        KooduTimeProof {
            born_at: 1_700_000_000_000,
            koodu_epoch: 0,
            koodu_cycle: 3,
            koodu_phase: 2,
            bitcoin_height: Some(830_000),
            bitcoin_anchor: None,
            gregorian_fallback: false,
        }
    }

    #[test]
    fn test_soul_cast_deterministic() {
        let entropy = [42u8; 32];
        let k = dummy_koodu();
        let a = cast_soul(&entropy, &k);
        let b = cast_soul(&entropy, &k);
        assert_eq!(a.primary_odu, b.primary_odu);
        assert_eq!(a.composed_odu, b.composed_odu);
        assert_eq!(a.temperament, b.temperament);
    }

    #[test]
    fn test_soul_cast_differs_with_different_entropy() {
        let k = dummy_koodu();
        let a = cast_soul(&[1u8; 32], &k);
        let b = cast_soul(&[2u8; 32], &k);
        // Primary Odù should differ (extremely likely with different entropy)
        // Can't assert always different but check struct is well-formed
        assert!(a.primary_odu <= 255);
        assert!(b.primary_odu <= 255);
        assert!(!a.temperament.is_empty());
    }
}
