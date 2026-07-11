use super::super::*;

#[test]
fn renames_defmacro_definition_and_macro_calls() {
    assert_function_rename! {
        input: "(defmacro helper (x) `(list ,x))\n(helper 1)\n(list #'helper (macro-function 'helper) helper '(helper 2))",
        dialect: Dialect::CommonLisp,
        from: "helper",
        to: "renamed",
        definitions: 1,
        calls: 3,
        changed: true,
        rewritten_contains: [
            "(defmacro renamed (x)",
            "(renamed 1)",
            "#'renamed",
            "(macro-function 'renamed)",
            "helper '(helper 2)"
        ]
    };
}

#[test]
fn renames_common_lisp_qualified_defmacro_definition_and_macro_calls() {
    assert_function_rename! {
        input: "(cl:defmacro helper (x) `(list ,x))\n(helper 1)\n(list #'helper helper)",
        dialect: Dialect::CommonLisp,
        from: "helper",
        to: "renamed",
        definitions: 1,
        calls: 2,
        changed: true,
        rewritten_contains: ["(cl:defmacro renamed (x)", "(renamed 1)", "#'renamed helper"]
    };
}

#[test]
fn renames_common_lisp_user_qualified_defmacro_definition_and_macro_calls() {
    assert_function_rename! {
        input: "(cl-user:defmacro helper (x) `(list ,x))\n(helper 1)\n(list #'helper helper)",
        dialect: Dialect::CommonLisp,
        from: "helper",
        to: "renamed",
        definitions: 1,
        calls: 2,
        changed: true,
        rewritten_contains: ["(cl-user:defmacro renamed (x)", "(renamed 1)", "#'renamed helper"]
    };
}

#[test]
fn renames_emacs_lisp_cl_defmacro_definition_and_macro_calls() {
    assert_function_rename! {
        input: "(cl-defmacro helper (x) `(list ,x))\n(helper 1)\n(defun caller () (helper 2) helper)",
        dialect: Dialect::EmacsLisp,
        from: "helper",
        to: "renamed",
        definitions: 1,
        calls: 2,
        changed: true,
        rewritten_contains: [
            "(cl-defmacro renamed (x)",
            "(renamed 1)",
            "(defun caller () (renamed 2) helper)"
        ]
    };
}

#[test]
fn renames_defmacro_definition_inside_reader_quoted_lambda_body_without_touching_shadowed_macro_body()
{
    assert_function_rename! {
        input: "(defmacro helper (x) `(list ,x))\n(defun caller () #'(lambda () (macrolet ((helper (value) (list #'helper (function helper) (helper value)))) (helper 1))))",
        dialect: Dialect::CommonLisp,
        from: "helper",
        to: "renamed",
        definitions: 1,
        calls: 3,
        changed: true,
        rewritten_contains: [
            "(defmacro renamed (x) `(list ,x))",
            "#'(lambda () (macrolet ((helper (value) (list #'renamed (function renamed) (renamed value)))) (helper 1)))"
        ]
    };
}

#[test]
fn renames_common_lisp_qualified_defmacro_definition_inside_reader_quoted_lambda_body_without_touching_shadowed_macro_body()
{
    assert_function_rename! {
        input: "(cl:defmacro helper (x) `(list ,x))\n(defun caller () #'(lambda () (macrolet ((helper (value) (list #'helper (function helper) (helper value)))) (helper 1))))",
        dialect: Dialect::CommonLisp,
        from: "helper",
        to: "renamed",
        definitions: 1,
        calls: 3,
        changed: true,
        rewritten_contains: [
            "(cl:defmacro renamed (x) `(list ,x))",
            "#'(lambda () (macrolet ((helper (value) (list #'renamed (function renamed) (renamed value)))) (helper 1)))"
        ]
    };
}

#[test]
fn renames_common_lisp_user_qualified_defmacro_definition_inside_reader_quoted_lambda_body_without_touching_shadowed_macro_body()
{
    assert_function_rename! {
        input: "(cl-user:defmacro helper (x) `(list ,x))\n(defun caller () #'(lambda () (macrolet ((helper (value) (list #'helper (function helper) (helper value)))) (helper 1))))",
        dialect: Dialect::CommonLisp,
        from: "helper",
        to: "renamed",
        definitions: 1,
        calls: 3,
        changed: true,
        rewritten_contains: [
            "(cl-user:defmacro renamed (x) `(list ,x))",
            "#'(lambda () (macrolet ((helper (value) (list #'renamed (function renamed) (renamed value)))) (helper 1)))"
        ]
    };
}
