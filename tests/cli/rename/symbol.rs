use super::*;

#[test]
fn cli_finds_symbol_atoms_without_string_or_comment_matches() {
    let mut cmd = paredit();
    cmd.args(["find-symbol", "--symbol", "foo"])
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
    cmd.args(["rename-symbol", "--from", "foo", "--to", "bar"])
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
    cmd.arg("rename-symbols")
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
    cmd.arg("rename-symbols")
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
    cmd.args(["rename-symbol", "--from", "foo", "--to", "bar"])
        .write_stdin("(define-condition foo (error) ())\n(error 'foo)")
        .assert()
        .success()
        .stdout("(define-condition bar (error) ())\n(error 'bar)");
}

#[test]
fn cli_refactor_preview_renames_bare_quoted_symbol_designator_in_symbol_and_function_mode() {
    for mode in ["symbol", "function"] {
        let dir = fresh_temp_dir(&format!("quoted-designator-{mode}"));
        let lisp_file = dir.join("core.lisp");
        fs::write(
            &lisp_file,
            "(define-condition foo (error) ())\n(defun signal-foo () (error 'foo))",
        )
        .expect("write lisp fixture");

        let mut cmd = paredit();
        cmd.arg("refactor")
            .arg("preview")
            .arg("--from")
            .arg("foo")
            .arg("--to")
            .arg("bar")
            .arg("--mode")
            .arg(mode)
            .arg("--write")
            .arg("--output")
            .arg("json")
            .arg(&lisp_file)
            .assert()
            .success()
            .stdout(predicate::str::contains("\"written\": true"));

        let rewritten = fs::read_to_string(&lisp_file).expect("read rewritten lisp");
        assert!(
            rewritten.contains("(error 'bar)"),
            "mode {mode}: quoted designator was not renamed: {rewritten}"
        );
        assert!(
            !rewritten.contains("'foo"),
            "mode {mode}: stale quoted reference left behind: {rewritten}"
        );

        // The rewritten file must still parse; `find-symbol` is a cheap way
        // to force the CLI to reparse it end-to-end.
        let mut find_cmd = paredit();
        find_cmd
            .args(["find-symbol", "--symbol", "bar", "--file"])
            .arg(&lisp_file)
            .assert()
            .success();
    }
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
    cmd.arg("rename-symbols")
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
