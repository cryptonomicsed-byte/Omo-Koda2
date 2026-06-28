use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OduSeed(pub [u8; 32]);

impl OduSeed {
    pub fn new(seed: [u8; 32]) -> Self {
        Self(seed)
    }

    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn len(&self) -> usize {
        32
    }

    pub fn is_empty(&self) -> bool {
        false
    }
}

impl AsRef<[u8]> for OduSeed {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OduIdentity {
    pub primary_index: u8,
    pub mnemonic: String,
}

/// An agent's identity index resolved into the IfáScript Odù corpus: the
/// canonical name, action vessel, and primary prescription that the index
/// `primary_index` stands for. This is the bridge between BIPỌ̀N39 identity
/// (the XOR-reduced index) and the meaning IfáScript assigns it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OduSign {
    pub index: u8,
    pub name: String,
    pub vessel: String,
    pub prescription: Option<String>,
}

impl OduIdentity {
    /// Resolve this identity's `primary_index` into its IfáScript Odù sign.
    ///
    /// `primary_index` is produced by BIPỌ̀N39 (an XOR reduction of the
    /// mnemonic's token indices over the 256 base Odù). On its own it is just a
    /// number; `sign()` looks it up in the IfáScript corpus so the identity
    /// carries its actual Odù — name, vessel, and prescription — not a bare u8.
    pub fn sign(&self) -> OduSign {
        let odu = ifascript::get_odu(self.primary_index);
        OduSign {
            index: self.primary_index,
            name: odu.universal_name.to_string(),
            vessel: format!("{:?}", odu.vessel),
            prescription: odu.prescriptions.first().map(|p| p.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sign_resolves_index_into_ifascript_odu() {
        let identity = OduIdentity {
            primary_index: 7,
            mnemonic: "Ogbe leads the way".to_string(),
        };
        let sign = identity.sign();
        // The sign preserves the identity index and pulls a real name/vessel
        // from the IfáScript corpus (never an empty string).
        assert_eq!(sign.index, 7);
        assert!(!sign.name.is_empty());
        assert!(!sign.vessel.is_empty());
    }
}
