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
