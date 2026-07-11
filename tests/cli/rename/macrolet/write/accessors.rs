use super::super::*;

macrolet_write_case!(
    cli_writes_macrolet_rename_without_touching_global_macro_cell_accessors,
    "rename-macrolet-global-accessors-write",
    "core.lisp",
    "(macrolet ((old-name (x) x)) (macro-function 'old-name) (compiler-macro-function 'old-name) (old-name 1) old-name)\n",
    "(macrolet ((new-name (x) x)) (macro-function 'old-name) (compiler-macro-function 'old-name) (new-name 1) old-name)\n",
    1
);

macrolet_write_case!(
    cli_writes_compiler_macrolet_rename_without_touching_global_macro_cell_accessors,
    "rename-compiler-macrolet-global-accessors-write",
    "core.lisp",
    "(compiler-macrolet ((old-name (x) x)) (macro-function 'old-name) (compiler-macro-function 'old-name) (old-name 1) old-name)\n",
    "(compiler-macrolet ((new-name (x) x)) (macro-function 'old-name) (compiler-macro-function 'old-name) (new-name 1) old-name)\n",
    1
);

macrolet_write_case!(
    cli_writes_macrolet_rename_without_touching_setf_function_call_heads,
    "rename-macrolet-setf-function-call-heads-write",
    "core.lisp",
    "(macrolet ((old-name (x) x)) ((setf old-name) 1 thing) (old-name 1) old-name)\n",
    "(macrolet ((new-name (x) x)) ((setf old-name) 1 thing) (new-name 1) old-name)\n",
    1
);
