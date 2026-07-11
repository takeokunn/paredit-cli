use super::*;

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
fn cli_reports_symbol_macrolet_bindings_without_counting_expansion_reference() {
    let mut cmd = paredit();
    cmd.args(["let-report", "--output", "json"])
        .write_stdin("(symbol-macrolet ((value (compute value)) (used other)) (list used))")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"form\": \"symbol-macrolet\""))
        .stdout(predicate::str::contains(
            "\"inline_supported_by_inline_let\": false",
        ))
        .stdout(predicate::str::contains("\"name\": \"value\""))
        .stdout(predicate::str::contains("\"reference_count\": 0"))
        .stdout(predicate::str::contains("\"name\": \"used\""))
        .stdout(predicate::str::contains("\"reference_count\": 1"))
        .stdout(predicate::str::contains("\"unused_binding_count\": 1"))
        .stdout(predicate::str::contains("\"unsupported-by-inline-let\""));
}

#[test]
fn cli_reports_single_binding_symbol_macrolet_supported_by_inline_let() {
    let mut cmd = paredit();
    cmd.args(["let-report", "--output", "json"])
        .write_stdin("(symbol-macrolet ((used other)) (list used))")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"form\": \"symbol-macrolet\""))
        .stdout(predicate::str::contains(
            "\"inline_supported_by_inline_let\": true",
        ))
        .stdout(predicate::str::contains("\"name\": \"used\""))
        .stdout(predicate::str::contains("\"reference_count\": 1"))
        .stdout(predicate::str::contains(
            "\"can_inline_without_duplication\": true",
        ));
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
fn cli_reports_bare_symbol_let_binding_as_implicit_nil() {
    let mut cmd = paredit();
    cmd.args(["let-report", "--output", "json"])
        .write_stdin("(defun f () (let ((opoint (point)) beg end) (setq beg 1) (setq end 2) (list opoint beg end)))")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"let_form_count\": 1"))
        .stdout(predicate::str::contains("\"name\": \"beg\""))
        .stdout(predicate::str::contains("\"name\": \"end\""))
        .stdout(predicate::str::contains("\"value\": \"nil\""));
}

#[test]
fn cli_reports_let_star_later_bare_symbol_binding_without_erroring() {
    let mut cmd = paredit();
    cmd.args(["let-report", "--output", "json"])
        .write_stdin("(let* ((x 1) y) (+ x (or y 0)))")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"form\": \"let*\""))
        .stdout(predicate::str::contains("\"name\": \"x\""))
        .stdout(predicate::str::contains("\"name\": \"y\""))
        .stdout(predicate::str::contains("\"value\": \"nil\""));
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
