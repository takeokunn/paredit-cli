use super::*;

#[test]
fn wraps_outermost_calls_and_skips_nested_and_existing() {
    let input = "(foo (foo x))\n(trace (foo y))";
    let plan = plan_wrap_function_calls(WrapFunctionCallsRequest {
        input,
        dialect: Dialect::CommonLisp,
        function: SymbolName::new("foo").unwrap(),
        wrapper: SymbolName::new("trace").unwrap(),
        scope: WrapFunctionCallsScope::AllCalls,
    })
    .unwrap();

    assert_eq!(plan.calls.len(), 1);
    assert_eq!(plan.skipped_nested.len(), 1);
    assert_eq!(plan.skipped_already_wrapped.len(), 1);
    assert_eq!(plan.rewritten, "(trace (foo (foo x)))\n(trace (foo y))");
    SyntaxTree::parse(&plan.rewritten).unwrap();
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn pbt_wrap_function_calls_output_remains_parseable(
        function in symbol_strategy(),
        wrapper in symbol_strategy(),
        arg in symbol_strategy(),
    ) {
        prop_assume!(function != wrapper);
        prop_assume!(function != arg);
        let input = format!("({function} {arg})");
        let plan = plan_wrap_function_calls(WrapFunctionCallsRequest {
            input: &input,
            dialect: Dialect::CommonLisp,
            function: SymbolName::new(function.clone()).unwrap(),
            wrapper: SymbolName::new(wrapper.clone()).unwrap(),
            scope: WrapFunctionCallsScope::AllCalls,
        }).unwrap();

        SyntaxTree::parse(&plan.rewritten).unwrap();
        prop_assert!(plan.changed);
        prop_assert_eq!(plan.rewritten, format!("({wrapper} ({function} {arg}))"));
    }
}
