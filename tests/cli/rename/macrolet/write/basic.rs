use super::super::*;

macrolet_write_case!(
    cli_writes_macrolet_rename_without_touching_function_designators,
    "rename-macrolet-function-designators-write",
    "core.lisp",
    "(macrolet ((old-name (x) (list #'old-name (function old-name) x))) #'old-name (function old-name) (old-name 1) old-name)\n",
    "(macrolet ((new-name (x) (list #'old-name (function old-name) x))) #'old-name (function old-name) (new-name 1) old-name)\n",
    1
);

macrolet_write_case!(
    cli_writes_macrolet_rename_across_files,
    "rename-macrolet-write",
    "core.lisp",
    "(macrolet ((old-name (x) (list old-name x))) (old-name 1))\n",
    "(macrolet ((new-name (x) (list old-name x))) (new-name 1))\n",
    1
);

macrolet_write_case!(
    cli_writes_compiler_macrolet_rename_without_touching_expander_or_noncall_values,
    "rename-compiler-macrolet-write",
    "core.lisp",
    "(compiler-macrolet ((old-name (x) (list old-name x))) (old-name 1) old-name)\n",
    "(compiler-macrolet ((new-name (x) (list old-name x))) (new-name 1) old-name)\n",
    1
);

macrolet_write_case!(
    cli_writes_cl_user_qualified_compiler_macrolet_rename_without_touching_expander_or_noncall_values,
    "rename-cl-user-compiler-macrolet-write",
    "core.lisp",
    "(cl-user:compiler-macrolet ((old-name (x) (list old-name x))) (old-name 1) old-name)\n",
    "(cl-user:compiler-macrolet ((new-name (x) (list old-name x))) (new-name 1) old-name)\n",
    1
);

macrolet_write_case!(
    cli_writes_cl_qualified_compiler_macrolet_rename_without_touching_expander_or_noncall_values,
    "rename-cl-compiler-macrolet-write",
    "core.lisp",
    "(cl:compiler-macrolet ((old-name (x) (list old-name x))) (old-name 1) old-name)\n",
    "(cl:compiler-macrolet ((new-name (x) (list old-name x))) (new-name 1) old-name)\n",
    1
);

macrolet_write_case!(
    cli_writes_cl_user_compiler_macrolet_rename_without_touching_expander_or_noncall_values,
    "rename-cl-user-compiler-macrolet-write-plain",
    "core.lisp",
    "(cl-user:compiler-macrolet ((old-name (x) (list old-name x))) (old-name 1) old-name)\n",
    "(cl-user:compiler-macrolet ((new-name (x) (list old-name x))) (new-name 1) old-name)\n",
    1
);

macrolet_write_case!(
    cli_writes_cl_macrolet_rename_without_touching_expander_or_noncall_values,
    "rename-cl-macrolet-write",
    "core.lisp",
    "(cl:macrolet ((old-name (x) (list old-name x))) (old-name 1) old-name)\n",
    "(cl:macrolet ((new-name (x) (list old-name x))) (new-name 1) old-name)\n",
    1
);
