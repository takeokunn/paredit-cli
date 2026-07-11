use super::*;

#[test]
fn cli_defaults_verify_refactor_to_pre_phase_and_reports_blocking_gates() {
    let dir = fresh_temp_dir("refactor verify-default-pre-phase");
    let first = dir.join("core.lisp");
    let second = dir.join("extra.lisp");
    fs::write(
        &first,
        r#"(defpackage #:demo.core (:use #:cl))
(in-package #:demo.core)
(defun render-pane (pane) pane)
"#,
    )
    .expect("write first verify refactor fixture");
    fs::write(
        &second,
        r#"(defpackage #:demo.extra (:use #:cl))
(in-package #:demo.extra)
(defun render-pane (pane) pane)
"#,
    )
    .expect("write second verify refactor fixture");

    let mut cmd = paredit();
    cmd.args(["refactor", "verify"])
        .arg("--symbol")
        .arg("render-pane")
        .arg("--output")
        .arg("json")
        .arg(&first)
        .arg(&second)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"operation\": \"rename\""))
        .stdout(predicate::str::contains("\"phase\": \"pre\""))
        .stdout(predicate::str::contains("\"passed\": false"))
        .stdout(predicate::str::contains("\"code\": \"preflight-gates\""))
        .stdout(predicate::str::contains(
            "\"code\": \"ambiguous-definition\"",
        ));
}

#[test]
fn cli_verifies_post_rename_refactor_invariants_for_agents() {
    let dir = fresh_temp_dir("refactor verify");
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
    cmd.args(["refactor", "verify"])
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
        .stdout(predicate::str::contains("\"target_kind\": \"callable\""))
        .stdout(predicate::str::contains("\"code\": \"old-symbol-removed\""))
        .stdout(predicate::str::contains("\"code\": \"new-symbol-present\""))
        .stdout(predicate::str::contains(
            "\"code\": \"new-symbol-signature-compatible\"",
        ));
}

#[test]
fn cli_reports_missing_new_symbol_for_post_rename_verification() {
    let dir = fresh_temp_dir("refactor verify-post-rename-missing-new-symbol");
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
    cmd.args(["refactor", "verify"])
        .arg("--symbol")
        .arg("render-pane")
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
        .stdout(predicate::str::contains("\"passed\": false"))
        .stdout(predicate::str::contains(
            "\"code\": \"new-symbol-required\"",
        ))
        .stdout(predicate::str::contains(
            "Post-rename verification requires --new-symbol",
        ));
}

#[test]
fn cli_skips_signature_verification_for_symbol_macros() {
    let dir = fresh_temp_dir("refactor verify-symbol-macro");
    let file = dir.join("core.lisp");
    fs::write(
        &file,
        r#"(defpackage #:demo.core (:use #:cl))
(in-package #:demo.core)
(define-symbol-macro paint-pane current-pane)
(defun caller () paint-pane)
"#,
    )
    .expect("write verify refactor fixture");

    let mut cmd = paredit();
    let assert = cmd
        .args(["refactor", "verify"])
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
        .stdout(predicate::str::contains("\"passed\": true"))
        .stdout(predicate::str::contains(
            "\"target_kind\": \"symbol_macro\"",
        ))
        .stdout(predicate::str::contains("\"code\": \"old-symbol-removed\""))
        .stdout(predicate::str::contains("\"code\": \"new-symbol-present\""));

    let stdout = String::from_utf8(assert.get_output().stdout.clone())
        .expect("verify refactor output should be valid utf-8");
    assert!(
        !stdout.contains("\"code\": \"new-symbol-signature-compatible\""),
        "symbol-macro verification should not require signature compatibility: {stdout}"
    );
}

#[test]
fn cli_skips_signature_verification_for_compiler_macros() {
    let dir = fresh_temp_dir("refactor verify-compiler-macro");
    let file = dir.join("core.lisp");
    fs::write(
        &file,
        r#"(defpackage #:demo.core (:use #:cl))
(in-package #:demo.core)
(define-compiler-macro paint-pane (pane) `(draw-pane ,pane))
(defun caller () (paint-pane window))
"#,
    )
    .expect("write verify refactor fixture");

    let mut cmd = paredit();
    let assert = cmd
        .args(["refactor", "verify"])
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
        .stdout(predicate::str::contains("\"passed\": true"))
        .stdout(predicate::str::contains(
            "\"target_kind\": \"compiler_macro\"",
        ))
        .stdout(predicate::str::contains("\"code\": \"old-symbol-removed\""))
        .stdout(predicate::str::contains("\"code\": \"new-symbol-present\""));

    let stdout = String::from_utf8(assert.get_output().stdout.clone())
        .expect("verify refactor output should be valid utf-8");
    assert!(
        !stdout.contains("\"code\": \"new-symbol-signature-compatible\""),
        "compiler-macro verification should not require signature compatibility: {stdout}"
    );
}

#[test]
fn cli_skips_signature_verification_for_setf_expanders() {
    let dir = fresh_temp_dir("refactor verify-setf-expander");
    let file = dir.join("core.lisp");
    fs::write(
        &file,
        r#"(defpackage #:demo.core (:use #:cl))
(in-package #:demo.core)
(define-setf-expander paint-pane (pane) (values nil nil '(store) '(writer store) '(reader pane)))
(defun caller (item) (setf (paint-pane item) 1))
"#,
    )
    .expect("write verify refactor fixture");

    let mut cmd = paredit();
    let assert = cmd
        .args(["refactor", "verify"])
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
        .stdout(predicate::str::contains("\"passed\": true"))
        .stdout(predicate::str::contains(
            "\"target_kind\": \"setf_expander\"",
        ))
        .stdout(predicate::str::contains("\"code\": \"old-symbol-removed\""))
        .stdout(predicate::str::contains("\"code\": \"new-symbol-present\""));

    let stdout = String::from_utf8(assert.get_output().stdout.clone())
        .expect("verify refactor output should be valid utf-8");
    assert!(
        !stdout.contains("\"code\": \"new-symbol-signature-compatible\""),
        "setf-expander verification should not require signature compatibility: {stdout}"
    );
}

#[test]
fn cli_skips_signature_verification_for_defsetf_targets() {
    let dir = fresh_temp_dir("refactor verify-defsetf");
    let file = dir.join("core.lisp");
    fs::write(
        &file,
        r#"(defpackage #:demo.core (:use #:cl))
(in-package #:demo.core)
(defsetf paint-pane accessor)
(defun caller (item) (setf (paint-pane item) 1))
"#,
    )
    .expect("write verify refactor fixture");

    let mut cmd = paredit();
    let assert = cmd
        .args(["refactor", "verify"])
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
        .stdout(predicate::str::contains("\"passed\": true"))
        .stdout(predicate::str::contains(
            "\"target_kind\": \"setf_expander\"",
        ))
        .stdout(predicate::str::contains("\"code\": \"old-symbol-removed\""))
        .stdout(predicate::str::contains("\"code\": \"new-symbol-present\""));

    let stdout = String::from_utf8(assert.get_output().stdout.clone())
        .expect("verify refactor output should be valid utf-8");
    assert!(
        !stdout.contains("\"code\": \"new-symbol-signature-compatible\""),
        "defsetf verification should not require signature compatibility: {stdout}"
    );
}

#[test]
fn cli_skips_signature_verification_for_signature_operations_on_macros() {
    let dir = fresh_temp_dir("refactor verify-signature-macro");
    let file = dir.join("core.lisp");
    fs::write(
        &file,
        r#"(defpackage #:demo.core (:use #:cl))
(in-package #:demo.core)
(defmacro paint-pane (pane) `(draw-pane ,pane))
(defun caller () (paint-pane window))
"#,
    )
    .expect("write verify refactor fixture");

    let mut cmd = paredit();
    let assert = cmd
        .args(["refactor", "verify"])
        .arg("--symbol")
        .arg("paint-pane")
        .arg("--operation")
        .arg("signature")
        .arg("--phase")
        .arg("post")
        .arg("--output")
        .arg("json")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"operation\": \"signature\""))
        .stdout(predicate::str::contains("\"target_kind\": \"macro\""));

    let stdout = String::from_utf8(assert.get_output().stdout.clone())
        .expect("verify refactor output should be valid utf-8");
    assert!(
        !stdout.contains("\"code\": \"signature-compatible\""),
        "macro signature verification should not require callable signature compatibility: {stdout}"
    );
}

#[test]
fn cli_skips_signature_verification_for_signature_operations_on_define_method_combination() {
    let dir = fresh_temp_dir("refactor verify-signature-define-method-combination");
    let file = dir.join("core.lisp");
    fs::write(
        &file,
        r#"(defpackage #:demo.core (:use #:cl))
(in-package #:demo.core)
(define-method-combination render-combination (pane theme) ((primary *)) (list pane theme primary))
(defun caller () (render-combination pane theme))
"#,
    )
    .expect("write verify refactor fixture");

    let mut cmd = paredit();
    let assert = cmd
        .args(["refactor", "verify"])
        .arg("--symbol")
        .arg("render-combination")
        .arg("--operation")
        .arg("signature")
        .arg("--phase")
        .arg("post")
        .arg("--output")
        .arg("json")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"operation\": \"signature\""))
        .stdout(predicate::str::contains("\"target_kind\": \"macro\""));

    let stdout = String::from_utf8(assert.get_output().stdout.clone())
        .expect("verify refactor output should be valid utf-8");
    assert!(
        !stdout.contains("\"code\": \"signature-compatible\""),
        "define-method-combination signature verification should not require callable signature compatibility: {stdout}"
    );
}

#[test]
fn cli_verifies_post_move_refactor_for_symbol_macros_without_new_symbol() {
    let dir = fresh_temp_dir("refactor verify-move-symbol-macro");
    let file = dir.join("core.lisp");
    fs::write(
        &file,
        r#"(defpackage #:demo.core (:use #:cl))
(in-package #:demo.core)
(define-symbol-macro paint-pane current-pane)
(defun caller () paint-pane)
"#,
    )
    .expect("write verify refactor fixture");

    let mut cmd = paredit();
    let _assert = cmd
        .args(["refactor", "verify"])
        .arg("--symbol")
        .arg("paint-pane")
        .arg("--operation")
        .arg("move")
        .arg("--phase")
        .arg("post")
        .arg("--output")
        .arg("json")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"operation\": \"move\""))
        .stdout(predicate::str::contains("\"phase\": \"post\""))
        .stdout(predicate::str::contains(
            "\"target_kind\": \"symbol_macro\"",
        ))
        .stdout(predicate::str::contains(
            "\"code\": \"moved-symbol-present\"",
        ))
        .stdout(predicate::str::contains("\"passed\": true"));
}
