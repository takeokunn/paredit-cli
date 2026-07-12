#[test]
fn renames_value_binding_from_reference() {
    let input = "(let ((value 1)) (+ value value))";
    let plan = plan_rename_at(RenameAtRequest {
        input,
        dialect: Dialect::CommonLisp,
        at: ByteOffset::new(input.rfind("value").unwrap()),
        to: SymbolName::new("count").unwrap(),
    })
    .expect("plan");
    assert_eq!(plan.namespace, RenameAtNamespace::Value);
    assert_eq!(plan.rewritten, "(let ((count 1)) (+ count count))");
}

// NOT ported: `renames_value_binding_through_local_special_declaration`
// (renaming a `let` binding also renames its own `(declare (special ...))`
// mention) needs `binding_rename_parts`'s reference collection to treat a
// `declare (special name)` mention as a renamable reference, which main's
// shared rename/binding walker does not do yet. See the note in
// `rename/at/tests.rs` above `nested_lambda_initializers` — same category of
// out-of-scope shared-engine enhancement.

#[test]
fn keeps_common_lisp_value_and_function_namespaces_separate() {
    let input = "(let ((value 1)) (list value (value)))";
    let plan = plan_rename_at(request(input, "value 1", "item")).expect("plan");
    assert_eq!(plan.rewritten, "(let ((item 1)) (list item (value)))");
}

#[test]
fn renames_macro_lambda_binding_as_a_value() {
    let input = "(defmacro emit (&whole whole &environment environment form) `(list ,whole ,environment ,form))";
    let plan =
        plan_rename_at(request(input, "environment form", "expansion-environment")).expect("plan");
    assert_eq!(plan.namespace, RenameAtNamespace::Value);
    assert_eq!(
        plan.rewritten,
        "(defmacro emit (&whole whole &environment expansion-environment form) `(list ,whole ,expansion-environment ,form))"
    );
}
use super::*;
