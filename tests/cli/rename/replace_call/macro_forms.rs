use super::*;

#[test]
fn cli_rejects_replace_function_calls_for_shadowed_compiler_macrolet_path() {
    let dir = fresh_temp_dir("replace-function-calls-compiler-macrolet-shadowed-path");
    let lisp_file = dir.join("service.lisp");
    fs::write(
        &lisp_file,
        "(defun render () (compiler-macrolet ((fetch-user (id) `(fetch-user ,id))) (fetch-user user)))\n",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("replace-function-calls")
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
fn cli_rejects_replace_function_calls_for_shadowed_cl_user_compiler_macrolet_path() {
    let dir = fresh_temp_dir("replace-function-calls-cl-user-compiler-macrolet-shadowed-path");
    let lisp_file = dir.join("service.lisp");
    fs::write(
        &lisp_file,
        "(defun render () (cl-user:compiler-macrolet ((fetch-user (id) `(fetch-user ,id))) (fetch-user user)))\n",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("replace-function-calls")
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
fn cli_rejects_replace_function_calls_for_shadowed_cl_user_macrolet_path() {
    let dir = fresh_temp_dir("replace-function-calls-cl-user-macrolet-shadowed-path");
    let lisp_file = dir.join("service.lisp");
    fs::write(
        &lisp_file,
        "(defun render () (cl-user:macrolet ((fetch-user (id) `(fetch-user ,id))) (fetch-user user)))\n",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("replace-function-calls")
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
fn cli_writes_replace_function_calls_inside_cl_user_macrolet_expanders_only() {
    let dir = fresh_temp_dir("replace-function-calls-cl-user-macrolet-all-calls");
    let lisp_file = dir.join("service.lisp");
    fs::write(
        &lisp_file,
        "(defun render () (cl-user:macrolet ((fetch-user (id) `(fetch-user ,id))) (fetch-user user)) (fetch-user root))\n",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("replace-function-calls")
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
        "(defun render () (cl-user:macrolet ((fetch-user (id) `(load-user ,id))) (fetch-user user)) (load-user root))\n"
    );
}

#[test]
fn cli_writes_replace_function_call_at_macrolet_expander_path() {
    let dir = fresh_temp_dir("replace-function-calls-macrolet-expander-path");
    let lisp_file = dir.join("service.lisp");
    fs::write(
        &lisp_file,
        "(defun render () (macrolet ((expand (id) `(fetch-user ,id))) (expand user)))\n",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("replace-function-calls")
        .arg(&lisp_file)
        .arg("--from")
        .arg("fetch-user")
        .arg("--to")
        .arg("load-user")
        .arg("--call-path")
        .arg("0.3.1.0.2")
        .arg("--write")
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"callCount\": 1"))
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(lisp_file).expect("read rewritten lisp"),
        "(defun render () (macrolet ((expand (id) `(load-user ,id))) (expand user)))\n"
    );
}
