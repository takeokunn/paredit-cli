use super::*;
#[test]
fn cli_removes_unused_block() {
    paredit()
        .args([
            "refactor",
            "remove-unused-block",
            "--dialect",
            "common-lisp",
            "--path",
            "0.1",
            "--name",
            "out",
        ])
        .write_stdin("(progn (block out (first) (second)))")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"rewritten\": \"(progn (progn (first) (second)))\"",
        ));
}
#[test]
fn cli_removes_unused_integer_tag() {
    paredit()
        .args([
            "refactor",
            "remove-unused-tag",
            "--dialect",
            "common-lisp",
            "--path",
            "0",
            "--name",
            "10",
        ])
        .write_stdin("(tagbody 10 (print 1))")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"rewritten\": \"(tagbody  (print 1))\"",
        ));
}
#[test]
fn cli_rejects_referenced_tag() {
    paredit()
        .args([
            "refactor",
            "remove-unused-tag",
            "--dialect",
            "common-lisp",
            "--path",
            "0",
            "--name",
            "start",
        ])
        .write_stdin("(tagbody start (go start))")
        .assert()
        .failure()
        .stderr(predicate::str::contains("matching go reference"));
}
