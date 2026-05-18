use omokoda_core::interpreter::Steward;
use omokoda_core::parser::parse;
use omokoda_core::session::MessageRole;

#[tokio::test]
async fn full_private_e2e_flow() {
    let mut steward = Steward::new();
    let session_dir = std::env::current_dir().unwrap().join("test_sessions_e2e");
    if !session_dir.exists() {
        std::fs::create_dir_all(&session_dir).unwrap();
    }

    // 1. Birth
    steward.set_session_dir(session_dir.clone());
    steward.set_mock_provider("42 is the answer".to_string());
    steward
        .dispatch(parse(r#"birth "luna" provider:ollama"#).unwrap()[0].clone())
        .await
        .unwrap();
    let agent_id = steward.agent_state().unwrap().id().clone();

    // 2. Private Think
    steward
        .dispatch(parse(r#"think "my secret is 42" /private"#).unwrap()[0].clone())
        .await
        .unwrap();

    // 3. Seal
    steward
        .dispatch(parse(r#"/seal "password123""#).unwrap()[0].clone())
        .await
        .unwrap();

    // Verify private data is gone from memory
    assert!(steward.agent_state().unwrap().private_data().is_none());
    let saved_path = steward.agent_storage_path(&agent_id);
    let saved_json = std::fs::read_to_string(&saved_path).unwrap();
    assert!(!saved_json.contains("my secret is 42"));
    assert!(!saved_json.contains("42 is the answer"));
    assert!(saved_json.contains("private_ciphertext"));

    // 4. Resume (new Steward)
    let mut steward2 = Steward::new();
    steward2.set_session_dir(session_dir.clone());
    steward2.set_mock_provider("42 is the answer".to_string());
    steward2.load_agent(&agent_id).unwrap();
    assert!(steward2.agent_state().unwrap().private_data().is_none());

    // 5. Unlock
    steward2
        .dispatch(parse(r#"/unlock "password123""#).unwrap()[0].clone())
        .await
        .unwrap();

    // Verify private messages are restored
    let pd = steward2.agent_state().unwrap().private_data().unwrap();
    assert!(pd.private_messages.iter().any(|m| {
        m.role == MessageRole::User
            && m.blocks.iter().any(|b| {
                if let omokoda_core::session::ContentBlock::Text { text } = b {
                    text.contains("secret is 42")
                } else {
                    false
                }
            })
    }));

    // 6. Act and Receipt
    let test_file = session_dir.join("test.txt");
    std::fs::write(&test_file, "hello integration").unwrap();
    steward2.set_reputation_for_test(100.0); // Ensure tier high enough
    steward2.set_permission_mode(omokoda_core::permissions::PermissionMode::Allow);
    let res = steward2
        .dispatch(parse(r#"act "read_file" "test_sessions_e2e/test.txt""#).unwrap()[0].clone())
        .await
        .unwrap();
    assert!(res.receipt.is_some());
    let receipt = res.receipt.unwrap();
    assert_eq!(receipt.action, "read_file");
    steward2
        .dispatch(parse(r#"/seal "password123""#).unwrap()[0].clone())
        .await
        .unwrap();
    let re_saved_json = std::fs::read_to_string(steward2.agent_storage_path(&agent_id)).unwrap();
    assert!(!re_saved_json.contains("secret is 42"));
    assert!(!re_saved_json.contains("now it works"));

    // Cleanup
    let _ = std::fs::remove_dir_all(session_dir);
}
