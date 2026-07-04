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
fn cli_formats_multiple_value_bind_indentation() {
    let dir = fresh_temp_dir("format-multiple-value-bind");
    let file = dir.join("multiple-value-bind.lisp");
    fs::write(
        &file,
        "(multiple-value-bind (value foundp) (gethash key table) (list value foundp))\n",
    )
    .expect("write source fixture");

    let mut cmd = paredit();
    cmd.arg("format")
        .arg("--file")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "(multiple-value-bind (value foundp) (gethash key table)\n  (list value foundp))\n",
        ));
}

#[test]
fn cli_formats_handler_case_indentation() {
    let dir = fresh_temp_dir("format-handler-case");
    let file = dir.join("handler-case.lisp");
    fs::write(
        &file,
        "(handler-case (risky) (error (condition) (recover condition) (log condition)) (:no-error (value) value))\n",
    )
    .expect("write source fixture");

    let mut cmd = paredit();
    cmd.arg("format")
        .arg("--file")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "(handler-case (risky)\n  (error (condition)\n    (recover condition)\n    (log condition))\n  (:no-error (value)\n    value))\n",
        ));
}

#[test]
fn cli_formats_restart_case_indentation() {
    let dir = fresh_temp_dir("format-restart-case");
    let file = dir.join("restart-case.lisp");
    fs::write(
        &file,
        "(restart-case (risky) (retry () (prepare) (risky)) (skip () nil))\n",
    )
    .expect("write source fixture");

    let mut cmd = paredit();
    cmd.arg("format")
        .arg("--file")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "(restart-case (risky)\n  (retry ()\n    (prepare)\n    (risky))\n  (skip ()\n    nil))\n",
        ));
}

#[test]
fn cli_formats_do_iteration_indentation() {
    let dir = fresh_temp_dir("format-do");
    let file = dir.join("do.lisp");
    fs::write(
        &file,
        "(do ((i 0 (1+ i)) (sum 0 (+ sum i))) ((>= i limit) sum total) (incf total sum))\n",
    )
    .expect("write source fixture");

    let mut cmd = paredit();
    cmd.arg("format")
        .arg("--file")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "(do ((i 0 (1+ i))\n     (sum 0 (+ sum i)))\n  ((>= i limit)\n    sum\n    total)\n  (incf total sum))\n",
        ));
}

#[test]
fn cli_formats_prog_iteration_indentation() {
    let dir = fresh_temp_dir("format-prog");
    let file = dir.join("prog.lisp");
    fs::write(
        &file,
        "(prog ((i 0) (sum 0)) start (incf sum i) (when (> sum limit) (return sum)) (incf i) (go start))\n",
    )
    .expect("write source fixture");

    let mut cmd = paredit();
    cmd.arg("format")
        .arg("--file")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "(prog ((i 0)\n       (sum 0))\n  start\n  (incf sum i)\n  (when (> sum limit)\n    (return sum))\n  (incf i)\n  (go start))\n",
        ));
}

#[test]
fn cli_formats_common_lisp_prefix_body_indentation() {
    let dir = fresh_temp_dir("format-prefix-body");
    let file = dir.join("prefix-body.lisp");
    fs::write(
        &file,
        "(block done (catch 'retry (unwind-protect (run job) (cleanup job) (release job))))\n",
    )
    .expect("write source fixture");

    let mut cmd = paredit();
    cmd.arg("format")
        .arg("--file")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "(block done\n  (catch 'retry\n    (unwind-protect (run job)\n      (cleanup job)\n      (release job))))\n",
        ));
}

#[test]
fn cli_formats_symbol_macrolet_indentation() {
    let dir = fresh_temp_dir("format-symbol-macrolet");
    let file = dir.join("symbol-macrolet.lisp");
    fs::write(
        &file,
        "(symbol-macrolet ((value (compute value)) (used other)) (list value used))\n",
    )
    .expect("write source fixture");

    let mut cmd = paredit();
    cmd.arg("format")
        .arg("--file")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "(symbol-macrolet ((value (compute value))\n                  (used other))\n  (list value used))\n",
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
            "(locally\n  (declare (optimize speed))\n  (declaim (inline f))\n  (proclaim (special x))\n  (f))\n",
        ));
}

#[test]
fn cli_formats_multiple_declaration_specs() {
    let dir = fresh_temp_dir("format-multiple-declarations");
    let file = dir.join("declarations.lisp");
    fs::write(
        &file,
        "(declare (optimize speed) (type fixnum index) (ignorable scratch))\n",
    )
    .expect("write source fixture");

    let mut cmd = paredit();
    cmd.arg("format")
        .arg("--file")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "(declare (optimize speed)\n         (type fixnum index)\n         (ignorable scratch))\n",
        ));
}
