pub(super) use super::*;

#[test]
fn cli_rejects_remove_common_lisp_parameter_after_allow_other_keys() {
    remove_command()
        .args([
            "remove-function-parameter",
            "--dialect",
            "common-lisp",
            "--definition-path",
            "0",
            "--name",
            "c",
            "--call-path",
            "1.1",
        ])
        .write_stdin(
            "(defun f (a &key b &allow-other-keys c) (list a b c))\n(print (f 1 :b 20 :c 30))",
        )
        .assert()
        .failure()
        .stderr(predicate::str::contains("after &allow-other-keys"));
}

#[test]
fn cli_rejects_remove_function_parameter_missing_argument() {
    remove_command()
        .args([
            "remove-function-parameter",
            "--definition-path",
            "0",
            "--name",
            "margin",
            "--call-path",
            "1.3",
        ])
        .write_stdin(
            "(defun area (width height margin) (* width height))\n(defun render () (area 10 20))",
        )
        .assert()
        .failure()
        .stderr(predicate::str::contains("does not have argument"));
}
