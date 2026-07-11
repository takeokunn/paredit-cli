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
    cmd.arg("symbol-report")
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
    cmd.arg("symbol-report")
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
