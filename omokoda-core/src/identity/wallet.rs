use blake2::digest::consts::U32;
use blake2::{Blake2b, Digest};
use ed25519_dalek::SigningKey;
use hmac::{Hmac, Mac};
use sha2::Sha512;

type Blake2b256 = Blake2b<U32>;

/// Derive the real Sui address from an Ed25519 public key: `0x` + hex of
/// `blake2b256(flag_byte || pubkey)`, `flag_byte = 0x00` for the Ed25519
/// signature scheme -- Sui's actual on-chain address format (SIP-6), not the
/// raw public key hex that was published here before. Ported from
/// vanity-cloakseed's `chainCrypto.ts::deriveSuiAddress`, verified against
/// the same algorithm.
pub fn sui_address_from_pubkey(pubkey: &[u8; 32]) -> String {
    let mut hasher = Blake2b256::new();
    hasher.update([0x00]);
    hasher.update(pubkey);
    let digest = hasher.finalize();
    format!("0x{}", hex::encode(digest))
}

pub struct Wallet;

impl Wallet {
    /// Derives an Ed25519 keypair from a mnemonic using SLIP-0010 (m/44'/784'/0'/0'/0')
    /// for Sui compatibility.
    pub fn derive_from_mnemonic(mnemonic: &str, passphrase: &str) -> Result<SigningKey, String> {
        // 1. Mnemonic to seed (BIP-39 standard seed derivation for the master key)
        // Note: The architecture spec mentions argon2id for "identity-critical" seeds,
        // but for a standard Sui wallet compatibility, BIP-39 PBKDF2 is usually expected
        // if interacting with other wallets.
        // However, the architecture says "Sui wallet — Ed25519 keypair, m/44'/784' derivation from mnemonic".
        // Let's use the PBKDF2 seed for the BIP-39 master seed, then SLIP-0010 for derivation.

        let mut seed = [0u8; 64];
        let salt = format!("mnemonic{}", passphrase);
        pbkdf2::pbkdf2::<Hmac<Sha512>>(mnemonic.as_bytes(), salt.as_bytes(), 2048, &mut seed)
            .expect("PBKDF2 failed");

        Self::derive_from_seed(&seed)
    }

    pub fn derive_from_seed(seed: &[u8; 64]) -> Result<SigningKey, String> {
        // SLIP-0010 master key derivation
        let mut hmac =
            Hmac::<Sha512>::new_from_slice(b"ed25519 seed").map_err(|e| e.to_string())?;
        hmac.update(seed);
        let intermediate = hmac.finalize().into_bytes();

        let mut il = [0u8; 32];
        let mut ir = [0u8; 32];
        il.copy_from_slice(&intermediate[..32]);
        ir.copy_from_slice(&intermediate[32..]);

        // Derivation path: m/44'/784'/0'/0'/0'
        // Every step is hardened for Ed25519 as per SLIP-0010.
        let path = [
            44 | 0x8000_0000,
            784 | 0x8000_0000,
            0x8000_0000,
            0x8000_0000,
            0x8000_0000,
        ];

        let (mut kl, mut kr) = (il, ir);
        for &index in &path {
            (kl, kr) = Self::derive_child(kl, kr, index)?;
        }

        Ok(SigningKey::from_bytes(&kl))
    }

    fn derive_child(
        kl: [u8; 32],
        kr: [u8; 32],
        index: u32,
    ) -> Result<([u8; 32], [u8; 32]), String> {
        let mut hmac = Hmac::<Sha512>::new_from_slice(&kr).map_err(|e| e.to_string())?;
        hmac.update(&[0u8]); // hardened indicator for SLIP-0010
        hmac.update(&kl);
        hmac.update(&index.to_be_bytes());
        let intermediate = hmac.finalize().into_bytes();

        let mut il = [0u8; 32];
        let mut ir = [0u8; 32];
        il.copy_from_slice(&intermediate[..32]);
        ir.copy_from_slice(&intermediate[32..]);
        Ok((il, ir))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Real vector: `sui keytool generate ed25519 --json` on this box printed
    /// suiAddress = 0x498724...71ba for peerId (raw pubkey hex)
    /// ff8790...fa32c -- independently verified with Python's hashlib
    /// (blake2b digest_size=32) before trusting it here. Not a synthetic
    /// vector; the actual `sui` binary computed this address.
    #[test]
    fn sui_address_matches_the_real_sui_cli() {
        let pubkey_hex = "ff879040047ab33258afade9e5505defc37e4d5dac2d770b702a526d40bfa32c";
        let pubkey: [u8; 32] = hex::decode(pubkey_hex).unwrap().try_into().unwrap();
        let address = sui_address_from_pubkey(&pubkey);
        assert_eq!(
            address,
            "0x498724481844b13ea6f8277c65af18774e03d5b81b6d40d4258cd8f12b2871ba"
        );
    }

    #[test]
    fn sui_address_is_deterministic_and_well_formed() {
        let pubkey = [7u8; 32];
        let a = sui_address_from_pubkey(&pubkey);
        let b = sui_address_from_pubkey(&pubkey);
        assert_eq!(a, b);
        assert!(a.starts_with("0x"));
        assert_eq!(a.len(), 66, "0x + 64 hex chars (32-byte digest)");
    }

    #[test]
    fn sui_address_differs_from_the_raw_pubkey_hex() {
        // The bug this replaces: publishing raw pubkey hex as if it were the
        // address. They must never be equal.
        let pubkey = [3u8; 32];
        let address = sui_address_from_pubkey(&pubkey);
        assert_ne!(address, format!("0x{}", hex::encode(pubkey)));
    }
}
