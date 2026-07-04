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
