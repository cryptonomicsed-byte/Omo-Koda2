use omokoda_core::interpreter::Steward;
use omokoda_core::parser::parse;
use omokoda_core::session::MessageRole;
use std::path::PathBuf;

macro_rules! test_steward {
    ($name:expr) => {{
        let mut path = std::env::current_dir().unwrap();
        path.push("target");
        path.push("test_sessions");
        path.push($name);
        if path.exists() {
            let _ = std::fs::remove_dir_all(&path);
        }
        std::fs::create_dir_all(&path).unwrap();
        Steward::new().with_session_dir(path)
    }};
}

#[tokio::test]
async fn full_private_e2e_flow() {
    let session_dir = std::env::current_dir()
        .unwrap()
        .join("target")
        .join("test_sessions")
        .join("full_private_e2e_flow");
    if session_dir.exists() {
        let _ = std::fs::remove_dir_all(&session_dir);
    }
    std::fs::create_dir_all(&session_dir).unwrap();

    let mut steward = Steward::new().with_session_dir(session_dir.clone());

    // 1. Birth
    steward.set_mock_provider("42 is the answer".to_string());
    steward
        .dispatch(parse(r#"birth "luna" provider:ollama"#).unwrap()[0].clone())
        .await
        .unwrap();
    let agent_id = steward.agent_core().unwrap().id().clone();

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
    assert!(steward.agent_core().unwrap().private_data().is_none());
    let saved_path = steward.agent_storage_path(&agent_id);
    let saved_json = std::fs::read_to_string(&saved_path).unwrap();
    assert!(!saved_json.contains("my secret is 42"));
    assert!(!saved_json.contains("42 is the answer"));
    assert!(saved_json.contains("private_ciphertext"));

    // 4. Resume (new Steward)
    let mut steward2 = Steward::new().with_session_dir(session_dir.clone());
    steward2.set_mock_provider("42 is the answer".to_string());
    steward2.load_agent(&agent_id).unwrap();
    assert!(steward2.agent_core().unwrap().private_data().is_none());

    // 5. Unlock
    steward2
        .dispatch(parse(r#"/unlock "password123""#).unwrap()[0].clone())
        .await
        .unwrap();

    // Verify private messages are restored
    let pd = steward2.agent_core().unwrap().private_data().unwrap();
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
        .dispatch(
            parse(r#"act "read_file" "target/test_sessions/full_private_e2e_flow/test.txt""#)
                .unwrap()[0]
                .clone(),
        )
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

#[tokio::test]
async fn multi_agent_storage_isolation() {
    let mut steward1 = test_steward!("multi_agent_storage_isolation_1");
    let mut steward2 = test_steward!("multi_agent_storage_isolation_2");

    steward1
        .dispatch(parse(r#"birth "agent1""#).unwrap()[0].clone())
        .await
        .unwrap();
    steward2
        .dispatch(parse(r#"birth "agent2""#).unwrap()[0].clone())
        .await
        .unwrap();

    let id1 = steward1.agent_core().unwrap().id().clone();
    let id2 = steward2.agent_core().unwrap().id().clone();

    assert!(id1 != id2);
    assert!(steward1.agent_storage_path(&id1).exists());
    assert!(steward2.agent_storage_path(&id2).exists());
}
