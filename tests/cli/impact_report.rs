use super::*;

#[test]
fn cli_reports_refactor_impact_across_dialects() {
    let dir = fresh_temp_dir("impact-report");
    let lisp_file = dir.join("core.lisp");
    let elisp_file = dir.join("init.el");
    fs::write(
        &lisp_file,
        "(defun area (width height) (* width height))\n(defun render () (area 10) area)\n",
    )
    .expect("write lisp fixture");
    fs::write(&elisp_file, "(defun draw () (area 5 6))\n").expect("write elisp fixture");

    let mut cmd = paredit();
    cmd.arg("inspect")
        .arg("impact")
        .arg("--symbol")
        .arg("area")
        .arg(&lisp_file)
        .arg(&elisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"symbol\": \"area\""))
        .stdout(predicate::str::contains("\"riskLevel\": \"warning\""))
        .stdout(predicate::str::contains("\"definition_count\": 1"))
        .stdout(predicate::str::contains("\"reference_count\": 3"))
        .stdout(predicate::str::contains("\"call_count\": 2"))
        .stdout(predicate::str::contains("\"inbound_edge_count\": 2"))
        .stdout(predicate::str::contains("\"non_call_reference_count\": 1"))
        .stdout(predicate::str::contains("\"code\": \"inbound-callers\""))
        .stdout(predicate::str::contains(
            "\"code\": \"non-call-references\"",
        ))
        .stdout(predicate::str::contains("\"code\": \"signature-mismatch\""))
        .stdout(predicate::str::contains("\"status\": \"exact\""))
        .stdout(predicate::str::contains(
            "\"status\": \"missing-arguments\"",
        ))
        .stdout(predicate::str::contains("\"dialect\": \"common-lisp\""))
        .stdout(predicate::str::contains("\"dialect\": \"emacs-lisp\""));
}

#[test]
fn cli_gates_refactor_impact_policy_for_ci() {
    let dir = fresh_temp_dir("impact-report-policy");
    let lisp_file = dir.join("core.lisp");
    let elisp_file = dir.join("init.el");
    fs::write(
        &lisp_file,
        "(defun area (width height) (* width height))\n(defun render () (area 10) area)\n",
    )
    .expect("write lisp impact policy fixture");
    fs::write(&elisp_file, "(defun draw () (area 5 6))\n")
        .expect("write elisp impact policy fixture");

    let mut cmd = paredit();
    cmd.arg("inspect")
        .arg("impact")
        .arg("--symbol")
        .arg("area")
        .arg("--fail-on-risk-level")
        .arg("warning")
        .arg("--require-definitions")
        .arg("2")
        .arg("--require-references")
        .arg("5")
        .arg("--require-calls")
        .arg("3")
        .arg("--output")
        .arg("json")
        .arg(&lisp_file)
        .arg(&elisp_file)
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"policy\""))
        .stdout(predicate::str::contains("\"passed\": false"))
        .stdout(predicate::str::contains(
            "\"fail_on_risk_level\": \"warning\"",
        ))
        .stdout(predicate::str::contains("\"risk_level\": \"warning\""))
        .stdout(predicate::str::contains(
            "--fail-on-risk-level warning failed with warning risk",
        ))
        .stdout(predicate::str::contains(
            "--require-definitions expected at least 2, found 1",
        ))
        .stdout(predicate::str::contains(
            "--require-references expected at least 5, found 3",
        ))
        .stdout(predicate::str::contains(
            "--require-calls expected at least 3, found 2",
        ))
        .stderr(predicate::str::contains("impact-report policy failed"));
}

#[test]
fn cli_accepts_refactor_impact_policy_when_thresholds_pass() {
    let dir = fresh_temp_dir("impact-report-policy-pass");
    let file = dir.join("core.lisp");
    fs::write(
        &file,
        "(defun area (width height) (* width height))\n(defun render () (area 10 20))\n",
    )
    .expect("write passing impact policy fixture");

    let mut cmd = paredit();
    cmd.arg("inspect")
        .arg("impact")
        .arg("--symbol")
        .arg("area")
        .arg("--fail-on-risk-level")
        .arg("error")
        .arg("--require-definitions")
        .arg("1")
        .arg("--require-references")
        .arg("1")
        .arg("--require-calls")
        .arg("1")
        .arg("--output")
        .arg("text")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("risk_level\twarning"))
        .stdout(predicate::str::contains("signature_mismatch_count\t0"))
        .stdout(predicate::str::contains("policy_passed\ttrue"));
}

#[test]
fn cli_reports_common_lisp_setf_callable_impact() {
    let dir = fresh_temp_dir("impact-report-common-lisp-setf");
    let file = dir.join("setf.lisp");
    fs::write(
        &file,
        "(define-setf-expander accessor (place) (values nil nil '(store) '(writer store) '(reader place)))\n(defun render (item) (setf (accessor item) 1) accessor)\n(defun wrapper (item) (setf (accessor item) 2))\n",
    )
    .expect("write setf impact fixture");

    let mut cmd = paredit();
    cmd.arg("inspect")
        .arg("impact")
        .arg("--symbol")
        .arg("accessor")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"definition_count\": 1"))
        .stdout(predicate::str::contains("\"reference_count\": 3"))
        .stdout(predicate::str::contains("\"call_count\": 2"))
        .stdout(predicate::str::contains("\"inbound_edge_count\": 2"))
        .stdout(predicate::str::contains("\"non_call_reference_count\": 1"))
        .stdout(predicate::str::contains("\"status\": \"exact\""));
}

#[test]
fn cli_excludes_symbol_macrolet_binding_names_and_shadowed_body_references_from_impact() {
    let dir = fresh_temp_dir("impact-report-symbol-macrolet");
    let file = dir.join("symbol-macrolet.lisp");
    fs::write(
        &file,
        "(in-package #:app)\n(defun helper () 1)\n(defun caller ()\n  (cl:symbol-macrolet ((helper (helper)))\n    helper))\n",
    )
    .expect("write symbol-macrolet impact fixture");

    let mut cmd = paredit();
    cmd.arg("inspect")
        .arg("impact")
        .arg("--symbol")
        .arg("helper")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"reference_count\": 1"))
        .stdout(predicate::str::contains("\"call_count\": 1"))
        .stdout(predicate::str::contains("\"inbound_edge_count\": 1"))
        .stdout(predicate::str::contains("\"non_call_reference_count\": 0"));
}
