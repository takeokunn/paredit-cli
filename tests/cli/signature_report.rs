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

#[test]
fn cli_reports_common_lisp_setf_callable_signature_usage() {
    let dir = fresh_temp_dir("signature-report-common-lisp-setf");
    let lisp_file = dir.join("setf.lisp");
    fs::write(
        &lisp_file,
        "(define-setf-expander accessor (place) (values nil nil '(store) '(writer store) '(reader place)))\n(defun render (item) (setf (accessor item) 1) accessor)\n",
    )
    .expect("write setf signature fixture");

    let mut cmd = paredit();
    cmd.arg("signature-report")
        .arg("--symbol")
        .arg("accessor")
        .arg(&lisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"definition_count\": 1"))
        .stdout(predicate::str::contains("\"call_count\": 1"))
        .stdout(predicate::str::contains("\"parameterCount\": 1"))
        .stdout(predicate::str::contains("\"expectedParameterCount\": 1"))
        .stdout(predicate::str::contains("\"status\": \"exact\""));
}

#[test]
fn cli_reports_common_lisp_defmacro_signature_usage() {
    let dir = fresh_temp_dir("signature-report-common-lisp-defmacro");
    let lisp_file = dir.join("defmacro.lisp");
    fs::write(
        &lisp_file,
        "(defmacro with-pane (pane theme) `(render ,pane ,theme))\n(defun use () (with-pane pane theme))\n",
    )
    .expect("write defmacro signature fixture");

    let mut cmd = paredit();
    cmd.arg("signature-report")
        .arg("--symbol")
        .arg("with-pane")
        .arg(&lisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"definition_count\": 1"))
        .stdout(predicate::str::contains("\"call_count\": 1"))
        .stdout(predicate::str::contains("\"category\": \"macro\""))
        .stdout(predicate::str::contains("\"parameterCount\": 2"))
        .stdout(predicate::str::contains("\"expectedParameterCount\": 2"))
        .stdout(predicate::str::contains("\"status\": \"exact\""));
}

#[test]
fn cli_reports_common_lisp_define_compiler_macro_signature_usage() {
    let dir = fresh_temp_dir("signature-report-common-lisp-compiler-macro");
    let lisp_file = dir.join("compiler-macro.lisp");
    fs::write(
        &lisp_file,
        "(define-compiler-macro optimize-render (pane theme) `(render ,pane ,theme))\n(defun use () (optimize-render pane theme))\n",
    )
    .expect("write compiler macro signature fixture");

    let mut cmd = paredit();
    cmd.arg("signature-report")
        .arg("--symbol")
        .arg("optimize-render")
        .arg(&lisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"definition_count\": 1"))
        .stdout(predicate::str::contains("\"call_count\": 1"))
        .stdout(predicate::str::contains("\"category\": \"macro\""))
        .stdout(predicate::str::contains("\"parameterCount\": 2"))
        .stdout(predicate::str::contains("\"expectedParameterCount\": 2"))
        .stdout(predicate::str::contains("\"status\": \"exact\""));
}

#[test]
fn cli_reports_common_lisp_define_modify_macro_signature_usage() {
    let dir = fresh_temp_dir("signature-report-common-lisp-modify-macro");
    let lisp_file = dir.join("modify-macro.lisp");
    fs::write(&lisp_file, "(define-modify-macro updatef (place) incf)\n")
        .expect("write modify macro signature fixture");

    let mut cmd = paredit();
    cmd.arg("signature-report")
        .arg("--symbol")
        .arg("updatef")
        .arg(&lisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"definition_count\": 1"))
        .stdout(predicate::str::contains("\"call_count\": 0"))
        .stdout(predicate::str::contains("\"category\": \"macro\""))
        .stdout(predicate::str::contains("\"parameterCount\": 1"));
}

#[test]
fn cli_reports_common_lisp_define_method_combination_signature_usage() {
    let dir = fresh_temp_dir("signature-report-common-lisp-method-combination");
    let lisp_file = dir.join("method-combination.lisp");
    fs::write(
        &lisp_file,
        "(define-method-combination render-combination (pane theme) ((primary *)) (list pane theme primary))\n",
    )
    .expect("write method combination signature fixture");

    let mut cmd = paredit();
    cmd.arg("signature-report")
        .arg("--symbol")
        .arg("render-combination")
        .arg(&lisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"definition_count\": 1"))
        .stdout(predicate::str::contains("\"call_count\": 0"))
        .stdout(predicate::str::contains("\"category\": \"macro\""))
        .stdout(predicate::str::contains("\"parameterCount\": 2"));
}

#[test]
fn cli_reports_common_lisp_defsetf_long_form_signature_usage() {
    let dir = fresh_temp_dir("signature-report-common-lisp-defsetf-long");
    let lisp_file = dir.join("defsetf-long.lisp");
    fs::write(
        &lisp_file,
        "(defsetf accessor (item) (value) `(writer ,item ,value))\n(defun render (item) (setf (accessor item) 1) accessor)\n",
    )
    .expect("write defsetf signature fixture");

    let mut cmd = paredit();
    cmd.arg("signature-report")
        .arg("--symbol")
        .arg("accessor")
        .arg(&lisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"definition_count\": 1"))
        .stdout(predicate::str::contains("\"call_count\": 1"))
        .stdout(predicate::str::contains("\"parameterCount\": 1"))
        .stdout(predicate::str::contains("\"expectedParameterCount\": 1"))
        .stdout(predicate::str::contains("\"status\": \"exact\""));
}

#[test]
fn cli_marks_common_lisp_short_defsetf_signature_as_unknown() {
    let dir = fresh_temp_dir("signature-report-common-lisp-defsetf-short");
    let lisp_file = dir.join("defsetf-short.lisp");
    fs::write(
        &lisp_file,
        "(defsetf accessor set-accessor)\n(defun render (item) (setf (accessor item) 1) accessor)\n",
    )
    .expect("write short defsetf signature fixture");

    let mut cmd = paredit();
    cmd.arg("signature-report")
        .arg("--symbol")
        .arg("accessor")
        .arg(&lisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"definition_count\": 0"))
        .stdout(predicate::str::contains("\"call_count\": 1"))
        .stdout(predicate::str::contains("\"expectedParameterCount\": null"))
        .stdout(predicate::str::contains(
            "\"status\": \"unknown-definition\"",
        ));
}

#[test]
fn cli_reports_common_lisp_symbol_macro_without_arity_signature() {
    let dir = fresh_temp_dir("signature-report-common-lisp-symbol-macro");
    let lisp_file = dir.join("symbol-macro.lisp");
    fs::write(
        &lisp_file,
        "(define-symbol-macro current-user (slot-value *session* 'user))\n(list current-user)\n",
    )
    .expect("write symbol macro signature fixture");

    let mut cmd = paredit();
    cmd.arg("signature-report")
        .arg("--symbol")
        .arg("current-user")
        .arg("--output")
        .arg("json")
        .arg(&lisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"definition_count\": 1"))
        .stdout(predicate::str::contains("\"call_count\": 0"))
        .stdout(predicate::str::contains("\"category\": \"variable\""))
        .stdout(predicate::str::contains("\"parameterCount\": null"));
}

#[test]
fn cli_reports_common_lisp_symbol_macrolet_expansion_and_body_calls_without_binding_name_calls() {
    let dir = fresh_temp_dir("signature-report-common-lisp-symbol-macrolet");
    let lisp_file = dir.join("symbol-macrolet.lisp");
    fs::write(
        &lisp_file,
        "(defun helper (x) (+ x 10))\n(defun target (x) x)\n(defun render () (symbol-macrolet ((helper (target 1))) (list helper (target 2))))\n",
    )
    .expect("write symbol-macrolet signature fixture");

    let mut cmd = paredit();
    cmd.arg("signature-report")
        .arg("--symbol")
        .arg("target")
        .arg("--output")
        .arg("json")
        .arg(&lisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"definition_count\": 1"))
        .stdout(predicate::str::contains("\"call_count\": 2"))
        .stdout(predicate::str::contains("\"head\": \"target\""))
        .stdout(predicate::str::contains("\"expectedParameterCount\": 1"))
        .stdout(predicate::str::contains("\"status\": \"exact\""))
        .stdout(predicate::str::contains("\"head\": \"helper\"").not());
}
