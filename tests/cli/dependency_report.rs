use super::*;

#[test]
fn cli_reports_dependency_inventory_for_agent_planning() {
    let dir = fresh_temp_dir("dependency-report");
    let file = dir.join("demo.asd");
    fs::write(
        &file,
        r#"(defpackage #:demo.core
  (:use #:cl #:alexandria)
  (:import-from #:uiop #:pathname-directory-pathname))
(in-package #:demo.core)
(asdf:defsystem #:demo
  :depends-on (#:alexandria "cl-ppcre")
  :components ((:file "package") (:file "core")))
(require :swank)
(provide 'demo.core)
(load "extra.lisp")
(defun render ()
  (alexandria:when-let ((x 1))
    (uiop:ensure-directory-pathname x)))
"#,
    )
    .expect("write dependency fixture");

    let mut cmd = paredit();
    cmd.arg("dependency-report")
        .arg("--output")
        .arg("json")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"file_count\": 1"))
        .stdout(predicate::str::contains("\"dependency_count\""))
        .stdout(predicate::str::contains("\"kind\": \"defpackage-use\""))
        .stdout(predicate::str::contains(
            "\"kind\": \"defpackage-import-from\"",
        ))
        .stdout(predicate::str::contains("\"kind\": \"asdf-depends-on\""))
        .stdout(predicate::str::contains("\"kind\": \"asdf-component\""))
        .stdout(predicate::str::contains("\"kind\": \"require\""))
        .stdout(predicate::str::contains("\"kind\": \"provide\""))
        .stdout(predicate::str::contains("\"kind\": \"load\""))
        .stdout(predicate::str::contains("\"kind\": \"qualified-symbol\""))
        .stdout(predicate::str::contains("\"target\": \"alexandria\""))
        .stdout(predicate::str::contains("\"target\": \"uiop\""));
}

#[test]
fn cli_excludes_symbol_macrolet_binding_names_and_shadowed_body_references_from_dependencies() {
    let dir = fresh_temp_dir("dependency-report-symbol-macrolet");
    let file = dir.join("symbol-macrolet.lisp");
    fs::write(
        &file,
        "(defun caller ()\n  (cl:symbol-macrolet ((cl-user:helper (uiop:ensure-pathname x)))\n    cl-user:helper))\n",
    )
    .expect("write symbol-macrolet dependency fixture");

    let mut cmd = paredit();
    cmd.arg("dependency-report")
        .arg("--output")
        .arg("json")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"dependency_count\": 2"))
        .stdout(predicate::str::contains("\"target\": \"cl\""))
        .stdout(predicate::str::contains("\"target\": \"uiop\""))
        .stdout(predicate::str::contains("\"target\": \"cl-user\"").not());
}

#[test]
fn cli_respects_local_callable_scopes_in_common_lisp_dependency_report() {
    let dir = fresh_temp_dir("dependency-report-local-callables");
    let file = dir.join("local-callables.lisp");
    fs::write(
        &file,
        "(defun caller ()\n  (cl:labels ((cl-user:helper (x)\n                (cl-user:helper x)\n                (uiop:ensure-pathname x)))\n    (cl-user:helper value))\n  (cl:flet ((cl-user:helper (x)\n              (cl-user:helper x)\n              (uiop:ensure-pathname x)))\n    (cl-user:helper value))\n  (cl:macrolet ((cl-user:helper (x) (uiop:ensure-pathname x)))\n    (cl-user:helper value)))\n",
    )
    .expect("write local-callable dependency fixture");

    let mut cmd = paredit();
    cmd.arg("dependency-report")
        .arg("--output")
        .arg("json")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"target\": \"cl\""))
        .stdout(predicate::str::contains("\"target\": \"uiop\""))
        .stdout(predicate::str::contains("\"target\": \"cl-user\""));
}

#[test]
fn cli_reports_quasiquote_unquotes_and_quote_wrapped_unquotes_as_dependencies() {
    // `,uiop:ensure-pathname` and `,@(cl-user:helper value)` are ordinary
    // unquoted/spliced live code. `',cl-user:quoted` — a quote wrapping an
    // unquote, the idiom for splicing a computed value as a literal into
    // generated code — is live too: the quote does not block traversal
    // once already inside the quasiquote, so the nested unquote's
    // reference to `cl-user:quoted` is still a real dependency. In every
    // case the reported target is the package prefix (`cl-user`/`uiop`),
    // never the bare symbol name (`quoted`) after the qualifier.
    let dir = fresh_temp_dir("dependency-report-quasiquote");
    let file = dir.join("quasiquote.lisp");
    fs::write(
        &file,
        "(defun caller ()\n  `(list ',cl-user:quoted ,uiop:ensure-pathname ,@(cl-user:helper value)))\n",
    )
    .expect("write quasiquote dependency fixture");

    let mut cmd = paredit();
    cmd.arg("dependency-report")
        .arg("--output")
        .arg("json")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"dependency_count\": 3"))
        .stdout(predicate::str::contains("\"target\": \"uiop\""))
        .stdout(predicate::str::contains("\"target\": \"cl-user\""))
        .stdout(predicate::str::contains("\"target\": \"quoted\"").not());
}

#[test]
fn cli_skips_reader_eval_bodies_in_dependency_analysis() {
    let dir = fresh_temp_dir("dependency-report-reader-eval");
    let file = dir.join("reader-eval.lisp");
    fs::write(
        &file,
        "(defun caller () #.(progn (cl-user:helper) (uiop:ensure-pathname value)) (cl-user:helper value))\n",
    )
    .expect("write reader-eval dependency fixture");

    let mut cmd = paredit();
    cmd.arg("dependency-report")
        .arg("--output")
        .arg("json")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"dependency_count\": 1"))
        .stdout(predicate::str::contains("\"target\": \"cl-user\""))
        .stdout(predicate::str::contains("\"target\": \"uiop\"").not());
}
