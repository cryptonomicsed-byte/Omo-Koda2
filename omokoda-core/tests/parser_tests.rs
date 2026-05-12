#[cfg(test)]
mod parser_tests {
    use omokoda_core::parser::{parse, Statement};

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
        assert!(parse(&long).is_err());
    }

    // ── HARD REJECTION: stdlib/internal names ────────────────────

    #[test]
    fn rejects_stdlib_name_in_input() {
        // The parser must reject any input containing internal identifiers
        let cases = vec![
            r#"think "metabolism.stake()""#,
            r#"act "dopamine" "burn:100""#,
            r#"think "call odu_vault directly""#,
            r#"birth "agent" hermetic_seed:abc123"#,
            r#"think "run synapse.transfer""#,
        ];
        for case in cases {
            let result = parse(case);
            assert!(
                result.is_err(),
                "Expected rejection of internal identifier in: {case}"
            );
        }
    }

    #[test]
    fn rejects_raw_key_material() {
        let cases = vec![
            r#"think "my key is 0x366327bb08232513abcdef""#,
            r#"act "vault" "k_root:deadbeef1234""#,
        ];
        for case in cases {
            assert!(parse(case).is_err());
        }
    }

    // ── MALFORMED INPUT ──────────────────────────────────────────

    #[test]
    fn rejects_birth_without_string() {
        assert!(parse("birth").is_err());
    }

    #[test]
    fn rejects_think_without_string() {
        assert!(parse("think").is_err());
    }

    #[test]
    fn rejects_act_with_one_string() {
        // act requires exactly two quoted strings
        assert!(parse(r#"act "web_search""#).is_err());
    }

    #[test]
    fn rejects_unquoted_string_args() {
        assert!(parse("birth luna").is_err());
        assert!(parse("think hello world").is_err());
    }

    #[test]
    fn rejects_empty_quoted_string() {
        assert!(parse(r#"birth """#).is_err());
    }

    #[test]
    fn rejects_unknown_slash_command() {
        // Unknown slash commands should fail, not silently pass
        assert!(parse("/unknowncommand").is_err());
    }

    // ── KNOWN SLASH COMMANDS ─────────────────────────────────────

    #[test]
    fn accepts_all_valid_slash_commands() {
        let valid = vec![
            "/private",
            "/publish",
            "/sandbox",
            "/status",
            "/transfer",
            "/configure",
            "/help",
            "/tools",
        ];
        for cmd in valid {
            assert!(parse(cmd).is_ok(), "Expected valid slash cmd: {cmd}");
        }
    }
}
