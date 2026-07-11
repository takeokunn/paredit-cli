use super::*;

#[test]
fn cli_reports_multi_file_call_sites_for_agent_refactor_planning() {
    let dir = fresh_temp_dir("call-report");
    let lisp_file = dir.join("core.lisp");
    let elisp_file = dir.join("init.el");
    fs::write(
        &lisp_file,
        "(defun area (width height) (* width height))\n(defun render () (area 10 20))\n(defun total () (+ (area 3 4) 1))\n",
    )
    .expect("write lisp fixture");
    fs::write(&elisp_file, "(defun demo-mode () (area 5 6))\n").expect("write elisp fixture");

    let mut cmd = paredit();
    cmd.arg("inspect")
        .arg("calls")
        .arg("--symbol")
        .arg("area")
        .arg(&lisp_file)
        .arg(&elisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"symbol\": \"area\""))
        .stdout(predicate::str::contains("\"file_count\": 2"))
        .stdout(predicate::str::contains("\"total_count\": 3"))
        .stdout(predicate::str::contains("\"dialect\": \"common-lisp\""))
        .stdout(predicate::str::contains("\"dialect\": \"emacs-lisp\""))
        .stdout(predicate::str::contains("\"head\": \"area\""))
        .stdout(predicate::str::contains("\"argumentCount\": 2"))
        .stdout(predicate::str::contains(
            "\"enclosingDefinition\": \"render\"",
        ))
        .stdout(predicate::str::contains(
            "\"enclosingDefinition\": \"total\"",
        ))
        .stdout(predicate::str::contains(
            "\"enclosingDefinition\": \"demo-mode\"",
        ));

    let mut cmd = paredit();
    cmd.arg("inspect")
        .arg("calls")
        .arg(&lisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"head\": \"area\""))
        .stdout(predicate::str::contains("\"head\": \"defun\"").not());
}

#[test]
fn cli_can_include_definition_forms_for_inventory_reports() {
    let dir = fresh_temp_dir("call-report-include-definitions");
    let lisp_file = dir.join("inventory.lisp");
    fs::write(
        &lisp_file,
        "(defun area (width height) (scale width height))\n",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("inspect")
        .arg("calls")
        .arg("--include-definitions")
        .arg("--output")
        .arg("json")
        .arg(&lisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"includeDefinitions\": true"))
        .stdout(predicate::str::contains("\"total_count\": 2"))
        .stdout(predicate::str::contains("\"head\": \"defun\""))
        .stdout(predicate::str::contains("\"category\": \"function\""))
        .stdout(predicate::str::contains("\"head\": \"scale\""));
}

#[test]
fn cli_skips_emacs_lisp_cl_flet_local_callable_calls() {
    let dir = fresh_temp_dir("call-report-emacs-cl-flet");
    let elisp_file = dir.join("locals.el");
    fs::write(
        &elisp_file,
        "(defun main () (cl-flet ((helper (x) (target x))) (helper 1) (target 2)))\n",
    )
    .expect("write elisp fixture");

    let mut cmd = paredit();
    cmd.arg("inspect")
        .arg("calls")
        .arg("--symbol")
        .arg("target")
        .arg("--output")
        .arg("json")
        .arg(&elisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"dialect\": \"emacs-lisp\""))
        .stdout(predicate::str::contains("\"total_count\": 2"))
        .stdout(predicate::str::contains("\"head\": \"target\""))
        .stdout(predicate::str::contains("\"head\": \"helper\"").not())
        .stdout(predicate::str::contains(
            "\"enclosingDefinition\": \"main\"",
        ));
}

#[test]
fn cli_skips_emacs_lisp_cl_labels_local_callable_calls_in_definition_bodies() {
    let dir = fresh_temp_dir("call-report-emacs-cl-labels");
    let elisp_file = dir.join("labels.el");
    fs::write(
        &elisp_file,
        "(defun main () (cl-labels ((helper (x) (helper x) (target x))) (helper 1)))\n",
    )
    .expect("write elisp fixture");

    let mut cmd = paredit();
    cmd.arg("inspect")
        .arg("calls")
        .arg("--symbol")
        .arg("target")
        .arg("--output")
        .arg("json")
        .arg(&elisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"dialect\": \"emacs-lisp\""))
        .stdout(predicate::str::contains("\"total_count\": 1"))
        .stdout(predicate::str::contains("\"head\": \"target\""))
        .stdout(predicate::str::contains("\"head\": \"helper\"").not())
        .stdout(predicate::str::contains(
            "\"enclosingDefinition\": \"main\"",
        ));
}

#[test]
fn cli_skips_emacs_lisp_cl_macrolet_local_macro_calls() {
    let dir = fresh_temp_dir("call-report-emacs-cl-macrolet");
    let elisp_file = dir.join("macrolet.el");
    fs::write(
        &elisp_file,
        "(defun main () (cl-macrolet ((helper (x) (list 'target x))) (helper 1)))\n",
    )
    .expect("write elisp fixture");

    let mut cmd = paredit();
    cmd.arg("inspect")
        .arg("calls")
        .arg("--symbol")
        .arg("helper")
        .arg("--output")
        .arg("json")
        .arg(&elisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"dialect\": \"emacs-lisp\""))
        .stdout(predicate::str::contains("\"total_count\": 0"))
        .stdout(predicate::str::contains("\"count\": 0"))
        .stdout(predicate::str::contains("\"head\": \"helper\"").not());
}

#[test]
fn cli_reports_common_lisp_symbol_macrolet_expansion_calls_without_binding_name_calls() {
    let dir = fresh_temp_dir("call-report-common-lisp-symbol-macrolet");
    let lisp_file = dir.join("symbol-macrolet.lisp");
    fs::write(
        &lisp_file,
        "(defun main () (symbol-macrolet ((helper place) (value (target 1))) (helper 1) value (target 2)))\n",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("inspect")
        .arg("calls")
        .arg("--symbol")
        .arg("target")
        .arg("--output")
        .arg("json")
        .arg(&lisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"dialect\": \"common-lisp\""))
        .stdout(predicate::str::contains("\"total_count\": 2"))
        .stdout(predicate::str::contains("\"head\": \"target\""))
        .stdout(predicate::str::contains("\"head\": \"helper\"").not())
        .stdout(predicate::str::contains(
            "\"enclosingDefinition\": \"main\"",
        ));
}

#[test]
fn cli_skips_reader_eval_bodies_when_reporting_calls() {
    let dir = fresh_temp_dir("call-report-reader-eval");
    let lisp_file = dir.join("reader-eval.lisp");
    fs::write(
        &lisp_file,
        "(defun helper (x) x)\n(defun caller () #.(list (helper 1) #'helper (function helper)) (helper 2))\n",
    )
    .expect("write reader-eval fixture");

    let mut cmd = paredit();
    cmd.arg("inspect")
        .arg("calls")
        .arg("--symbol")
        .arg("helper")
        .arg("--output")
        .arg("json")
        .arg(&lisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"total_count\": 1"))
        .stdout(predicate::str::contains("\"count\": 1"))
        .stdout(predicate::str::contains("\"head\": \"helper\""))
        .stdout(predicate::str::contains("\"head\": \"list\"").not())
        .stdout(predicate::str::contains("\"head\": \"function\"").not());
}
