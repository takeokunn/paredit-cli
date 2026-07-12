use super::super::*;

#[test]
fn renames_outer_function_inside_macrolet_expander_but_not_shadowed_body() {
    assert_function_rename! {
        input: "(defun helper (x) x)\n(defun caller () (macrolet ((helper () #'helper (function helper) (helper 1))) (helper) #'helper (function helper) (helper 2)))",
        dialect: Dialect::CommonLisp,
        from: "helper",
        to: "renamed",
        definitions: 1,
        calls: 3,
        changed: true,
        rewritten_contains: [
            "(defun renamed (x) x)",
            "(macrolet ((helper () #'renamed (function renamed) (renamed 1))) (helper) #'helper (function helper) (helper 2)))"
        ]
    };
}

#[test]
fn renames_outer_function_inside_compiler_macrolet_expander_but_not_shadowed_body() {
    assert_function_rename! {
        input: "(defun helper (x) x)\n(defun caller () (compiler-macrolet ((helper () #'helper (function helper) (helper 1))) (helper) #'helper (function helper) (helper 2)))",
        dialect: Dialect::CommonLisp,
        from: "helper",
        to: "renamed",
        definitions: 1,
        calls: 3,
        changed: true,
        rewritten_contains: [
            "(defun renamed (x) x)",
            "(compiler-macrolet ((helper () #'renamed (function renamed) (renamed 1))) (helper) #'helper (function helper) (helper 2)))"
        ]
    };
}

#[test]
fn renames_outer_callable_designators_inside_macrolet_expanders_only() {
    assert_function_rename! {
        input: "(defmacro helper (x) x)\n(defun caller () (macrolet ((helper () (list #'helper (function helper) (macro-function 'helper) (compiler-macro-function 'helper) (symbol-function 'helper) (fdefinition 'helper)))) (helper) #'helper (function helper) (macro-function 'helper) (compiler-macro-function 'helper) (symbol-function 'helper) (fdefinition 'helper)))",
        dialect: Dialect::CommonLisp,
        from: "helper",
        to: "renamed",
        definitions: 1,
        calls: 6,
        changed: true,
        rewritten_contains: [
            "(defmacro renamed (x) x)",
            "(macrolet ((helper () (list #'renamed (function renamed) (macro-function 'renamed) (compiler-macro-function 'renamed) (symbol-function 'renamed) (fdefinition 'renamed)))) (helper) #'helper (function helper) (macro-function 'helper) (compiler-macro-function 'helper) (symbol-function 'helper) (fdefinition 'helper))"
        ]
    };
}

#[test]
fn renames_outer_callable_designators_inside_compiler_macrolet_expanders_only() {
    assert_function_rename! {
        input: "(defmacro helper (x) x)\n(defun caller () (compiler-macrolet ((helper () (list #'helper (function helper) (macro-function 'helper) (compiler-macro-function 'helper) (symbol-function 'helper) (fdefinition 'helper)))) (helper) #'helper (function helper) (macro-function 'helper) (compiler-macro-function 'helper) (symbol-function 'helper) (fdefinition 'helper)))",
        dialect: Dialect::CommonLisp,
        from: "helper",
        to: "renamed",
        definitions: 1,
        calls: 6,
        changed: true,
        rewritten_contains: [
            "(defmacro renamed (x) x)",
            "(compiler-macrolet ((helper () (list #'renamed (function renamed) (macro-function 'renamed) (compiler-macro-function 'renamed) (symbol-function 'renamed) (fdefinition 'renamed)))) (helper) #'helper (function helper) (macro-function 'helper) (compiler-macro-function 'helper) (symbol-function 'helper) (fdefinition 'helper))"
        ]
    };
}

#[test]
fn renames_package_qualified_function_references_inside_macrolet_body_and_expander() {
    assert_function_rename! {
        input: "(defun helper (x) x)\n(defun caller () (macrolet ((helper () #'cl-user:helper (function cl-user:helper) (cl-user:helper 1))) (helper) #'cl-user:helper (function cl-user:helper) (cl-user:helper 2)))",
        dialect: Dialect::CommonLisp,
        from: "helper",
        to: "renamed",
        definitions: 1,
        calls: 6,
        changed: true,
        rewritten_contains: [
            "(defun renamed (x) x)",
            "(macrolet ((helper () #'renamed (function renamed) (renamed 1))) (helper) #'renamed (function renamed) (renamed 2)))"
        ]
    };
}

#[test]
fn renames_package_qualified_function_references_inside_compiler_macrolet_body_and_expander() {
    assert_function_rename! {
        input: "(defun helper (x) x)\n(defun caller () (compiler-macrolet ((helper () #'cl-user:helper (function cl-user:helper) (cl-user:helper 1))) (helper) #'cl-user:helper (function cl-user:helper) (cl-user:helper 2)))",
        dialect: Dialect::CommonLisp,
        from: "helper",
        to: "renamed",
        definitions: 1,
        calls: 6,
        changed: true,
        rewritten_contains: [
            "(defun renamed (x) x)",
            "(compiler-macrolet ((helper () #'renamed (function renamed) (renamed 1))) (helper) #'renamed (function renamed) (renamed 2)))"
        ]
    };
}
