use super::*;

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
fn preserves_unquote_prefixes_when_renaming_function_calls_inside_quasiquote() {
    assert_function_rename! {
        input: "(defun helper (x) x)\n(defmacro build () `(list ,(helper 1) ,@(helper 2) (helper 3)))",
        dialect: Dialect::CommonLisp,
        from: "helper",
        to: "renamed",
        definitions: 1,
        calls: 2,
        changed: true,
        rewritten_contains: ["(defun renamed (x) x)", "`(list ,(renamed 1) ,@(renamed 2) (helper 3))"]
    };
}

#[test]
fn renames_function_designators_but_skips_quoted_data() {
    assert_function_rename! {
        input: "(defun helper (x) x)\n(list '(helper 1) #'helper (function helper) `(helper ,value) (helper 2) (symbol-function 'helper) (fdefinition 'helper))",
        dialect: Dialect::CommonLisp,
        from: "helper",
        to: "renamed",
        definitions: 1,
        calls: 5,
        changed: true,
        rewritten_contains: [
            "(defun renamed (x) x)",
            "'(helper 1)",
            "#'renamed",
            "(function renamed)",
            "`(helper ,value)",
            "(renamed 2)",
            "(symbol-function 'renamed)",
            "(fdefinition 'renamed)"
        ]
    };
}

#[test]
fn renames_unquoted_callable_designators_inside_quasiquote() {
    assert_function_rename! {
        input: "(defun helper (x) x)\n(defun caller () `(list ,#'helper ,(function helper) ,(symbol-function 'helper) ,(fdefinition 'helper)))",
        dialect: Dialect::CommonLisp,
        from: "helper",
        to: "renamed",
        definitions: 1,
        calls: 4,
        changed: true,
        rewritten_contains: [
            "(defun renamed (x) x)",
            "(defun caller () `(list ,#'renamed ,(function renamed) ,(symbol-function 'renamed) ,(fdefinition 'renamed)))"
        ]
    };
}

#[test]
fn renames_unquoted_macro_function_designators_inside_quasiquote() {
    assert_function_rename! {
        input: "(defmacro helper (x) x)\n(defun caller () `(list ,(macro-function 'helper) ,(macro-function 'other) '(macro-function helper)))",
        dialect: Dialect::CommonLisp,
        from: "helper",
        to: "renamed",
        definitions: 1,
        calls: 1,
        changed: true,
        rewritten_contains: [
            "(defmacro renamed (x) x)",
            "(defun caller () `(list ,(macro-function 'renamed) ,(macro-function 'other) '(macro-function helper)))"
        ]
    };
}

#[test]
fn renames_unquoted_compiler_macro_function_designators_inside_quasiquote() {
    assert_function_rename! {
        input: "(define-compiler-macro helper (x) x)\n(defun caller () `(list ,(compiler-macro-function 'helper) ,(compiler-macro-function 'other) '(compiler-macro-function helper)))",
        dialect: Dialect::CommonLisp,
        from: "helper",
        to: "renamed",
        definitions: 1,
        calls: 1,
        changed: true,
        rewritten_contains: [
            "(define-compiler-macro renamed (x) x)",
            "(defun caller () `(list ,(compiler-macro-function 'renamed) ,(compiler-macro-function 'other) '(compiler-macro-function helper)))"
        ]
    };
}

#[test]
fn renames_qualified_macro_function_designators_inside_quasiquote() {
    assert_function_rename! {
        input: "(defmacro helper (x) x)\n(define-compiler-macro helper (x) x)\n(defun caller () `(list ,(cl:macro-function 'cl-user:helper) ,(cl-user:compiler-macro-function 'common-lisp-user:helper) '(cl:macro-function helper) '(cl-user:compiler-macro-function helper)))",
        dialect: Dialect::CommonLisp,
        from: "helper",
        to: "renamed",
        definitions: 2,
        calls: 2,
        changed: true,
        rewritten_contains: [
            "(defmacro renamed (x) x)",
            "(define-compiler-macro renamed (x) x)",
            "(defun caller () `(list ,(cl:macro-function 'renamed) ,(cl-user:compiler-macro-function 'renamed) '(cl:macro-function helper) '(cl-user:compiler-macro-function helper)))"
        ]
    };
}

#[test]
fn renames_quoted_setf_function_names_in_fdefinition() {
    assert_function_rename! {
        input: "(defun accessor (x) x)\n(fdefinition '(setf accessor))",
        dialect: Dialect::CommonLisp,
        from: "accessor",
        to: "slot-accessor",
        definitions: 1,
        calls: 1,
        changed: true,
        rewritten_contains: [
            "(defun slot-accessor (x) x)",
            "(fdefinition '(setf slot-accessor))"
        ]
    };
}

#[test]
fn renames_unquoted_setf_function_designators_inside_quasiquote() {
    assert_function_rename! {
        input: "(defun accessor (x) x)\n(defun caller () `(list ,#'(setf accessor) ,(function (setf accessor)) ,(fdefinition '(setf accessor))))",
        dialect: Dialect::CommonLisp,
        from: "accessor",
        to: "slot-accessor",
        definitions: 1,
        calls: 3,
        changed: true,
        rewritten_contains: [
            "(defun slot-accessor (x) x)",
            "(defun caller () `(list ,#'(setf slot-accessor) ,(function (setf slot-accessor)) ,(fdefinition '(setf slot-accessor))))"
        ]
    };
}

#[test]
fn renames_function_calls_inside_reader_quoted_lambda_bodies() {
    assert_function_rename! {
        input: "(defun helper (x) x)\n(defun caller (x) #'(lambda (y) (helper y)))",
        dialect: Dialect::CommonLisp,
        from: "helper",
        to: "renamed",
        definitions: 1,
        calls: 1,
        changed: true,
        rewritten_contains: ["#'(lambda (y) (renamed y))"]
    };
}

#[test]
fn skips_non_lambda_lists_inside_reader_quoted_function_prefixes() {
    assert_function_rename! {
        input: "(defun helper (x) x)\n(defun caller () #'(foo (helper 1)))",
        dialect: Dialect::CommonLisp,
        from: "helper",
        to: "renamed",
        definitions: 1,
        calls: 0,
        changed: true,
        rewritten_contains: ["#'(foo (helper 1))"]
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
fn renames_outer_function_without_touching_nested_symbol_macrolet_shadowing() {
    assert_function_rename! {
        input: "(defun helper (x) x)\n(defun caller () (symbol-macrolet ((helper other)) helper #'helper (function helper) (helper 1)) (helper 2) #'helper (function helper) (helper 3))",
        dialect: Dialect::CommonLisp,
        from: "helper",
        to: "renamed",
        definitions: 1,
        calls: 7,
        changed: true,
        rewritten_contains: [
            "(defun renamed (x) x)",
            "(symbol-macrolet ((helper other)) helper #'renamed (function renamed) (renamed 1))",
            "(renamed 2) #'renamed (function renamed) (renamed 3)"
        ]
    };
}

#[test]
fn renames_outer_function_without_touching_qualified_symbol_macrolet_shadowing() {
    assert_function_rename! {
        input: "(cl:defun helper (x) x)\n(defun caller () (cl:symbol-macrolet ((helper other)) helper #'helper (function helper) (helper 1)) (cl-user:symbol-macrolet ((helper other)) helper #'helper (function helper) (helper 2)) (helper 3) #'helper (function helper) (helper 4))",
        dialect: Dialect::CommonLisp,
        from: "helper",
        to: "renamed",
        definitions: 1,
        calls: 10,
        changed: true,
        rewritten_contains: [
            "(cl:defun renamed (x) x)",
            "(cl:symbol-macrolet ((helper other)) helper #'renamed (function renamed) (renamed 1))",
            "(cl-user:symbol-macrolet ((helper other)) helper #'renamed (function renamed) (renamed 2))",
            "(renamed 3) #'renamed (function renamed) (renamed 4)"
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
fn renames_outer_function_without_touching_package_qualified_symbol_macrolet_binding_names() {
    assert_function_rename! {
        input: "(defun helper (x) x)\n(defun caller () (symbol-macrolet ((cl-user:helper other)) helper #'helper (function helper) (helper 1)) (helper 2) #'helper (function helper) (helper 3))",
        dialect: Dialect::CommonLisp,
        from: "helper",
        to: "renamed",
        definitions: 1,
        calls: 7,
        changed: true,
        rewritten_contains: [
            "(defun renamed (x) x)",
            "(symbol-macrolet ((cl-user:helper other)) helper #'renamed (function renamed) (renamed 1))",
            "(renamed 2) #'renamed (function renamed) (renamed 3)"
        ]
    };
}

#[test]
fn renames_function_calls_inside_symbol_macrolet_expansion_forms() {
    assert_function_rename! {
        input: "(defun helper (x) x)\n(defun caller () (symbol-macrolet ((helper (helper 0))) helper #'helper (function helper) (helper 1)) (helper 2))",
        dialect: Dialect::CommonLisp,
        from: "helper",
        to: "renamed",
        definitions: 1,
        calls: 5,
        changed: true,
        rewritten_contains: [
            "(defun renamed (x) x)",
            "(symbol-macrolet ((helper (renamed 0))) helper #'renamed (function renamed) (renamed 1))",
            "(renamed 2)"
        ]
    };
}

#[test]
fn renames_outer_function_references_inside_reader_quoted_lambda_bodies_only() {
    assert_function_rename! {
        input: "(defun helper (x) x)\n(defun caller () #'(lambda () (symbol-macrolet ((helper other)) helper) helper #'helper (function helper) (helper 1)))",
        dialect: Dialect::CommonLisp,
        from: "helper",
        to: "renamed",
        definitions: 1,
        calls: 3,
        changed: true,
        rewritten_contains: [
            "(defun renamed (x) x)",
            "#'(lambda () (symbol-macrolet ((helper other)) helper) helper #'renamed (function renamed) (renamed 1))"
        ]
    };
}

#[test]
fn skips_reader_eval_bodies_when_renaming_functions() {
    assert_function_rename! {
        input: "(defun helper (x) x)\n(defun caller () #.(list helper #'helper (function helper) (helper 1)) (helper 2))",
        dialect: Dialect::CommonLisp,
        from: "helper",
        to: "renamed",
        definitions: 1,
        calls: 1,
        changed: true,
        rewritten_contains: [
            "(defun renamed (x) x)",
            "(defun caller () #.(list helper #'helper (function helper) (helper 1)) (renamed 2))"
        ]
    };
}
