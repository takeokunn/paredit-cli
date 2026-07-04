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
fn plans_binding_rename_without_touching_dolist_iteration_scope() {
    let input = "(let ((value 1) (items 2)) (list value (dolist (value items value) value) value))";
    let plan = plan_rename_binding(RenameBindingRequest {
        input,
        dialect: Dialect::CommonLisp,
        target: RenameTarget::Path(Path::from_indexes(vec![0])),
        from: SymbolName::new("value").unwrap(),
        to: SymbolName::new("seed").unwrap(),
    })
    .unwrap();

    assert_eq!(plan.form, "let");
    assert_eq!(plan.references.len(), 2);
    assert_eq!(plan.shadowed_scope_count, 1);
    assert_eq!(
        plan.rewritten,
        "(let ((seed 1) (items 2)) (list seed (dolist (value items value) value) seed))"
    );
    SyntaxTree::parse(&plan.rewritten).unwrap();
}

#[test]
fn plans_binding_rename_without_touching_with_slots_scope() {
    let input =
        "(let ((value 1)) (with-slots (value (alias value)) value (list value alias)) value)";
    let plan = plan_rename_binding(RenameBindingRequest {
        input,
        dialect: Dialect::CommonLisp,
        target: RenameTarget::Path(Path::from_indexes(vec![0])),
        from: SymbolName::new("value").unwrap(),
        to: SymbolName::new("outer").unwrap(),
    })
    .unwrap();

    assert_eq!(plan.form, "let");
    assert_eq!(plan.references.len(), 2);
    assert_eq!(plan.shadowed_scope_count, 1);
    assert_eq!(
        plan.rewritten,
        "(let ((outer 1)) (with-slots (value (alias value)) outer (list value alias)) outer)"
    );
    SyntaxTree::parse(&plan.rewritten).unwrap();
}

#[test]
fn plans_binding_rename_without_touching_with_accessors_scope() {
    let input = "(let ((value 1)) (with-accessors ((value get-value) (alias value)) value (list value alias)) value)";
    let plan = plan_rename_binding(RenameBindingRequest {
        input,
        dialect: Dialect::CommonLisp,
        target: RenameTarget::Path(Path::from_indexes(vec![0])),
        from: SymbolName::new("value").unwrap(),
        to: SymbolName::new("outer").unwrap(),
    })
    .unwrap();

    assert_eq!(plan.form, "let");
    assert_eq!(plan.references.len(), 2);
    assert_eq!(plan.shadowed_scope_count, 1);
    assert_eq!(
        plan.rewritten,
        "(let ((outer 1)) (with-accessors ((value get-value) (alias value)) outer (list value alias)) outer)"
    );
    SyntaxTree::parse(&plan.rewritten).unwrap();
}

#[test]
fn plans_dolist_iteration_binding_rename_without_touching_source() {
    let input = "(dolist (value items value) (collect value) items)";
    let plan = plan_rename_binding(RenameBindingRequest {
        input,
        dialect: Dialect::CommonLisp,
        target: RenameTarget::Path(Path::from_indexes(vec![0])),
        from: SymbolName::new("value").unwrap(),
        to: SymbolName::new("item").unwrap(),
    })
    .unwrap();

    assert_eq!(plan.form, "dolist");
    assert_eq!(plan.references.len(), 2);
    assert_eq!(
        plan.rewritten,
        "(dolist (item items item) (collect item) items)"
    );
    SyntaxTree::parse(&plan.rewritten).unwrap();
}

#[test]
fn plans_dotimes_iteration_binding_rename_without_touching_count() {
    let input = "(dotimes (index limit index) (push index result) limit)";
    let plan = plan_rename_binding(RenameBindingRequest {
        input,
        dialect: Dialect::CommonLisp,
        target: RenameTarget::Path(Path::from_indexes(vec![0])),
        from: SymbolName::new("index").unwrap(),
        to: SymbolName::new("i").unwrap(),
    })
    .unwrap();

    assert_eq!(plan.form, "dotimes");
    assert_eq!(plan.references.len(), 2);
    assert_eq!(
        plan.rewritten,
        "(dotimes (i limit i) (push i result) limit)"
    );
    SyntaxTree::parse(&plan.rewritten).unwrap();
}

#[test]
fn plans_binding_rename_without_touching_do_scope() {
    let input =
        "(let ((value 1)) (list value (do ((value value (1+ value))) ((done) value) value) value))";
    let plan = plan_rename_binding(RenameBindingRequest {
        input,
        dialect: Dialect::CommonLisp,
        target: RenameTarget::Path(Path::from_indexes(vec![0])),
        from: SymbolName::new("value").unwrap(),
        to: SymbolName::new("outer").unwrap(),
    })
    .unwrap();

    assert_eq!(plan.form, "let");
    assert_eq!(plan.references.len(), 3);
    assert_eq!(plan.shadowed_scope_count, 1);
    assert_eq!(
        plan.rewritten,
        "(let ((outer 1)) (list outer (do ((value outer (1+ value))) ((done) value) value) outer))"
    );
    SyntaxTree::parse(&plan.rewritten).unwrap();
}

#[test]
fn plans_binding_rename_without_touching_prog_scope() {
    let input =
        "(let ((value 1)) (list value (prog ((value value) (copy value)) (return value)) value))";
    let plan = plan_rename_binding(RenameBindingRequest {
        input,
        dialect: Dialect::CommonLisp,
        target: RenameTarget::Path(Path::from_indexes(vec![0])),
        from: SymbolName::new("value").unwrap(),
        to: SymbolName::new("outer").unwrap(),
    })
    .unwrap();

    assert_eq!(plan.form, "let");
    assert_eq!(plan.references.len(), 4);
    assert_eq!(plan.shadowed_scope_count, 1);
    assert_eq!(
        plan.rewritten,
        "(let ((outer 1)) (list outer (prog ((value outer) (copy outer)) (return value)) outer))"
    );
    SyntaxTree::parse(&plan.rewritten).unwrap();
}

#[test]
fn plans_do_binding_rename_across_steps_end_clause_and_body() {
    let input = "(do ((value seed (1+ value))) ((done value) value) (collect value seed))";
    let plan = plan_rename_binding(RenameBindingRequest {
        input,
        dialect: Dialect::CommonLisp,
        target: RenameTarget::Path(Path::from_indexes(vec![0])),
        from: SymbolName::new("value").unwrap(),
        to: SymbolName::new("item").unwrap(),
    })
    .unwrap();

    assert_eq!(plan.form, "do");
    assert_eq!(plan.references.len(), 4);
    assert_eq!(
        plan.rewritten,
        "(do ((item seed (1+ item))) ((done item) item) (collect item seed))"
    );
    SyntaxTree::parse(&plan.rewritten).unwrap();
}

#[test]
fn plans_prog_star_binding_rename_across_later_inits_and_body() {
    let input = "(prog* ((value seed) (copy value)) (return (list value copy)))";
    let plan = plan_rename_binding(RenameBindingRequest {
        input,
        dialect: Dialect::CommonLisp,
        target: RenameTarget::Path(Path::from_indexes(vec![0])),
        from: SymbolName::new("value").unwrap(),
        to: SymbolName::new("item").unwrap(),
    })
    .unwrap();

    assert_eq!(plan.form, "prog*");
    assert_eq!(plan.references.len(), 2);
    assert_eq!(
        plan.rewritten,
        "(prog* ((item seed) (copy item)) (return (list item copy)))"
    );
    SyntaxTree::parse(&plan.rewritten).unwrap();
}

#[test]
fn plans_with_slots_binding_rename_preserves_bare_slot_name() {
    let input = "(with-slots (value (alias slot-name)) object (list value alias object))";
    let plan = plan_rename_binding(RenameBindingRequest {
        input,
        dialect: Dialect::CommonLisp,
        target: RenameTarget::Path(Path::from_indexes(vec![0])),
        from: SymbolName::new("value").unwrap(),
        to: SymbolName::new("slot-value").unwrap(),
    })
    .unwrap();

    assert_eq!(plan.form, "with-slots");
    assert_eq!(plan.references.len(), 1);
    assert_eq!(
        plan.rewritten,
        "(with-slots ((slot-value value) (alias slot-name)) object (list slot-value alias object))"
    );
    SyntaxTree::parse(&plan.rewritten).unwrap();
}

#[test]
fn plans_with_accessors_binding_rename_preserves_accessor_name() {
    let input =
        "(with-accessors ((value get-value) (alias get-alias)) object (list value alias object))";
    let plan = plan_rename_binding(RenameBindingRequest {
        input,
        dialect: Dialect::CommonLisp,
        target: RenameTarget::Path(Path::from_indexes(vec![0])),
        from: SymbolName::new("value").unwrap(),
        to: SymbolName::new("slot-value").unwrap(),
    })
    .unwrap();

    assert_eq!(plan.form, "with-accessors");
    assert_eq!(plan.references.len(), 1);
    assert_eq!(
        plan.rewritten,
        "(with-accessors ((slot-value get-value) (alias get-alias)) object (list slot-value alias object))"
    );
    SyntaxTree::parse(&plan.rewritten).unwrap();
}

#[test]
fn rejects_ambiguous_with_slots_binding_rename() {
    let input = "(with-slots (value (value slot-name)) object value)";
    let error = plan_rename_binding(RenameBindingRequest {
        input,
        dialect: Dialect::CommonLisp,
        target: RenameTarget::Path(Path::from_indexes(vec![0])),
        from: SymbolName::new("value").unwrap(),
        to: SymbolName::new("slot-value").unwrap(),
    })
    .unwrap_err();

    assert!(error
        .to_string()
        .contains("binding 'value' was found in multiple selected with-slots specs"));
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

#[test]
fn plans_handler_case_clause_parameter_rename_without_touching_other_scopes() {
    let input = "(handler-case (risky condition) (error (condition) (recover condition outer)) (:no-error (value) (finish value condition)))";
    let plan = plan_rename_binding(RenameBindingRequest {
        input,
        dialect: Dialect::CommonLisp,
        target: RenameTarget::Path(Path::from_indexes(vec![0])),
        from: SymbolName::new("condition").unwrap(),
        to: SymbolName::new("err").unwrap(),
    })
    .unwrap();

    assert_eq!(plan.form, "handler-case");
    assert_eq!(plan.references.len(), 1);
    assert_eq!(
        plan.rewritten,
        "(handler-case (risky condition) (error (err) (recover err outer)) (:no-error (value) (finish value condition)))"
    );
    SyntaxTree::parse(&plan.rewritten).unwrap();
}

#[test]
fn plans_restart_case_clause_parameter_rename_without_crossing_nested_clause_shadow() {
    let input = "(restart-case (risky condition) (retry (condition) (recover condition (handler-case (again) (error (condition) condition)))) (skip () condition))";
    let plan = plan_rename_binding(RenameBindingRequest {
        input,
        dialect: Dialect::CommonLisp,
        target: RenameTarget::Path(Path::from_indexes(vec![0])),
        from: SymbolName::new("condition").unwrap(),
        to: SymbolName::new("reason").unwrap(),
    })
    .unwrap();

    assert_eq!(plan.form, "restart-case");
    assert_eq!(plan.references.len(), 1);
    assert_eq!(plan.shadowed_scope_count, 1);
    assert_eq!(
        plan.rewritten,
        "(restart-case (risky condition) (retry (reason) (recover reason (handler-case (again) (error (condition) condition)))) (skip () condition))"
    );
    SyntaxTree::parse(&plan.rewritten).unwrap();
}

#[test]
fn rejects_ambiguous_handler_case_clause_parameter_rename() {
    let input =
        "(handler-case (risky) (error (condition) condition) (warning (condition) condition))";
    let error = plan_rename_binding(RenameBindingRequest {
        input,
        dialect: Dialect::CommonLisp,
        target: RenameTarget::Path(Path::from_indexes(vec![0])),
        from: SymbolName::new("condition").unwrap(),
        to: SymbolName::new("err").unwrap(),
    })
    .unwrap_err();

    assert!(error
        .to_string()
        .contains("multiple selected handler-case clauses"));
}
