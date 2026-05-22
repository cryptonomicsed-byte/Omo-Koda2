// Privacy enforcement layer — gates provider selection, storage, and hive access
// based on the resolved UserIdentity privacy mode.
use crate::identity::user::{PrivacyMode, UserIdentity};

pub struct PrivacyEnforcer;

impl PrivacyEnforcer {
    /// Returns Err if a non-local provider is attempted while user requires local-only inference.
    pub fn check_provider_allowed(mode: &PrivacyMode, provider_is_local: bool) -> Result<(), String> {
        if *mode != PrivacyMode::Public && !provider_is_local {
            Err(format!(
                "HARD FAIL: {:?} mode requires a local provider — external inference not permitted",
                mode
            ))
        } else {
            Ok(())
        }
    }

    /// Returns Err if a storage write is attempted for an incognito user.
    pub fn check_storage_allowed(identity: &UserIdentity) -> Result<(), String> {
        if identity.is_incognito() {
            Err("HARD FAIL: incognito session — no persistent storage permitted".to_string())
        } else {
            Ok(())
        }
    }

    /// Returns Err if a hive contribution is attempted without explicit public consent.
    pub fn check_hive_allowed(identity: &UserIdentity) -> Result<(), String> {
        if !identity.allows_hive_contribution() {
            Err(format!(
                "HARD FAIL: {:?} mode — hive contribution requires explicit Public consent",
                identity.privacy
            ))
        } else {
            Ok(())
        }
    }

    /// Single gate: run all privacy checks for a full think turn.
    pub fn gate_think(identity: &UserIdentity, provider_is_local: bool) -> Result<(), String> {
        Self::check_provider_allowed(&identity.privacy, provider_is_local)?;
        Ok(())
    }

    /// Single gate: run all privacy checks for an act (tool execution) turn.
    pub fn gate_act(identity: &UserIdentity) -> Result<(), String> {
        // Act execution itself is always allowed; storage of the receipt depends on privacy mode.
        // Incognito agents can still act — their receipts are not persisted.
        let _ = identity;
        Ok(())
    }

    /// Single gate: run all privacy checks before writing a memory cell.
    pub fn gate_store_memory(identity: &UserIdentity) -> Result<(), String> {
        Self::check_storage_allowed(identity)
    }

    /// Single gate: run all privacy checks before contributing to the public hive.
    pub fn gate_hive_write(identity: &UserIdentity) -> Result<(), String> {
        Self::check_hive_allowed(identity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn public_user() -> UserIdentity {
        let mut u = UserIdentity::from_seed("pub-test");
        u.apply_privacy(PrivacyMode::Public);
        u
    }

    fn private_user() -> UserIdentity {
        UserIdentity::from_seed("priv-test") // seeds default to Private
    }

    fn incognito_user() -> UserIdentity {
        let mut u = UserIdentity::from_seed("incog-test");
        u.apply_privacy(PrivacyMode::Incognito);
        u
    }

    #[test]
    fn public_user_allows_external_provider() {
        assert!(PrivacyEnforcer::check_provider_allowed(&PrivacyMode::Public, false).is_ok());
    }

    #[test]
    fn private_mode_blocks_external_provider() {
        let res = PrivacyEnforcer::check_provider_allowed(&PrivacyMode::Private, false);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("HARD FAIL"));
    }

    #[test]
    fn incognito_blocks_external_provider() {
        let res = PrivacyEnforcer::check_provider_allowed(&PrivacyMode::Incognito, false);
        assert!(res.is_err());
    }

    #[test]
    fn incognito_blocks_storage() {
        let u = incognito_user();
        let res = PrivacyEnforcer::check_storage_allowed(&u);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("HARD FAIL"));
    }

    #[test]
    fn private_user_allows_storage() {
        let u = private_user();
        assert!(PrivacyEnforcer::check_storage_allowed(&u).is_ok());
    }

    #[test]
    fn only_public_user_may_write_hive() {
        assert!(PrivacyEnforcer::check_hive_allowed(&public_user()).is_ok());
        assert!(PrivacyEnforcer::check_hive_allowed(&private_user()).is_err());
        assert!(PrivacyEnforcer::check_hive_allowed(&incognito_user()).is_err());
    }

    #[test]
    fn gate_think_public_local_ok() {
        assert!(PrivacyEnforcer::gate_think(&public_user(), true).is_ok());
    }

    #[test]
    fn gate_think_private_external_hard_fail() {
        assert!(PrivacyEnforcer::gate_think(&private_user(), false).is_err());
    }
}
