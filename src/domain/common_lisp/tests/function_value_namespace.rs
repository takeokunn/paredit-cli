use crate::domain::common_lisp::function_value_namespace_diagnostics;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::SyntaxTree;

#[test]
fn finds_local_functions_used_as_values() {
    let tree = SyntaxTree::parse(
        "(flet ((finish-attempt (value) value)) (funcall finish-attempt 1))\n         (labels ((retry (&rest values) values)) (apply retry '(1)))",
    )
    .expect("parse local callable forms");

    let diagnostics = function_value_namespace_diagnostics(&tree, Dialect::CommonLisp)
        .expect("collect diagnostics");

    assert_eq!(diagnostics.len(), 2);
    assert_eq!(diagnostics[0].name, "finish-attempt");
    assert_eq!(diagnostics[1].name, "retry");
    assert!(diagnostics
        .iter()
        .all(|diagnostic| diagnostic.code() == "function-used-as-value"));
}

#[test]
fn ignores_value_bindings_and_inert_reader_data() {
    let tree = SyntaxTree::parse(
        r#"(flet ((finish-attempt (value) value))
              (let ((finish-attempt #'identity)) (funcall finish-attempt 1))
              ((lambda (finish-attempt) (funcall finish-attempt 1)) #'identity)
              (multiple-value-bind (finish-attempt) (values #'identity)
                (funcall finish-attempt 1))
              '(funcall finish-attempt 1))"#,
    )
    .expect("parse flet");

    let diagnostics = function_value_namespace_diagnostics(&tree, Dialect::CommonLisp)
        .expect("collect diagnostics");

    assert!(diagnostics.is_empty());
}

#[test]
fn checks_unknown_dialect_for_standard_input() {
    let tree = SyntaxTree::parse("(flet ((finish-attempt () nil)) (funcall finish-attempt))")
        .expect("parse flet");

    let diagnostics =
        function_value_namespace_diagnostics(&tree, Dialect::Unknown).expect("collect diagnostics");

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code(), "function-used-as-value");
}
