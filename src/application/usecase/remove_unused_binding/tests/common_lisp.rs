use super::*;

#[test]
fn plans_common_lisp_single_unused_binding() {
    let input = "(let ((unused 1) (used 2)) used)";
    let plan = plan_remove_unused_binding(RemoveUnusedBindingRequest {
        input,
        dialect: Dialect::CommonLisp,
        path: Some("0".parse().expect("path")),
        target: target(input),
        name: Some(&SymbolName::new("unused").expect("symbol")),
        all_bindings: false,
        allow_drop_value: false,
    })
    .expect("plan");

    assert_eq!(plan.binding_name.as_deref(), Some("unused"));
    assert_eq!(plan.binding_value.as_deref(), Some("1"));
    assert_eq!(plan.reference_count, Some(0));
    assert_eq!(plan.replacement, "(let ( (used 2)) used)");
    assert_eq!(plan.rewritten, "(let ( (used 2)) used)");
    assert!(plan.dropped_value_requires_review);
    assert!(plan.changed);
}

#[test]
fn rejects_referenced_binding() {
    let input = "(let ((x 1)) x)";
    let error = plan_remove_unused_binding(RemoveUnusedBindingRequest {
        input,
        dialect: Dialect::CommonLisp,
        path: None,
        target: target(input),
        name: Some(&SymbolName::new("x").expect("symbol")),
        all_bindings: false,
        allow_drop_value: true,
    })
    .expect_err("referenced binding");

    assert!(error.to_string().contains("zero in-scope references"));
}

#[test]
fn plans_unused_binding_ignoring_shadowed_lambda_parameter() {
    let input = "(let ((x 1) (used 2)) (list used (lambda (x) x)))";
    let plan = plan_remove_unused_binding(RemoveUnusedBindingRequest {
        input,
        dialect: Dialect::CommonLisp,
        path: None,
        target: target(input),
        name: Some(&SymbolName::new("x").expect("symbol")),
        all_bindings: false,
        allow_drop_value: true,
    })
    .expect("plan");

    assert_eq!(plan.binding_name.as_deref(), Some("x"));
    assert_eq!(plan.reference_count, Some(0));
    assert_eq!(
        plan.replacement,
        "(let ( (used 2)) (list used (lambda (x) x)))"
    );
}

#[test]
fn rejects_reference_before_shadowed_lambda_parameter() {
    let input = "(let ((x 1)) (list x (lambda (x) x)))";
    let error = plan_remove_unused_binding(RemoveUnusedBindingRequest {
        input,
        dialect: Dialect::CommonLisp,
        path: None,
        target: target(input),
        name: Some(&SymbolName::new("x").expect("symbol")),
        all_bindings: false,
        allow_drop_value: true,
    })
    .expect_err("outer reference");

    assert!(error.to_string().contains("zero in-scope references"));
}

#[test]
fn plans_unused_binding_ignoring_shadowed_inner_let() {
    let input = "(let ((x 1) (used 2)) (let ((x 3)) x) used)";
    let plan = plan_remove_unused_binding(RemoveUnusedBindingRequest {
        input,
        dialect: Dialect::CommonLisp,
        path: None,
        target: target(input),
        name: Some(&SymbolName::new("x").expect("symbol")),
        all_bindings: false,
        allow_drop_value: true,
    })
    .expect("plan");

    assert_eq!(plan.binding_name.as_deref(), Some("x"));
    assert_eq!(plan.reference_count, Some(0));
    assert_eq!(plan.replacement, "(let ( (used 2)) (let ((x 3)) x) used)");
}

#[test]
fn plans_unused_binding_ignoring_shadowed_dolist_variable() {
    let input = "(let ((value 1) (used 2)) (list used (dolist (value items value) value)))";
    let plan = plan_remove_unused_binding(RemoveUnusedBindingRequest {
        input,
        dialect: Dialect::CommonLisp,
        path: None,
        target: target(input),
        name: Some(&SymbolName::new("value").expect("symbol")),
        all_bindings: false,
        allow_drop_value: true,
    })
    .expect("plan");

    assert_eq!(plan.binding_name.as_deref(), Some("value"));
    assert_eq!(plan.reference_count, Some(0));
    assert_eq!(
        plan.replacement,
        "(let ( (used 2)) (list used (dolist (value items value) value)))"
    );
}

#[test]
fn plans_unused_binding_ignoring_shadowed_with_slots_variable() {
    let input = "(let ((value 1) (used 2)) (list used (with-slots (value) object value)))";
    let plan = plan_remove_unused_binding(RemoveUnusedBindingRequest {
        input,
        dialect: Dialect::CommonLisp,
        path: None,
        target: target(input),
        name: Some(&SymbolName::new("value").expect("symbol")),
        all_bindings: false,
        allow_drop_value: true,
    })
    .expect("plan");

    assert_eq!(plan.binding_name.as_deref(), Some("value"));
    assert_eq!(plan.reference_count, Some(0));
    assert_eq!(
        plan.replacement,
        "(let ( (used 2)) (list used (with-slots (value) object value)))"
    );
}

#[test]
fn plans_all_unused_bindings_by_replacing_form_with_body() {
    let input = "(let ((unused 1)) body)";
    let plan = plan_remove_unused_binding(RemoveUnusedBindingRequest {
        input,
        dialect: Dialect::CommonLisp,
        path: None,
        target: target(input),
        name: None,
        all_bindings: true,
        allow_drop_value: true,
    })
    .expect("plan");

    assert_eq!(plan.bindings.len(), 1);
    assert_eq!(plan.replacement, "body");
    assert_eq!(plan.rewritten, "body");
}

#[test]
fn plans_unused_symbol_macrolet_without_counting_expansion_reference() {
    let input = "(symbol-macrolet ((value (compute value)) (used other)) (list used))";
    let plan = plan_remove_unused_binding(RemoveUnusedBindingRequest {
        input,
        dialect: Dialect::CommonLisp,
        path: None,
        target: target(input),
        name: Some(&SymbolName::new("value").expect("symbol")),
        all_bindings: false,
        allow_drop_value: true,
    })
    .expect("plan");

    assert_eq!(plan.form, "symbol-macrolet");
    assert_eq!(plan.binding_name.as_deref(), Some("value"));
    assert_eq!(plan.binding_value.as_deref(), Some("(compute value)"));
    assert_eq!(plan.reference_count, Some(0));
    assert_eq!(
        plan.replacement,
        "(symbol-macrolet ( (used other)) (list used))"
    );
}

#[test]
fn rejects_referenced_symbol_macrolet_binding() {
    let input = "(symbol-macrolet ((value (compute))) (list value))";
    let error = plan_remove_unused_binding(RemoveUnusedBindingRequest {
        input,
        dialect: Dialect::CommonLisp,
        path: None,
        target: target(input),
        name: Some(&SymbolName::new("value").expect("symbol")),
        all_bindings: false,
        allow_drop_value: true,
    })
    .expect_err("referenced symbol macro");

    assert!(error.to_string().contains("zero in-scope references"));
}

#[test]
fn plans_unused_macrolet_without_counting_expander_body_reference() {
    let input = "(macrolet ((value (x) (compute value x)) (used (y) (list y))) (list used))";
    let plan = plan_remove_unused_binding(RemoveUnusedBindingRequest {
        input,
        dialect: Dialect::CommonLisp,
        path: None,
        target: target(input),
        name: Some(&SymbolName::new("value").expect("symbol")),
        all_bindings: false,
        allow_drop_value: true,
    })
    .expect("plan");

    assert_eq!(plan.form, "macrolet");
    assert_eq!(plan.binding_name.as_deref(), Some("value"));
    assert_eq!(plan.binding_value.as_deref(), Some("(x) (compute value x)"));
    assert_eq!(plan.reference_count, Some(0));
    assert_eq!(
        plan.replacement,
        "(macrolet ( (used (y) (list y))) (list used))"
    );
}

#[test]
fn rejects_referenced_macrolet_binding() {
    let input = "(macrolet ((value (x) (compute x))) (list (value 1)))";
    let error = plan_remove_unused_binding(RemoveUnusedBindingRequest {
        input,
        dialect: Dialect::CommonLisp,
        path: None,
        target: target(input),
        name: Some(&SymbolName::new("value").expect("symbol")),
        all_bindings: false,
        allow_drop_value: true,
    })
    .expect_err("referenced macro");

    assert!(error.to_string().contains("zero in-scope references"));
}

#[test]
fn plans_unused_compiler_macrolet_without_counting_expander_body_reference() {
    let input =
        "(compiler-macrolet ((value (x) (compute value x)) (used (y) (list y))) (list used))";
    let plan = plan_remove_unused_binding(RemoveUnusedBindingRequest {
        input,
        dialect: Dialect::CommonLisp,
        path: None,
        target: target(input),
        name: Some(&SymbolName::new("value").expect("symbol")),
        all_bindings: false,
        allow_drop_value: true,
    })
    .expect("plan");

    assert_eq!(plan.form, "compiler-macrolet");
    assert_eq!(plan.binding_name.as_deref(), Some("value"));
    assert_eq!(plan.binding_value.as_deref(), Some("(x) (compute value x)"));
    assert_eq!(plan.reference_count, Some(0));
    assert_eq!(
        plan.replacement,
        "(compiler-macrolet ( (used (y) (list y))) (list used))"
    );
}

#[test]
fn rejects_referenced_compiler_macrolet_binding() {
    let input = "(compiler-macrolet ((value (x) (compute x))) (list (value 1)))";
    let error = plan_remove_unused_binding(RemoveUnusedBindingRequest {
        input,
        dialect: Dialect::CommonLisp,
        path: None,
        target: target(input),
        name: Some(&SymbolName::new("value").expect("symbol")),
        all_bindings: false,
        allow_drop_value: true,
    })
    .expect_err("referenced compiler macro");

    assert!(error.to_string().contains("zero in-scope references"));
}

#[test]
fn plans_unused_flet_binding_ignoring_definition_body_reference() {
    let input = "(flet ((unused () (unused)) (used () (used))) (used))";
    let plan = plan_remove_unused_binding(RemoveUnusedBindingRequest {
        input,
        dialect: Dialect::CommonLisp,
        path: None,
        target: target(input),
        name: Some(&SymbolName::new("unused").expect("symbol")),
        all_bindings: false,
        allow_drop_value: true,
    })
    .expect("plan");

    assert_eq!(plan.form, "flet");
    assert_eq!(plan.binding_name.as_deref(), Some("unused"));
    assert_eq!(plan.reference_count, Some(0));
    assert_eq!(plan.replacement, "(flet ( (used () (used))) (used))");
}

#[test]
fn rejects_referenced_flet_binding() {
    let input = "(flet ((unused () 1)) (unused))";
    let error = plan_remove_unused_binding(RemoveUnusedBindingRequest {
        input,
        dialect: Dialect::CommonLisp,
        path: None,
        target: target(input),
        name: Some(&SymbolName::new("unused").expect("symbol")),
        all_bindings: false,
        allow_drop_value: true,
    })
    .expect_err("referenced flet binding");

    assert!(error.to_string().contains("zero in-scope references"));
}

#[test]
fn rejects_recursive_labels_binding() {
    let input = "(labels ((unused () (unused)) (used () (list used))) (used))";
    let error = plan_remove_unused_binding(RemoveUnusedBindingRequest {
        input,
        dialect: Dialect::CommonLisp,
        path: None,
        target: target(input),
        name: Some(&SymbolName::new("unused").expect("symbol")),
        all_bindings: false,
        allow_drop_value: true,
    })
    .expect_err("recursive labels binding");

    assert!(error.to_string().contains("zero in-scope references"));
}
