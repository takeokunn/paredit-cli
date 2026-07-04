use super::*;

#[test]
fn cli_infers_extract_function_params() {
    let mut cmd = paredit();
    cmd.args([
        "extract-function",
        "--path",
        "0.3",
        "--name",
        "area-with-margin",
        "--infer-params",
        "--output",
        "json",
    ])
    .write_stdin("(defun render (width height margin) (+ (* width height) margin))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"inferred_params\": ["))
    .stdout(predicate::str::contains("\"width\""))
    .stdout(predicate::str::contains("\"height\""))
    .stdout(predicate::str::contains("\"margin\""))
    .stdout(predicate::str::contains(
        "\"call\": \"(area-with-margin width height margin)\"",
    ))
    .stdout(predicate::str::contains(
        "\"definition\": \"(defun area-with-margin (width height margin) (+ (* width height) margin))\"",
    ));
}

#[test]
fn cli_infers_extract_function_params_without_call_heads_or_literals() {
    let mut cmd = paredit();
    cmd.args([
        "extract-function",
        "--path",
        "0.3",
        "--name",
        "measure",
        "--infer-params",
        "--output",
        "json",
    ])
    .write_stdin("(defun render (width) (+ width 1 :px nil))")
    .assert()
    .success()
    .stdout(predicate::str::contains(
        "\"params\": [\n    \"width\"\n  ]",
    ))
    .stdout(predicate::str::contains(
        "\"inferred_params\": [\n    \"width\"\n  ]",
    ))
    .stdout(predicate::str::contains("\"call\": \"(measure width)\""))
    .stdout(predicate::str::contains(
        "\"definition\": \"(defun measure (width) (+ width 1 :px nil))\"",
    ));
}

#[test]
fn cli_infers_extract_function_params_without_local_let_bindings() {
    let mut cmd = paredit();
    cmd.args([
        "extract-function",
        "--path",
        "0.3",
        "--name",
        "compute",
        "--infer-params",
        "--output",
        "json",
    ])
    .write_stdin("(defun render (y z) (let ((x y)) (+ x z)))")
    .assert()
    .success()
    .stdout(predicate::str::contains(
        "\"params\": [\n    \"y\",\n    \"z\"\n  ]",
    ))
    .stdout(predicate::str::contains(
        "\"inferred_params\": [\n    \"y\",\n    \"z\"\n  ]",
    ))
    .stdout(predicate::str::contains("\"call\": \"(compute y z)\""))
    .stdout(predicate::str::contains(
        "\"definition\": \"(defun compute (y z) (let ((x y)) (+ x z)))\"",
    ));
}

#[test]
fn cli_infers_extract_function_params_without_sequential_let_bindings() {
    let mut cmd = paredit();
    cmd.args([
        "extract-function",
        "--path",
        "0.3",
        "--name",
        "compute",
        "--infer-params",
        "--output",
        "json",
    ])
    .write_stdin("(defun render (y) (let* ((x y) (z (+ x 1))) (+ z y)))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"params\": [\n    \"y\"\n  ]"))
    .stdout(predicate::str::contains(
        "\"inferred_params\": [\n    \"y\"\n  ]",
    ))
    .stdout(predicate::str::contains("\"call\": \"(compute y)\""))
    .stdout(predicate::str::contains(
        "\"definition\": \"(defun compute (y) (let* ((x y) (z (+ x 1))) (+ z y)))\"",
    ));
}

#[test]
fn cli_infers_extract_function_params_without_clojure_destructuring_let_bindings() {
    let mut cmd = paredit();
    cmd.args([
        "extract-function",
        "--dialect",
        "clojure",
        "--path",
        "0.3",
        "--name",
        "compute",
        "--infer-params",
        "--output",
        "json",
    ])
    .write_stdin("(defn render [point scale] (let [[x y] point total (+ x y)] (* total scale)))")
    .assert()
    .success()
    .stdout(predicate::str::contains(
        "\"params\": [\n    \"point\",\n    \"scale\"\n  ]",
    ))
    .stdout(predicate::str::contains(
        "\"inferred_params\": [\n    \"point\",\n    \"scale\"\n  ]",
    ))
    .stdout(predicate::str::contains("\"call\": \"(compute point scale)\""))
    .stdout(predicate::str::contains(
        "\"definition\": \"(defn compute [point scale] (let [[x y] point total (+ x y)] (* total scale)))\"",
    ));
}

#[test]
fn cli_infers_extract_function_params_without_clojure_destructuring_lambda_bindings() {
    let mut cmd = paredit();
    cmd.args([
        "extract-function",
        "--dialect",
        "clojure",
        "--path",
        "0.3",
        "--name",
        "compute",
        "--infer-params",
        "--output",
        "json",
    ])
    .write_stdin("(defn render [scale] (fn [[x y]] (+ x y scale)))")
    .assert()
    .success()
    .stdout(predicate::str::contains(
        "\"params\": [\n    \"scale\"\n  ]",
    ))
    .stdout(predicate::str::contains(
        "\"inferred_params\": [\n    \"scale\"\n  ]",
    ))
    .stdout(predicate::str::contains("\"call\": \"(compute scale)\""))
    .stdout(predicate::str::contains(
        "\"definition\": \"(defn compute [scale] (fn [[x y]] (+ x y scale)))\"",
    ));
}
