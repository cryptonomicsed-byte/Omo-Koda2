//! Proves a skill forged into the live session (via add_session_skill) is
//! immediately gated and invocable through the registry, without a reload.
#[cfg(test)]
mod skillforge_hotadd {
    use omokoda_core::permissions::{PermissionMode, PermissionPolicy};
    use omokoda_core::tools::skills::SkillManifestEntry;
    use omokoda_core::tools::{ExecutionContext, ToolRegistry};
    use std::collections::HashMap;

    fn ctx(tier: u8) -> ExecutionContext {
        ExecutionContext {
            agent_id: omokoda_core::identity::AgentId::from_str("agent-forge"),
            name: "forge".to_string(),
            tier,
            reputation: 0.0,
            odu_identity: omokoda_core::identity::odu::OduIdentity {
                primary_index: 0,
                mnemonic: "".into(),
            },
            workspace_root: std::env::current_dir().unwrap(),
            sandbox_mode: false,
        }
    }

    #[tokio::test]
    async fn forged_skill_is_gated_and_invocable_same_session() {
        // A mock upstream the forged skill will call.
        let server = httpmock::MockServer::start();
        let mock = server.mock(|when, then| {
            when.method("GET").path("/health");
            then.status(200).body("{\"status\":\"ok\"}");
        });

        let registry = ToolRegistry::new();

        // Before forging: unknown tool, not allowed at any tier.
        assert!(!registry.is_allowed("myforge", 5));

        let mut routes = HashMap::new();
        routes.insert("health".to_string(), "GET /health".to_string());
        let entry = SkillManifestEntry {
            name: "myforge".to_string(),
            description: "forged in-session".to_string(),
            base_url: server.base_url(), // literal URL, resolve_env passes through
            auth_header: None,
            auth_env: None,
            auth_value: None,
            required_tier: 2,
            write: false,
            routes,
        };

        // Hot-add — the SkillForge path.
        registry.add_session_skill(entry);

        // Tier gate honored: tier 1 too low, tier 2 allowed.
        assert!(!registry.is_allowed("myforge", 1));
        assert!(registry.is_allowed("myforge", 2));

        // Invocable immediately, no reload.
        let policy = PermissionPolicy::default_steward_policy(PermissionMode::DangerFullAccess);
        let (out, _usage) = registry
            .execute("myforge", "{\"route\":\"health\"}", ctx(2), &policy, None)
            .await
            .expect("forged skill should execute");
        assert!(out.contains("ok"), "got: {out}");
        mock.assert();
    }
}
