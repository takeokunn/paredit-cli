use super::*;

#[test]
fn plans_common_lisp_dry_run_and_replaces_only_the_selection() {
    let mut cmd = paredit();
    cmd.args([
        "refactor",
        "extract-constant",
        "--dialect",
        "common-lisp",
        "--path",
        "0.3.1",
        "--name",
        "+answer+",
    ])
    .write_stdin("(defun f () (list (+ 40 2) (+ 40 2)))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"written\": false"))
    .stdout(predicate::str::contains(
        "(defun f () (list +answer+ (+ 40 2)))\\n\\n(defconstant +answer+ (+ 40 2))",
    ));
}

#[test]
fn plans_common_lisp_vector_literal_constant() {
    let mut cmd = paredit();
    cmd.args([
        "refactor",
        "extract-constant",
        "--dialect",
        "common-lisp",
        "--path",
        "0.1",
        "--name",
        "+helper-vector+",
    ])
    .write_stdin("(render #(helper value))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"written\": false"))
    .stdout(predicate::str::contains(
        "(render +helper-vector+)\\n\\n(defconstant +helper-vector+ #(helper value))",
    ));
}

#[test]
fn plans_emacs_lisp_from_byte_offset() {
    let input = "(defun f () (+ 40 2))";
    let offset = input.find("40").unwrap();
    let mut cmd = paredit();
    cmd.args([
        "refactor",
        "extract-constant",
        "--dialect",
        "emacs-lisp",
        "--at",
        &offset.to_string(),
        "--name",
        "answer",
        "--output",
        "text",
    ])
    .write_stdin(input)
    .assert()
    .success()
    .stdout(predicate::str::contains("path\t0.3.1"))
    .stdout(predicate::str::contains("definition\t(defconst answer 40)"));
}

#[test]
fn inserts_before_a_top_level_anchor() {
    let mut cmd = paredit();
    cmd.args([
        "refactor",
        "extract-constant",
        "--dialect",
        "common-lisp",
        "--path",
        "1.3",
        "--name",
        "+answer+",
        "--insert",
        "before",
        "--anchor-path",
        "0",
    ])
    .write_stdin("(in-package :app)\n(defun f () (+ 40 2))")
    .assert()
    .success()
    .stdout(predicate::str::contains(
        "(defconstant +answer+ (+ 40 2))\\n\\n(in-package :app)\\n(defun f () +answer+)",
    ));
}

#[test]
fn writes_emacs_lisp_file() {
    let dir = fresh_temp_dir("extract-constant");
    let file = dir.join("sample.el");
    fs::write(&file, "(defun f () (+ 40 2))\n").unwrap();

    let mut cmd = paredit();
    cmd.args([
        "refactor",
        "extract-constant",
        "--file",
        file.to_str().unwrap(),
        "--path",
        "0.3",
        "--name",
        "answer",
        "--write",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(&file).unwrap(),
        "(defun f () answer)\n\n(defconst answer (+ 40 2))\n"
    );
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn rejects_quote_and_quasiquote_contexts() {
    for (input, path) in [
        ("(defun f () '(+ 40 2))", "0.3.1"),
        ("(defun f () `(+ 40 2))", "0.3.1"),
    ] {
        let mut cmd = paredit();
        cmd.args([
            "refactor",
            "extract-constant",
            "--dialect",
            "common-lisp",
            "--path",
            path,
            "--name",
            "+answer+",
        ])
        .write_stdin(input)
        .assert()
        .failure()
        .stderr(predicate::str::contains("quote or quasiquote"));
    }
}

#[test]
fn rejects_structurally_invalid_targets() {
    for (path, message) in [("0", "entire top-level form"), ("0.0", "definition head")] {
        let mut cmd = paredit();
        cmd.args([
            "refactor",
            "extract-constant",
            "--dialect",
            "common-lisp",
            "--path",
            path,
            "--name",
            "+answer+",
        ])
        .write_stdin("(defun f () (+ 40 2))")
        .assert()
        .failure()
        .stderr(predicate::str::contains(message));
    }
}

#[test]
fn rejects_invalid_cli_combinations_and_dialect() {
    let mut missing_anchor = paredit();
    missing_anchor
        .args([
            "refactor",
            "extract-constant",
            "--dialect",
            "common-lisp",
            "--path",
            "0.3",
            "--name",
            "+answer+",
            "--insert",
            "after",
        ])
        .write_stdin("(defun f () 42)")
        .assert()
        .failure()
        .stderr(predicate::str::contains("requires --anchor-path"));

    let mut unsupported = paredit();
    unsupported
        .args([
            "refactor",
            "extract-constant",
            "--dialect",
            "clojure",
            "--path",
            "0.1",
            "--name",
            "answer",
        ])
        .write_stdin("(+ 40 2)")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "supports only common-lisp and emacs-lisp",
        ));
}
