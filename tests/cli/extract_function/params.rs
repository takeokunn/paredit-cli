use super::*;

#[test]
fn cli_plans_parameterized_extract_function() {
    let mut cmd = paredit();
    cmd.args(["refactor", "extract-function",
        "--dialect",
        "common-lisp",
        "--path",
        "0.3",
        "--name",
        "area-with-margin",
        "--param",
        "width",
        "--param",
        "height",
        "--param",
        "margin",
        "--output",
        "json",
    ])
    .write_stdin("(defun render (width height margin) (+ (* width height) margin))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"params\": ["))
    .stdout(predicate::str::contains("\"width\""))
    .stdout(predicate::str::contains("\"height\""))
    .stdout(predicate::str::contains("\"margin\""))
    .stdout(predicate::str::contains(
        "\"call\": \"(area-with-margin width height margin)\"",
    ))
    .stdout(predicate::str::contains(
        "\"definition\": \"(defun area-with-margin (width height margin) (+ (* width height) margin))\"",
    ))
    .stdout(predicate::str::contains(
        "(defun render (width height margin) (area-with-margin width height margin))",
    ));
}

#[test]
fn cli_merges_explicit_and_inferred_extract_function_params() {
    let mut cmd = paredit();
    cmd.args([
        "refactor",
        "extract-function",
        "--dialect",
        "common-lisp",
        "--path",
        "0.3",
        "--name",
        "area-with-margin",
        "--param",
        "margin",
        "--infer-params",
        "--output",
        "json",
    ])
    .write_stdin("(defun render (width height margin) (+ (* width height) margin))")
    .assert()
    .success()
    .stdout(predicate::str::contains(
        "\"params\": [\n    \"margin\",\n    \"width\",\n    \"height\"\n  ]",
    ))
    .stdout(predicate::str::contains(
        "\"inferred_params\": [\n    \"width\",\n    \"height\"\n  ]",
    ))
    .stdout(predicate::str::contains(
        "\"call\": \"(area-with-margin margin width height)\"",
    ));
}
