use super::*;

#[test]
fn cli_reports_workspace_inventory_from_directory_roots() {
    let dir = fresh_temp_dir("workspace report");
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
    cmd.args(["inspect", "workspace"])
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

#[test]
fn cli_reports_workspace_inventory_with_include_flags() {
    let dir = fresh_temp_dir("workspace report-include-flags");
    let hidden_dir = dir.join(".hidden");
    let generated_dir = dir.join("target");
    fs::create_dir_all(&hidden_dir).expect("create hidden dir");
    fs::create_dir_all(&generated_dir).expect("create generated dir");

    let lisp_file = dir.join("core.lisp");
    let hidden_file = hidden_dir.join("secret.el");
    let generated_file = generated_dir.join("generated.scm");
    let unknown_file = dir.join("notes.txt");
    fs::write(&lisp_file, "(defun area (width height) (* width height))\n")
        .expect("write common lisp fixture");
    fs::write(&hidden_file, "(defun secret () (message \"hidden\"))\n")
        .expect("write hidden fixture");
    fs::write(&generated_file, "(define generated #t)\n").expect("write generated fixture");
    fs::write(&unknown_file, "plain-text-note\n").expect("write unknown fixture");

    let mut cmd = paredit();
    cmd.args(["inspect", "workspace"])
        .arg("--include-hidden")
        .arg("--include-generated")
        .arg("--include-unknown")
        .arg("--output")
        .arg("json")
        .arg(&dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"file_count\": 4"))
        .stdout(predicate::str::contains("\"parsed_count\": 4"))
        .stdout(predicate::str::contains("\"parse_error_count\": 0"))
        .stdout(predicate::str::contains("\"hidden\": 0"))
        .stdout(predicate::str::contains("\"generated\": 0"))
        .stdout(predicate::str::contains("\"unknown\": 0"))
        .stdout(predicate::str::contains("\"dialect\": \"common-lisp\""))
        .stdout(predicate::str::contains("\"dialect\": \"emacs-lisp\""))
        .stdout(predicate::str::contains("\"dialect\": \"scheme\""))
        .stdout(predicate::str::contains("\"dialect\": \"unknown\""));
}

#[test]
fn cli_reports_workspace_inventory_with_max_depth_limit() {
    let dir = fresh_temp_dir("workspace report-max-depth");
    let nested_dir = dir.join("nested");
    fs::create_dir_all(&nested_dir).expect("create nested dir");

    let root_file = dir.join("root.lisp");
    let nested_file = nested_dir.join("deep.lisp");
    fs::write(&root_file, "(defun root () t)\n").expect("write root fixture");
    fs::write(&nested_file, "(defun deep () t)\n").expect("write nested fixture");

    let mut cmd = paredit();
    cmd.args(["inspect", "workspace"])
        .arg("--max-depth")
        .arg("1")
        .arg("--output")
        .arg("json")
        .arg(&dir)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"file_count\": 1"))
        .stdout(predicate::str::contains("\"parsed_count\": 1"))
        .stdout(predicate::str::contains("\"parse_error_count\": 0"))
        .stdout(predicate::str::contains(root_file.display().to_string()))
        .stdout(predicate::str::contains(nested_file.display().to_string()).not());
}
