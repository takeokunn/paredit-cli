use super::*;

#[test]
fn cli_rejects_wrap_function_calls_for_shadowed_compiler_macrolet_path() {
    let dir = fresh_temp_dir("wrap-function-calls-compiler-macrolet-shadowed-path");
    let lisp_file = dir.join("service.lisp");
    fs::write(
        &lisp_file,
        "(defun render () (compiler-macrolet ((fetch-user (id) `(fetch-user ,id))) (fetch-user user)))\n",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("wrap-function-calls")
        .arg(&lisp_file)
        .arg("--function")
        .arg("fetch-user")
        .arg("--wrapper")
        .arg("with-cache")
        .arg("--call-path")
        .arg("0.3.2")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "call-path 0.3.2 is shadowed by a local callable named fetch-user",
        ));
}

#[test]
fn cli_rejects_wrap_function_calls_for_shadowed_cl_user_macrolet_path() {
    let dir = fresh_temp_dir("wrap-function-calls-cl-user-macrolet-shadowed-path");
    let lisp_file = dir.join("service.lisp");
    fs::write(
        &lisp_file,
        "(defun render () (cl-user:macrolet ((fetch-user (id) `(fetch-user ,id))) (fetch-user user)))\n",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("wrap-function-calls")
        .arg(&lisp_file)
        .arg("--function")
        .arg("fetch-user")
        .arg("--wrapper")
        .arg("with-cache")
        .arg("--call-path")
        .arg("0.3.2")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "call-path 0.3.2 is shadowed by a local callable named fetch-user",
        ));
}

#[test]
fn cli_writes_wrap_function_calls_inside_cl_user_macrolet_expanders_only() {
    let dir = fresh_temp_dir("wrap-function-calls-cl-user-macrolet");
    let lisp_file = dir.join("service.lisp");
    fs::write(
        &lisp_file,
        "(defun render () (cl-user:macrolet ((fetch-user (id) `(fetch-user ,id))) (fetch-user user)) (fetch-user root))\n",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("wrap-function-calls")
        .arg(&lisp_file)
        .arg("--function")
        .arg("fetch-user")
        .arg("--wrapper")
        .arg("with-cache")
        .arg("--all-calls")
        .arg("--write")
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"callCount\": 2"))
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(lisp_file).expect("read wrapped lisp"),
        "(defun render () (cl-user:macrolet ((fetch-user (id) `(with-cache (fetch-user ,id)))) (fetch-user user)) (with-cache (fetch-user root)))\n"
    );
}

#[test]
fn cli_rejects_wrap_function_calls_for_shadowed_cl_user_compiler_macrolet_path() {
    let dir = fresh_temp_dir("wrap-function-calls-cl-user-compiler-macrolet-shadowed-path");
    let lisp_file = dir.join("service.lisp");
    fs::write(
        &lisp_file,
        "(defun render () (cl-user:compiler-macrolet ((fetch-user (id) `(fetch-user ,id))) (fetch-user user)))\n",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("wrap-function-calls")
        .arg(&lisp_file)
        .arg("--function")
        .arg("fetch-user")
        .arg("--wrapper")
        .arg("with-cache")
        .arg("--call-path")
        .arg("0.3.2")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "call-path 0.3.2 is shadowed by a local callable named fetch-user",
        ));
}
