use super::*;

#[test]
fn plans_macrolet_rename_without_touching_expander_body_references() {
    let input = "(macrolet ((foo (x) (list foo x))) (foo 1) foo)\n";
    let plan = plan_rename_macrolet(RenameMacroletRequest {
        input,
        dialect: Dialect::CommonLisp,
        from: SymbolName::new("foo").unwrap(),
        to: SymbolName::new("bar").unwrap(),
    })
    .unwrap();

    assert_eq!(plan.definitions.len(), 1);
    assert_eq!(plan.calls.len(), 1);
    assert!(plan.changed);
    assert!(
        plan.rewritten
            .contains("(macrolet ((bar (x) (list foo x))) (bar 1) foo)")
    );
}

#[test]
fn plans_compiler_macrolet_rename_without_touching_expander_body_references() {
    let input = "(compiler-macrolet ((foo (x) (list foo x))) (foo 1) foo)\n";
    let plan = plan_rename_macrolet(RenameMacroletRequest {
        input,
        dialect: Dialect::CommonLisp,
        from: SymbolName::new("foo").unwrap(),
        to: SymbolName::new("bar").unwrap(),
    })
    .unwrap();

    assert_eq!(plan.definitions.len(), 1);
    assert_eq!(plan.calls.len(), 1);
    assert!(plan.changed);
    assert!(
        plan.rewritten
            .contains("(compiler-macrolet ((bar (x) (list foo x))) (bar 1) foo)")
    );
}

#[test]
fn skips_nested_macrolet_calls_when_renaming_outer_macrolet() {
    let input = "(macrolet ((foo (x) x)) (macrolet ((foo (y) (foo y))) (foo 1)) (foo 2))\n";
    let plan = plan_rename_macrolet(RenameMacroletRequest {
        input,
        dialect: Dialect::CommonLisp,
        from: SymbolName::new("foo").unwrap(),
        to: SymbolName::new("bar").unwrap(),
    })
    .unwrap();

    assert_eq!(plan.definitions.len(), 1);
    assert_eq!(plan.calls.len(), 1);
    assert!(plan.rewritten.contains("(macrolet ((bar (x) x))"));
    assert!(
        plan.rewritten
            .contains("(macrolet ((foo (y) (foo y))) (foo 1))")
    );
    assert!(plan.rewritten.contains("(bar 2)"));
}

#[test]
fn skips_flet_calls_that_shadow_macrolet_binding() {
    let input = "(macrolet ((foo (x) x)) (flet ((foo (y) y)) (foo 1)) (foo 2))\n";
    let plan = plan_rename_macrolet(RenameMacroletRequest {
        input,
        dialect: Dialect::CommonLisp,
        from: SymbolName::new("foo").unwrap(),
        to: SymbolName::new("bar").unwrap(),
    })
    .unwrap();

    assert_eq!(plan.definitions.len(), 1);
    assert_eq!(plan.calls.len(), 1);
    assert!(plan.rewritten.contains("(macrolet ((bar (x) x))"));
    assert!(plan.rewritten.contains("(flet ((foo (y) y)) (foo 1))"));
    assert!(plan.rewritten.contains("(bar 2)"));
}

#[test]
fn skips_labels_calls_that_shadow_macrolet_binding() {
    let input = "(macrolet ((foo (x) x)) (labels ((foo (y) (foo y))) (foo 1)) (foo 2))\n";
    let plan = plan_rename_macrolet(RenameMacroletRequest {
        input,
        dialect: Dialect::CommonLisp,
        from: SymbolName::new("foo").unwrap(),
        to: SymbolName::new("bar").unwrap(),
    })
    .unwrap();

    assert_eq!(plan.definitions.len(), 1);
    assert_eq!(plan.calls.len(), 1);
    assert!(plan.rewritten.contains("(macrolet ((bar (x) x))"));
    assert!(
        plan.rewritten
            .contains("(labels ((foo (y) (foo y))) (foo 1))")
    );
    assert!(plan.rewritten.contains("(bar 2)"));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn pbt_rename_macrolet_output_remains_parseable_and_preserves_inner_body_refs(
        from in symbol_strategy(),
        to in symbol_strategy(),
    ) {
        prop_assume!(from != to);
        let input = format!("(macrolet (({from} (x) (list {from} x))) ({from} 1) {from})");
        let plan = plan_rename_macrolet(RenameMacroletRequest {
            input: &input,
            dialect: Dialect::CommonLisp,
            from: SymbolName::new(from.clone()).unwrap(),
            to: SymbolName::new(to.clone()).unwrap(),
        }).unwrap();

        SyntaxTree::parse(&plan.rewritten).unwrap();
        prop_assert!(plan.changed);
        let rewritten_definition = format!("(macrolet (({} (x)", to);
        let rewritten_call = format!("({} 1)", to);
        let preserved_inner_reference = format!("(list {} x)", from);

        prop_assert!(plan.rewritten.contains(&rewritten_definition));
        prop_assert!(plan.rewritten.contains(&rewritten_call));
        prop_assert!(plan.rewritten.contains(&preserved_inner_reference));
    }
}
