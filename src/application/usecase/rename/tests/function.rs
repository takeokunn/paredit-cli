use super::*;

#[test]
fn plans_function_rename_without_value_references() {
    let input = "(defun foo (x) (list foo x))\n(defun caller () (foo 1))";
    let plan = plan_rename_function(RenameFunctionRequest {
        input,
        dialect: Dialect::CommonLisp,
        from: SymbolName::new("foo").unwrap(),
        to: SymbolName::new("baz").unwrap(),
    })
    .unwrap();

    assert_eq!(plan.definitions.len(), 1);
    assert_eq!(plan.calls.len(), 1);
    assert!(plan.changed);
    assert!(plan.rewritten.contains("(defun baz (x)"));
    assert!(plan.rewritten.contains("(baz 1)"));
    assert!(plan.rewritten.contains("(list foo x)"));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn pbt_rename_function_output_remains_parseable_and_preserves_value_refs(
        from in symbol_strategy(),
        to in symbol_strategy(),
    ) {
        prop_assume!(from != to);
        let input = format!("(defun {from} (x) (list {from} x))\n(defun caller () ({from} 1))");
        let plan = plan_rename_function(RenameFunctionRequest {
            input: &input,
            dialect: Dialect::CommonLisp,
            from: SymbolName::new(from.clone()).unwrap(),
            to: SymbolName::new(to.clone()).unwrap(),
        }).unwrap();

        SyntaxTree::parse(&plan.rewritten).unwrap();
        prop_assert!(plan.changed);
        let rewritten_definition = format!("(defun {} (x)", to);
        let rewritten_call = format!("({} 1)", to);
        let preserved_value_reference = format!("(list {} x)", from);

        prop_assert!(plan.rewritten.contains(&rewritten_definition));
        prop_assert!(plan.rewritten.contains(&rewritten_call));
        prop_assert!(plan.rewritten.contains(&preserved_value_reference));
    }
}
