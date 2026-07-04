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

#[test]
fn cli_plans_inline_let_for_common_lisp() {
    let mut cmd = paredit();
    cmd.args(["inline-let", "--path", "0.3", "--output", "json"])
        .write_stdin("(defun render () (let ((product (* width height))) (+ product margin)))")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"dialect\": \"unknown\""))
        .stdout(predicate::str::contains("\"binding_name\": \"product\""))
        .stdout(predicate::str::contains(
            "\"binding_value\": \"(* width height)\"",
        ))
        .stdout(predicate::str::contains("\"reference_count\": 1"))
        .stdout(predicate::str::contains(
            "(defun render () (+ (* width height) margin))",
        ));
}

#[test]
fn cli_plans_inline_let_with_multiple_body_expressions() {
    let mut cmd = paredit();
    cmd.args([
        "inline-let",
        "--path",
        "0.3",
        "--allow-duplicate-evaluation",
        "--output",
        "json",
    ])
    .write_stdin(
        "(defun render () (let ((product (* width height))) (log product) (+ product margin)))",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"body_count\": 2"))
    .stdout(predicate::str::contains("\"reference_count\": 2"))
    .stdout(predicate::str::contains(
        "(defun render () (log (* width height)) (+ (* width height) margin))",
    ));
}

#[test]
fn cli_reports_let_inline_safety_for_common_lisp() {
    let mut cmd = paredit();
    cmd.args(["let-report", "--output", "json"])
        .write_stdin("(defun render () (let ((product (* width height))) (+ product margin)))")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"let_form_count\": 1"))
        .stdout(predicate::str::contains("\"path\": \"0.3\""))
        .stdout(predicate::str::contains("\"binding_style\": \"list-pair\""))
        .stdout(predicate::str::contains("\"name\": \"product\""))
        .stdout(predicate::str::contains("\"reference_count\": 1"))
        .stdout(predicate::str::contains(
            "\"can_inline_without_duplication\": true",
        ));
}

#[test]
fn cli_reports_multi_body_let_supported_by_inline_let() {
    let mut cmd = paredit();
    cmd.args(["let-report", "--output", "json"])
        .write_stdin("(let ((x 1)) (print x) (+ x 2))")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"body_count\": 2"))
        .stdout(predicate::str::contains(
            "\"inline_supported_by_inline_let\": true",
        ))
        .stdout(predicate::str::contains("\"reference_count\": 2"))
        .stdout(predicate::str::contains("\"duplicate-evaluation\""));
}

#[test]
fn cli_reports_let_duplicate_evaluation_risk() {
    let mut cmd = paredit();
    cmd.args(["let-report", "--output", "json"])
        .write_stdin("(let ((x (compute))) (+ x x))")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"let_form_count\": 1"))
        .stdout(predicate::str::contains("\"name\": \"x\""))
        .stdout(predicate::str::contains("\"reference_count\": 2"))
        .stdout(predicate::str::contains("\"duplicate-evaluation\""))
        .stdout(predicate::str::contains(
            "\"can_inline_without_duplication\": false",
        ));
}

#[test]
fn cli_reports_let_star_references_from_later_bindings() {
    let mut cmd = paredit();
    cmd.args(["let-report", "--output", "json"])
        .write_stdin("(let* ((x 1) (y (+ x 2))) y)")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"form\": \"let*\""))
        .stdout(predicate::str::contains("\"name\": \"x\""))
        .stdout(predicate::str::contains("\"reference_count\": 1"))
        .stdout(predicate::str::contains("\"unused_binding_count\": 0"));
}

#[test]
fn cli_reports_clojure_vector_let_bindings() {
    let mut cmd = paredit();
    cmd.args(["let-report", "--dialect", "clojure", "--output", "json"])
        .write_stdin("(let [x 1 y (+ x 2)] (+ x y))")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"dialect\": \"clojure\""))
        .stdout(predicate::str::contains("\"binding_style\": \"vector\""))
        .stdout(predicate::str::contains("\"name\": \"x\""))
        .stdout(predicate::str::contains("\"name\": \"y\""))
        .stdout(predicate::str::contains("\"multiple-bindings\""));
}

#[test]
fn cli_reports_clojure_vector_let_references_from_later_bindings() {
    let mut cmd = paredit();
    cmd.args(["let-report", "--dialect", "clojure", "--output", "json"])
        .write_stdin("(let [x 1 y (+ x 2)] y)")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"dialect\": \"clojure\""))
        .stdout(predicate::str::contains("\"binding_style\": \"vector\""))
        .stdout(predicate::str::contains("\"name\": \"x\""))
        .stdout(predicate::str::contains("\"reference_count\": 1"))
        .stdout(predicate::str::contains("\"unused_binding_count\": 0"));
}

#[test]
fn cli_fails_let_report_policy_after_printing_json() {
    let mut cmd = paredit();
    cmd.args([
        "let-report",
        "--fail-on-duplicate-evaluation",
        "--fail-on-unused-binding",
        "--require-inlineable-bindings",
        "2",
        "--output",
        "json",
    ])
    .write_stdin("(let ((x (compute)) (unused 1)) (+ x x))")
    .assert()
    .failure()
    .stdout(predicate::str::contains("\"policy\""))
    .stdout(predicate::str::contains("\"passed\": false"))
    .stdout(predicate::str::contains(
        "\"duplicate_evaluation_count\": 1",
    ))
    .stdout(predicate::str::contains("\"unused_binding_count\": 1"))
    .stderr(predicate::str::contains("let-report policy failed"));
}

#[test]
fn cli_writes_inline_let_for_emacs_lisp_file() {
    let dir = fresh_temp_dir("inline-let");
    let elisp_file = dir.join("render.el");
    fs::write(
        &elisp_file,
        "(defun render () (let ((product (* width height))) (+ product margin)))\n",
    )
    .expect("write elisp fixture");

    let mut cmd = paredit();
    cmd.arg("inline-let")
        .arg("--file")
        .arg(&elisp_file)
        .arg("--path")
        .arg("0.3")
        .arg("--write")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"dialect\": \"emacs-lisp\""))
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(elisp_file).expect("read inline let elisp"),
        "(defun render () (+ (* width height) margin))\n"
    );
}

#[test]
fn cli_rejects_inline_let_duplicate_evaluation_by_default() {
    let mut cmd = paredit();
    cmd.args(["inline-let", "--path", "0"])
        .write_stdin("(let ((x (compute))) (+ x x))")
        .assert()
        .failure()
        .stderr(predicate::str::contains("would duplicate"));
}

#[test]
fn cli_allows_inline_let_duplicate_evaluation_when_explicit() {
    let mut cmd = paredit();
    cmd.args([
        "inline-let",
        "--path",
        "0",
        "--allow-duplicate-evaluation",
        "--output",
        "json",
    ])
    .write_stdin("(let ((x (compute))) (+ x x))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"reference_count\": 2"))
    .stdout(predicate::str::contains("(+ (compute) (compute))"));
}

#[test]
fn cli_plans_inline_let_for_clojure_vector_binding() {
    let mut cmd = paredit();
    cmd.args([
        "inline-let",
        "--dialect",
        "clojure",
        "--path",
        "0",
        "--output",
        "json",
    ])
    .write_stdin("(let [product (* width height)] (+ product margin))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"dialect\": \"clojure\""))
    .stdout(predicate::str::contains("\"binding_name\": \"product\""))
    .stdout(predicate::str::contains("(+ (* width height) margin)"));
}

#[test]
fn cli_plans_inline_let_without_touching_shadowed_lambda_parameter() {
    let mut cmd = paredit();
    cmd.args(["inline-let", "--path", "0", "--output", "json"])
        .write_stdin("(let ((x 1)) (list x (lambda (x) x)))")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"binding_name\": \"x\""))
        .stdout(predicate::str::contains("\"reference_count\": 1"))
        .stdout(predicate::str::contains("(list 1 (lambda (x) x))"));
}
