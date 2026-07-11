use super::*;

#[test]
fn cli_plans_flet_rename_without_touching_definition_body_or_noncall_values() {
    assert_cli_local_function_plan(
        "rename-local-function-flet-plan",
        "lisp",
        "(flet ((old-name (x) (old-name x))) (old-name 1) old-name)\n",
        1,
        1,
        "(flet ((new-name (x) (old-name x))) (new-name 1) old-name)",
    );
}

#[test]
fn cli_writes_labels_rename_with_recursive_calls() {
    assert_cli_local_function_write(
        "rename-local-function-labels-write",
        "lisp",
        "(labels ((old-name (x) (old-name x))) (old-name 1) old-name)\n",
        1,
        2,
        "(labels ((new-name (x) (new-name x))) (new-name 1) old-name)\n",
    );
}

#[test]
fn cli_plans_package_qualified_flet_rename() {
    assert_cli_local_function_plan(
        "rename-local-function-qualified-flet-plan",
        "lisp",
        "(cl:flet ((old-name (x) (old-name x))) (old-name 1) old-name)\n",
        1,
        1,
        "(cl:flet ((new-name (x) (old-name x))) (new-name 1) old-name)",
    );
}

#[test]
fn cli_writes_package_qualified_labels_rename_with_recursive_calls() {
    assert_cli_local_function_write(
        "rename-local-function-qualified-labels-write",
        "lisp",
        "(cl-user:labels ((old-name (x) (old-name x))) (old-name 1) old-name)\n",
        1,
        2,
        "(cl-user:labels ((new-name (x) (new-name x))) (new-name 1) old-name)\n",
    );
}

#[test]
fn cli_plans_emacs_lisp_cl_flet_rename_without_touching_definition_body_or_noncall_values() {
    assert_cli_local_function_plan(
        "rename-local-function-emacs-cl-flet-plan",
        "el",
        "(cl-flet ((old-name (x) (old-name x))) (old-name 1) old-name)\n",
        1,
        1,
        "(cl-flet ((new-name (x) (old-name x))) (new-name 1) old-name)",
    );
}

#[test]
fn cli_writes_emacs_lisp_cl_labels_rename_with_recursive_calls() {
    assert_cli_local_function_write(
        "rename-local-function-emacs-cl-labels-write",
        "el",
        "(cl-labels ((old-name (x) (old-name x))) (old-name 1) old-name)\n",
        1,
        2,
        "(cl-labels ((new-name (x) (new-name x))) (new-name 1) old-name)\n",
    );
}

#[test]
fn cli_writes_local_function_rename_without_crossing_nested_shadow() {
    assert_cli_local_function_write(
        "rename-local-function-shadow-write",
        "lisp",
        "(flet ((old-name (x) x)) (labels ((old-name (y) (old-name y))) (old-name 1)) (old-name 2))\n",
        1,
        1,
        "(flet ((new-name (x) x)) (labels ((old-name (y) (old-name y))) (old-name 1)) (new-name 2))\n",
    );
}

#[test]
fn cli_writes_labels_rename_with_function_designators() {
    assert_cli_local_function_write(
        "rename-local-function-designators-write",
        "lisp",
        "(labels ((old-name (x) #'old-name (function old-name) (old-name x))) #'old-name (function old-name) old-name)\n",
        1,
        5,
        "(labels ((new-name (x) #'new-name (function new-name) (new-name x))) #'new-name (function new-name) old-name)\n",
    );
}

#[test]
fn cli_writes_flet_rename_inside_reader_quoted_lambda_bodies() {
    assert_cli_local_function_write(
        "rename-local-function-reader-quoted-lambda-write",
        "lisp",
        "(flet ((old-name (x) #'(lambda () (old-name x) old-name))) (old-name 1) old-name)\n",
        1,
        3,
        "(flet ((new-name (x) #'(lambda () (new-name x) new-name))) (new-name 1) old-name)\n",
    );
}
