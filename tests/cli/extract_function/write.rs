use super::*;

#[test]
fn cli_writes_extract_function_for_emacs_lisp_file() {
    let dir = fresh_temp_dir("extract");
    let elisp_file = dir.join("render.el");
    fs::write(&elisp_file, "(defun render () (+ 1 2))\n").expect("write elisp fixture");

    let mut cmd = paredit();
    cmd.arg("refactor")
        .arg("extract-function")
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
fn cli_writes_parameterized_extract_function_before_anchor() {
    let dir = fresh_temp_dir("extract-before-anchor");
    let lisp_file = dir.join("render.lisp");
    fs::write(
        &lisp_file,
        "(in-package #:demo)\n(defun render (width height margin)\n  (+ (* width height) margin))\n(defun boot () :boot)\n",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("refactor")
        .arg("extract-function")
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
fn cli_writes_extract_function_for_common_lisp_macrolet_body() {
    let dir = fresh_temp_dir("extract-macrolet");
    let lisp_file = dir.join("render.lisp");
    fs::write(
        &lisp_file,
        "(defun render (outer input) (macrolet ((with-local (local) (list local outer))) (with-local input)))\n",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("refactor")
        .arg("extract-function")
        .arg("--file")
        .arg(&lisp_file)
        .arg("--path")
        .arg("0.3")
        .arg("--name")
        .arg("build")
        .arg("--infer-params")
        .arg("--write")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(lisp_file).expect("read extracted lisp"),
        "(defun render (outer input) (build outer input))\n\n(defun build (outer input) (macrolet ((with-local (local) (list local outer))) (with-local input)))\n"
    );
}

#[test]
fn cli_writes_extract_function_for_common_lisp_symbol_macrolet_body() {
    let dir = fresh_temp_dir("extract-symbol-macrolet");
    let lisp_file = dir.join("render.lisp");
    fs::write(
        &lisp_file,
        "(defun render (outer) (symbol-macrolet ((local (compute outer))) (list local outer)))\n",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("refactor")
        .arg("extract-function")
        .arg("--file")
        .arg(&lisp_file)
        .arg("--path")
        .arg("0.3")
        .arg("--name")
        .arg("build")
        .arg("--infer-params")
        .arg("--write")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(lisp_file).expect("read extracted lisp"),
        "(defun render (outer) (build outer))\n\n(defun build (outer) (symbol-macrolet ((local (compute outer))) (list local outer)))\n"
    );
}
