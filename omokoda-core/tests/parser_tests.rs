#[cfg(test)]
mod parser_tests {
    use omokoda_core::parser::{parse, ParseErrorCode, Statement};

    // ── VALID INPUT ──────────────────────────────────────────────

    #[test]
    fn birth_minimal() {
        let result = parse(r#"birth "luna""#);
        assert!(result.is_ok());
        let stmts = result.unwrap();
        assert_eq!(stmts.len(), 1);
        assert!(matches!(stmts[0], Statement::Birth { .. }));
    }

    #[test]
    fn birth_with_metadata() {
        let result = parse(r#"birth "luna" tier:3 budget:1000"#);
        assert!(result.is_ok());
    }

    #[test]
    fn think_minimal() {
        let result = parse(r#"think "what is sovereignty""#);
        assert!(result.is_ok());
        assert!(matches!(
            result.unwrap()[0],
            Statement::Think { private: true, .. }
        ));
    }

    #[test]
    fn think_publish_flag_makes_thought_public() {
        let result = parse(r#"think "publishable thought" /publish"#);
        assert!(result.is_ok());
        assert!(matches!(
            result.unwrap()[0],
            Statement::Think { private: false, .. }
        ));
    }

    #[test]
    fn think_private_flag() {
        let result = parse(r#"think "private thought" /private"#);
        assert!(result.is_ok());
        assert!(matches!(
            result.unwrap()[0],
            Statement::Think { private: true, .. }
        ));
    }

    #[test]
    fn think_accepts_natural_modifiers_without_new_primitives() {
        let result = parse(
            r#"think "plan a release" loop:true max_iterations:5 priority:high /sandbox /private"#,
        );
        assert!(result.is_ok());
        let stmts = result.unwrap();
        assert_eq!(stmts.len(), 1);
        match &stmts[0] {
            Statement::Think {
                private, modifiers, ..
            } => {
                assert!(*private);
                assert!(modifiers.loop_enabled);
                assert_eq!(modifiers.max_iterations, Some(5));
                assert_eq!(modifiers.priority.as_deref(), Some("high"));
                assert!(modifiers.sandbox);
            }
            _ => panic!("expected think"),
        }
    }

    #[test]
    fn act_minimal() {
        let result = parse(r#"act "web_search" "bitcoin origin""#);
        assert!(result.is_ok());
        assert!(matches!(
            result.unwrap()[0],
            Statement::Act { sandbox: false, .. }
        ));
    }

    #[test]
    fn act_sandbox_flag() {
        let result = parse(r#"act "web_search" "test query" /sandbox"#);
        assert!(result.is_ok());
        assert!(matches!(
            result.unwrap()[0],
            Statement::Act { sandbox: true, .. }
        ));
    }

    #[test]
    fn text_fallback_becomes_think() {
        let result = parse("hello, what can you do?");
        assert!(result.is_ok());
        assert!(matches!(
            result.unwrap()[0],
            Statement::Think { private: true, .. }
        ));
    }

    #[test]
    fn slash_command_status() {
        let result = parse("/status");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap()[0], Statement::SlashCmd { .. }));
    }

    #[test]
    fn multi_statement_program() {
        let input = r#"
birth "luna"
think "hello"
act "log" "started"
"#;
        let result = parse(input);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 3);
    }

    // ── INPUT LIMITS ─────────────────────────────────────────────

    #[test]
    fn rejects_input_over_4096_bytes() {
        let long = format!(r#"think "{}""#, "a".repeat(4100));
        let err = parse(&long).unwrap_err();
        assert_eq!(err.code, ParseErrorCode::InputTooLong);
    }

    // ── HARD REJECTION: stdlib/internal names ────────────────────

    #[test]
    fn rejects_stdlib_name_in_input() {
        let cases = vec![
            r#"think "k_root""#,
            r#"act "odu_vault" "burn:100""#,
            r#"birth "agent" kdf:argon2id"#,
        ];
        for case in cases {
            let result = parse(case);
            let err = result.unwrap_err();
            assert_eq!(err.code, ParseErrorCode::BlockedIdentifier);
        }
    }

    #[test]
    fn rejects_raw_key_material() {
        let cases = vec![
            r#"think "my key is 0x366327bb08232513abcdef""#,
            r#"act "vault" "key:deadbeef1234""#,
        ];
        for case in cases {
            let err = parse(case).unwrap_err();
            assert_eq!(err.code, ParseErrorCode::RawKeyMaterial);
        }
    }

    // ── MALFORMED INPUT ──────────────────────────────────────────

    #[test]
    fn rejects_birth_without_string() {
        let err = parse("birth").unwrap_err();
        assert_eq!(err.code, ParseErrorCode::MissingArgument);
    }

    #[test]
    fn rejects_think_without_string() {
        let err = parse("think").unwrap_err();
        assert_eq!(err.code, ParseErrorCode::MissingArgument);
    }

    #[test]
    fn rejects_act_with_one_string() {
        let err = parse(r#"act "web_search""#).unwrap_err();
        assert_eq!(err.code, ParseErrorCode::MissingArgument);
    }

    #[test]
    fn rejects_unquoted_string_args() {
        let err = parse("birth luna").unwrap_err();
        assert_eq!(err.code, ParseErrorCode::MissingArgument);
    }

    #[test]
    fn rejects_empty_quoted_string() {
        let err = parse(r#"birth """#).unwrap_err();
        assert_eq!(err.code, ParseErrorCode::EmptyArgument);
    }

    #[test]
    fn unknown_slash_routes_to_skill_act() {
        // Slash sugar: a non-builtin slash command is no longer an error — it is
        // `act` on that skill name (resolution happens later at the tool layer).
        let stmts = parse("/unknowncommand").unwrap();
        assert_eq!(stmts.len(), 1);
        match &stmts[0] {
            Statement::Act { tool, sandbox, .. } => {
                assert_eq!(tool, "unknowncommand");
                assert!(!sandbox);
            }
            other => panic!("expected Act, got {other:?}"),
        }
    }
}
