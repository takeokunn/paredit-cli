use super::*;

#[test]
fn cli_plans_extract_function_for_common_lisp() {
    let mut cmd = paredit();
    cmd.args([
        "refactor",
        "extract-function",
        "--dialect",
        "common-lisp",
        "--path",
        "0.3",
        "--name",
        "compute-sum",
        "--output",
        "json",
    ])
    .write_stdin("(defun render () (+ 1 2))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"dialect\": \"common-lisp\""))
    .stdout(predicate::str::contains("\"call\": \"(compute-sum)\""))
    .stdout(predicate::str::contains(
        "\"definition\": \"(defun compute-sum () (+ 1 2))\"",
    ))
    .stdout(predicate::str::contains(
        "(defun render () (compute-sum))\\n\\n(defun compute-sum () (+ 1 2))",
    ));
}

#[test]
fn cli_plans_extract_function_for_common_lisp_macrolet_body() {
    let mut cmd = paredit();
    cmd.args(["refactor", "extract-function",
        "--dialect",
        "common-lisp",
        "--path",
        "0.3",
        "--name",
        "build",
        "--infer-params",
        "--output",
        "json",
    ])
    .write_stdin(
        "(defun render (outer input) (macrolet ((with-local (local) (list local outer))) (with-local input)))",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"call\": \"(build outer input)\""))
    .stdout(predicate::str::contains(
        "\"definition\": \"(defun build (outer input) (macrolet ((with-local (local) (list local outer))) (with-local input)))\"",
    ))
    .stdout(predicate::str::contains(
        "(defun render (outer input) (build outer input))\\n\\n(defun build (outer input) (macrolet ((with-local (local) (list local outer))) (with-local input)))",
    ));
}

#[test]
fn cli_plans_extract_function_for_common_lisp_symbol_macrolet_body() {
    let mut cmd = paredit();
    cmd.args(["refactor", "extract-function",
        "--dialect",
        "common-lisp",
        "--path",
        "0.3",
        "--name",
        "build",
        "--infer-params",
        "--output",
        "json",
    ])
    .write_stdin(
        "(defun render (outer) (symbol-macrolet ((local (compute outer))) (list local outer)))",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"call\": \"(build outer)\""))
    .stdout(predicate::str::contains(
        "\"definition\": \"(defun build (outer) (symbol-macrolet ((local (compute outer))) (list local outer)))\"",
    ))
    .stdout(predicate::str::contains(
        "(defun render (outer) (build outer))\\n\\n(defun build (outer) (symbol-macrolet ((local (compute outer))) (list local outer)))",
    ));
}
