use super::*;

#[test]
fn cli_plans_replace_function_calls_without_writing() {
    let dir = fresh_temp_dir("replace-function-calls-plan");
    let lisp_file = dir.join("service.lisp");
    let source = "(defun fetch-user (id) (list fetch-user id))\n(defun render () (fetch-user id) (fetch-user other))\n";
    fs::write(&lisp_file, source).expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("replace-function-calls")
        .arg(&lisp_file)
        .arg("--from")
        .arg("fetch-user")
        .arg("--to")
        .arg("load-user")
        .arg("--all-calls")
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"from\": \"fetch-user\""))
        .stdout(predicate::str::contains("\"to\": \"load-user\""))
        .stdout(predicate::str::contains("\"callCount\": 2"))
        .stdout(predicate::str::contains(
            "(defun fetch-user (id) (list fetch-user id))",
        ))
        .stdout(predicate::str::contains("(load-user id)"))
        .stdout(predicate::str::contains("(load-user other)"));

    assert_eq!(
        fs::read_to_string(lisp_file).expect("read unchanged lisp"),
        source
    );
}

#[test]
fn cli_writes_replace_function_calls_at_call_path() {
    let dir = fresh_temp_dir("replace-function-calls-path");
    let lisp_file = dir.join("service.lisp");
    fs::write(
        &lisp_file,
        "(defun render () (fetch-user id) (fetch-user other))\n",
    )
    .expect("write lisp fixture");

    let output = paredit()
        .arg("replace-function-calls")
        .arg(&lisp_file)
        .arg("--from")
        .arg("fetch-user")
        .arg("--to")
        .arg("load-user")
        .arg("--call-path")
        .arg("0.4")
        .arg("--write")
        .arg("--output")
        .arg("json")
        .output()
        .expect("run paredit");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report = parse_replace_call_report(&output.stdout).expect("parse replace-call report");
    assert_eq!(report.call_count, 1);
    assert_eq!(report.files.first().map(|file| file.written), Some(true));
    assert!(String::from_utf8_lossy(&output.stdout).contains("\"path\": \"0.4\""));

    assert_eq!(
        fs::read_to_string(lisp_file).expect("read path-targeted lisp"),
        "(defun render () (fetch-user id) (load-user other))\n"
    );
}

#[test]
fn cli_rejects_replace_function_calls_without_explicit_scope() {
    let dir = fresh_temp_dir("replace-function-calls-no-scope");
    let lisp_file = dir.join("service.lisp");
    fs::write(&lisp_file, "(defun render () (fetch-user id))\n").expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("replace-function-calls")
        .arg(&lisp_file)
        .arg("--from")
        .arg("fetch-user")
        .arg("--to")
        .arg("load-user")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "replace-function-calls requires either --all-calls or repeated --call-path",
        ));
}

#[test]
fn cli_rejects_replace_function_calls_with_conflicting_scope_flags() {
    let dir = fresh_temp_dir("replace-function-calls-conflicting-scope");
    let lisp_file = dir.join("service.lisp");
    fs::write(&lisp_file, "(defun render () (fetch-user id))\n").expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("replace-function-calls")
        .arg(&lisp_file)
        .arg("--from")
        .arg("fetch-user")
        .arg("--to")
        .arg("load-user")
        .arg("--all-calls")
        .arg("--call-path")
        .arg("0.4")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "replace-function-calls requires either --all-calls or repeated --call-path",
        ));
}

#[test]
fn cli_replace_function_calls_enforces_required_call_count() {
    let dir = fresh_temp_dir("replace-function-calls-require");
    let lisp_file = dir.join("service.lisp");
    fs::write(&lisp_file, "(defun render () (fetch-user id))\n").expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("replace-function-calls")
        .arg(&lisp_file)
        .arg("--from")
        .arg("fetch-user")
        .arg("--to")
        .arg("load-user")
        .arg("--all-calls")
        .arg("--require-calls")
        .arg("2")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "expected at least 2 changed call sites but found 1",
        ));
}

#[test]
fn cli_replace_function_calls_fails_when_selected_calls_do_not_change() {
    let dir = fresh_temp_dir("replace-function-calls-no-change");
    let lisp_file = dir.join("service.lisp");
    fs::write(&lisp_file, "(defun fetch-user (id) id)\n").expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("replace-function-calls")
        .arg(&lisp_file)
        .arg("--from")
        .arg("fetch-user")
        .arg("--to")
        .arg("load-user")
        .arg("--all-calls")
        .arg("--fail-on-no-change")
        .assert()
        .failure()
        .stderr(predicate::str::contains("no selected call site changed"));
}

#[test]
fn cli_replace_function_calls_aggregates_counts_across_multiple_files() {
    let dir = fresh_temp_dir("replace-function-calls-multi-file");
    let changed_file = dir.join("changed.lisp");
    let unchanged_file = dir.join("unchanged.lisp");
    fs::write(&changed_file, "(defun render () (fetch-user id))\n").expect("write changed fixture");
    fs::write(&unchanged_file, "(defun keep () :ok)\n").expect("write unchanged fixture");

    let output = paredit()
        .arg("replace-function-calls")
        .arg(&changed_file)
        .arg(&unchanged_file)
        .arg("--from")
        .arg("fetch-user")
        .arg("--to")
        .arg("load-user")
        .arg("--all-calls")
        .arg("--write")
        .arg("--output")
        .arg("json")
        .output()
        .expect("run paredit");
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let report = parse_replace_call_report(&output.stdout).expect("parse replace-call report");
    assert_eq!(report.call_count, 1);
    assert_eq!(
        report.files,
        vec![
            CliWrittenFileReport { written: true },
            CliWrittenFileReport { written: false },
        ]
    );

    assert_eq!(
        fs::read_to_string(changed_file).expect("read changed file"),
        "(defun render () (load-user id))\n"
    );
    assert_eq!(
        fs::read_to_string(unchanged_file).expect("read unchanged file"),
        "(defun keep () :ok)\n"
    );
}
