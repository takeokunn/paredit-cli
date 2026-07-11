use super::*;

#[test]
fn cli_writes_outer_flet_rename_inside_macrolet_expander_only() {
    assert_cli_local_function_write(
        "rename-local-function-macrolet-expander-write",
        "lisp",
        "(flet ((old-name (x) x)) (macrolet ((old-name () #'old-name (function old-name) (old-name 1))) (old-name) #'old-name (function old-name) (old-name 2)))\n",
        1,
        3,
        "(flet ((new-name (x) x)) (macrolet ((old-name () #'new-name (function new-name) (new-name 1))) (old-name) #'old-name (function old-name) (old-name 2)))\n",
    );
}

#[test]
fn cli_writes_outer_flet_rename_inside_compiler_macrolet_expander_only() {
    assert_cli_local_function_write(
        "rename-local-function-compiler-macrolet-expander-write",
        "lisp",
        "(flet ((old-name (x) x)) (compiler-macrolet ((old-name () #'old-name (function old-name) (old-name 1))) (old-name) #'old-name (function old-name) (old-name 2)))\n",
        1,
        3,
        "(flet ((new-name (x) x)) (compiler-macrolet ((old-name () #'new-name (function new-name) (new-name 1))) (old-name) #'old-name (function old-name) (old-name 2)))\n",
    );
}

#[test]
fn cli_plans_outer_setf_flet_rename_inside_macrolet_expander_only() {
    assert_cli_local_function_plan(
        "rename-local-function-setf-macrolet-expander-plan",
        "lisp",
        "(flet (((setf old-name) (value object) value)) (macrolet ((old-name () #'(setf old-name) (function (setf old-name)) ((setf old-name) 1 thing))) (old-name) #'(setf old-name) (function (setf old-name)) ((setf old-name) 2 thing)))\n",
        1,
        3,
        "\"replacement\": \"new-name\"",
    );
}

#[test]
fn cli_writes_outer_setf_flet_rename_inside_macrolet_expander_only() {
    assert_cli_local_function_write(
        "rename-local-function-setf-macrolet-expander-write",
        "lisp",
        "(flet (((setf old-name) (value object) value)) (macrolet ((old-name () #'(setf old-name) (function (setf old-name)) ((setf old-name) 1 thing))) (old-name) #'(setf old-name) (function (setf old-name)) ((setf old-name) 2 thing)))\n",
        1,
        3,
        "(flet (((setf new-name) (value object) value)) (macrolet ((old-name () #'(setf new-name) (function (setf new-name)) ((setf new-name) 1 thing))) (old-name) #'(setf old-name) (function (setf old-name)) ((setf old-name) 2 thing)))\n",
    );
}

#[test]
fn cli_plans_outer_setf_flet_rename_inside_compiler_macrolet_expander_only() {
    assert_cli_local_function_plan(
        "rename-local-function-setf-compiler-macrolet-expander-plan",
        "lisp",
        "(flet (((setf old-name) (value object) value)) (compiler-macrolet ((old-name () #'(setf old-name) (function (setf old-name)) ((setf old-name) 1 thing))) (old-name) #'(setf old-name) (function (setf old-name)) ((setf old-name) 2 thing)))\n",
        1,
        3,
        "\"replacement\": \"new-name\"",
    );
}

#[test]
fn cli_writes_outer_setf_flet_rename_inside_compiler_macrolet_expander_only() {
    assert_cli_local_function_write(
        "rename-local-function-setf-compiler-macrolet-expander-write",
        "lisp",
        "(flet (((setf old-name) (value object) value)) (compiler-macrolet ((old-name () #'(setf old-name) (function (setf old-name)) ((setf old-name) 1 thing))) (old-name) #'(setf old-name) (function (setf old-name)) ((setf old-name) 2 thing)))\n",
        1,
        3,
        "(flet (((setf new-name) (value object) value)) (compiler-macrolet ((old-name () #'(setf new-name) (function (setf new-name)) ((setf new-name) 1 thing))) (old-name) #'(setf old-name) (function (setf old-name)) ((setf old-name) 2 thing)))\n",
    );
}

#[test]
fn cli_writes_package_qualified_outer_flet_rename_inside_macrolet_expander_only() {
    assert_cli_local_function_write(
        "rename-local-function-qualified-macrolet-expander-write",
        "lisp",
        "(cl-user:flet ((old-name (x) x)) (cl-user:macrolet ((old-name () #'old-name (function old-name) (old-name 1))) (old-name) #'old-name (function old-name) (old-name 2)))\n",
        1,
        3,
        "(cl-user:flet ((new-name (x) x)) (cl-user:macrolet ((old-name () #'new-name (function new-name) (new-name 1))) (old-name) #'old-name (function old-name) (old-name 2)))\n",
    );
}

#[test]
fn cli_writes_package_qualified_outer_flet_rename_inside_compiler_macrolet_expander_only() {
    assert_cli_local_function_write(
        "rename-local-function-qualified-compiler-macrolet-expander-write",
        "lisp",
        "(cl-user:flet ((old-name (x) x)) (cl-user:compiler-macrolet ((old-name () #'old-name (function old-name) (old-name 1))) (old-name) #'old-name (function old-name) (old-name 2)))\n",
        1,
        3,
        "(cl-user:flet ((new-name (x) x)) (cl-user:compiler-macrolet ((old-name () #'new-name (function new-name) (new-name 1))) (old-name) #'old-name (function old-name) (old-name 2)))\n",
    );
}
