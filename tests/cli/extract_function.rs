use super::*;

#[test]
fn cli_plans_extract_function_for_common_lisp() {
    let mut cmd = paredit();
    cmd.args([
        "extract-function",
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
    .stdout(predicate::str::contains("\"dialect\": \"unknown\""))
    .stdout(predicate::str::contains("\"call\": \"(compute-sum)\""))
    .stdout(predicate::str::contains(
        "\"definition\": \"(defun compute-sum () (+ 1 2))\"",
    ))
    .stdout(predicate::str::contains(
        "(defun render () (compute-sum))\\n\\n(defun compute-sum () (+ 1 2))",
    ));
}

#[test]
fn cli_writes_extract_function_for_emacs_lisp_file() {
    let dir = fresh_temp_dir("extract");
    let elisp_file = dir.join("render.el");
    fs::write(&elisp_file, "(defun render () (+ 1 2))\n").expect("write elisp fixture");

    let mut cmd = paredit();
    cmd.arg("extract-function")
        .arg("--file")
        .arg(&elisp_file)
        .arg("--path")
        .arg("0.3")
        .arg("--name")
        .arg("render-sum")
        .arg("--write")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"dialect\": \"emacs-lisp\""))
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(elisp_file).expect("read extracted elisp"),
        "(defun render () (render-sum))\n\n(defun render-sum () (+ 1 2))\n"
    );
}

#[test]
fn cli_plans_parameterized_extract_function() {
    let mut cmd = paredit();
    cmd.args([
        "extract-function",
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

#[test]
fn cli_merges_explicit_and_inferred_extract_function_params() {
    let mut cmd = paredit();
    cmd.args([
        "extract-function",
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

#[test]
fn cli_writes_parameterized_extract_function_before_anchor() {
    let dir = fresh_temp_dir("extract-before-anchor");
    let lisp_file = dir.join("render.lisp");
    fs::write(
        &lisp_file,
        "(in-package #:demo)\n(defun render (width height margin)\n  (+ (* width height) margin))\n(defun boot () :boot)\n",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("extract-function")
        .arg("--file")
        .arg(&lisp_file)
        .arg("--path")
        .arg("1.3")
        .arg("--name")
        .arg("area-with-margin")
        .arg("--param")
        .arg("width")
        .arg("--param")
        .arg("height")
        .arg("--param")
        .arg("margin")
        .arg("--insert")
        .arg("before")
        .arg("--anchor-path")
        .arg("2")
        .arg("--write")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"insert\": \"before\""))
        .stdout(predicate::str::contains("\"anchor_path\": \"2\""))
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(lisp_file).expect("read extracted lisp"),
        "(in-package #:demo)\n(defun render (width height margin)\n  (area-with-margin width height margin))\n(defun area-with-margin (width height margin) (+ (* width height) margin))\n\n(defun boot () :boot)\n"
    );
}

#[test]
fn cli_rejects_extract_function_write_without_file() {
    let mut cmd = paredit();
    cmd.args([
        "extract-function",
        "--path",
        "0.3",
        "--name",
        "compute-sum",
        "--write",
    ])
    .write_stdin("(defun render () (+ 1 2))")
    .assert()
    .failure()
    .stderr(predicate::str::contains("--write requires --file"));
}
