use super::*;

#[test]
fn cli_writes_setf_local_callable_rename_updates_definition_and_call_site() {
    assert_cli_local_function_write(
        "rename-local-function-setf-write",
        "lisp",
        "(flet (((setf old-name) (value object) value)) ((setf old-name) 1 thing) old-name)\n",
        1,
        1,
        "(flet (((setf new-name) (value object) value)) ((setf new-name) 1 thing) old-name)\n",
    );
}

#[test]
fn cli_writes_labels_setf_local_callable_rename_updates_definition_and_call_site() {
    assert_cli_local_function_write(
        "rename-local-function-labels-setf-write",
        "lisp",
        "(labels (((setf old-name) (value object) value)) ((setf old-name) 1 thing) old-name)\n",
        1,
        1,
        "(labels (((setf new-name) (value object) value)) ((setf new-name) 1 thing) old-name)\n",
    );
}

#[test]
fn cli_writes_package_qualified_setf_local_callable_rename_updates_definition_and_call_site() {
    assert_cli_local_function_write(
        "rename-local-function-qualified-setf-write",
        "lisp",
        "(cl-user:flet (((setf old-name) (value object) value)) ((setf old-name) 1 thing) old-name)\n",
        1,
        1,
        "(cl-user:flet (((setf new-name) (value object) value)) ((setf new-name) 1 thing) old-name)\n",
    );
}

#[test]
fn cli_writes_flet_setf_local_callable_reader_designators() {
    assert_cli_local_function_write(
        "rename-local-function-setf-designators-write",
        "lisp",
        "(flet (((setf old-name) (value object) value)) #'(setf old-name) (function (setf old-name)) ((setf old-name) 1 thing) old-name)\n",
        1,
        3,
        "(flet (((setf new-name) (value object) value)) #'(setf new-name) (function (setf new-name)) ((setf new-name) 1 thing) old-name)\n",
    );
}

#[test]
fn cli_writes_labels_setf_local_callable_reader_designators_and_recursion() {
    assert_cli_local_function_write(
        "rename-local-function-labels-setf-designators-write",
        "lisp",
        "(labels (((setf old-name) (value object) #'(setf old-name) (function (setf old-name)) ((setf old-name) value object))) #'(setf old-name) (function (setf old-name)) old-name)\n",
        1,
        5,
        "(labels (((setf new-name) (value object) #'(setf new-name) (function (setf new-name)) ((setf new-name) value object))) #'(setf new-name) (function (setf new-name)) old-name)\n",
    );
}
