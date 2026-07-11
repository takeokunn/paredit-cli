use super::*;

#[test]
fn cli_writes_define_symbol_macro_rename_skipping_lexically_shadowed_bindings_and_parameters() {
    assert_write_case(
        "rename-symbol-macro-lexical-shadowing",
        "(define-symbol-macro old-name current-user) (let ((old-name 1)) old-name) (lambda (old-name) old-name) old-name\n",
        "(define-symbol-macro new-name current-user) (let ((old-name 1)) old-name) (lambda (old-name) old-name) new-name\n",
        1,
        1,
    );
}

#[test]
fn cli_writes_define_symbol_macro_rename_skipping_shadowing_symbol_macrolet_bindings() {
    assert_write_case(
        "rename-symbol-macro-symbol-macrolet-shadowing",
        "(define-symbol-macro old-name current-user) (symbol-macrolet ((old-name other-user)) old-name) old-name\n",
        "(define-symbol-macro new-name current-user) (symbol-macrolet ((old-name other-user)) old-name) new-name\n",
        1,
        1,
    );
}

#[test]
fn cli_writes_define_symbol_macro_rename_skipping_cl_user_symbol_macrolet_shadowing() {
    assert_write_case(
        "rename-symbol-macro-cl-user-symbol-macrolet-shadowing",
        "(define-symbol-macro old-name current-user) (cl-user:symbol-macrolet ((old-name other-user)) old-name) old-name\n",
        "(define-symbol-macro new-name current-user) (cl-user:symbol-macrolet ((old-name other-user)) old-name) new-name\n",
        1,
        1,
    );
}

#[test]
fn cli_writes_define_symbol_macro_rename_skipping_cl_symbol_macrolet_shadowing() {
    assert_write_case(
        "rename-symbol-macro-cl-symbol-macrolet-shadowing",
        "(define-symbol-macro old-name current-user) (cl:symbol-macrolet ((old-name other-user)) old-name) old-name\n",
        "(define-symbol-macro new-name current-user) (cl:symbol-macrolet ((old-name other-user)) old-name) new-name\n",
        1,
        1,
    );
}

#[test]
fn cli_writes_define_symbol_macro_rename_preserving_symbol_macrolet_body_shadowing_while_updating_rhs()
 {
    assert_write_case(
        "rename-symbol-macro-symbol-macrolet-rhs",
        "(define-symbol-macro old-name current-user) (symbol-macrolet ((old-name old-name)) old-name) old-name\n",
        "(define-symbol-macro new-name current-user) (symbol-macrolet ((old-name new-name)) old-name) new-name\n",
        1,
        2,
    );
}

#[test]
fn cli_writes_define_symbol_macro_rename_skipping_nested_symbol_macrolet_shadowing() {
    assert_write_case(
        "rename-symbol-macro-nested-shadowing",
        "(define-symbol-macro old-name current-user) (symbol-macrolet ((old-name other-user)) (symbol-macrolet ((old-name inner-user)) old-name) old-name) old-name\n",
        "(define-symbol-macro new-name current-user) (symbol-macrolet ((old-name other-user)) (symbol-macrolet ((old-name inner-user)) old-name) old-name) new-name\n",
        1,
        1,
    );
}

#[test]
fn cli_writes_define_symbol_macro_rename_without_touching_common_lisp_shadowing_forms() {
    assert_write_case(
        "rename-symbol-macro-shadowing",
        "(define-symbol-macro old-name current-user) (with-slots (old-name) object old-name) (with-accessors ((old-name get-old-name)) object old-name) (do ((old-name seed (1+ old-name))) ((done) old-name) old-name) (prog ((old-name seed)) (return old-name)) (loop for old-name in values collect old-name finally (return old-name)) (handler-bind ((error (lambda (old-name) old-name))) old-name) (restart-bind ((retry (lambda () old-name) :report (lambda (old-name) old-name))) old-name) old-name\n",
        "(define-symbol-macro new-name current-user) (with-slots (old-name) object old-name) (with-accessors ((old-name get-old-name)) object old-name) (do ((old-name seed (1+ old-name))) ((done) old-name) old-name) (prog ((old-name seed)) (return old-name)) (loop for old-name in values collect old-name finally (return old-name)) (handler-bind ((error (lambda (old-name) old-name))) new-name) (restart-bind ((retry (lambda () new-name) :report (lambda (old-name) old-name))) new-name) new-name\n",
        1,
        4,
    );
}

#[test]
fn cli_writes_define_symbol_macro_rename_without_touching_cl_user_shadowing_forms() {
    assert_write_case(
        "rename-symbol-macro-cl-user-shadowing",
        "(define-symbol-macro old-name current-user) (cl-user:with-slots (old-name) object old-name) (cl-user:with-accessors ((old-name get-old-name)) object old-name) old-name\n",
        "(define-symbol-macro new-name current-user) (cl-user:with-slots (old-name) object old-name) (cl-user:with-accessors ((old-name get-old-name)) object old-name) new-name\n",
        1,
        1,
    );
}
