//! End-to-end: drive the real SkillForgeTool through analysis → transformation
//! → audit → registration on a live (tiny) GitHub repo. Sandbox is disabled so
//! the test stays fast and network/docker-light; the Docker sandbox path is
//! covered separately. Requires outbound git (network) — ignored by default.
#[cfg(test)]
mod skillforge_e2e {
    use omokoda_core::tools::skillforge::SkillForgeTool;
    use omokoda_core::tools::{ExecutionContext, Tool};
    use std::sync::{Arc, Mutex};

    fn ctx() -> ExecutionContext {
        ExecutionContext {
            agent_id: omokoda_core::identity::AgentId::from_str("agent-forge"),
            name: "forge".to_string(),
            tier: 3,
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
    #[ignore] // needs network; run with `cargo test -- --ignored`
    async fn forge_transforms_and_registers_live() {
        let tmp = std::env::temp_dir().join(format!("sf-e2e-{}", std::process::id()));
        std::env::set_var("SKILLFORGE_FORGE", tmp.join("forge"));
        std::env::set_var("SKILLFORGE_MANIFEST", tmp.join("skills.forged.json"));
        std::env::set_var("SKILLFORGE_REVIEW", tmp.join("review"));
        std::env::set_var("SKILLFORGE_SCAN_WAIT", "6"); // keep Strix poll short in test
        std::env::set_var("SKILLFORGE_REQUIRE_FULL_AUDIT", "false"); // default lenient mode

        let skills = Arc::new(Mutex::new(Vec::new()));
        let tool = SkillForgeTool::new(skills.clone());

        // store defaults true → repo is created in Gitea and scanned.
        let params = r#"{"url":"https://github.com/octocat/Hello-World.git","sandbox":false,"approve":true}"#;
        let (out, _usage) = tool
            .execute(params, &ctx())
            .await
            .expect("forge should run");

        let receipt: serde_json::Value = serde_json::from_str(&out).unwrap();
        // stored in Gitea + security scan ran
        let storage = &receipt["storage"];
        assert!(
            storage["repo"].as_str().unwrap_or("").contains("/skill-"),
            "expected gitea repo, got: {out}"
        );
        assert!(
            storage["pushed"]
                .as_array()
                .map(|a| !a.is_empty())
                .unwrap_or(false),
            "expected pushed files, got: {out}"
        );
        // full Strix pentest dispatched (scan_id present)
        assert!(
            storage["security_scan"]["strix"]["scan_id"].is_number(),
            "expected strix scan dispatched, got: {out}"
        );
        // registered in Vantage platform skill registry
        assert!(
            receipt["vantage_registry"]["registered"]
                .as_bool()
                .unwrap_or(false),
            "expected vantage registry registration, got: {out}"
        );
        // native nuclei file scan ran as part of analysis (works now, no docker)
        assert_eq!(
            receipt["analysis"]["nuclei"]["ran"], true,
            "expected nuclei scan to run, got: {out}"
        );
        // transformation ran and added agent-native surfaces
        let surfaces = &receipt["transformation"]["added_surfaces"];
        assert!(
            surfaces.is_array() && !surfaces.as_array().unwrap().is_empty(),
            "expected transformation surfaces, got: {out}"
        );
        // approved → registered live in the shared registry
        assert_eq!(receipt["status"], "registered_with_override");
        let guard = skills.lock().unwrap();
        assert!(
            guard
                .iter()
                .any(|s| s.name == receipt["skill"]["name"].as_str().unwrap()),
            "forged skill should be hot-added to the live registry"
        );
        // gateway routes present (mcp discovery/invoke)
        let routes = &receipt["skill"]["routes"];
        assert!(
            routes.get("mcp_discover").is_some(),
            "gateway routes missing: {out}"
        );
    }

    /// Fail-closed: if the full Strix audit does not complete in the window,
    /// the skill is HELD for review even without any critical findings and
    /// without `approve`. Proves the full security audit gates registration.
    #[tokio::test]
    #[ignore] // needs network; run with `cargo test -- --ignored --test-threads=1`
    async fn forge_gates_on_incomplete_full_audit() {
        let tmp = std::env::temp_dir().join(format!("sf-gate-{}", std::process::id()));
        std::env::set_var("SKILLFORGE_FORGE", tmp.join("forge"));
        std::env::set_var("SKILLFORGE_MANIFEST", tmp.join("skills.forged.json"));
        std::env::set_var("SKILLFORGE_REVIEW", tmp.join("review"));
        std::env::set_var("SKILLFORGE_SCAN_WAIT", "1"); // Strix cannot finish in 1s
        std::env::set_var("SKILLFORGE_REQUIRE_FULL_AUDIT", "true"); // strict mode

        let skills = Arc::new(Mutex::new(Vec::new()));
        let tool = SkillForgeTool::new(skills.clone());

        // no "approve" → must be held because the full audit is incomplete
        let params = r#"{"url":"https://github.com/octocat/Hello-World.git","sandbox":false}"#;
        let (out, _usage) = tool
            .execute(params, &ctx())
            .await
            .expect("forge should run");
        let receipt: serde_json::Value = serde_json::from_str(&out).unwrap();

        assert_eq!(receipt["status"], "pending_human_review", "got: {out}");
        assert_eq!(
            receipt["storage"]["security_scan"]["audit_complete"], false,
            "expected incomplete audit, got: {out}"
        );
        let reasons = receipt["audit"]["reasons"].to_string();
        assert!(
            reasons.contains("Strix") || reasons.contains("audit"),
            "expected audit reason, got: {out}"
        );
        // not hot-added to the live registry while held
        assert!(
            skills.lock().unwrap().is_empty(),
            "held skill must not be registered live, got: {out}"
        );
    }
}
