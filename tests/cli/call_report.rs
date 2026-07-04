use super::*;

#[test]
fn cli_reports_multi_file_call_sites_for_agent_refactor_planning() {
    let dir = fresh_temp_dir("call-report");
    let lisp_file = dir.join("core.lisp");
    let elisp_file = dir.join("init.el");
    fs::write(
        &lisp_file,
        "(defun area (width height) (* width height))\n(defun render () (area 10 20))\n(defun total () (+ (area 3 4) 1))\n",
    )
    .expect("write lisp fixture");
    fs::write(&elisp_file, "(defun demo-mode () (area 5 6))\n").expect("write elisp fixture");

    let mut cmd = paredit();
    cmd.arg("call-report")
        .arg("--symbol")
        .arg("area")
        .arg(&lisp_file)
        .arg(&elisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"symbol\": \"area\""))
        .stdout(predicate::str::contains("\"file_count\": 2"))
        .stdout(predicate::str::contains("\"total_count\": 3"))
        .stdout(predicate::str::contains("\"dialect\": \"common-lisp\""))
        .stdout(predicate::str::contains("\"dialect\": \"emacs-lisp\""))
        .stdout(predicate::str::contains("\"head\": \"area\""))
        .stdout(predicate::str::contains("\"argumentCount\": 2"))
        .stdout(predicate::str::contains(
            "\"enclosingDefinition\": \"render\"",
        ))
        .stdout(predicate::str::contains(
            "\"enclosingDefinition\": \"total\"",
        ))
        .stdout(predicate::str::contains(
            "\"enclosingDefinition\": \"demo-mode\"",
        ));

    let mut cmd = paredit();
    cmd.arg("call-report")
        .arg(&lisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"head\": \"area\""))
        .stdout(predicate::str::contains("\"head\": \"defun\"").not());
}
