use super::*;

#[test]
fn unwraps_outermost_unary_wrappers_and_skips_unsafe_sites() {
    let input = "(trace (foo (trace (foo x))))\n(trace (foo y) :label \"y\")";
    let plan = plan_unwrap_function_calls(UnwrapFunctionCallsRequest {
        input,
        dialect: Dialect::CommonLisp,
        function: SymbolName::new("foo").unwrap(),
        wrapper: SymbolName::new("trace").unwrap(),
        scope: UnwrapFunctionCallsScope::AllCalls,
    })
    .unwrap();

    assert_eq!(plan.calls.len(), 1);
    assert_eq!(plan.skipped_nested.len(), 1);
    assert_eq!(plan.skipped_non_unary_wrapper.len(), 1);
    assert_eq!(
        plan.rewritten,
        "(foo (trace (foo x)))\n(trace (foo y) :label \"y\")"
    );
    SyntaxTree::parse(&plan.rewritten).unwrap();
}

#[test]
fn explicit_path_rejects_non_wrapper_targets() {
    let input = "(defun render () (foo x))";
    let error = plan_unwrap_function_calls(UnwrapFunctionCallsRequest {
        input,
        dialect: Dialect::CommonLisp,
        function: SymbolName::new("foo").unwrap(),
        wrapper: SymbolName::new("trace").unwrap(),
        scope: UnwrapFunctionCallsScope::ExplicitPaths(vec![Path::from_indexes(vec![0, 2])]),
    })
    .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("call-path 0.2 is not a unary trace wrapper around foo")
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn pbt_unwrap_function_calls_output_remains_parseable(
        function in symbol_strategy(),
        wrapper in symbol_strategy(),
        arg in symbol_strategy(),
    ) {
        prop_assume!(function != wrapper);
        prop_assume!(function != arg);
        let input = format!("({wrapper} ({function} {arg}))");
        let plan = plan_unwrap_function_calls(UnwrapFunctionCallsRequest {
            input: &input,
            dialect: Dialect::CommonLisp,
            function: SymbolName::new(function.clone()).unwrap(),
            wrapper: SymbolName::new(wrapper.clone()).unwrap(),
            scope: UnwrapFunctionCallsScope::AllCalls,
        }).unwrap();

        SyntaxTree::parse(&plan.rewritten).unwrap();
        prop_assert!(plan.changed);
        prop_assert_eq!(plan.rewritten, format!("({function} {arg})"));
    }

    #[test]
    fn pbt_wrap_then_unwrap_round_trips_simple_calls(
        function in symbol_strategy(),
        wrapper in symbol_strategy(),
        arg in symbol_strategy(),
    ) {
        prop_assume!(function != wrapper);
        prop_assume!(function != arg);
        let input = format!("({function} {arg})");
        let wrapped = plan_wrap_function_calls(WrapFunctionCallsRequest {
            input: &input,
            dialect: Dialect::CommonLisp,
            function: SymbolName::new(function.clone()).unwrap(),
            wrapper: SymbolName::new(wrapper.clone()).unwrap(),
            scope: WrapFunctionCallsScope::AllCalls,
        }).unwrap();
        let unwrapped = plan_unwrap_function_calls(UnwrapFunctionCallsRequest {
            input: &wrapped.rewritten,
            dialect: Dialect::CommonLisp,
            function: SymbolName::new(function).unwrap(),
            wrapper: SymbolName::new(wrapper).unwrap(),
            scope: UnwrapFunctionCallsScope::AllCalls,
        }).unwrap();

        SyntaxTree::parse(&unwrapped.rewritten).unwrap();
        prop_assert_eq!(unwrapped.rewritten, input);
    }
}
