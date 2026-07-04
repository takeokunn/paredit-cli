use super::*;

#[test]
fn cli_formats_common_lisp_indentation() {
    let dir = fresh_temp_dir("format-common-lisp");
    let file = dir.join("core.lisp");
    fs::write(
        &file,
        "(defun add (x y) (let ((sum (+ x y)) (label :ok)) (list label sum)))\n",
    )
    .expect("write source fixture");

    let mut cmd = paredit();
    cmd.arg("format")
        .arg("--file")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "(defun add (x y)\n  (let ((sum (+ x y))\n        (label :ok))\n    (list label sum)))\n",
        ));
}

#[test]
fn cli_formats_handler_bind_indentation() {
    let dir = fresh_temp_dir("format-handler-bind");
    let file = dir.join("handler-bind.lisp");
    fs::write(
        &file,
        "(handler-bind ((error #'handle-error) (warning #'muffle-warning)) (risky) (recover))\n",
    )
    .expect("write source fixture");

    let mut cmd = paredit();
    cmd.arg("format")
        .arg("--file")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "(handler-bind ((error #'handle-error)\n               (warning #'muffle-warning))\n  (risky)\n  (recover))\n",
        ));
}

#[test]
fn cli_formats_macrolet_indentation() {
    let dir = fresh_temp_dir("format-macrolet");
    let file = dir.join("macrolet.lisp");
    fs::write(
        &file,
        "(macrolet ((with-x (x) (list x outer))) (with-x 1) (with-x 2))\n",
    )
    .expect("write source fixture");

    let mut cmd = paredit();
    cmd.arg("format")
        .arg("--file")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "(macrolet ((with-x (x) (list x outer)))\n  (with-x 1)\n  (with-x 2))\n",
        ));
}

#[test]
fn cli_formats_compiler_macrolet_indentation() {
    let dir = fresh_temp_dir("format-compiler-macrolet");
    let file = dir.join("compiler-macrolet.lisp");
    fs::write(
        &file,
        "(compiler-macrolet ((with-x (x) (list x outer))) (with-x 1) (with-x 2))\n",
    )
    .expect("write source fixture");

    let mut cmd = paredit();
    cmd.arg("format")
        .arg("--file")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "(compiler-macrolet ((with-x (x) (list x outer)))\n  (with-x 1)\n  (with-x 2))\n",
        ));
}

#[test]
fn cli_formats_multiple_local_callable_bindings() {
    let dir = fresh_temp_dir("format-multiple-local-callables");
    let file = dir.join("local-callables.lisp");
    fs::write(
        &file,
        "(macrolet ((with-a (x) (list x outer)) (with-b (y) (list y outer))) (with-a 1) (with-b 2))\n",
    )
    .expect("write source fixture");

    let mut cmd = paredit();
    cmd.arg("format")
        .arg("--file")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "(macrolet ((with-a (x) (list x outer))\n           (with-b (y) (list y outer)))\n  (with-a 1)\n  (with-b 2))\n",
        ));
}

#[test]
fn cli_formats_define_compiler_macro_indentation() {
    let dir = fresh_temp_dir("format-define-compiler-macro");
    let file = dir.join("compiler-macro.lisp");
    fs::write(
        &file,
        "(define-compiler-macro fast-add (x y) (list '+ x y))\n",
    )
    .expect("write source fixture");

    let mut cmd = paredit();
    cmd.arg("format")
        .arg("--file")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "(define-compiler-macro fast-add (x y)\n  (list '+ x y))\n",
        ));
}

#[test]
fn cli_formats_define_setf_expander_indentation() {
    let dir = fresh_temp_dir("format-define-setf-expander");
    let file = dir.join("setf-expander.lisp");
    fs::write(
        &file,
        "(define-setf-expander place (env) (values) (list place env))\n",
    )
    .expect("write source fixture");

    let mut cmd = paredit();
    cmd.arg("format")
        .arg("--file")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "(define-setf-expander place (env)\n  (values)\n  (list place env))\n",
        ));
}

#[test]
fn cli_formats_declarations_indentation() {
    let dir = fresh_temp_dir("format-declarations");
    let file = dir.join("declarations.lisp");
    fs::write(
        &file,
        "(locally (declare (optimize speed)) (declaim (inline f)) (proclaim (special x)) (f))\n",
    )
    .expect("write source fixture");

    let mut cmd = paredit();
    cmd.arg("format")
        .arg("--file")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "(locally\n  (declare\n    (optimize speed))\n  (declaim\n    (inline f))\n  (proclaim\n    (special x))\n  (f))\n",
        ));
}
