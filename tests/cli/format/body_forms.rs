use super::assert_format_output;

#[test]
fn cli_formats_common_lisp_indentation() {
    assert_format_output(
        "format-common-lisp",
        "core.lisp",
        "(defun add (x y) (let ((sum (+ x y)) (label :ok)) (list label sum)))\n",
        "(defun add (x y)\n  (let ((sum (+ x y))\n        (label :ok))\n    (list label sum)))\n",
    );
}

#[test]
fn cli_formats_defmethod_specialized_lambda_list() {
    assert_format_output(
        "format-defmethod",
        "defmethod.lisp",
        "(defmethod render ((node widget) stream) (draw node stream) (finish-output stream))\n",
        "(defmethod render ((node widget) stream)\n  (draw node stream)\n  (finish-output stream))\n",
    );
}

#[test]
fn cli_formats_defmethod_qualifiers_with_specialized_lambda_list() {
    assert_format_output(
        "format-defmethod-qualifier",
        "defmethod-qualifier.lisp",
        "(defmethod render :around ((node widget) stream) (call-next-method) (finish-output stream))\n",
        "(defmethod render :around ((node widget) stream)\n  (call-next-method)\n  (finish-output stream))\n",
    );
}

#[test]
fn cli_formats_handler_bind_indentation() {
    assert_format_output(
        "format-handler-bind",
        "handler-bind.lisp",
        "(handler-bind ((error #'handle-error) (warning #'muffle-warning)) (risky) (recover))\n",
        "(handler-bind ((error #'handle-error)\n               (warning #'muffle-warning))\n  (risky)\n  (recover))\n",
    );
}

#[test]
fn cli_formats_multiple_value_bind_indentation() {
    assert_format_output(
        "format-multiple-value-bind",
        "multiple-value-bind.lisp",
        "(multiple-value-bind (value foundp) (gethash key table) (list value foundp))\n",
        "(multiple-value-bind (value foundp) (gethash key table)\n  (list value foundp))\n",
    );
}

#[test]
fn cli_formats_handler_case_indentation() {
    assert_format_output(
        "format-handler-case",
        "handler-case.lisp",
        "(handler-case (risky) (error (condition) (recover condition) (log condition)) (:no-error (value) value))\n",
        "(handler-case (risky)\n  (error (condition)\n    (recover condition)\n    (log condition))\n  (:no-error (value)\n    value))\n",
    );
}

#[test]
fn cli_formats_restart_case_indentation() {
    assert_format_output(
        "format-restart-case",
        "restart-case.lisp",
        "(restart-case (risky) (retry () (prepare) (risky)) (skip () nil))\n",
        "(restart-case (risky)\n  (retry ()\n    (prepare)\n    (risky))\n  (skip ()\n    nil))\n",
    );
}

#[test]
fn cli_formats_do_iteration_indentation() {
    assert_format_output(
        "format-do",
        "do.lisp",
        "(do ((i 0 (1+ i)) (sum 0 (+ sum i))) ((>= i limit) sum total) (incf total sum))\n",
        "(do ((i 0 (1+ i))\n     (sum 0 (+ sum i)))\n  ((>= i limit)\n    sum\n    total)\n  (incf total sum))\n",
    );
}

#[test]
fn cli_formats_prog_iteration_indentation() {
    assert_format_output(
        "format-prog",
        "prog.lisp",
        "(prog ((i 0) (sum 0)) start (incf sum i) (when (> sum limit) (return sum)) (incf i) (go start))\n",
        "(prog ((i 0)\n       (sum 0))\n  start\n  (incf sum i)\n  (when (> sum limit)\n    (return sum))\n  (incf i)\n  (go start))\n",
    );
}

#[test]
fn cli_formats_common_lisp_prefix_body_indentation() {
    assert_format_output(
        "format-prefix-body",
        "prefix-body.lisp",
        "(block done (catch 'retry (unwind-protect (run job) (cleanup job) (release job))))\n",
        "(block done\n  (catch 'retry\n    (unwind-protect (run job)\n      (cleanup job)\n      (release job))))\n",
    );
}

#[test]
fn cli_formats_common_lisp_with_body_macros() {
    assert_format_output(
        "format-with-body-macros",
        "with-body-macros.lisp",
        "(with-input-from-string (stream text) (read stream) (finish stream))\n(with-output-to-string (stream) (write value :stream stream) (finish-output stream))\n(with-hash-table-iterator (next table) (multiple-value-bind (more key value) (next) (when more (collect key value))))\n(with-package-iterator (next package :internal :external) (multiple-value-bind (more symbol status package) (next) (when more (collect symbol status package))))\n",
        "(with-input-from-string (stream text)\n  (read stream)\n  (finish stream))\n\n(with-output-to-string (stream)\n  (write value :stream stream)\n  (finish-output stream))\n\n(with-hash-table-iterator (next table)\n  (multiple-value-bind (more key value) (next)\n    (when more\n      (collect key value))))\n\n(with-package-iterator (next package :internal :external)\n  (multiple-value-bind (more symbol status package) (next)\n    (when more\n      (collect symbol status package))))\n",
    );
}

#[test]
fn cli_formats_loop_clause_indentation() {
    assert_format_output(
        "format-loop",
        "loop.lisp",
        "(loop for item in items when (valid-p item) collect (transform item) finally (return result))\n",
        "(loop for item in items\n      when (valid-p item)\n        collect (transform item)\n      finally (return result))\n",
    );
}

#[test]
fn cli_formats_common_lisp_assignment_pairs() {
    assert_format_output(
        "format-assignment-pairs",
        "assignment-pairs.lisp",
        "(setf (slot-value user 'name) (compute-name user) (slot-value user 'age) (compute-age user))\n",
        "(setf (slot-value user 'name) (compute-name user)\n      (slot-value user 'age) (compute-age user))\n",
    );
}
