#[cfg(test)]
mod parser_composition_tests {
    use omokoda_core::parser::{parse, Statement};

    #[test]
    fn think_with_multiple_flags_order_1() {
        let result = parse(r#"think "test" /private /publish"#);
        // /publish overrides /private if both are present in our simple while loop
        assert!(result.is_ok());
        let stmts = result.unwrap();
        assert_eq!(stmts.len(), 1);
        assert!(matches!(stmts[0], Statement::Think { private: false, .. }));
    }

    #[test]
    fn think_with_multiple_flags_order_2() {
        let result = parse(r#"think "test" /publish /private"#);
        assert!(result.is_ok());
        let stmts = result.unwrap();
        assert_eq!(stmts.len(), 1);
        assert!(matches!(stmts[0], Statement::Think { private: true, .. }));
    }

    #[test]
    fn act_with_multiple_flags() {
        let result = parse(r#"act "tool" "params" /sandbox /sandbox"#);
        assert!(result.is_ok());
        let stmts = result.unwrap();
        assert_eq!(stmts.len(), 1);
        assert!(matches!(stmts[0], Statement::Act { sandbox: true, .. }));
    }
}
