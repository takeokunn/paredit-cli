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
fn cli_formats_defmethod_specialized_lambda_list() {
    let dir = fresh_temp_dir("format-defmethod");
    let file = dir.join("defmethod.lisp");
    fs::write(
        &file,
        "(defmethod render ((node widget) stream) (draw node stream) (finish-output stream))\n",
    )
    .expect("write source fixture");

    let mut cmd = paredit();
    cmd.arg("format")
        .arg("--file")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "(defmethod render ((node widget) stream)\n  (draw node stream)\n  (finish-output stream))\n",
        ));
}

#[test]
fn cli_formats_defmethod_qualifiers_with_specialized_lambda_list() {
    let dir = fresh_temp_dir("format-defmethod-qualifier");
    let file = dir.join("defmethod-qualifier.lisp");
    fs::write(
        &file,
        "(defmethod render :around ((node widget) stream) (call-next-method) (finish-output stream))\n",
    )
    .expect("write source fixture");

    let mut cmd = paredit();
    cmd.arg("format")
        .arg("--file")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "(defmethod render :around ((node widget) stream)\n  (call-next-method)\n  (finish-output stream))\n",
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
fn cli_formats_common_lisp_with_body_macros() {
    let dir = fresh_temp_dir("format-with-body-macros");
    let file = dir.join("with-body-macros.lisp");
    fs::write(
        &file,
        "(with-input-from-string (stream text) (read stream) (finish stream))\n(with-output-to-string (stream) (write value :stream stream) (finish-output stream))\n(with-hash-table-iterator (next table) (multiple-value-bind (more key value) (next) (when more (collect key value))))\n(with-package-iterator (next package :internal :external) (multiple-value-bind (more symbol status package) (next) (when more (collect symbol status package))))\n",
    )
    .expect("write source fixture");

    let mut cmd = paredit();
    cmd.arg("format")
        .arg("--file")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "(with-input-from-string (stream text)\n  (read stream)\n  (finish stream))\n\n(with-output-to-string (stream)\n  (write value :stream stream)\n  (finish-output stream))\n\n(with-hash-table-iterator (next table)\n  (multiple-value-bind (more key value) (next)\n    (when more\n      (collect key value))))\n\n(with-package-iterator (next package :internal :external)\n  (multiple-value-bind (more symbol status package) (next)\n    (when more\n      (collect symbol status package))))\n",
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
            "(macrolet ((with-x (x)\n             (list x outer)))\n  (with-x 1)\n  (with-x 2))\n",
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
            "(compiler-macrolet ((with-x (x)\n                      (list x outer)))\n  (with-x 1)\n  (with-x 2))\n",
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
            "(macrolet ((with-a (x)\n             (list x outer))\n           (with-b (y)\n             (list y outer)))\n  (with-a 1)\n  (with-b 2))\n",
        ));
}

#[test]
fn cli_formats_local_callable_bodies_on_dedicated_lines() {
    let dir = fresh_temp_dir("format-local-callable-bodies");
    let file = dir.join("local-callable-bodies.lisp");
    fs::write(
        &file,
        "(labels ((parse (x) (validate x) (build x)) (emit (y) (write y) (finish))) (parse input) (emit output))\n",
    )
    .expect("write source fixture");

    let mut cmd = paredit();
    cmd.arg("format")
        .arg("--file")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "(labels ((parse (x)\n           (validate x)\n           (build x))\n         (emit (y)\n           (write y)\n           (finish)))\n  (parse input)\n  (emit output))\n",
        ));
}

#[test]
fn cli_formats_loop_clause_indentation() {
    let dir = fresh_temp_dir("format-loop");
    let file = dir.join("loop.lisp");
    fs::write(
        &file,
        "(loop for item in items when (valid-p item) collect (transform item) finally (return result))\n",
    )
    .expect("write source fixture");

    let mut cmd = paredit();
    cmd.arg("format")
        .arg("--file")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "(loop for item in items\n      when (valid-p item)\n        collect (transform item)\n      finally (return result))\n",
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
fn cli_formats_common_lisp_assignment_pairs() {
    let dir = fresh_temp_dir("format-assignment-pairs");
    let file = dir.join("assignment-pairs.lisp");
    fs::write(
        &file,
        "(setf (slot-value user 'name) (compute-name user) (slot-value user 'age) (compute-age user))\n",
    )
    .expect("write source fixture");

    let mut cmd = paredit();
    cmd.arg("format")
        .arg("--file")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "(setf (slot-value user 'name) (compute-name user)\n      (slot-value user 'age) (compute-age user))\n",
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
