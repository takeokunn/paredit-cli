use super::assert_format_output;

#[test]
fn cli_formats_symbol_macrolet_indentation() {
    assert_format_output(
        "format-symbol-macrolet",
        "symbol-macrolet.lisp",
        "(symbol-macrolet ((value (compute value)) (used other)) (list value used))\n",
        "(symbol-macrolet ((value (compute value))\n                  (used other))\n  (list value used))\n",
    );
}

#[test]
fn cli_formats_macrolet_indentation() {
    assert_format_output(
        "format-macrolet",
        "macrolet.lisp",
        "(macrolet ((with-x (x) (list x outer))) (with-x 1) (with-x 2))\n",
        "(macrolet ((with-x (x)\n             (list x outer)))\n  (with-x 1)\n  (with-x 2))\n",
    );
}

#[test]
fn cli_formats_compiler_macrolet_indentation() {
    assert_format_output(
        "format-compiler-macrolet",
        "compiler-macrolet.lisp",
        "(compiler-macrolet ((with-x (x) (list x outer))) (with-x 1) (with-x 2))\n",
        "(compiler-macrolet ((with-x (x)\n                      (list x outer)))\n  (with-x 1)\n  (with-x 2))\n",
    );
}

#[test]
fn cli_formats_multiple_local_callable_bindings() {
    assert_format_output(
        "format-multiple-local-callables",
        "local-callables.lisp",
        "(macrolet ((with-a (x) (list x outer)) (with-b (y) (list y outer))) (with-a 1) (with-b 2))\n",
        "(macrolet ((with-a (x)\n             (list x outer))\n           (with-b (y)\n             (list y outer)))\n  (with-a 1)\n  (with-b 2))\n",
    );
}

#[test]
fn cli_formats_local_callable_bodies_on_dedicated_lines() {
    assert_format_output(
        "format-local-callable-bodies",
        "local-callable-bodies.lisp",
        "(labels ((parse (x) (validate x) (build x)) (emit (y) (write y) (finish))) (parse input) (emit output))\n",
        "(labels ((parse (x)\n           (validate x)\n           (build x))\n         (emit (y)\n           (write y)\n           (finish)))\n  (parse input)\n  (emit output))\n",
    );
}

#[test]
fn cli_formats_define_compiler_macro_indentation() {
    assert_format_output(
        "format-define-compiler-macro",
        "compiler-macro.lisp",
        "(define-compiler-macro fast-add (x y) (list '+ x y))\n",
        "(define-compiler-macro fast-add (x y)\n  (list '+ x y))\n",
    );
}

#[test]
fn cli_formats_define_setf_expander_indentation() {
    assert_format_output(
        "format-define-setf-expander",
        "setf-expander.lisp",
        "(define-setf-expander place (env) (values) (list place env))\n",
        "(define-setf-expander place (env)\n  (values)\n  (list place env))\n",
    );
}
