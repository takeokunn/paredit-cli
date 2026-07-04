use super::*;

#[test]
fn cli_verifies_post_rename_refactor_invariants_for_agents() {
    let dir = fresh_temp_dir("verify-refactor");
    let file = dir.join("core.lisp");
    fs::write(
        &file,
        r#"(defpackage #:demo.core (:use #:cl))
(in-package #:demo.core)
(defun paint-pane (pane) (draw-pane pane))
(defun caller () (paint-pane window))
"#,
    )
    .expect("write verify refactor fixture");

    let mut cmd = paredit();
    cmd.arg("verify-refactor")
        .arg("--symbol")
        .arg("render-pane")
        .arg("--new-symbol")
        .arg("paint-pane")
        .arg("--operation")
        .arg("rename")
        .arg("--phase")
        .arg("post")
        .arg("--output")
        .arg("json")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"operation\": \"rename\""))
        .stdout(predicate::str::contains("\"phase\": \"post\""))
        .stdout(predicate::str::contains("\"passed\": true"))
        .stdout(predicate::str::contains("\"code\": \"old-symbol-removed\""))
        .stdout(predicate::str::contains("\"code\": \"new-symbol-present\""))
        .stdout(predicate::str::contains(
            "\"code\": \"new-symbol-signature-compatible\"",
        ));
}
