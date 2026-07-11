use super::super::*;

macrolet_write_case!(
    cli_writes_emacs_lisp_cl_macrolet_rename_without_crossing_cl_labels_shadow,
    "rename-cl-macrolet-labels-shadow-write",
    "core.el",
    "(cl-macrolet ((old-name (x) x)) (cl-labels ((old-name (y) (old-name y))) (old-name 1)) (old-name 2))\n",
    "(cl-macrolet ((new-name (x) x)) (cl-labels ((old-name (y) (old-name y))) (old-name 1)) (new-name 2))\n",
    1
);

macrolet_write_case!(
    cli_writes_macrolet_rename_without_crossing_qualified_symbol_macrolet_shadow,
    "rename-qualified-symbol-macrolet-shadow-write",
    "core.lisp",
    "(cl:macrolet ((old-name (x) x)) (cl:symbol-macrolet ((old-name other)) old-name) (cl-user:symbol-macrolet ((old-name other)) old-name) (old-name 2))\n",
    "(cl:macrolet ((new-name (x) x)) (cl:symbol-macrolet ((old-name other)) old-name) (cl-user:symbol-macrolet ((old-name other)) old-name) (new-name 2))\n",
    1
);

macrolet_write_case!(
    cli_writes_macrolet_rename_across_same_name_nested_macrolet_expander_body,
    "rename-macrolet-nested-shadow-write",
    "core.lisp",
    "(macrolet ((old-name (x) x)) (macrolet ((old-name (y) (old-name y))) (old-name 1)) (old-name 2))\n",
    "(macrolet ((new-name (x) x)) (macrolet ((old-name (y) (new-name y))) (old-name 1)) (new-name 2))\n",
    2
);

macrolet_write_case!(
    cli_writes_compiler_macrolet_rename_across_same_name_nested_compiler_macrolet_expander_body,
    "rename-compiler-macrolet-nested-shadow-write",
    "core.lisp",
    "(compiler-macrolet ((old-name (x) x)) (compiler-macrolet ((old-name (y) (old-name y))) (old-name 1)) (old-name 2))\n",
    "(compiler-macrolet ((new-name (x) x)) (compiler-macrolet ((old-name (y) (new-name y))) (old-name 1)) (new-name 2))\n",
    2
);

macrolet_write_case!(
    cli_writes_macrolet_rename_without_crossing_labels_function_shadow,
    "rename-macrolet-labels-shadow-write",
    "core.lisp",
    "(macrolet ((old-name (x) x)) (labels ((old-name (y) (old-name y))) (old-name 1)) (old-name 2))\n",
    "(macrolet ((new-name (x) x)) (labels ((old-name (y) (old-name y))) (old-name 1)) (new-name 2))\n",
    1
);

macrolet_write_case!(
    cli_writes_macrolet_rename_without_crossing_flet_function_shadow,
    "rename-macrolet-flet-shadow-write",
    "core.lisp",
    "(macrolet ((old-name (x) x)) (flet ((old-name (y) y)) (old-name 1)) (old-name 2))\n",
    "(macrolet ((new-name (x) x)) (flet ((old-name (y) y)) (old-name 1)) (new-name 2))\n",
    1
);

macrolet_write_case!(
    cli_writes_qualified_macrolet_rename_without_crossing_qualified_labels_shadow,
    "rename-qualified-macrolet-labels-shadow-write",
    "core.lisp",
    "(cl:macrolet ((old-name (x) x)) (cl:labels ((old-name (y) (old-name y))) (old-name 1)) (old-name 2))\n",
    "(cl:macrolet ((new-name (x) x)) (cl:labels ((old-name (y) (old-name y))) (old-name 1)) (new-name 2))\n",
    1
);

macrolet_write_case!(
    cli_writes_cl_user_qualified_macrolet_rename_without_crossing_cl_user_qualified_labels_shadow,
    "rename-cl-user-qualified-macrolet-labels-shadow-write",
    "core.lisp",
    "(cl-user:macrolet ((old-name (x) x)) (cl-user:labels ((old-name (y) (old-name y))) (old-name 1)) (old-name 2))\n",
    "(cl-user:macrolet ((new-name (x) x)) (cl-user:labels ((old-name (y) (old-name y))) (old-name 1)) (new-name 2))\n",
    1
);

macrolet_write_case!(
    cli_writes_macrolet_rename_inside_nested_expander_body,
    "rename-macrolet-nested-expander-body-write",
    "core.lisp",
    "(macrolet ((old-name (x) x)) (macrolet ((helper (y) (old-name y))) (helper 1)) (old-name 2))\n",
    "(macrolet ((new-name (x) x)) (macrolet ((helper (y) (new-name y))) (helper 1)) (new-name 2))\n",
    2
);

macrolet_write_case!(
    cli_writes_independent_macrolet_rename_inside_expander_body,
    "rename-macrolet-inside-expander-body-write",
    "core.lisp",
    "(macrolet ((outer (x) (macrolet ((old-name (y) y)) (old-name x)))) (outer 1))\n",
    "(macrolet ((outer (x) (macrolet ((new-name (y) y)) (new-name x)))) (outer 1))\n",
    1
);
