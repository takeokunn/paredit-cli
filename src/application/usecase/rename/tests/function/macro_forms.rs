use super::*;

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
fn renames_define_modify_macro_definition_and_macro_calls() {
    assert_function_rename! {
        input: "(define-modify-macro bumpf (delta) +)\n(bumpf place 1)\n(list #'bumpf bumpf)",
        dialect: Dialect::CommonLisp,
        from: "bumpf",
        to: "stepf",
        definitions: 1,
        calls: 2,
        changed: true,
        rewritten_contains: ["(define-modify-macro stepf (delta) +)", "(stepf place 1)", "#'stepf bumpf"]
    };
}

#[test]
fn renames_common_lisp_qualified_define_modify_macro_definition_and_macro_calls() {
    assert_function_rename! {
        input: "(cl:define-modify-macro bumpf (delta) +)\n(bumpf place 1)\n(list #'bumpf bumpf)",
        dialect: Dialect::CommonLisp,
        from: "bumpf",
        to: "stepf",
        definitions: 1,
        calls: 2,
        changed: true,
        rewritten_contains: ["(cl:define-modify-macro stepf (delta) +)", "(stepf place 1)", "#'stepf bumpf"]
    };
}

#[test]
fn renames_common_lisp_user_qualified_define_modify_macro_definition_and_macro_calls() {
    assert_function_rename! {
        input: "(cl-user:define-modify-macro bumpf (delta) +)\n(bumpf place 1)\n(list #'bumpf bumpf)",
        dialect: Dialect::CommonLisp,
        from: "bumpf",
        to: "stepf",
        definitions: 1,
        calls: 2,
        changed: true,
        rewritten_contains: ["(cl-user:define-modify-macro stepf (delta) +)", "(stepf place 1)", "#'stepf bumpf"]
    };
}

#[test]
fn renames_define_compiler_macro_definition_and_designators() {
    assert_function_rename! {
        input: "(define-compiler-macro fast-add (x y) `(+ ,x ,y))\n(list #'fast-add (function fast-add) (compiler-macro-function 'fast-add) fast-add)",
        dialect: Dialect::CommonLisp,
        from: "fast-add",
        to: "optimized-add",
        definitions: 1,
        calls: 3,
        changed: true,
        rewritten_contains: [
            "(define-compiler-macro optimized-add (x y)",
            "#'optimized-add",
            "(function optimized-add)",
            "(compiler-macro-function 'optimized-add)",
            " fast-add)"
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

#[test]
fn renames_common_lisp_qualified_define_compiler_macro_definition_and_designators() {
    assert_function_rename! {
        input: "(cl:define-compiler-macro fast-add (x y) `(+ ,x ,y))\n(list #'fast-add (function fast-add) fast-add)",
        dialect: Dialect::CommonLisp,
        from: "fast-add",
        to: "optimized-add",
        definitions: 1,
        calls: 2,
        changed: true,
        rewritten_contains: [
            "(cl:define-compiler-macro optimized-add (x y)",
            "#'optimized-add",
            "(function optimized-add)",
            " fast-add)"
        ]
    };
}

#[test]
fn renames_common_lisp_user_qualified_define_compiler_macro_definition_inside_reader_quoted_lambda_body_without_touching_shadowed_macro_body()
 {
    assert_function_rename! {
        input: "(cl-user:define-compiler-macro fast-add (x y) `(+ ,x ,y))\n(defun caller () #'(lambda () (compiler-macrolet ((fast-add (value) (list #'fast-add (function fast-add) (fast-add value)))) (fast-add 1))))",
        dialect: Dialect::CommonLisp,
        from: "fast-add",
        to: "optimized-add",
        definitions: 1,
        calls: 3,
        changed: true,
        rewritten_contains: [
            "(cl-user:define-compiler-macro optimized-add (x y) `(+ ,x ,y))",
            "#'(lambda () (compiler-macrolet ((fast-add (value) (list #'optimized-add (function optimized-add) (optimized-add value)))) (fast-add 1)))"
        ]
    };
}

#[test]
fn renames_common_lisp_qualified_define_compiler_macro_definition_inside_reader_quoted_lambda_body_without_touching_shadowed_macro_body()
 {
    assert_function_rename! {
        input: "(cl:define-compiler-macro fast-add (x y) `(+ ,x ,y))\n(defun caller () #'(lambda () (compiler-macrolet ((fast-add (value) (list #'fast-add (function fast-add) (fast-add value)))) (fast-add 1))))",
        dialect: Dialect::CommonLisp,
        from: "fast-add",
        to: "optimized-add",
        definitions: 1,
        calls: 3,
        changed: true,
        rewritten_contains: [
            "(cl:define-compiler-macro optimized-add (x y) `(+ ,x ,y))",
            "#'(lambda () (compiler-macrolet ((fast-add (value) (list #'optimized-add (function optimized-add) (optimized-add value)))) (fast-add 1)))"
        ]
    };
}

#[test]
fn renames_common_lisp_qualified_define_compiler_macro_definition_inside_reader_quoted_lambda_body_without_touching_qualified_shadowed_macro_body()
 {
    assert_function_rename! {
        input: "(cl:define-compiler-macro fast-add (x y) `(+ ,x ,y))\n(defun caller () #'(lambda () (cl:compiler-macrolet ((fast-add (value) (list #'fast-add (function fast-add) (fast-add value)))) (fast-add 1))))",
        dialect: Dialect::CommonLisp,
        from: "fast-add",
        to: "optimized-add",
        definitions: 1,
        calls: 3,
        changed: true,
        rewritten_contains: [
            "(cl:define-compiler-macro optimized-add (x y) `(+ ,x ,y))",
            "#'(lambda () (cl:compiler-macrolet ((fast-add (value) (list #'optimized-add (function optimized-add) (optimized-add value)))) (fast-add 1)))"
        ]
    };
}

#[test]
fn renames_common_lisp_user_qualified_define_compiler_macro_definition_inside_reader_quoted_lambda_body_without_touching_qualified_shadowed_macro_body()
 {
    assert_function_rename! {
        input: "(cl-user:define-compiler-macro fast-add (x y) `(+ ,x ,y))\n(defun caller () #'(lambda () (cl-user:compiler-macrolet ((fast-add (value) (list #'fast-add (function fast-add) (fast-add value)))) (fast-add 1))))",
        dialect: Dialect::CommonLisp,
        from: "fast-add",
        to: "optimized-add",
        definitions: 1,
        calls: 3,
        changed: true,
        rewritten_contains: [
            "(cl-user:define-compiler-macro optimized-add (x y) `(+ ,x ,y))",
            "#'(lambda () (cl-user:compiler-macrolet ((fast-add (value) (list #'optimized-add (function optimized-add) (optimized-add value)))) (fast-add 1)))"
        ]
    };
}

#[test]
fn renames_define_compiler_macro_definition_inside_reader_quoted_lambda_body_without_touching_shadowed_macro_body()
 {
    assert_function_rename! {
        input: "(define-compiler-macro fast-add (x y) `(+ ,x ,y))\n(defun caller () #'(lambda () (compiler-macrolet ((fast-add (value) (list #'fast-add (function fast-add) (fast-add value)))) (fast-add 1))))",
        dialect: Dialect::CommonLisp,
        from: "fast-add",
        to: "optimized-add",
        definitions: 1,
        calls: 3,
        changed: true,
        rewritten_contains: [
            "(define-compiler-macro optimized-add (x y) `(+ ,x ,y))",
            "#'(lambda () (compiler-macrolet ((fast-add (value) (list #'optimized-add (function optimized-add) (optimized-add value)))) (fast-add 1)))"
        ]
    };
}

#[test]
fn renames_common_lisp_user_qualified_define_compiler_macro_definition_and_designators() {
    assert_function_rename! {
        input: "(cl-user:define-compiler-macro fast-add (x y) `(+ ,x ,y))\n(list #'fast-add (function fast-add) fast-add)",
        dialect: Dialect::CommonLisp,
        from: "fast-add",
        to: "optimized-add",
        definitions: 1,
        calls: 2,
        changed: true,
        rewritten_contains: [
            "(cl-user:define-compiler-macro optimized-add (x y)",
            "#'optimized-add",
            "(function optimized-add)",
            " fast-add)"
        ]
    };
}

#[test]
fn renames_define_setf_expander_definition_place_uses_and_designators() {
    assert_function_rename! {
        input: "(define-setf-expander accessor (place) (values nil nil '(store) '(writer store) '(reader place)))\n(setf (accessor item) 1)\n(list #'accessor (function accessor) accessor)",
        dialect: Dialect::CommonLisp,
        from: "accessor",
        to: "slot-accessor",
        definitions: 1,
        calls: 3,
        changed: true,
        rewritten_contains: [
            "(define-setf-expander slot-accessor (place)",
            "(setf (slot-accessor item) 1)",
            "#'slot-accessor",
            "(function slot-accessor)",
            " accessor)"
        ]
    };
}

#[test]
fn renames_common_lisp_user_qualified_define_setf_expander_definition_place_uses_and_designators() {
    assert_function_rename! {
        input: "(cl-user:define-setf-expander accessor (place) (values nil nil '(store) '(writer store) '(reader place)))\n(setf (accessor item) 1)\n(list #'accessor (function accessor) accessor)",
        dialect: Dialect::CommonLisp,
        from: "accessor",
        to: "slot-accessor",
        definitions: 1,
        calls: 3,
        changed: true,
        rewritten_contains: [
            "(cl-user:define-setf-expander slot-accessor (place)",
            "(setf (slot-accessor item) 1)",
            "#'slot-accessor",
            "(function slot-accessor)",
            " accessor)"
        ]
    };
}

#[test]
fn renames_defsetf_definition_place_uses_and_designators() {
    assert_function_rename! {
        input: "(defsetf accessor set-accessor)\n(setf (accessor item) 1)\n(list #'accessor (function accessor) accessor)",
        dialect: Dialect::CommonLisp,
        from: "accessor",
        to: "slot-accessor",
        definitions: 1,
        calls: 3,
        changed: true,
        rewritten_contains: [
            "(defsetf slot-accessor set-accessor)",
            "(setf (slot-accessor item) 1)",
            "#'slot-accessor",
            "(function slot-accessor)",
            " accessor)"
        ]
    };
}

#[test]
fn renames_common_lisp_user_qualified_defsetf_definition_place_uses_and_designators() {
    assert_function_rename! {
        input: "(cl-user:defsetf accessor set-accessor)\n(setf (accessor item) 1)\n(list #'accessor (function accessor) accessor)",
        dialect: Dialect::CommonLisp,
        from: "accessor",
        to: "slot-accessor",
        definitions: 1,
        calls: 3,
        changed: true,
        rewritten_contains: [
            "(cl-user:defsetf slot-accessor set-accessor)",
            "(setf (slot-accessor item) 1)",
            "#'slot-accessor",
            "(function slot-accessor)",
            " accessor)"
        ]
    };
}

#[test]
fn renames_common_lisp_qualified_defsetf_definition_place_uses_and_designators() {
    assert_function_rename! {
        input: "(cl:defsetf accessor set-accessor)\n(setf (accessor item) 1)\n(list #'accessor (function accessor) accessor)",
        dialect: Dialect::CommonLisp,
        from: "accessor",
        to: "slot-accessor",
        definitions: 1,
        calls: 3,
        changed: true,
        rewritten_contains: [
            "(cl:defsetf slot-accessor set-accessor)",
            "(setf (slot-accessor item) 1)",
            "#'slot-accessor",
            "(function slot-accessor)",
            " accessor)"
        ]
    };
}
