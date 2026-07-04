use super::*;

#[test]
fn cli_reports_workspace_inventory_from_directory_roots() {
    let dir = fresh_temp_dir("workspace-report");
    let src_dir = dir.join("src");
    fs::create_dir_all(&src_dir).expect("create source dir");

    let lisp_file = src_dir.join("core.lisp");
    let elisp_file = src_dir.join("init.el");
    let scheme_file = src_dir.join("broken.scm");
    let unknown_file = src_dir.join("notes.txt");
    fs::write(
        &lisp_file,
        "(in-package #:demo)\n(defun area (width height) (* width height))\n",
    )
    .expect("write common lisp fixture");
    fs::write(&elisp_file, "(defun draw () (area 5 6))\n").expect("write elisp fixture");
    fs::write(&scheme_file, "(define (broken x)").expect("write scheme fixture");
    fs::write(&unknown_file, "not lisp").expect("write unknown fixture");

    let mut cmd = paredit();
    cmd.arg("workspace-report")
        .arg("--output")
        .arg("json")
        .arg(&dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"file_count\": 3"))
        .stdout(predicate::str::contains("\"parsed_count\": 2"))
        .stdout(predicate::str::contains("\"parse_error_count\": 1"))
        .stdout(predicate::str::contains("\"definition_count\": 2"))
        .stdout(predicate::str::contains("\"unknown\": 1"))
        .stdout(predicate::str::contains("\"dialect\": \"common-lisp\""))
        .stdout(predicate::str::contains("\"dialect\": \"emacs-lisp\""))
        .stdout(predicate::str::contains("\"dialect\": \"scheme\""))
        .stdout(predicate::str::contains("\"status\": \"parse-error\""));
}
