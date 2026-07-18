use super::*;

#[test]
fn cli_finds_symbol_atoms_without_string_or_comment_matches() {
    let mut cmd = paredit();
    cmd.args(["inspect", "find-symbol", "--symbol", "foo"])
        .write_stdin("(defun foo (foo) \"foo\" ; foo\n  foo)")
        .assert()
        .success()
        .stdout(predicate::str::contains("0.1\t7..10\tfoo"))
        .stdout(predicate::str::contains("0.2.0\t12..15\tfoo"))
        .stdout(predicate::str::contains("0.4\t31..34\tfoo"));
}

#[test]
fn cli_renames_symbol_atoms_without_string_or_comment_matches() {
    let mut cmd = paredit();
    cmd.args(["refactor", "rename-symbol", "--from", "foo", "--to", "bar"])
        .write_stdin("(defun foo (foo) \"foo\" ; foo\n  foo)")
        .assert()
        .success()
        .stdout("(defun bar (bar) \"foo\" ; foo\n  bar)");
}

#[test]
fn cli_plans_multi_file_symbol_rename() {
    let dir = fresh_temp_dir("plan");
    let lisp_file = dir.join("core.lisp");
    let elisp_file = dir.join("init.el");
    fs::write(&lisp_file, "(defun old-name () old-name)").expect("write lisp fixture");
    fs::write(
        &elisp_file,
        "(defun old-name () (message \"old-name\") old-name) ; old-name",
    )
    .expect("write elisp fixture");

    let mut cmd = paredit();
    cmd.arg("refactor")
        .arg("rename-symbols")
        .arg("--from")
        .arg("old-name")
        .arg("--to")
        .arg("new-name")
        .arg(&lisp_file)
        .arg(&elisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"write\": false"))
        .stdout(predicate::str::contains("\"dialect\": \"common-lisp\""))
        .stdout(predicate::str::contains("\"dialect\": \"emacs-lisp\""))
        .stdout(predicate::str::contains("\"count\": 2"));
}

#[test]
fn cli_writes_multi_file_symbol_rename_without_string_or_comment_matches() {
    let dir = fresh_temp_dir("write");
    let lisp_file = dir.join("core.lisp");
    let elisp_file = dir.join("init.el");
    fs::write(&lisp_file, "(defun old-name () old-name)").expect("write lisp fixture");
    fs::write(
        &elisp_file,
        "(defun old-name () (message \"old-name\") old-name) ; old-name",
    )
    .expect("write elisp fixture");

    let mut cmd = paredit();
    cmd.arg("refactor")
        .arg("rename-symbols")
        .arg("--from")
        .arg("old-name")
        .arg("--to")
        .arg("new-name")
        .arg("--write")
        .arg(&lisp_file)
        .arg(&elisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(lisp_file).expect("read rewritten lisp"),
        "(defun new-name () new-name)"
    );
    assert_eq!(
        fs::read_to_string(elisp_file).expect("read rewritten elisp"),
        "(defun new-name () (message \"old-name\") new-name) ; old-name"
    );
}

#[test]
fn cli_renames_bare_quoted_symbol_designator_references() {
    // `'foo` is the standard Common Lisp idiom for referencing a symbol as
    // data (condition/class designators passed to `error`, `typep`,
    // `make-instance`, etc). A rename that skipped it would silently leave
    // a dangling reference to a definition that no longer exists.
    let mut cmd = paredit();
    cmd.args(["refactor", "rename-symbol", "--from", "foo", "--to", "bar"])
        .write_stdin("(define-condition foo (error) ())\n(error 'foo)")
        .assert()
        .success()
        .stdout("(define-condition bar (error) ())\n(error 'bar)");
}

#[test]
fn cli_refactor_preview_symbol_mode_renames_bare_quoted_data() {
    let dir = fresh_temp_dir("quoted-data-symbol-mode");
    let lisp_file = dir.join("core.lisp");
    fs::write(
        &lisp_file,
        "(define-condition foo (error) ())\n(defun signal-foo () (error 'foo))",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.args([
        "refactor", "preview", "--from", "foo", "--to", "bar", "--mode", "symbol", "--write",
        "--output", "json",
    ])
    .arg(&lisp_file)
    .assert()
    .success()
    .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(&lisp_file).expect("read rewritten lisp"),
        "(define-condition bar (error) ())\n(defun signal-foo () (error 'bar))"
    );
}

#[test]
fn cli_refactor_preview_function_mode_preserves_bare_quote_and_renames_explicit_designators() {
    let dir = fresh_temp_dir("quoted-data-function-mode");
    let lisp_file = dir.join("core.lisp");
    fs::write(
        &lisp_file,
        "(defun foo () nil)\n(defun caller () (list 'foo #'foo (function foo)))",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.args([
        "refactor", "preview", "--from", "foo", "--to", "bar", "--mode", "function", "--write",
        "--output", "json",
    ])
    .arg(&lisp_file)
    .assert()
    .success()
    .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(&lisp_file).expect("read rewritten lisp"),
        "(defun bar () nil)\n(defun caller () (list 'foo #'bar (function bar)))"
    );
}

#[test]
fn cli_renames_unqualified_occurrences_of_package_qualified_common_lisp_symbol() {
    let dir = fresh_temp_dir("rename-qualified-common-lisp-symbol");
    let lisp_file = dir.join("core.lisp");
    fs::write(
        &lisp_file,
        "(defun cl-user:old-name () old-name)\n(old-name cl-user:old-name)",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("refactor")
        .arg("rename-symbols")
        .arg("--from")
        .arg("cl-user:old-name")
        .arg("--to")
        .arg("new-name")
        .arg("--write")
        .arg(&lisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"count\": 4"))
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(lisp_file).expect("read rewritten lisp"),
        "(defun new-name () new-name)\n(new-name new-name)"
    );
}

#[test]
fn cli_rename_symbol_fail_on_no_change_gate_fails_when_symbol_absent() {
    let mut cmd = paredit();
    cmd.args([
        "refactor",
        "rename-symbol",
        "--from",
        "missing-name",
        "--to",
        "new-name",
        "--fail-on-no-change",
    ])
    .write_stdin("(defun keep (x) x)")
    .assert()
    .failure()
    .stderr(predicate::str::contains(
        "rename-symbol policy failed: no occurrence changed",
    ));
}

#[test]
fn cli_rename_symbol_fail_on_no_change_gate_passes_on_rewrite() {
    let mut cmd = paredit();
    cmd.args([
        "refactor",
        "rename-symbol",
        "--from",
        "keep",
        "--to",
        "hold",
        "--fail-on-no-change",
    ])
    .write_stdin("(defun keep (x) x)")
    .assert()
    .success()
    .stdout(predicate::str::contains("(defun hold (x) x)"));
}
