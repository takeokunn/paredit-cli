use super::*;

#[test]
fn cli_reports_definition_inventory_for_refactor_planning() {
    let dir = fresh_temp_dir("definition-report");
    let lisp_file = dir.join("core.lisp");
    let elisp_file = dir.join("mode.el");
    fs::write(
        &lisp_file,
        "(in-package #:demo)\n\
         (defun render-pane (session pane) (list session pane))\n\
         (defmacro with-pane ((pane) &body body) `(progn ,pane ,@body))\n\
         (defclass renderer () ())\n\
         (defstruct point x y)\n\
         (define-modify-macro updatef (place) incf)\n\
         (define-symbol-macro current-user (slot-value *session* 'user))\n\
         (deftest split-window () (is (= 2 (pane-count))))\n",
    )
    .expect("write lisp fixture");
    fs::write(
        &elisp_file,
        "(defun demo-render (buffer) (message \"%s\" buffer))\n\
         (define-minor-mode demo-mode \"Demo mode\" :lighter \" Demo\")\n\
         (cl-defgeneric demo-renderer (buffer))\n\
         (cl-defmethod demo-renderer ((buffer string)) (message \"%s\" buffer))\n",
    )
    .expect("write elisp fixture");

    let mut cmd = paredit();
    cmd.arg("definition-report")
        .arg("--output")
        .arg("json")
        .arg(&lisp_file)
        .arg(&elisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"file_count\": 2"))
        .stdout(predicate::str::contains("\"definition_count\": 11"))
        .stdout(predicate::str::contains("\"dialect\": \"common-lisp\""))
        .stdout(predicate::str::contains("\"dialect\": \"emacs-lisp\""))
        .stdout(predicate::str::contains("\"package\": \"#:demo\""))
        .stdout(predicate::str::contains("\"category\": \"function\""))
        .stdout(predicate::str::contains("\"category\": \"macro\""))
        .stdout(predicate::str::contains("\"category\": \"class\""))
        .stdout(predicate::str::contains("\"category\": \"struct\""))
        .stdout(predicate::str::contains("\"category\": \"variable\""))
        .stdout(predicate::str::contains("\"category\": \"test\""))
        .stdout(predicate::str::contains("\"category\": \"mode\""))
        .stdout(predicate::str::contains(
            "\"category\": \"generic-function\"",
        ))
        .stdout(predicate::str::contains("\"category\": \"method\""))
        .stdout(predicate::str::contains("\"parameter_count\": 2"))
        .stdout(predicate::str::contains("\"name\": \"current-user\""))
        .stdout(predicate::str::contains("\"name\": \"render-pane\""))
        .stdout(predicate::str::contains("\"name\": \"renderer\""))
        .stdout(predicate::str::contains("\"name\": \"point\""))
        .stdout(predicate::str::contains("\"name\": \"demo-mode\""))
        .stdout(predicate::str::contains("\"name\": \"demo-renderer\""));
}

#[test]
fn cli_reports_unrecognized_define_style_macros_as_unknown_macro_category() {
    let dir = fresh_temp_dir("definition-report-unknown-macro");
    let lisp_file = dir.join("strategy.lisp");
    fs::write(
        &lisp_file,
        "(define-trading-strategy d1-momentum :parameters 42)\n",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("definition-report")
        .arg("--output")
        .arg("json")
        .arg(&lisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"category\": \"unknown-macro\""))
        .stdout(predicate::str::contains("\"name\": \"d1-momentum\""));
}

#[test]
fn cli_reports_unused_definitions_for_dead_code_planning() {
    let dir = fresh_temp_dir("unused-definition-report");
    let lisp_file = dir.join("core.lisp");
    let elisp_file = dir.join("mode.el");
    fs::write(
        &lisp_file,
        "(defun used () :ok)\n\
         (defun caller () (used))\n\
         (defun unused () (unused))\n",
    )
    .expect("write lisp fixture");
    fs::write(
        &elisp_file,
        "(defun demo-mode () (caller))\n\
         (defun elisp-unused () (message \"unused\"))\n",
    )
    .expect("write elisp fixture");

    let mut cmd = paredit();
    cmd.arg("unused-definition-report")
        .arg("--output")
        .arg("json")
        .arg(&lisp_file)
        .arg(&elisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"file_count\": 2"))
        .stdout(predicate::str::contains("\"definition_count\": 5"))
        .stdout(predicate::str::contains("\"candidate_count\": 3"))
        .stdout(predicate::str::contains("\"dialect\": \"common-lisp\""))
        .stdout(predicate::str::contains("\"dialect\": \"emacs-lisp\""))
        .stdout(predicate::str::contains("\"name\": \"unused\""))
        .stdout(predicate::str::contains("\"name\": \"elisp-unused\""))
        .stdout(predicate::str::contains("\"reference_count\": 0"))
        .stdout(predicate::str::contains("\"unused\": true"))
        .stdout(predicate::str::contains("\"name\": \"used\""))
        .stdout(predicate::str::contains("\"reference_count\": 1"));
}

#[test]
fn cli_treats_a_shadowing_let_binding_as_no_reference_to_the_global_definition() {
    // Regression test: `unused-definition-report` must agree with
    // `remove-unused-definitions` on whether a same-named local binding
    // shadows the global definition. Previously the report used a flat,
    // scope-blind atom scan, so `foo` here was counted as referenced by the
    // `let`-shadowed occurrences inside `bar` even though nothing calls the
    // actual global `foo`.
    let dir = fresh_temp_dir("unused-definition-report-shadowed-let");
    let lisp_file = dir.join("core.lisp");
    fs::write(
        &lisp_file,
        "(defun foo () :global)\n\
         (defun bar () (let ((foo 1)) (+ foo foo)))\n",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("unused-definition-report")
        .arg("--output")
        .arg("json")
        .arg(&lisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"candidate_count\": 2"))
        .stdout(predicate::str::contains("\"name\": \"foo\""))
        .stdout(predicate::str::contains("\"name\": \"bar\""));
}

#[test]
fn cli_reports_package_qualified_common_lisp_definition_as_used_by_unqualified_reference() {
    let dir = fresh_temp_dir("unused-definition-report-qualified-use");
    let file = dir.join("core.lisp");
    fs::write(
        &file,
        "(defun cl-user:used () :ok)\n\
         (defun caller () (used))\n",
    )
    .expect("write fixture");

    let mut cmd = paredit();
    cmd.arg("unused-definition-report")
        .arg("--output")
        .arg("json")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"name\": \"cl-user:used\""))
        .stdout(predicate::str::contains("\"reference_count\": 1"))
        .stdout(predicate::str::contains("\"unused\": false"));
}

#[test]
fn cli_gates_unused_definition_report_for_ci() {
    let dir = fresh_temp_dir("unused-definition-report-policy");
    let file = dir.join("core.lisp");
    fs::write(
        &file,
        "(defun used () :ok)\n\
         (defun caller () (used))\n\
         (caller)\n\
         (defun stale-helper () :stale)\n",
    )
    .expect("write fixture");

    let mut cmd = paredit();
    cmd.arg("unused-definition-report")
        .arg("--fail-on-unused")
        .arg("--require-unused-definitions")
        .arg("2")
        .arg("--output")
        .arg("json")
        .arg(&file)
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"candidate_count\": 1"))
        .stdout(predicate::str::contains("\"policy\""))
        .stdout(predicate::str::contains("\"fail_on_unused\": true"))
        .stdout(predicate::str::contains(
            "\"require_unused_definitions\": 2",
        ))
        .stdout(predicate::str::contains("\"passed\": false"))
        .stdout(predicate::str::contains(
            "candidate_count 1 is below required 2",
        ))
        .stderr(predicate::str::contains(
            "unused-definition-report policy failed",
        ));
}

#[test]
fn cli_fail_on_unused_ignores_protected_category_candidates() {
    // A `deftest` is invoked by name from a test runner, not referenced by
    // symbol from other Lisp forms, so it having zero direct references is
    // its normal, expected state. `--fail-on-unused` must gate on the
    // actionable (bulk-removable-category) count so an ordinary test suite
    // does not fail CI on its own.
    let dir = fresh_temp_dir("unused-definition-report-protected-gate");
    let file = dir.join("core.lisp");
    fs::write(
        &file,
        "(defun used () :ok)\n\
         (defun caller () (used))\n\
         (caller)\n\
         (deftest stale-test () (is t))\n",
    )
    .expect("write fixture");

    let mut cmd = paredit();
    cmd.arg("unused-definition-report")
        .arg("--fail-on-unused")
        .arg("--output")
        .arg("json")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"candidate_count\": 1"))
        .stdout(predicate::str::contains(
            "\"actionable_candidate_count\": 0",
        ))
        .stdout(predicate::str::contains("\"passed\": true"))
        .stdout(predicate::str::contains("\"bulk_removable\": false"));
}

#[test]
fn cli_accepts_unused_definition_requirement_when_satisfied() {
    let dir = fresh_temp_dir("unused-definition-report-requirement");
    let file = dir.join("core.lisp");
    fs::write(
        &file,
        "(defun used () :ok)\n\
         (defun caller () (used))\n\
         (caller)\n\
         (defun stale-helper-a () :stale)\n\
         (defun stale-helper-b () :stale)\n",
    )
    .expect("write fixture");

    let mut cmd = paredit();
    cmd.arg("unused-definition-report")
        .arg("--require-unused-definitions")
        .arg("2")
        .arg("--output")
        .arg("text")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("candidate_count\t2"))
        .stdout(predicate::str::contains("policy_passed\ttrue"));
}
