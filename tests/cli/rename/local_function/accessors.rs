use super::*;

#[test]
fn cli_writes_flet_rename_without_touching_global_function_cell_accessors() {
    assert_cli_local_function_write(
        "rename-local-function-global-accessors-flet-write",
        "lisp",
        "(flet ((old-name (x) x)) (macro-function 'old-name) (compiler-macro-function 'old-name) (symbol-function 'old-name) (fdefinition 'old-name) (old-name 1) old-name)\n",
        1,
        1,
        "(flet ((new-name (x) x)) (macro-function 'old-name) (compiler-macro-function 'old-name) (symbol-function 'old-name) (fdefinition 'old-name) (new-name 1) old-name)\n",
    );
}

#[test]
fn cli_writes_labels_rename_without_touching_global_function_cell_accessors() {
    assert_cli_local_function_write(
        "rename-local-function-global-accessors-labels-write",
        "lisp",
        "(labels ((old-name (x) (old-name x))) (macro-function 'old-name) (compiler-macro-function 'old-name) (symbol-function 'old-name) (fdefinition 'old-name) (old-name 1) old-name)\n",
        1,
        2,
        "(labels ((new-name (x) (new-name x))) (macro-function 'old-name) (compiler-macro-function 'old-name) (symbol-function 'old-name) (fdefinition 'old-name) (new-name 1) old-name)\n",
    );
}
