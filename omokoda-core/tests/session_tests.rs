use omokoda_core::identity::odu::{OduIdentity, OduSeed};
use omokoda_core::identity::AgentId;
use omokoda_core::session::{
    ContentBlock, ConversationMessage, PrivateSessionData, Session, SESSION_VERSION,
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
    session.add_message(ConversationMessage::new_user("hello".to_string(), false));
    session.add_message(ConversationMessage::new_assistant("hi".to_string(), false));

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
    };

    // Seal
    session.seal_private(&private_data, &odu_seed).unwrap();
    assert!(session.encrypted_private.is_some());

    // Unseal
    let decrypted = session.unseal_private(&odu_seed).unwrap();
    assert_eq!(decrypted.private_messages.len(), 1);
    
    if let ContentBlock::Text { text } = &decrypted.private_messages[0].blocks[0] {
        assert_eq!(text, "secret");
    } else {
        panic!("Wrong block type");
    }
}

#[test]
fn session_decryption_fails_with_wrong_seed() {
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
    };

    session.seal_private(&private_data, &odu_seed).unwrap();

    let wrong_seed = OduSeed::new([2u8; 32]);
    let result = session.unseal_private(&wrong_seed);
    assert!(result.is_err());
}
