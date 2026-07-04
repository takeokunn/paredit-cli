use super::*;

#[test]
fn plans_flet_rename_without_touching_definition_body_references() {
    let input = "(flet ((foo (x) (foo x))) (foo 1) foo)\n";
    let plan = plan_rename_local_function(RenameLocalFunctionRequest {
        input,
        dialect: Dialect::CommonLisp,
        from: SymbolName::new("foo").unwrap(),
        to: SymbolName::new("bar").unwrap(),
    })
    .unwrap();

    assert_eq!(plan.definitions.len(), 1);
    assert_eq!(plan.calls.len(), 1);
    assert!(plan.changed);
    assert!(plan
        .rewritten
        .contains("(flet ((bar (x) (foo x))) (bar 1) foo)"));
}

#[test]
fn plans_labels_rename_with_recursive_definition_body_references() {
    let input = "(labels ((foo (x) (foo x))) (foo 1) foo)\n";
    let plan = plan_rename_local_function(RenameLocalFunctionRequest {
        input,
        dialect: Dialect::CommonLisp,
        from: SymbolName::new("foo").unwrap(),
        to: SymbolName::new("bar").unwrap(),
    })
    .unwrap();

    assert_eq!(plan.definitions.len(), 1);
    assert_eq!(plan.calls.len(), 2);
    assert!(plan.changed);
    assert!(plan
        .rewritten
        .contains("(labels ((bar (x) (bar x))) (bar 1) foo)"));
}

#[test]
fn skips_nested_labels_calls_when_renaming_outer_flet() {
    let input = "(flet ((foo (x) x)) (labels ((foo (y) (foo y))) (foo 1)) (foo 2))\n";
    let plan = plan_rename_local_function(RenameLocalFunctionRequest {
        input,
        dialect: Dialect::CommonLisp,
        from: SymbolName::new("foo").unwrap(),
        to: SymbolName::new("bar").unwrap(),
    })
    .unwrap();

    assert_eq!(plan.definitions.len(), 1);
    assert_eq!(plan.calls.len(), 1);
    assert!(plan.rewritten.contains("(flet ((bar (x) x))"));
    assert!(plan
        .rewritten
        .contains("(labels ((foo (y) (foo y))) (foo 1))"));
    assert!(plan.rewritten.contains("(bar 2)"));
}

#[test]
fn nested_flet_definition_body_can_still_reference_outer_function() {
    let input = "(flet ((foo (x) x)) (flet ((foo (y) (foo y))) (foo 1)) (foo 2))\n";
    let plan = plan_rename_local_function(RenameLocalFunctionRequest {
        input,
        dialect: Dialect::CommonLisp,
        from: SymbolName::new("foo").unwrap(),
        to: SymbolName::new("bar").unwrap(),
    })
    .unwrap();

    assert_eq!(plan.definitions.len(), 1);
    assert_eq!(plan.calls.len(), 2);
    assert!(plan.rewritten.contains("(flet ((bar (x) x))"));
    assert!(plan
        .rewritten
        .contains("(flet ((foo (y) (bar y))) (foo 1))"));
    assert!(plan.rewritten.contains("(bar 2)"));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn pbt_rename_labels_output_remains_parseable_and_updates_recursive_calls(
        from in symbol_strategy(),
        to in symbol_strategy(),
    ) {
        prop_assume!(from != to);
        let input = format!("(labels (({from} (x) ({from} x))) ({from} 1) {from})");
        let plan = plan_rename_local_function(RenameLocalFunctionRequest {
            input: &input,
            dialect: Dialect::CommonLisp,
            from: SymbolName::new(from.clone()).unwrap(),
            to: SymbolName::new(to.clone()).unwrap(),
        }).unwrap();

        SyntaxTree::parse(&plan.rewritten).unwrap();
        prop_assert!(plan.changed);
        let rewritten_definition = format!("(labels (({} (x)", to);
        let rewritten_recursive_call = format!("({} x)", to);
        let rewritten_body_call = format!("({} 1)", to);

        prop_assert!(plan.rewritten.contains(&rewritten_definition));
        prop_assert!(plan.rewritten.contains(&rewritten_recursive_call));
        prop_assert!(plan.rewritten.contains(&rewritten_body_call));
    }
}
