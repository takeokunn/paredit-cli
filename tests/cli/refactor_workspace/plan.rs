use super::*;

#[test]
fn cli_builds_gated_refactor_plan_for_agents() {
    let dir = fresh_temp_dir("refactor-plan");
    let file = dir.join("core.lisp");
    fs::write(
        &file,
        r#"(defpackage #:demo.core (:use #:cl))
(in-package #:demo.core)
(defun render-pane (pane) (draw-pane pane))
(defun caller () (render-pane window) render-pane)
"#,
    )
    .expect("write refactor plan fixture");

    let mut cmd = paredit();
    cmd.arg("refactor-plan")
        .arg("--symbol")
        .arg("render-pane")
        .arg("--operation")
        .arg("rename")
        .arg("--output")
        .arg("json")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"operation\": \"rename\""))
        .stdout(predicate::str::contains("\"symbol\": \"render-pane\""))
        .stdout(predicate::str::contains("\"decision\""))
        .stdout(predicate::str::contains("\"status\": \"ready\""))
        .stdout(predicate::str::contains(
            "\"next_action\": \"apply-symbol-rename\"",
        ))
        .stdout(predicate::str::contains("\"safe_to_automate\": true"))
        .stdout(predicate::str::contains("\"policy_passed\": true"))
        .stdout(predicate::str::contains("\"blocking_gate_count\": 0"))
        .stdout(predicate::str::contains("\"name\": \"plan-policy\""))
        .stdout(predicate::str::contains(
            "\"name\": \"manual-review-gates\"",
        ))
        .stdout(predicate::str::contains("\"name\": \"apply-plan\""))
        .stdout(predicate::str::contains("\"status\": \"scheduled\""))
        .stdout(predicate::str::contains("\"definition_count\": 1"))
        .stdout(predicate::str::contains("\"call_count\": 1"))
        .stdout(predicate::str::contains(
            "\"code\": \"non-call-references\"",
        ))
        .stdout(predicate::str::contains(
            "\"action\": \"run-impact-report\"",
        ))
        .stdout(predicate::str::contains("--fail-on-risk-level warning"))
        .stdout(predicate::str::contains("--require-definitions 1"))
        .stdout(predicate::str::contains("--require-references 1"))
        .stdout(predicate::str::contains("--require-calls 1"))
        .stdout(predicate::str::contains(
            "paredit rename-symbols --from 'render-pane' --to <new-symbol> --output json",
        ));
}

#[test]
fn cli_builds_workspace_refactor_plan_from_directory_roots() {
    let dir = fresh_temp_dir("workspace-refactor-plan");
    let src_dir = dir.join("src");
    fs::create_dir_all(&src_dir).expect("create source dir");

    let file = src_dir.join("core.lisp");
    let ignored = src_dir.join("notes.txt");
    fs::write(
        &file,
        r#"(defpackage #:demo.core (:use #:cl))
(in-package #:demo.core)
(defun render-pane (pane) (draw-pane pane))
(defun caller () (render-pane window) render-pane)
"#,
    )
    .expect("write workspace refactor fixture");
    fs::write(&ignored, "render-pane is mentioned in plain text")
        .expect("write ignored workspace fixture");

    let mut cmd = paredit();
    cmd.arg("workspace-refactor-plan")
        .arg("--symbol")
        .arg("render-pane")
        .arg("--operation")
        .arg("rename")
        .arg("--output")
        .arg("json")
        .arg(&dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"operation\": \"rename\""))
        .stdout(predicate::str::contains("\"symbol\": \"render-pane\""))
        .stdout(predicate::str::contains("\"workspace\""))
        .stdout(predicate::str::contains("\"discovered_file_count\": 1"))
        .stdout(predicate::str::contains("\"unknown\": 1"))
        .stdout(predicate::str::contains("\"file_count\": 1"))
        .stdout(predicate::str::contains("\"definition_count\": 1"))
        .stdout(predicate::str::contains("\"call_count\": 1"))
        .stdout(predicate::str::contains(
            "paredit rename-symbols --from 'render-pane' --to <new-symbol> --output json",
        ));
}

#[test]
fn cli_builds_workspace_remove_plan_with_unused_definition_cleanup_command() {
    let dir = fresh_temp_dir("workspace-remove-plan");
    let src_dir = dir.join("src");
    fs::create_dir_all(&src_dir).expect("create source dir");

    let file = src_dir.join("core.lisp");
    fs::write(
        &file,
        r#"(defpackage #:demo.core (:use #:cl))
(in-package #:demo.core)
(defun stale-helper (value) value)
(defun caller () 42)
"#,
    )
    .expect("write workspace remove fixture");

    let mut cmd = paredit();
    cmd.arg("workspace-refactor-plan")
        .arg("--symbol")
        .arg("stale-helper")
        .arg("--operation")
        .arg("remove")
        .arg("--output")
        .arg("json")
        .arg(&dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"operation\": \"remove\""))
        .stdout(predicate::str::contains("\"symbol\": \"stale-helper\""))
        .stdout(predicate::str::contains(
            "\"action\": \"apply-unused-definition-removal\"",
        ))
        .stdout(predicate::str::contains(
            "paredit remove-unused-definitions --output json",
        ))
        .stdout(predicate::str::contains(
            "paredit verify-refactor --symbol 'stale-helper' --operation remove --phase post --output json",
        ));
}

#[test]
fn cli_fails_refactor_plan_policy_after_printing_json() {
    let dir = fresh_temp_dir("refactor-plan-policy");
    let file = dir.join("core.lisp");
    fs::write(
        &file,
        r#"(defpackage #:demo.core (:use #:cl))
(in-package #:demo.core)
(defun render-pane (pane) (draw-pane pane))
(defun caller () (render-pane window) render-pane)
"#,
    )
    .expect("write refactor plan policy fixture");

    let mut cmd = paredit();
    cmd.arg("refactor-plan")
        .arg("--symbol")
        .arg("render-pane")
        .arg("--operation")
        .arg("rename")
        .arg("--fail-on-blocking-gate")
        .arg("--require-definitions")
        .arg("2")
        .arg("--output")
        .arg("json")
        .arg(&file)
        .assert()
        .failure()
        .stdout(predicate::str::contains("\"policy\""))
        .stdout(predicate::str::contains("\"decision\""))
        .stdout(predicate::str::contains("\"status\": \"policy_failed\""))
        .stdout(predicate::str::contains(
            "\"next_action\": \"resolve-policy-violations\"",
        ))
        .stdout(predicate::str::contains("\"safe_to_automate\": false"))
        .stdout(predicate::str::contains("\"policy_passed\": false"))
        .stdout(predicate::str::contains("\"name\": \"plan-policy\""))
        .stdout(predicate::str::contains("\"status\": \"failed\""))
        .stdout(predicate::str::contains("\"name\": \"apply-plan\""))
        .stdout(predicate::str::contains("\"status\": \"skipped\""))
        .stdout(predicate::str::contains("\"passed\": false"))
        .stdout(predicate::str::contains("\"fail_on_blocking_gate\": true"))
        .stdout(predicate::str::contains(
            "--require-definitions expected at least 2, found 1",
        ))
        .stderr(predicate::str::contains("refactor-plan policy failed"));
}

#[test]
fn cli_marks_refactor_plan_manual_review_when_blocking_gates_do_not_fail_policy() {
    let dir = fresh_temp_dir("refactor-plan-manual-review");
    let file = dir.join("core.lisp");
    fs::write(
        &file,
        r#"(defpackage #:demo.core (:use #:cl))
(in-package #:demo.core)
(defun render-pane (pane) (draw-pane pane))
(defun render-pane (pane theme) (draw-themed-pane pane theme))
(defun caller () (render-pane window))
"#,
    )
    .expect("write manual review refactor plan fixture");

    let mut cmd = paredit();
    cmd.arg("refactor-plan")
        .arg("--symbol")
        .arg("render-pane")
        .arg("--operation")
        .arg("rename")
        .arg("--output")
        .arg("json")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"decision\""))
        .stdout(predicate::str::contains("\"status\": \"manual_review\""))
        .stdout(predicate::str::contains("\"policy_passed\": true"))
        .stdout(predicate::str::contains("\"safe_to_automate\": false"))
        .stdout(predicate::str::contains("\"blocking_gate_count\": 2"))
        .stdout(predicate::str::contains(
            "\"name\": \"manual-review-gates\"",
        ))
        .stdout(predicate::str::contains("\"status\": \"scheduled\""))
        .stdout(predicate::str::contains(
            "\"next_action\": \"review-rename-scope\"",
        ))
        .stdout(predicate::str::contains(
            "\"code\": \"ambiguous-definition\"",
        ))
        .stdout(predicate::str::contains("\"code\": \"signature-mismatch\""))
        .stdout(predicate::str::contains("\"command\": null"));
}
