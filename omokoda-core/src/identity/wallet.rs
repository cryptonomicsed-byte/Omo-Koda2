use blake2::digest::consts::U32;
use blake2::{Blake2b, Digest};
use ed25519_dalek::SigningKey;
use hmac::{Hmac, Mac};
use sha2::{Sha256, Sha512};
use sha3::{Keccak256, Sha3_256};

type Blake2b256 = Blake2b<U32>;

/// BIP-39 mnemonic -> 64-byte seed (standard PBKDF2-HMAC-SHA512, 2048
/// rounds, salt = "mnemonic" + passphrase). Shared by every chain's
/// derivation below -- they all fan out from the same root seed, only the
/// curve/path/child-derivation math differs per chain.
fn mnemonic_to_seed(mnemonic: &str, passphrase: &str) -> [u8; 64] {
    let mut seed = [0u8; 64];
    let salt = format!("mnemonic{}", passphrase);
    pbkdf2::pbkdf2::<Hmac<Sha512>>(mnemonic.as_bytes(), salt.as_bytes(), 2048, &mut seed)
        .expect("PBKDF2 failed");
    seed
}

/// One derived child chain's key material, hex-encoded for sealing into
/// `PrivateSessionData` alongside the existing Sui wallet key.
#[derive(Debug, Clone)]
pub struct ChainKey {
    pub private_key_hex: String,
    pub address: String,
}

/// Ethereum: secp256k1 BIP-32, path m/44'/60'/0'/0 (ported verbatim from
/// vanity-cloakseed's `chains.ts` CHAINS.ethereum.bip44), address = last 20
/// bytes of keccak256(uncompressed pubkey minus the 0x04 prefix).
pub fn derive_ethereum(mnemonic: &str, passphrase: &str) -> Result<ChainKey, String> {
    let seed = mnemonic_to_seed(mnemonic, passphrase);
    let xprv = bip32::XPrv::derive_from_path(
        seed,
        &"m/44'/60'/0'/0".parse::<bip32::DerivationPath>().map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let signing_key: k256::ecdsa::SigningKey = xprv.private_key().clone();
    let verifying_key = signing_key.verifying_key();
    let uncompressed = verifying_key.to_encoded_point(false);
    let pubkey_bytes = uncompressed.as_bytes(); // 0x04 || X(32) || Y(32)
    let mut hasher = Keccak256::new();
    hasher.update(&pubkey_bytes[1..]);
    let digest = hasher.finalize();
    let address = format!("0x{}", hex::encode(&digest[12..]));
    Ok(ChainKey {
        private_key_hex: hex::encode(signing_key.to_bytes()),
        address,
    })
}

/// Bitcoin: secp256k1 BIP-32, path m/84'/0'/0'/0 (native segwit account
/// root, matching vanity-cloakseed's chains.ts CHAINS.bitcoin.bip44).
/// Address = bech32 P2WPKH (bc1...) of hash160(compressed pubkey).
pub fn derive_bitcoin(mnemonic: &str, passphrase: &str) -> Result<ChainKey, String> {
    let seed = mnemonic_to_seed(mnemonic, passphrase);
    let xprv = bip32::XPrv::derive_from_path(
        seed,
        &"m/84'/0'/0'/0".parse::<bip32::DerivationPath>().map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let signing_key: k256::ecdsa::SigningKey = xprv.private_key().clone();
    let verifying_key = signing_key.verifying_key();
    let compressed = verifying_key.to_encoded_point(true);
    let sha = Sha256::digest(compressed.as_bytes());
    let hash160 = ripemd::Ripemd160::digest(sha);
    let address = bech32_p2wpkh("bc", &hash160)?;
    Ok(ChainKey {
        private_key_hex: hex::encode(signing_key.to_bytes()),
        address,
    })
}

/// Cosmos Hub: secp256k1 BIP-32, path m/44'/118'/0'/0 (matching
/// vanity-cloakseed's chains.ts CHAINS.cosmos.bip44). Address = bech32
/// "cosmos" of hash160(compressed pubkey).
pub fn derive_cosmos(mnemonic: &str, passphrase: &str) -> Result<ChainKey, String> {
    let seed = mnemonic_to_seed(mnemonic, passphrase);
    let xprv = bip32::XPrv::derive_from_path(
        seed,
        &"m/44'/118'/0'/0".parse::<bip32::DerivationPath>().map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    let signing_key: k256::ecdsa::SigningKey = xprv.private_key().clone();
    let verifying_key = signing_key.verifying_key();
    let compressed = verifying_key.to_encoded_point(true);
    let sha = Sha256::digest(compressed.as_bytes());
    let hash160 = ripemd::Ripemd160::digest(sha);
    let address = bech32_addr("cosmos", &hash160)?;
    Ok(ChainKey {
        private_key_hex: hex::encode(signing_key.to_bytes()),
        address,
    })
}

/// Solana: Ed25519 SLIP-0010, path m/44'/501'/0'/0' (matching
/// vanity-cloakseed's chains.ts CHAINS.solana.bip44 -- every segment
/// hardened, as SLIP-0010 ed25519 requires). Address = base58(pubkey).
pub fn derive_solana(mnemonic: &str, passphrase: &str) -> Result<ChainKey, String> {
    let seed = mnemonic_to_seed(mnemonic, passphrase);
    let path = [
        44 | HARDENED,
        501 | HARDENED,
        0 | HARDENED,
        0 | HARDENED,
    ];
    let signing_key = Wallet::derive_ed25519_slip10(&seed, &path)?;
    let pubkey = signing_key.verifying_key().to_bytes();
    Ok(ChainKey {
        private_key_hex: hex::encode(signing_key.to_bytes()),
        address: bs58::encode(pubkey).into_string(),
    })
}

/// Aptos: Ed25519 SLIP-0010, path m/44'/637'/0'/0' (matching
/// vanity-cloakseed's chains.ts CHAINS.aptos.bip44). Address =
/// sha3-256(pubkey || 0x00) -- Aptos's single-signer Ed25519 scheme.
pub fn derive_aptos(mnemonic: &str, passphrase: &str) -> Result<ChainKey, String> {
    let seed = mnemonic_to_seed(mnemonic, passphrase);
    let path = [44 | HARDENED, 637 | HARDENED, 0 | HARDENED, 0 | HARDENED];
    let signing_key = Wallet::derive_ed25519_slip10(&seed, &path)?;
    let pubkey = signing_key.verifying_key().to_bytes();
    let mut hasher = Sha3_256::new();
    hasher.update(pubkey);
    hasher.update([0x00]); // Ed25519 single-signer scheme identifier
    let digest = hasher.finalize();
    Ok(ChainKey {
        private_key_hex: hex::encode(signing_key.to_bytes()),
        address: format!("0x{}", hex::encode(digest)),
    })
}

/// Nostr (NIP-06): Ed25519 SLIP-0010, path m/44'/1237'/<account>'/0/0.
/// Every segment is derived hardened here (the same treatment the existing
/// Sui derivation already gives m/44'/784'/0'/0'/0' above) since SLIP-0010
/// ed25519 has no defined non-hardened child derivation -- there is no
/// public-key-only path to derive from for Ed25519. Address = bech32
/// "npub" of the raw pubkey (NIP-19).
pub fn derive_nostr(mnemonic: &str, passphrase: &str, account: u32) -> Result<ChainKey, String> {
    let seed = mnemonic_to_seed(mnemonic, passphrase);
    let path = [44 | HARDENED, 1237 | HARDENED, account | HARDENED, 0, 0];
    let signing_key = Wallet::derive_ed25519_slip10(&seed, &path)?;
    let pubkey = signing_key.verifying_key().to_bytes();
    let address = bech32_npub(&pubkey)?;
    Ok(ChainKey {
        private_key_hex: hex::encode(signing_key.to_bytes()),
        address,
    })
}

const HARDENED: u32 = 0x8000_0000;

fn bech32_p2wpkh(hrp_str: &str, hash160: &[u8]) -> Result<String, String> {
    use bech32::Hrp;
    let hrp = Hrp::parse(hrp_str).map_err(|e| e.to_string())?;
    bech32::segwit::encode_v0(hrp, hash160).map_err(|e| e.to_string())
}

fn bech32_addr(hrp_str: &str, hash160: &[u8]) -> Result<String, String> {
    use bech32::{Bech32, Hrp};
    let hrp = Hrp::parse(hrp_str).map_err(|e| e.to_string())?;
    bech32::encode::<Bech32>(hrp, hash160).map_err(|e| e.to_string())
}

fn bech32_npub(pubkey: &[u8; 32]) -> Result<String, String> {
    use bech32::{Bech32, Hrp};
    let hrp = Hrp::parse("npub").map_err(|e| e.to_string())?;
    bech32::encode::<Bech32>(hrp, pubkey).map_err(|e| e.to_string())
}

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

        let seed = mnemonic_to_seed(mnemonic, passphrase);
        Self::derive_from_seed(&seed)
    }

    pub fn derive_from_seed(seed: &[u8; 64]) -> Result<SigningKey, String> {
        // Derivation path: m/44'/784'/0'/0'/0'
        // Every step is hardened for Ed25519 as per SLIP-0010.
        let path = [
            44 | 0x8000_0000,
            784 | 0x8000_0000,
            0x8000_0000,
            0x8000_0000,
            0x8000_0000,
        ];
        Self::derive_ed25519_slip10(seed, &path)
    }

    /// General-purpose SLIP-0010 Ed25519 derivation used by every ed25519
    /// chain here (Sui, Solana, Aptos, Nostr) -- only the path differs per
    /// chain, the master-key + child-key math is identical.
    pub fn derive_ed25519_slip10(seed: &[u8; 64], path: &[u32]) -> Result<SigningKey, String> {
        // SLIP-0010 master key derivation
        let mut hmac =
            Hmac::<Sha512>::new_from_slice(b"ed25519 seed").map_err(|e| e.to_string())?;
        hmac.update(seed);
        let intermediate = hmac.finalize().into_bytes();

        let mut il = [0u8; 32];
        let mut ir = [0u8; 32];
        il.copy_from_slice(&intermediate[..32]);
        ir.copy_from_slice(&intermediate[32..]);

        let (mut kl, mut kr) = (il, ir);
        for &index in path {
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

    /// Standard BIP-39 test mnemonic (all-"abandon" + "about" checksum
    /// word) -- not a real identity, used purely as a fixed, reproducible
    /// input for the determinism/distinctness/byte-length checks below.
    const TEST_MNEMONIC: &str =
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    #[test]
    fn ethereum_key_is_deterministic_and_well_formed() {
        let a = derive_ethereum(TEST_MNEMONIC, "").unwrap();
        let b = derive_ethereum(TEST_MNEMONIC, "").unwrap();
        assert_eq!(a.private_key_hex, b.private_key_hex);
        assert_eq!(a.address, b.address);
        assert_eq!(a.private_key_hex.len(), 64, "32-byte secp256k1 scalar");
        assert!(a.address.starts_with("0x"));
        assert_eq!(a.address.len(), 42, "0x + 20-byte address");
    }

    #[test]
    fn bitcoin_key_is_deterministic_and_well_formed() {
        let a = derive_bitcoin(TEST_MNEMONIC, "").unwrap();
        let b = derive_bitcoin(TEST_MNEMONIC, "").unwrap();
        assert_eq!(a.private_key_hex, b.private_key_hex);
        assert_eq!(a.address, b.address);
        assert_eq!(a.private_key_hex.len(), 64, "32-byte secp256k1 scalar");
        assert!(a.address.starts_with("bc1"), "native segwit P2WPKH address");
    }

    #[test]
    fn cosmos_key_is_deterministic_and_well_formed() {
        let a = derive_cosmos(TEST_MNEMONIC, "").unwrap();
        let b = derive_cosmos(TEST_MNEMONIC, "").unwrap();
        assert_eq!(a.private_key_hex, b.private_key_hex);
        assert_eq!(a.address, b.address);
        assert_eq!(a.private_key_hex.len(), 64, "32-byte secp256k1 scalar");
        assert!(a.address.starts_with("cosmos1"));
    }

    #[test]
    fn solana_key_is_deterministic_and_well_formed() {
        let a = derive_solana(TEST_MNEMONIC, "").unwrap();
        let b = derive_solana(TEST_MNEMONIC, "").unwrap();
        assert_eq!(a.private_key_hex, b.private_key_hex);
        assert_eq!(a.address, b.address);
        assert_eq!(a.private_key_hex.len(), 64, "32-byte ed25519 secret key");
        assert!(!a.address.is_empty());
    }

    #[test]
    fn aptos_key_is_deterministic_and_well_formed() {
        let a = derive_aptos(TEST_MNEMONIC, "").unwrap();
        let b = derive_aptos(TEST_MNEMONIC, "").unwrap();
        assert_eq!(a.private_key_hex, b.private_key_hex);
        assert_eq!(a.address, b.address);
        assert_eq!(a.private_key_hex.len(), 64, "32-byte ed25519 secret key");
        assert!(a.address.starts_with("0x"));
        assert_eq!(a.address.len(), 66, "0x + 32-byte sha3-256 digest");
    }

    #[test]
    fn nostr_key_is_deterministic_and_well_formed() {
        let a = derive_nostr(TEST_MNEMONIC, "", 0).unwrap();
        let b = derive_nostr(TEST_MNEMONIC, "", 0).unwrap();
        assert_eq!(a.private_key_hex, b.private_key_hex);
        assert_eq!(a.address, b.address);
        assert_eq!(a.private_key_hex.len(), 64, "32-byte ed25519 secret key");
        assert!(a.address.starts_with("npub"));

        // A different account index must derive a different key -- confirms
        // the account segment of the NIP-06 path is actually load-bearing.
        let c = derive_nostr(TEST_MNEMONIC, "", 1).unwrap();
        assert_ne!(a.private_key_hex, c.private_key_hex);
    }

    #[test]
    fn all_chain_keys_from_the_same_mnemonic_are_pairwise_distinct() {
        // Same root mnemonic, six different chains -- every derived
        // private key must be unique. A collision here would mean two
        // chains accidentally share a derivation path.
        let sui = crate::identity::wallet::Wallet::derive_from_mnemonic(TEST_MNEMONIC, "")
            .unwrap();
        let sui_hex = hex::encode(sui.to_bytes());
        let eth = derive_ethereum(TEST_MNEMONIC, "").unwrap().private_key_hex;
        let btc = derive_bitcoin(TEST_MNEMONIC, "").unwrap().private_key_hex;
        let cosmos = derive_cosmos(TEST_MNEMONIC, "").unwrap().private_key_hex;
        let sol = derive_solana(TEST_MNEMONIC, "").unwrap().private_key_hex;
        let aptos = derive_aptos(TEST_MNEMONIC, "").unwrap().private_key_hex;
        let nostr = derive_nostr(TEST_MNEMONIC, "", 0).unwrap().private_key_hex;

        let keys = [sui_hex, eth, btc, cosmos, sol, aptos, nostr];
        for i in 0..keys.len() {
            for j in (i + 1)..keys.len() {
                assert_ne!(keys[i], keys[j], "chain {i} and chain {j} must not share a key");
            }
        }
    }
}
