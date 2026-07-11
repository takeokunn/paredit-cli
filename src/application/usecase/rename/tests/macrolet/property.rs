use super::*;

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
