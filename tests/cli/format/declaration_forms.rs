use super::assert_format_output;

#[test]
fn cli_formats_declarations_indentation() {
    assert_format_output(
        "format-declarations",
        "declarations.lisp",
        "(locally (declare (optimize speed)) (declaim (inline f)) (proclaim (special x)) (f))\n",
        "(locally\n  (declare (optimize speed))\n  (declaim (inline f))\n  (proclaim (special x))\n  (f))\n",
    );
}

#[test]
fn cli_formats_multiple_declaration_specs() {
    assert_format_output(
        "format-multiple-declarations",
        "declarations.lisp",
        "(declare (optimize speed) (type fixnum index) (ignorable scratch))\n",
        "(declare (optimize speed)\n         (type fixnum index)\n         (ignorable scratch))\n",
    );
}
