use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::PathBuf;

#[allow(clippy::unwrap_used)]
fn paredit() -> Command {
    Command::cargo_bin("paredit").expect("binary")
}

#[test]
fn check_accepts_valid_input() {
    let mut cmd = paredit();
    cmd.arg("check")
        .write_stdin("(defun add (x y) (+ x y))")
        .assert()
        .success()
        .stdout("ok\n");
}

#[test]
fn check_rejects_invalid_input() {
    let mut cmd = paredit();
    cmd.arg("check")
        .write_stdin("(defun add (x y)")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unclosed list"));
}

#[test]
fn cli_selects_by_path() {
    let mut cmd = paredit();
    cmd.args(["select", "--path", "0.2"])
        .write_stdin("(defun add (x y) (+ x y))")
        .assert()
        .success()
        .stdout("(x y)");
}

#[test]
fn cli_replaces_by_path() {
    let mut cmd = paredit();
    cmd.args(["replace", "--path", "0.1", "--with", "sum"])
        .write_stdin("(defun add (x y) (+ x y))")
        .assert()
        .success()
        .stdout("(defun sum (x y) (+ x y))");
}

#[test]
fn cli_detects_emacs_lisp_from_extension() {
    let mut cmd = paredit();
    cmd.args(["dialect", "--file", "tests/fixtures/sample.el"])
        .assert()
        .success()
        .stdout("emacs-lisp\n");
}

#[test]
fn cli_prints_definition_outline() {
    let mut cmd = paredit();
    cmd.args(["outline", "--file", "tests/fixtures/sample.el"])
        .assert()
        .success()
        .stdout(predicate::str::contains("0\t0..37\tdefun\ttrue"));
}

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
fn cli_plans_extract_function_for_common_lisp() {
    let mut cmd = paredit();
    cmd.args([
        "extract-function",
        "--path",
        "0.3",
        "--name",
        "compute-sum",
        "--output",
        "json",
    ])
    .write_stdin("(defun render () (+ 1 2))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"dialect\": \"unknown\""))
    .stdout(predicate::str::contains("\"call\": \"(compute-sum)\""))
    .stdout(predicate::str::contains(
        "\"definition\": \"(defun compute-sum () (+ 1 2))\"",
    ))
    .stdout(predicate::str::contains(
        "(defun render () (compute-sum))\\n\\n(defun compute-sum () (+ 1 2))",
    ));
}

#[test]
fn cli_writes_extract_function_for_emacs_lisp_file() {
    let dir = fresh_temp_dir("extract");
    let elisp_file = dir.join("render.el");
    fs::write(&elisp_file, "(defun render () (+ 1 2))\n").expect("write elisp fixture");

    let mut cmd = paredit();
    cmd.arg("extract-function")
        .arg("--file")
        .arg(&elisp_file)
        .arg("--path")
        .arg("0.3")
        .arg("--name")
        .arg("render-sum")
        .arg("--write")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"dialect\": \"emacs-lisp\""))
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(elisp_file).expect("read extracted elisp"),
        "(defun render () (render-sum))\n\n(defun render-sum () (+ 1 2))\n"
    );
}

#[test]
fn cli_rejects_extract_function_write_without_file() {
    let mut cmd = paredit();
    cmd.args([
        "extract-function",
        "--path",
        "0.3",
        "--name",
        "compute-sum",
        "--write",
    ])
    .write_stdin("(defun render () (+ 1 2))")
    .assert()
    .failure()
    .stderr(predicate::str::contains("--write requires --file"));
}

#[test]
fn cli_plans_introduce_let_for_common_lisp() {
    let mut cmd = paredit();
    cmd.args([
        "introduce-let",
        "--path",
        "0.3.1",
        "--name",
        "product",
        "--output",
        "json",
    ])
    .write_stdin("(defun render () (+ (* width height) margin))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"dialect\": \"unknown\""))
    .stdout(predicate::str::contains(
        "\"binding_value\": \"(* width height)\"",
    ))
    .stdout(predicate::str::contains(
        "\"replacement\": \"(let ((product (* width height))) (+ product margin))\"",
    ))
    .stdout(predicate::str::contains(
        "(defun render () (let ((product (* width height))) (+ product margin)))",
    ));
}

#[test]
fn cli_writes_introduce_let_for_emacs_lisp_file() {
    let dir = fresh_temp_dir("introduce-let");
    let elisp_file = dir.join("render.el");
    fs::write(
        &elisp_file,
        "(defun render () (+ (* width height) margin))\n",
    )
    .expect("write elisp fixture");

    let mut cmd = paredit();
    cmd.arg("introduce-let")
        .arg("--file")
        .arg(&elisp_file)
        .arg("--path")
        .arg("0.3.1")
        .arg("--name")
        .arg("product")
        .arg("--write")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"dialect\": \"emacs-lisp\""))
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(elisp_file).expect("read introduced let elisp"),
        "(defun render () (let ((product (* width height))) (+ product margin)))\n"
    );
}

#[test]
fn cli_rejects_introduce_let_write_without_file() {
    let mut cmd = paredit();
    cmd.args([
        "introduce-let",
        "--path",
        "0.3.1",
        "--name",
        "product",
        "--write",
    ])
    .write_stdin("(defun render () (+ (* width height) margin))")
    .assert()
    .failure()
    .stderr(predicate::str::contains("--write requires --file"));
}

#[test]
fn cli_prints_agent_report_json() {
    let mut cmd = paredit();
    cmd.args([
        "agent-report",
        "--file",
        "tests/fixtures/system.asd",
        "--output",
        "json",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("\"label\": \"common-lisp\""))
    .stdout(predicate::str::contains("\"definitionLike\": true"));
}

fn fresh_temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "paredit-cli-{name}-{}-{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}
