use super::*;

#[test]
fn cli_plans_define_symbol_macro_rename_without_touching_call_heads_or_quoted_forms() {
    assert_plan_case(
        "rename-symbol-macro-plan",
        "(define-symbol-macro old-name current-user) (list old-name (old-name 1) #'old-name `(slot ,old-name old-name))\n",
        "(define-symbol-macro new-name current-user) (list new-name (old-name 1) #'old-name `(slot ,new-name old-name))",
        1,
        2,
    );
}

#[test]
fn cli_plans_define_symbol_macro_rename_skipping_quoted_forms_and_preserving_unquote_references() {
    assert_plan_case(
        "rename-symbol-macro-quoted",
        "(define-symbol-macro old-name current-user) '(old-name) #'old-name `(list ,old-name old-name)\n",
        "(define-symbol-macro new-name current-user) '(old-name) #'old-name `(list ,new-name old-name)",
        1,
        1,
    );
}

#[test]
fn cli_plans_define_symbol_macro_rename_inside_reader_quoted_lambda_body_without_touching_function_designators()
 {
    assert_plan_case(
        "rename-symbol-macro-reader-quoted-lambda-function-designators",
        "(define-symbol-macro old-name current-user) #'(lambda () (define-symbol-macro old-name (list #'old-name (function old-name) old-name)) old-name) old-name\n",
        "(define-symbol-macro new-name current-user) #'(lambda () (define-symbol-macro new-name (list #'old-name (function old-name) old-name)) new-name) new-name",
        2,
        2,
    );
}

#[test]
fn cli_plans_define_symbol_macro_rename_inside_reader_quoted_lambda_with_cl_symbol_macrolet_shadowing()
 {
    assert_plan_case(
        "rename-symbol-macro-reader-quoted-cl-shadowing",
        "(define-symbol-macro old-name current-user) #'(lambda () (cl:symbol-macrolet ((old-name other-user)) old-name) old-name) old-name\n",
        "(define-symbol-macro new-name current-user) #'(lambda () (cl:symbol-macrolet ((old-name other-user)) old-name) new-name) new-name",
        1,
        2,
    );
}

#[test]
fn cli_plans_define_symbol_macro_rename_inside_reader_quoted_lambda_with_cl_user_symbol_macrolet_shadowing()
 {
    assert_plan_case(
        "rename-symbol-macro-reader-quoted-cl-user-shadowing",
        "(define-symbol-macro old-name current-user) #'(lambda () (cl-user:symbol-macrolet ((old-name other-user)) old-name) old-name) old-name\n",
        "(define-symbol-macro new-name current-user) #'(lambda () (cl-user:symbol-macrolet ((old-name other-user)) old-name) new-name) new-name",
        1,
        2,
    );
}

#[test]
fn cli_plans_define_symbol_macro_rename_in_locally_body_without_counting_declarations() {
    assert_plan_case(
        "rename-symbol-macro-locally-body",
        "(define-symbol-macro old-name current-user) (locally (declare (special old-name)) old-name) (locally (declaim (special old-name)) (proclaim (special old-name)) old-name) old-name\n",
        "(define-symbol-macro new-name current-user) (locally (declare (special old-name)) new-name) (locally (declaim (special old-name)) (proclaim (special old-name)) new-name) new-name",
        1,
        3,
    );
}

#[test]
fn cli_plans_define_symbol_macro_rename_skipping_definition_scope_boundaries() {
    assert_plan_case(
        "rename-symbol-macro-definition-boundaries",
        "(define-symbol-macro old-name current-user) (list old-name (define-setf-expander slot (place) (list old-name place)) (define-compiler-macro render (place) (list old-name place)) old-name)\n",
        "(define-symbol-macro new-name current-user) (list new-name (define-setf-expander slot (place) (list old-name place)) (define-compiler-macro render (place) (list old-name place)) new-name)",
        1,
        2,
    );
}
