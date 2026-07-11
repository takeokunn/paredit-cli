use super::*;

#[test]
fn cli_rejects_replace_function_calls_for_shadowed_labels_path() {
    let dir = fresh_temp_dir("replace-function-calls-shadowed-path");
    let lisp_file = dir.join("service.lisp");
    fs::write(
        &lisp_file,
        "(defun render () (labels ((fetch-user (id) (fetch-user id))) (fetch-user user)))\n",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("refactor")
        .arg("replace-function-calls")
        .arg(&lisp_file)
        .arg("--from")
        .arg("fetch-user")
        .arg("--to")
        .arg("load-user")
        .arg("--call-path")
        .arg("0.3.2")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "call-path 0.3.2 is shadowed by a local callable named fetch-user",
        ));
}

#[test]
fn cli_rejects_replace_function_calls_for_shadowed_cl_user_flet_path() {
    let dir = fresh_temp_dir("replace-function-calls-cl-user-flet-shadowed-path");
    let lisp_file = dir.join("service.lisp");
    fs::write(
        &lisp_file,
        "(defun render () (cl-user:flet ((fetch-user (id) (fetch-user id))) (fetch-user user)))\n",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("refactor")
        .arg("replace-function-calls")
        .arg(&lisp_file)
        .arg("--from")
        .arg("fetch-user")
        .arg("--to")
        .arg("load-user")
        .arg("--call-path")
        .arg("0.3.2")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "call-path 0.3.2 is shadowed by a local callable named fetch-user",
        ));
}

#[test]
fn cli_writes_replace_function_calls_skipping_labels_local_calls() {
    let dir = fresh_temp_dir("replace-function-calls-labels-all-calls");
    let lisp_file = dir.join("service.lisp");
    fs::write(
        &lisp_file,
        "(defun main () (labels ((fetch-user (id) (fetch-user id))) (fetch-user user)))\n(fetch-user root)\n",
    )
    .expect("write lisp fixture");

    let output = paredit()
        .arg("refactor")
        .arg("replace-function-calls")
        .arg(&lisp_file)
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
    assert_eq!(report.files.first().map(|file| file.written), Some(true));

    assert_eq!(
        fs::read_to_string(lisp_file).expect("read rewritten lisp"),
        "(defun main () (labels ((fetch-user (id) (fetch-user id))) (fetch-user user)))\n(load-user root)\n"
    );
}

#[test]
fn cli_writes_replace_function_calls_inside_flet_binding_bodies_only() {
    let dir = fresh_temp_dir("replace-function-calls-flet-all-calls");
    let lisp_file = dir.join("service.lisp");
    fs::write(
        &lisp_file,
        "(defun main () (flet ((fetch-user (id) (fetch-user id))) (fetch-user user)))\n(fetch-user root)\n",
    )
    .expect("write lisp fixture");

    let output = paredit()
        .arg("refactor")
        .arg("replace-function-calls")
        .arg(&lisp_file)
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
    assert_eq!(report.call_count, 2);
    assert_eq!(report.files.first().map(|file| file.written), Some(true));

    assert_eq!(
        fs::read_to_string(lisp_file).expect("read rewritten lisp"),
        "(defun main () (flet ((fetch-user (id) (load-user id))) (fetch-user user)))\n(load-user root)\n"
    );
}

#[test]
fn cli_writes_replace_function_calls_inside_cl_user_flet_binding_bodies_only() {
    let dir = fresh_temp_dir("replace-function-calls-cl-user-flet-all-calls");
    let lisp_file = dir.join("service.lisp");
    fs::write(
        &lisp_file,
        "(defun main () (cl-user:flet ((fetch-user (id) (fetch-user id))) (fetch-user user)))\n(fetch-user root)\n",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("refactor")
        .arg("replace-function-calls")
        .arg(&lisp_file)
        .arg("--from")
        .arg("fetch-user")
        .arg("--to")
        .arg("load-user")
        .arg("--all-calls")
        .arg("--write")
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"callCount\": 2"))
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(lisp_file).expect("read rewritten lisp"),
        "(defun main () (cl-user:flet ((fetch-user (id) (load-user id))) (fetch-user user)))\n(load-user root)\n"
    );
}
