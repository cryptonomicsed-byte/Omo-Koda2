use ed25519_dalek::SigningKey;
use hmac::{Hmac, Mac};
use sha2::Sha512;

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
        let mut hmac = Hmac::<Sha512>::new_from_slice(b"ed25519 seed").map_err(|e| e.to_string())?;
        hmac.update(seed);
        let intermediate = hmac.finalize().into_bytes();
        
        let mut il = [0u8; 32];
        let mut ir = [0u8; 32];
        il.copy_from_slice(&intermediate[..32]);
        ir.copy_from_slice(&intermediate[32..]);

        // Derivation path: m/44'/784'/0'/0'/0'
        // Every step is hardened for Ed25519 as per SLIP-0010.
        let path = [44 | 0x8000_0000, 784 | 0x8000_0000, 0 | 0x8000_0000, 0 | 0x8000_0000, 0 | 0x8000_0000];
        
        let (mut kl, mut kr) = (il, ir);
        for &index in &path {
            (kl, kr) = Self::derive_child(kl, kr, index)?;
        }

        Ok(SigningKey::from_bytes(&kl))
    }

    fn derive_child(kl: [u8; 32], kr: [u8; 32], index: u32) -> Result<([u8; 32], [u8; 32]), String> {
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
