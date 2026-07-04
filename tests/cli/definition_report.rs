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
         (define-symbol-macro current-user (slot-value *session* 'user))\n\
         (deftest split-window () (is (= 2 (pane-count))))\n",
    )
    .expect("write lisp fixture");
    fs::write(
        &elisp_file,
        "(defun demo-render (buffer) (message \"%s\" buffer))\n\
         (define-minor-mode demo-mode \"Demo mode\" :lighter \" Demo\")\n",
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
        .stdout(predicate::str::contains("\"definition_count\": 6"))
        .stdout(predicate::str::contains("\"dialect\": \"common-lisp\""))
        .stdout(predicate::str::contains("\"dialect\": \"emacs-lisp\""))
        .stdout(predicate::str::contains("\"package\": \"#:demo\""))
        .stdout(predicate::str::contains("\"category\": \"function\""))
        .stdout(predicate::str::contains("\"category\": \"macro\""))
        .stdout(predicate::str::contains("\"category\": \"variable\""))
        .stdout(predicate::str::contains("\"category\": \"test\""))
        .stdout(predicate::str::contains("\"category\": \"mode\""))
        .stdout(predicate::str::contains("\"parameter_count\": 2"))
        .stdout(predicate::str::contains("\"name\": \"current-user\""))
        .stdout(predicate::str::contains("\"name\": \"render-pane\""))
        .stdout(predicate::str::contains("\"name\": \"demo-mode\""));
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
