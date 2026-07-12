use super::super::*;

#[test]
fn plans_function_rename_without_value_references() {
    assert_function_rename! {
        input: "(defun foo (x) (list foo x))\n(defun caller () (foo 1))",
        dialect: Dialect::CommonLisp,
        from: "foo",
        to: "baz",
        definitions: 1,
        calls: 1,
        changed: true,
        rewritten_contains: ["(defun baz (x)", "(baz 1)", "(list foo x)"]
    };
}

#[test]
fn renames_function_calls_inside_bare_lambda_bodies_without_touching_shadowing_parameter() {
    assert_function_rename! {
        input: "(defun helper (v) (+ v 1))\n(defun main () (let ((fn (lambda (helper) (helper 1)))) (funcall fn (helper 2))))",
        dialect: Dialect::CommonLisp,
        from: "helper",
        to: "renamed",
        definitions: 1,
        calls: 2,
        changed: true,
        rewritten_contains: [
            "(defun renamed (v) (+ v 1))",
            "(lambda (helper) (renamed 1))",
            "(funcall fn (renamed 2))"
        ]
    };
}

#[test]
fn skips_labels_local_function_calls_when_renaming_function() {
    assert_function_rename! {
        input: "(defun helper (x) x)\n(defun main () (labels ((helper (x) (helper x))) (helper 1)))\n(defun caller () (helper 2))",
        dialect: Dialect::CommonLisp,
        from: "helper",
        to: "renamed",
        definitions: 1,
        calls: 1,
        changed: true,
        rewritten_contains: [
            "(defun renamed (x)",
            "(labels ((helper (x) (helper x))) (helper 1))",
            "(defun caller () (renamed 2))"
        ]
    };
}

#[test]
fn renames_outer_function_calls_inside_flet_binding_bodies_only() {
    assert_function_rename! {
        input: "(defun helper (x) x)\n(defun main () (flet ((helper (x) (helper x))) (helper 1)))",
        dialect: Dialect::CommonLisp,
        from: "helper",
        to: "renamed",
        definitions: 1,
        calls: 1,
        changed: true,
        rewritten_contains: ["(flet ((helper (x) (renamed x))) (helper 1))"]
    };
}

#[test]
fn renames_ordinary_calls_inside_flet_with_setf_binding_of_same_name() {
    assert_function_rename! {
        input: "(defun helper (x) x)\n(defun main () (flet (((setf helper) (value object) (list value object))) (helper 1)))",
        dialect: Dialect::CommonLisp,
        from: "helper",
        to: "renamed",
        definitions: 1,
        calls: 1,
        changed: true,
        rewritten_contains: [
            "(defun renamed (x) x)",
            "(flet (((setf helper) (value object) (list value object))) (renamed 1))"
        ]
    };
}

#[test]
fn renames_ordinary_calls_inside_labels_with_setf_binding_of_same_name() {
    assert_function_rename! {
        input: "(defun helper (x) x)\n(defun main () (labels (((setf helper) (value object) (list value object))) (helper 1)))",
        dialect: Dialect::CommonLisp,
        from: "helper",
        to: "renamed",
        definitions: 1,
        calls: 1,
        changed: true,
        rewritten_contains: [
            "(defun renamed (x) x)",
            "(labels (((setf helper) (value object) (list value object))) (renamed 1))"
        ]
    };
}

#[test]
fn renames_emacs_lisp_function_calls_and_designators_without_value_references() {
    assert_function_rename! {
        input: "(defun helper (x) x)\n(defun caller () (helper 1) #'helper (function helper) helper)",
        dialect: Dialect::EmacsLisp,
        from: "helper",
        to: "renamed",
        definitions: 1,
        calls: 3,
        changed: true,
        rewritten_contains: [
            "(defun renamed (x) x)",
            "(defun caller () (renamed 1) #'renamed (function renamed) helper)"
        ]
    };
}
