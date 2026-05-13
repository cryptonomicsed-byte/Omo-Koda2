#[cfg(test)]
mod parser_overblocking_tests {
    use omokoda_core::parser::parse;

    #[test]
    fn does_not_overblock_normal_text() {
        // "metabolism" is no longer a blocked identifier
        let input = r#"think "I am concerned about my metabolism""#;
        let result = parse(input);
        assert!(result.is_ok(), "Should not overblock ordinary user text");
    }

    #[test]
    fn still_blocks_identifier_as_word() {
        // "k_root" is still a blocked identifier
        let input = r#"think "k_root""#;
        let result = parse(input);
        assert!(result.is_err());
    }
}
