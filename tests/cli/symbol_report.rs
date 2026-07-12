use super::*;

#[test]
fn cli_reports_multi_file_symbol_occurrences_with_outline_context() {
    let dir = fresh_temp_dir("symbol-report");
    let lisp_file = dir.join("core.lisp");
    let elisp_file = dir.join("init.el");
    fs::write(
        &lisp_file,
        "(defun target () target)\n(defun other () (let ((target 1)) target))",
    )
    .expect("write lisp fixture");
    fs::write(
        &elisp_file,
        "(defun target () (message \"target\") target) ; target",
    )
    .expect("write elisp fixture");

    let mut cmd = paredit();
    cmd.arg("inspect")
        .arg("symbols")
        .arg("--symbol")
        .arg("target")
        .arg(&lisp_file)
        .arg(&elisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"symbol\": \"target\""))
        .stdout(predicate::str::contains("\"file_count\": 2"))
        .stdout(predicate::str::contains("\"total_count\": 6"))
        .stdout(predicate::str::contains("\"dialect\": \"common-lisp\""))
        .stdout(predicate::str::contains("\"dialect\": \"emacs-lisp\""))
        .stdout(predicate::str::contains("\"head\": \"defun\""))
        .stdout(predicate::str::contains("\"definitionLike\": true"))
        .stdout(predicate::str::contains("\"count\": 4"))
        .stdout(predicate::str::contains("\"count\": 2"));
}

#[test]
fn cli_reports_unqualified_query_for_package_qualified_common_lisp_symbol() {
    let dir = fresh_temp_dir("symbol-report-qualified-common-lisp");
    let lisp_file = dir.join("core.lisp");
    fs::write(
        &lisp_file,
        "(defun cl-user:target () target)\n(target cl-user:target)",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("inspect")
        .arg("symbols")
        .arg("--symbol")
        .arg("target")
        .arg(&lisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"symbol\": \"target\""))
        .stdout(predicate::str::contains("\"file_count\": 1"))
        .stdout(predicate::str::contains("\"total_count\": 4"))
        .stdout(predicate::str::contains("\"dialect\": \"common-lisp\""));
}

#[test]
fn cli_find_symbol_require_occurrences_gate_fails_on_undermatch() {
    let mut cmd = paredit();
    cmd.args([
        "inspect",
        "find-symbol",
        "--symbol",
        "foo",
        "--require-occurrences",
        "3",
        "--output",
        "json",
    ])
    .write_stdin("(foo 1)\n(bar (foo 2))\n")
    .assert()
    .failure()
    .stdout(predicate::str::contains("\"symbol\": \"foo\""))
    .stderr(predicate::str::contains(
        "require-occurrences policy failed",
    ));
}

#[test]
fn cli_symbols_require_occurrences_gate_passes_when_met() {
    let dir = fresh_temp_dir("symbols-require-occurrences");
    let file = dir.join("source.lisp");
    fs::write(&file, "(foo 1)\n(bar (foo 2))\n").expect("write source fixture");

    let mut cmd = paredit();
    cmd.args([
        "inspect",
        "symbols",
        "--symbol",
        "foo",
        "--require-occurrences",
        "2",
    ])
    .arg(&file)
    .assert()
    .success()
    .stdout(predicate::str::contains("\"total_count\": 2"));
}
