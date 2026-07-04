use super::*;

#[test]
fn plans_binding_rename_without_shadowed_inner_binding() {
    let input = "(let ((value 1)) (+ value (let ((value 2)) value) value))";
    let plan = plan_rename_binding(RenameBindingRequest {
        input,
        dialect: Dialect::CommonLisp,
        target: RenameTarget::Path(Path::from_indexes(vec![0])),
        from: SymbolName::new("value").unwrap(),
        to: SymbolName::new("product").unwrap(),
    })
    .unwrap();

    assert_eq!(plan.form, "let");
    assert_eq!(plan.references.len(), 2);
    assert_eq!(plan.shadowed_scope_count, 1);
    assert_eq!(
        plan.rewritten,
        "(let ((product 1)) (+ product (let ((value 2)) value) product))"
    );
    SyntaxTree::parse(&plan.rewritten).unwrap();
}

#[test]
fn plans_clojure_vector_binding_rename_through_later_binding_values() {
    let input = "(let [value 1 next (+ value 1)] [value next])";
    let plan = plan_rename_binding(RenameBindingRequest {
        input,
        dialect: Dialect::Clojure,
        target: RenameTarget::Path(Path::from_indexes(vec![0])),
        from: SymbolName::new("value").unwrap(),
        to: SymbolName::new("seed").unwrap(),
    })
    .unwrap();

    assert_eq!(plan.form, "let");
    assert_eq!(plan.references.len(), 2);
    assert_eq!(plan.rewritten, "(let [seed 1 next (+ seed 1)] [seed next])");
    SyntaxTree::parse(&plan.rewritten).unwrap();
}

#[test]
fn plans_symbol_macrolet_binding_rename_without_touching_expansion_reference() {
    let input = "(symbol-macrolet ((value (compute value))) (list value value))";
    let plan = plan_rename_binding(RenameBindingRequest {
        input,
        dialect: Dialect::CommonLisp,
        target: RenameTarget::Path(Path::from_indexes(vec![0])),
        from: SymbolName::new("value").unwrap(),
        to: SymbolName::new("slot").unwrap(),
    })
    .unwrap();

    assert_eq!(plan.form, "symbol-macrolet");
    assert_eq!(plan.references.len(), 2);
    assert_eq!(
        plan.rewritten,
        "(symbol-macrolet ((slot (compute value))) (list slot slot))"
    );
    SyntaxTree::parse(&plan.rewritten).unwrap();
}

#[test]
fn plans_lambda_parameter_rename_without_shadowed_inner_binding() {
    let input = "(lambda (value) (list value (lambda (value) value) value))";
    let plan = plan_rename_binding(RenameBindingRequest {
        input,
        dialect: Dialect::CommonLisp,
        target: RenameTarget::Path(Path::from_indexes(vec![0])),
        from: SymbolName::new("value").unwrap(),
        to: SymbolName::new("product").unwrap(),
    })
    .unwrap();

    assert_eq!(plan.form, "lambda");
    assert_eq!(plan.references.len(), 2);
    assert_eq!(plan.shadowed_scope_count, 1);
    assert_eq!(
        plan.rewritten,
        "(lambda (product) (list product (lambda (value) value) product))"
    );
    SyntaxTree::parse(&plan.rewritten).unwrap();
}

#[test]
fn plans_defun_parameter_rename() {
    let input = "(defun render (value other) (list value other))";
    let plan = plan_rename_binding(RenameBindingRequest {
        input,
        dialect: Dialect::CommonLisp,
        target: RenameTarget::Path(Path::from_indexes(vec![0])),
        from: SymbolName::new("value").unwrap(),
        to: SymbolName::new("product").unwrap(),
    })
    .unwrap();

    assert_eq!(plan.form, "defun");
    assert_eq!(plan.references.len(), 1);
    assert_eq!(
        plan.rewritten,
        "(defun render (product other) (list product other))"
    );
    SyntaxTree::parse(&plan.rewritten).unwrap();
}

#[test]
fn plans_defmacro_optional_parameter_rename_without_touching_default_form() {
    let input =
        "(defmacro wrap (&optional (value (default value) supplied)) (list value supplied))";
    let plan = plan_rename_binding(RenameBindingRequest {
        input,
        dialect: Dialect::CommonLisp,
        target: RenameTarget::Path(Path::from_indexes(vec![0])),
        from: SymbolName::new("value").unwrap(),
        to: SymbolName::new("form").unwrap(),
    })
    .unwrap();

    assert_eq!(plan.form, "defmacro");
    assert_eq!(plan.references.len(), 1);
    assert_eq!(
        plan.rewritten,
        "(defmacro wrap (&optional (form (default value) supplied)) (list form supplied))"
    );
    SyntaxTree::parse(&plan.rewritten).unwrap();
}

#[test]
fn plans_define_setf_expander_environment_parameter_rename() {
    let input = "(define-setf-expander slot (&whole whole &environment env target) (list whole env target))";
    let plan = plan_rename_binding(RenameBindingRequest {
        input,
        dialect: Dialect::CommonLisp,
        target: RenameTarget::Path(Path::from_indexes(vec![0])),
        from: SymbolName::new("env").unwrap(),
        to: SymbolName::new("macro-env").unwrap(),
    })
    .unwrap();

    assert_eq!(plan.form, "define-setf-expander");
    assert_eq!(plan.references.len(), 1);
    assert_eq!(
        plan.rewritten,
        "(define-setf-expander slot (&whole whole &environment macro-env target) (list whole macro-env target))"
    );
    SyntaxTree::parse(&plan.rewritten).unwrap();
}

#[test]
fn plans_define_compiler_macro_environment_parameter_rename() {
    let input = "(define-compiler-macro render (&whole whole &environment env target) (list whole env target))";
    let plan = plan_rename_binding(RenameBindingRequest {
        input,
        dialect: Dialect::CommonLisp,
        target: RenameTarget::Path(Path::from_indexes(vec![0])),
        from: SymbolName::new("env").unwrap(),
        to: SymbolName::new("macro-env").unwrap(),
    })
    .unwrap();

    assert_eq!(plan.form, "define-compiler-macro");
    assert_eq!(plan.references.len(), 1);
    assert_eq!(
        plan.rewritten,
        "(define-compiler-macro render (&whole whole &environment macro-env target) (list whole macro-env target))"
    );
    SyntaxTree::parse(&plan.rewritten).unwrap();
}

#[test]
fn plans_destructuring_bind_rename_without_touching_value_form() {
    let input = "(destructuring-bind (value other) (parse value) (list value other))";
    let plan = plan_rename_binding(RenameBindingRequest {
        input,
        dialect: Dialect::CommonLisp,
        target: RenameTarget::Path(Path::from_indexes(vec![0])),
        from: SymbolName::new("value").unwrap(),
        to: SymbolName::new("slot").unwrap(),
    })
    .unwrap();

    assert_eq!(plan.form, "destructuring-bind");
    assert_eq!(plan.references.len(), 1);
    assert_eq!(
        plan.rewritten,
        "(destructuring-bind (slot other) (parse value) (list slot other))"
    );
    SyntaxTree::parse(&plan.rewritten).unwrap();
}

#[test]
fn plans_multiple_value_bind_rename_without_shadowed_inner_binding() {
    let input = "(multiple-value-bind (value other) (compute) (list value (destructuring-bind (value) row value) other value))";
    let plan = plan_rename_binding(RenameBindingRequest {
        input,
        dialect: Dialect::CommonLisp,
        target: RenameTarget::Path(Path::from_indexes(vec![0])),
        from: SymbolName::new("value").unwrap(),
        to: SymbolName::new("slot").unwrap(),
    })
    .unwrap();

    assert_eq!(plan.form, "multiple-value-bind");
    assert_eq!(plan.references.len(), 2);
    assert_eq!(plan.shadowed_scope_count, 1);
    assert_eq!(
        plan.rewritten,
        "(multiple-value-bind (slot other) (compute) (list slot (destructuring-bind (value) row value) other slot))"
    );
    SyntaxTree::parse(&plan.rewritten).unwrap();
}
