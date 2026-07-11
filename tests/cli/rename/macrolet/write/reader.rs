use super::super::*;

macrolet_write_case!(
    cli_writes_macrolet_rename_inside_reader_quoted_lambda_bodies,
    "rename-macrolet-reader-quoted-lambda-write",
    "core.lisp",
    "(macrolet ((old-name (x) #'(lambda () (old-name x) old-name))) (old-name 1) old-name)\n",
    "(macrolet ((new-name (x) #'(lambda () (new-name x) new-name))) (new-name 1) old-name)\n",
    3
);

macrolet_write_case!(
    cli_writes_macrolet_rename_inside_reader_quoted_lambda_bodies_without_touching_function_designators,
    "rename-macrolet-reader-quoted-lambda-function-designator-write",
    "core.lisp",
    "(macrolet ((old-name (x) #'(lambda () (list #'old-name (function old-name) (old-name x) old-name)))) (old-name 1) old-name)\n",
    "(macrolet ((new-name (x) #'(lambda () (list #'old-name (function old-name) (new-name x) old-name)))) (new-name 1) old-name)\n",
    2
);

macrolet_write_case!(
    cli_writes_compiler_macrolet_rename_inside_reader_quoted_lambda_bodies_without_touching_function_designators,
    "rename-compiler-macrolet-reader-quoted-lambda-function-designator-write",
    "core.lisp",
    "(compiler-macrolet ((old-name (x) #'(lambda () (list #'old-name (function old-name) (old-name x) old-name)))) (old-name 1) old-name)\n",
    "(compiler-macrolet ((new-name (x) #'(lambda () (list #'old-name (function old-name) (new-name x) old-name)))) (new-name 1) old-name)\n",
    2
);

macrolet_write_case!(
    cli_writes_cl_qualified_compiler_macrolet_rename_inside_reader_quoted_lambda_bodies_without_touching_function_designators,
    "rename-cl-cmacrolet-reader-fn-write",
    "core.lisp",
    "(cl:compiler-macrolet ((old-name (x) #'(lambda () (list #'old-name (function old-name) (old-name x) old-name)))) (old-name 1) old-name)\n",
    "(cl:compiler-macrolet ((new-name (x) #'(lambda () (list #'old-name (function old-name) (new-name x) old-name)))) (new-name 1) old-name)\n",
    2
);

macrolet_write_case!(
    cli_writes_cl_user_qualified_compiler_macrolet_rename_inside_reader_quoted_lambda_bodies_without_touching_function_designators,
    "rename-cl-user-cmacrolet-reader-fn-write",
    "core.lisp",
    "(cl-user:compiler-macrolet ((old-name (x) #'(lambda () (list #'old-name (function old-name) (old-name x) old-name)))) (old-name 1) old-name)\n",
    "(cl-user:compiler-macrolet ((new-name (x) #'(lambda () (list #'old-name (function old-name) (new-name x) old-name)))) (new-name 1) old-name)\n",
    2
);

macrolet_write_case!(
    cli_writes_macrolet_rename_inside_quasiquote_with_unquote_prefixes,
    "rename-macrolet-quasiquote-write",
    "core.lisp",
    "(macrolet ((old-name (x) x)) `(list ,(old-name 1) ,@(old-name 2) (old-name 3)))\n",
    "(macrolet ((new-name (x) x)) `(list ,(new-name 1) ,@(new-name 2) (old-name 3)))\n",
    2
);

macrolet_write_case!(
    cli_skips_macrolet_definitions_and_calls_inside_quoted_data,
    "rename-macrolet-quoted-data-write",
    "core.lisp",
    "(macrolet ((old-name (x) x)) '(macrolet ((old-name (y) y)) (old-name 1)) `(progn (macrolet ((old-name (z) z)) (old-name 2)) ,(old-name 3)))\n",
    "(macrolet ((new-name (x) x)) '(macrolet ((old-name (y) y)) (old-name 1)) `(progn (macrolet ((old-name (z) z)) (old-name 2)) ,(new-name 3)))\n",
    1
);
