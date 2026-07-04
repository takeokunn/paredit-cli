use super::*;

#[test]
fn cli_reports_signature_arity_across_dialects() {
    let dir = fresh_temp_dir("signature-report");
    let lisp_file = dir.join("core.lisp");
    let elisp_file = dir.join("init.el");
    fs::write(
        &lisp_file,
        "(defun area (width height) (* width height))\n(defun render () (area 10 20))\n(defun too-short () (area 10))\n(defun too-long () (area 10 20 30))\n",
    )
    .expect("write lisp fixture");
    fs::write(&elisp_file, "(defun draw () (area 5 6))\n").expect("write elisp fixture");

    let mut cmd = paredit();
    cmd.arg("signature-report")
        .arg("--symbol")
        .arg("area")
        .arg(&lisp_file)
        .arg(&elisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"symbol\": \"area\""))
        .stdout(predicate::str::contains("\"definition_count\": 1"))
        .stdout(predicate::str::contains("\"call_count\": 4"))
        .stdout(predicate::str::contains("\"dialect\": \"common-lisp\""))
        .stdout(predicate::str::contains("\"dialect\": \"emacs-lisp\""))
        .stdout(predicate::str::contains("\"parameterCount\": 2"))
        .stdout(predicate::str::contains("\"expectedParameterCount\": 2"))
        .stdout(predicate::str::contains("\"status\": \"exact\""))
        .stdout(predicate::str::contains(
            "\"status\": \"missing-arguments\"",
        ))
        .stdout(predicate::str::contains("\"status\": \"extra-arguments\""));
}

#[test]
fn cli_gates_signature_report_policy_for_ci() {
    let dir = fresh_temp_dir("signature-report-policy");
    let lisp_file = dir.join("core.lisp");
    fs::write(
        &lisp_file,
        "(defun render (pane theme) pane)\n(defun draw () (render pane))\n",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("signature-report")
        .arg("--symbol")
        .arg("render")
        .arg("--fail-on-mismatch")
        .arg("--require-definitions")
        .arg("2")
        .arg("--require-calls")
        .arg("2")
        .arg("--output")
        .arg("json")
        .arg(&lisp_file)
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"definition_count\": 1"))
        .stdout(predicate::str::contains("\"call_count\": 1"))
        .stdout(predicate::str::contains("\"mismatch_count\": 1"))
        .stdout(predicate::str::contains("\"fail_on_mismatch\": true"))
        .stdout(predicate::str::contains("\"require_definitions\": 2"))
        .stdout(predicate::str::contains("\"require_calls\": 2"))
        .stdout(predicate::str::contains("\"passed\": false"))
        .stdout(predicate::str::contains(
            "--fail-on-mismatch found 1 incompatible call(s)",
        ))
        .stdout(predicate::str::contains(
            "--require-definitions expected at least 2, found 1",
        ))
        .stdout(predicate::str::contains(
            "--require-calls expected at least 2, found 1",
        ))
        .stderr(predicate::str::contains("signature-report policy failed"));
}

#[test]
fn cli_accepts_signature_report_policy_when_compatible() {
    let dir = fresh_temp_dir("signature-report-policy-compatible");
    let lisp_file = dir.join("core.lisp");
    fs::write(
        &lisp_file,
        "(defun render (pane theme) pane)\n(defun draw () (render pane theme))\n",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("signature-report")
        .arg("--symbol")
        .arg("render")
        .arg("--fail-on-mismatch")
        .arg("--require-definitions")
        .arg("1")
        .arg("--require-calls")
        .arg("1")
        .arg("--output")
        .arg("text")
        .arg(&lisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("definition_count\t1"))
        .stdout(predicate::str::contains("call_count\t1"))
        .stdout(predicate::str::contains("mismatch_count\t0"))
        .stdout(predicate::str::contains("policy_passed\ttrue"))
        .stdout(predicate::str::contains("status\texact\t1"));
}
