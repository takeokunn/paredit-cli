use super::*;

#[test]
fn cli_plans_introduce_let_for_common_lisp() {
    let mut cmd = paredit();
    cmd.args([
        "introduce-let",
        "--path",
        "0.3.1",
        "--name",
        "product",
        "--output",
        "json",
    ])
    .write_stdin("(defun render () (+ (* width height) margin))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"dialect\": \"unknown\""))
    .stdout(predicate::str::contains(
        "\"binding_value\": \"(* width height)\"",
    ))
    .stdout(predicate::str::contains(
        "\"replacement\": \"(let ((product (* width height))) (+ product margin))\"",
    ))
    .stdout(predicate::str::contains(
        "(defun render () (let ((product (* width height))) (+ product margin)))",
    ));
}

#[test]
fn cli_plans_introduce_let_for_all_equivalent_occurrences() {
    let mut cmd = paredit();
    cmd.args([
        "introduce-let",
        "--path",
        "0.3.1",
        "--name",
        "product",
        "--all-occurrences",
        "--output",
        "json",
    ])
    .write_stdin("(defun render () (+ (* width height) margin (*  width height)))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"occurrence_count\": 2"))
    .stdout(predicate::str::contains(
        "\"replacement\": \"(let ((product (* width height))) (+ product margin product))\"",
    ))
    .stdout(predicate::str::contains(
        "(defun render () (let ((product (* width height))) (+ product margin product)))",
    ));
}

#[test]
fn cli_plans_introduce_let_skips_shadowed_all_occurrences() {
    let mut cmd = paredit();
    cmd.args([
        "introduce-let",
        "--path",
        "0.3.1",
        "--name",
        "product",
        "--all-occurrences",
        "--output",
        "json",
    ])
    .write_stdin("(defun render () (+ (* width height) (let ((product 1)) (* width height))))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"occurrence_count\": 1"))
    .stdout(predicate::str::contains(
        "\"skipped_shadowed_occurrence_count\": 1",
    ))
    .stdout(predicate::str::contains(
        "(defun render () (let ((product (* width height))) (+ product (let ((product 1)) (* width height)))))",
    ));
}

#[test]
fn cli_plans_introduce_let_skips_symbol_macrolet_shadowed_all_occurrences() {
    let mut cmd = paredit();
    cmd.args([
        "introduce-let",
        "--path",
        "0.3.1",
        "--name",
        "product",
        "--all-occurrences",
        "--output",
        "json",
    ])
    .write_stdin(
        "(defun render () (+ (* width height) (symbol-macrolet ((product 1)) (* width height))))",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"occurrence_count\": 1"))
    .stdout(predicate::str::contains(
        "\"skipped_shadowed_occurrence_count\": 1",
    ))
    .stdout(predicate::str::contains(
        "(defun render () (let ((product (* width height))) (+ product (symbol-macrolet ((product 1)) (* width height)))))",
    ));
}

#[test]
fn cli_plans_introduce_let_keeps_let_star_same_initializer_outer_scope() {
    let mut cmd = paredit();
    cmd.args([
        "introduce-let",
        "--path",
        "0.3.1",
        "--name",
        "product",
        "--all-occurrences",
        "--output",
        "json",
    ])
    .write_stdin(
        "(defun render () (+ (* width height) (let* ((product (* width height))) (* width height))))",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"occurrence_count\": 2"))
    .stdout(predicate::str::contains(
        "\"skipped_shadowed_occurrence_count\": 1",
    ))
    .stdout(predicate::str::contains(
        "(defun render () (let ((product (* width height))) (+ product (let* ((product product)) (* width height)))))",
    ));
}

#[test]
fn cli_plans_introduce_let_skips_define_setf_expander_shadowed_all_occurrences() {
    let mut cmd = paredit();
    cmd.args([
        "introduce-let",
        "--path",
        "0.3.1",
        "--name",
        "product",
        "--all-occurrences",
        "--output",
        "json",
    ])
    .write_stdin(
        "(defun render () (+ (* width height) (define-setf-expander slot (&environment product place) (* width height))))",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"occurrence_count\": 1"))
    .stdout(predicate::str::contains(
        "\"skipped_shadowed_occurrence_count\": 1",
    ))
    .stdout(predicate::str::contains(
        "(defun render () (let ((product (* width height))) (+ product (define-setf-expander slot (&environment product place) (* width height)))))",
    ));
}

#[test]
fn cli_plans_introduce_let_skips_define_compiler_macro_shadowed_all_occurrences() {
    let mut cmd = paredit();
    cmd.args([
        "introduce-let",
        "--path",
        "0.3.1",
        "--name",
        "product",
        "--all-occurrences",
        "--output",
        "json",
    ])
    .write_stdin(
        "(defun render () (+ (* width height) (define-compiler-macro slot (&environment product place) (* width height))))",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"occurrence_count\": 1"))
    .stdout(predicate::str::contains(
        "\"skipped_shadowed_occurrence_count\": 1",
    ))
    .stdout(predicate::str::contains(
        "(defun render () (let ((product (* width height))) (+ product (define-compiler-macro slot (&environment product place) (* width height)))))",
    ));
}

#[test]
fn cli_plans_introduce_let_skips_handler_case_clause_shadow_only() {
    let mut cmd = paredit();
    cmd.args([
        "introduce-let",
        "--path",
        "0.3.1",
        "--name",
        "product",
        "--all-occurrences",
        "--output",
        "json",
    ])
    .write_stdin("(defun render () (+ (* width height) (handler-case (* width height) (error (product) (* width height)) (:no-error (value) (* width height)))))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"occurrence_count\": 3"))
    .stdout(predicate::str::contains(
        "\"skipped_shadowed_occurrence_count\": 1",
    ))
    .stdout(predicate::str::contains(
        "(defun render () (let ((product (* width height))) (+ product (handler-case product (error (product) (* width height)) (:no-error (value) product)))))",
    ));
}

#[test]
fn cli_plans_introduce_let_skips_macrolet_lambda_body_shadow_only() {
    let mut cmd = paredit();
    cmd.args([
        "introduce-let",
        "--path",
        "0.3.1",
        "--name",
        "product",
        "--all-occurrences",
        "--output",
        "json",
    ])
    .write_stdin("(defun render () (+ (* width height) (macrolet ((with-product (product) (* width height))) (* width height))))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"occurrence_count\": 2"))
    .stdout(predicate::str::contains(
        "\"skipped_shadowed_occurrence_count\": 1",
    ))
    .stdout(predicate::str::contains(
        "(defun render () (let ((product (* width height))) (+ product (macrolet ((with-product (product) (* width height))) product))))",
    ));
}

#[test]
fn cli_writes_introduce_let_for_emacs_lisp_file() {
    let dir = fresh_temp_dir("introduce-let");
    let elisp_file = dir.join("render.el");
    fs::write(
        &elisp_file,
        "(defun render () (+ (* width height) margin))\n",
    )
    .expect("write elisp fixture");

    let mut cmd = paredit();
    cmd.arg("introduce-let")
        .arg("--file")
        .arg(&elisp_file)
        .arg("--path")
        .arg("0.3.1")
        .arg("--name")
        .arg("product")
        .arg("--write")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"dialect\": \"emacs-lisp\""))
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(elisp_file).expect("read introduced let elisp"),
        "(defun render () (let ((product (* width height))) (+ product margin)))\n"
    );
}

#[test]
fn cli_writes_introduce_let_for_all_equivalent_occurrences() {
    let dir = fresh_temp_dir("introduce-let-all-occurrences");
    let lisp_file = dir.join("render.lisp");
    fs::write(
        &lisp_file,
        "(defun render () (+ (* width height) margin (*  width height)))\n",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("introduce-let")
        .arg("--file")
        .arg(&lisp_file)
        .arg("--path")
        .arg("0.3.1")
        .arg("--name")
        .arg("product")
        .arg("--all-occurrences")
        .arg("--write")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"occurrence_count\": 2"))
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(lisp_file).expect("read introduced let lisp"),
        "(defun render () (let ((product (* width height))) (+ product margin product)))\n"
    );
}

#[test]
fn cli_writes_introduce_let_without_shadowed_occurrence_capture() {
    let dir = fresh_temp_dir("introduce-let-shadowed-all-occurrences");
    let lisp_file = dir.join("render.lisp");
    fs::write(
        &lisp_file,
        "(defun render () (+ (* width height) (let ((product 1)) (* width height))))\n",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("introduce-let")
        .arg("--file")
        .arg(&lisp_file)
        .arg("--path")
        .arg("0.3.1")
        .arg("--name")
        .arg("product")
        .arg("--all-occurrences")
        .arg("--write")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"occurrence_count\": 1"))
        .stdout(predicate::str::contains(
            "\"skipped_shadowed_occurrence_count\": 1",
        ))
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(lisp_file).expect("read introduced let lisp"),
        "(defun render () (let ((product (* width height))) (+ product (let ((product 1)) (* width height)))))\n"
    );
}

#[test]
fn cli_rejects_introduce_let_write_without_file() {
    let mut cmd = paredit();
    cmd.args([
        "introduce-let",
        "--path",
        "0.3.1",
        "--name",
        "product",
        "--write",
    ])
    .write_stdin("(defun render () (+ (* width height) margin))")
    .assert()
    .failure()
    .stderr(predicate::str::contains("--write requires --file"));
}
