use omokoda_core::identity::odu::{OduIdentity, OduSeed};
use omokoda_core::identity::AgentId;
use omokoda_core::session::{
    ContentBlock, ConversationMessage, PrivateSessionData, SensitiveKey, Session,
    ARGON2_ITERATIONS, ARGON2_MEMORY_KB, ARGON2_OUTPUT_LEN, ARGON2_PARALLELISM,
    ENCRYPTED_SESSION_VERSION, SESSION_VERSION,
};
use std::fs;

#[test]
fn session_new_initializes_fields() {
    let agent_id = AgentId::from_str("agent-1");
    let session = Session::new(agent_id.clone(), "luna".to_string(), 12345);

    assert_eq!(session.version, SESSION_VERSION);
    assert_eq!(session.agent_id, agent_id);
    assert_eq!(session.name, "luna");
    assert_eq!(session.birth_timestamp, 12345);
    assert_eq!(session.reputation, 0.0);
    assert!(session.public_messages.is_empty());
    assert!(session.encrypted_private.is_none());
}

#[test]
fn session_persistence_roundtrip() {
    let path = std::path::Path::new("test_session_roundtrip.json");

    let agent_id = AgentId::from_str("agent-1");
    let mut session = Session::new(agent_id, "luna".to_string(), 12345);
    session.add_message(
        ConversationMessage::new_user("hello".to_string(), false),
        0.0,
    );
    session.add_message(
        ConversationMessage::new_assistant("hi".to_string(), false),
        0.0,
    );

    session.save(path).unwrap();

    let content = fs::read_to_string(path).unwrap();
    let loaded: Session = serde_json::from_str(&content).unwrap();

    assert_eq!(loaded.name, "luna");
    assert_eq!(loaded.public_messages.len(), 2);

    if let ContentBlock::Text { text } = &loaded.public_messages[0].blocks[0] {
        assert_eq!(text, "hello");
    } else {
        panic!("Wrong block type");
    }

    let _ = std::fs::remove_file(path);
}

#[test]
fn session_encryption_roundtrip() {
    let agent_id = AgentId::from_str("agent-1");
    let mut session = Session::new(agent_id, "luna".to_string(), 12345);

    let mut entropy = [0u8; 32];
    entropy[0] = 1;
    let odu_seed = OduSeed::new(entropy);
    let odu_identity = OduIdentity {
        primary_index: 0,
        mnemonic: "test mnemonic".to_string(),
    };

    let private_data = PrivateSessionData {
        odu_seed: odu_seed.clone(),
        odu_identity: odu_identity.clone(),
        private_messages: vec![ConversationMessage::new_user("secret".to_string(), true)],
        vantage_api_key: None,
        wallet_private_key_hex: None,
    };

    // Seal
    let password_key = [0u8; 32];
    session.seal_private(&private_data, &password_key).unwrap();
    assert!(session.encrypted_private.is_some());
    let encrypted = session.encrypted_private.as_ref().unwrap();
    assert_eq!(encrypted.version, ENCRYPTED_SESSION_VERSION);
    assert!(!encrypted.private_ciphertext.is_empty());
    assert_eq!(encrypted.kdf.memory_kb, ARGON2_MEMORY_KB);
    assert_eq!(encrypted.kdf.iterations, ARGON2_ITERATIONS);
    assert_eq!(encrypted.kdf.parallelism, ARGON2_PARALLELISM);
    assert_eq!(encrypted.kdf.output_len, ARGON2_OUTPUT_LEN);

    // Unseal
    let decrypted = session.unseal_private(&password_key).unwrap();
    assert_eq!(decrypted.private_messages.len(), 1);

    if let ContentBlock::Text { text } = &decrypted.private_messages[0].blocks[0] {
        assert_eq!(text, "secret");
    } else {
        panic!("Wrong block type");
    }
}

#[test]
fn session_decryption_fails_with_wrong_password_key() {
    let agent_id = AgentId::from_str("agent-1");
    let mut session = Session::new(agent_id, "luna".to_string(), 12345);

    let odu_seed = OduSeed::new([1u8; 32]);
    let odu_identity = OduIdentity {
        primary_index: 0,
        mnemonic: "test".to_string(),
    };

    let private_data = PrivateSessionData {
        odu_seed: odu_seed.clone(),
        odu_identity: odu_identity.clone(),
        private_messages: vec![ConversationMessage::new_user("secret".to_string(), true)],
        vantage_api_key: None,
        wallet_private_key_hex: None,
    };

    let password_key = [0u8; 32];
    session.seal_private(&private_data, &password_key).unwrap();

    let wrong_key = [1u8; 32];
    let result = session.unseal_private(&wrong_key);
    assert!(result.is_err());
}

#[test]
fn session_key_rotation_works() {
    let agent_id = AgentId::from_str("agent-1");
    let mut session = Session::new(agent_id, "luna".to_string(), 12345);

    let odu_seed = OduSeed::new([1u8; 32]);
    let odu_identity = OduIdentity {
        primary_index: 0,
        mnemonic: "test".to_string(),
    };

    let private_data = PrivateSessionData {
        odu_seed: odu_seed.clone(),
        odu_identity: odu_identity.clone(),
        private_messages: vec![ConversationMessage::new_user("secret".to_string(), true)],
        vantage_api_key: None,
        wallet_private_key_hex: None,
    };

    let old_key = [0u8; 32];
    session.seal_private(&private_data, &old_key).unwrap();

    let new_key = [2u8; 32];
    session.rotate_key(&old_key, &new_key).unwrap();

    // Old key should fail
    assert!(session.unseal_private(&old_key).is_err());

    // New key should work
    let decrypted = session.unseal_private(&new_key).unwrap();
    if let ContentBlock::Text { text } = &decrypted.private_messages[0].blocks[0] {
        assert_eq!(text, "secret");
    } else {
        panic!("Wrong block type");
    }
}

#[test]
fn session_leakage_test() {
    let agent_id = AgentId::from_str("agent-1");
    let mut session = Session::new(agent_id, "luna".to_string(), 12345);

    let odu_seed = OduSeed::new([1u8; 32]);
    let odu_identity = OduIdentity {
        primary_index: 0,
        mnemonic: "test".to_string(),
    };

    let private_data = PrivateSessionData {
        odu_seed: odu_seed.clone(),
        odu_identity: odu_identity.clone(),
        private_messages: vec![ConversationMessage::new_user(
            "THIS_IS_A_SECRET".to_string(),
            true,
        )],
        vantage_api_key: None,
        wallet_private_key_hex: None,
    };

    let password_key = [0u8; 32];
    session.seal_private(&private_data, &password_key).unwrap();

    let serialized = serde_json::to_string(&session).unwrap();

    // The secret string should NOT appear in the serialized JSON
    assert!(!serialized.contains("THIS_IS_A_SECRET"));

    // But the encrypted data should be there
    assert!(serialized.contains("encrypted_private"));
    assert!(serialized.contains("private_ciphertext"));
    assert!(!serialized.contains("private_messages"));
}

#[test]
fn session_export_keeps_private_content_encrypted() {
    let agent_id = AgentId::from_str("agent-1");
    let mut session = Session::new(agent_id, "luna".to_string(), 12345);
    let odu_seed = OduSeed::new([3u8; 32]);
    let private_data = PrivateSessionData {
        odu_seed: odu_seed.clone(),
        odu_identity: OduIdentity {
            primary_index: 0,
            mnemonic: "secret mnemonic phrase".to_string(),
        },
        private_messages: vec![ConversationMessage::new_user(
            "PRIVATE_EXPORT_SECRET".to_string(),
            true,
        )],
        vantage_api_key: None,
        wallet_private_key_hex: None,
    };

    session.seal_private(&private_data, &[7u8; 32]).unwrap();
    let exported = session.export_json().unwrap();

    assert!(!exported.contains("PRIVATE_EXPORT_SECRET"));
    assert!(!exported.contains("secret mnemonic phrase"));
    assert!(exported.contains("private_ciphertext"));
}

#[test]
fn sensitive_key_zeroize_clears_key_bytes() {
    let mut key = SensitiveKey::new([9u8; 32]);
    assert_eq!(key.expose(), &[9u8; 32]);
    key.zeroize_now();
    assert_eq!(key.expose(), &[0u8; 32]);
}

#[cfg(unix)]
#[test]
fn session_save_uses_strict_file_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let path = std::path::Path::new("test_secure_session_dir/session.json");
    let agent_id = AgentId::from_str("agent-1");
    let session = Session::new(agent_id, "luna".to_string(), 12345);

    session.save(path).unwrap();
    let file_mode = fs::metadata(path).unwrap().permissions().mode() & 0o777;
    let dir_mode = fs::metadata(path.parent().unwrap())
        .unwrap()
        .permissions()
        .mode()
        & 0o777;

    assert_eq!(file_mode, 0o600);
    assert_eq!(dir_mode, 0o700);

    let _ = fs::remove_dir_all("test_secure_session_dir");
}
