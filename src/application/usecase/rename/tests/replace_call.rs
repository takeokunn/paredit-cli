use super::*;

#[test]
fn replaces_call_heads_without_touching_definitions_or_value_references() {
    let input = "(defun fetch-user (id) (list fetch-user id))\n(defun render () (fetch-user id))";
    let plan = plan_replace_function_calls(ReplaceFunctionCallsRequest {
        input,
        dialect: Dialect::CommonLisp,
        from: SymbolName::new("fetch-user").unwrap(),
        to: SymbolName::new("load-user").unwrap(),
        scope: ReplaceFunctionCallsScope::AllCalls,
    })
    .unwrap();

    assert_eq!(plan.calls.len(), 1);
    assert_eq!(
        plan.rewritten,
        "(defun fetch-user (id) (list fetch-user id))\n(defun render () (load-user id))"
    );
    SyntaxTree::parse(&plan.rewritten).unwrap();
}

#[test]
fn replaces_only_explicit_call_paths() {
    let input = "(defun render () (fetch-user id) (fetch-user other))";
    let plan = plan_replace_function_calls(ReplaceFunctionCallsRequest {
        input,
        dialect: Dialect::CommonLisp,
        from: SymbolName::new("fetch-user").unwrap(),
        to: SymbolName::new("load-user").unwrap(),
        scope: ReplaceFunctionCallsScope::ExplicitPaths(vec!["0.4".parse::<Path>().unwrap()]),
    })
    .unwrap();

    assert_eq!(plan.calls.len(), 1);
    assert_eq!(plan.calls[0].path, "0.4");
    assert_eq!(
        plan.rewritten,
        "(defun render () (fetch-user id) (load-user other))"
    );
    SyntaxTree::parse(&plan.rewritten).unwrap();
}

#[test]
fn explicit_path_must_select_matching_call() {
    let input = "(defun render () (fetch-user id))";
    let error = plan_replace_function_calls(ReplaceFunctionCallsRequest {
        input,
        dialect: Dialect::CommonLisp,
        from: SymbolName::new("fetch-user").unwrap(),
        to: SymbolName::new("load-user").unwrap(),
        scope: ReplaceFunctionCallsScope::ExplicitPaths(vec!["0".parse::<Path>().unwrap()]),
    })
    .unwrap_err();

    assert!(error.to_string().contains("call-path 0 is not a call"));
}

#[test]
fn all_calls_skip_labels_local_function_calls() {
    let input = "(defun main () (labels ((fetch-user (id) (fetch-user id))) (fetch-user user)))\n(fetch-user root)";
    let plan = plan_replace_function_calls(ReplaceFunctionCallsRequest {
        input,
        dialect: Dialect::CommonLisp,
        from: SymbolName::new("fetch-user").unwrap(),
        to: SymbolName::new("load-user").unwrap(),
        scope: ReplaceFunctionCallsScope::AllCalls,
    })
    .unwrap();

    assert_eq!(plan.calls.len(), 1);
    assert!(plan
        .rewritten
        .contains("(labels ((fetch-user (id) (fetch-user id))) (fetch-user user))"));
    assert!(plan.rewritten.contains("(load-user root)"));
    SyntaxTree::parse(&plan.rewritten).unwrap();
}

#[test]
fn all_calls_replaces_outer_calls_inside_flet_binding_bodies_only() {
    let input = "(defun main () (flet ((fetch-user (id) (fetch-user id))) (fetch-user user)))\n(fetch-user root)";
    let plan = plan_replace_function_calls(ReplaceFunctionCallsRequest {
        input,
        dialect: Dialect::CommonLisp,
        from: SymbolName::new("fetch-user").unwrap(),
        to: SymbolName::new("load-user").unwrap(),
        scope: ReplaceFunctionCallsScope::AllCalls,
    })
    .unwrap();

    assert_eq!(plan.calls.len(), 2);
    assert!(plan
        .rewritten
        .contains("(flet ((fetch-user (id) (load-user id))) (fetch-user user))"));
    assert!(plan.rewritten.contains("(load-user root)"));
    SyntaxTree::parse(&plan.rewritten).unwrap();
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn pbt_replace_function_calls_output_remains_parseable(
        from in symbol_strategy(),
        to in symbol_strategy(),
        arg in symbol_strategy(),
    ) {
        prop_assume!(from != to);
        prop_assume!(from != arg);
        prop_assume!(to != arg);
        let input = format!("(defun keep () {from})\n({from} {arg})");
        let plan = plan_replace_function_calls(ReplaceFunctionCallsRequest {
            input: &input,
            dialect: Dialect::CommonLisp,
            from: SymbolName::new(from.clone()).unwrap(),
            to: SymbolName::new(to.clone()).unwrap(),
            scope: ReplaceFunctionCallsScope::AllCalls,
        }).unwrap();

        SyntaxTree::parse(&plan.rewritten).unwrap();
        prop_assert!(plan.changed);
        prop_assert_eq!(plan.calls.len(), 1);
        let replaced_call = format!("({} {})", to, arg);
        let preserved_definition = format!("(defun keep () {})", from);
        prop_assert!(plan.rewritten.contains(&replaced_call));
        prop_assert!(plan.rewritten.contains(&preserved_definition));
    }
}
