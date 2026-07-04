use super::*;

#[test]
fn formats_short_atom_lists_inline() {
    let input = "(defun add (x y) (+ x y))";
    let tree = SyntaxTree::parse(input).expect("valid");
    assert_eq!(
        Formatter::new(2).format(&tree),
        "(defun add (x y)\n  (+ x y))\n"
    );
}

#[test]
fn formats_binding_forms_with_aligned_bindings() {
    let input = "(let ((x 1) (y (+ x 2))) (list x y))";
    let tree = SyntaxTree::parse(input).expect("valid");
    assert_eq!(
        Formatter::new(2).format(&tree),
        "(let ((x 1)\n      (y (+ x 2)))\n  (list x y))\n"
    );
}

#[test]
fn formats_bracket_binding_forms_as_name_value_pairs() {
    let input = "(let [x 1 y (+ x 2)] (list x y))";
    let tree = SyntaxTree::parse(input).expect("valid");
    assert_eq!(
        Formatter::new(2).format(&tree),
        "(let [x 1\n      y (+ x 2)]\n  (list x y))\n"
    );
}

#[test]
fn formats_handler_bind_like_a_binding_form() {
    let input =
        "(handler-bind ((error #'handle-error) (warning #'muffle-warning)) (risky) (recover))";
    let tree = SyntaxTree::parse(input).expect("valid");
    assert_eq!(
        Formatter::new(2).format(&tree),
        "(handler-bind ((error #'handle-error)\n               (warning #'muffle-warning))\n  (risky)\n  (recover))\n"
    );
}

#[test]
fn formats_restart_bind_like_a_binding_form() {
    let input = "(restart-bind ((retry #'retry :report report-retry) (skip #'skip)) (work))";
    let tree = SyntaxTree::parse(input).expect("valid");
    assert_eq!(
        Formatter::new(2).format(&tree),
        "(restart-bind ((retry #'retry :report report-retry)\n               (skip #'skip))\n  (work))\n"
    );
}

#[test]
fn formats_macro_and_cond_body_forms() {
    let input = "(defmacro when-let ((name value)) (list 'when value (list 'let (list (list name value)) name)))\n(cond ((null x) nil) (t (car x)))";
    let tree = SyntaxTree::parse(input).expect("valid");
    assert_eq!(
        Formatter::new(2).format(&tree),
        "(defmacro when-let ((name value))\n  (list 'when value (list 'let (list (list name value)) name)))\n\n(cond\n  ((null x) nil)\n  (t (car x)))\n"
    );
}

#[test]
fn formats_multi_body_cond_clauses_on_separate_lines() {
    let input = "(cond ((ready-p value) (prepare value) (run value)) (t (fallback)))";
    let tree = SyntaxTree::parse(input).expect("valid");
    assert_eq!(
        Formatter::new(2).format(&tree),
        "(cond\n  ((ready-p value)\n    (prepare value)\n    (run value))\n  (t (fallback)))\n"
    );
}

#[test]
fn formats_case_keyform_and_multi_body_clauses() {
    let input = "(case kind (:ready (prepare value) (run value)) ((:skip :noop) value) (otherwise (fallback)))";
    let tree = SyntaxTree::parse(input).expect("valid");
    assert_eq!(
        Formatter::new(2).format(&tree),
        "(case kind\n  (:ready\n    (prepare value)\n    (run value))\n  ((:skip :noop) value)\n  (otherwise (fallback)))\n"
    );
}

#[test]
fn formats_do_iteration_specs_and_end_clause() {
    let input = "(do ((i 0 (1+ i)) (sum 0 (+ sum i))) ((>= i limit) sum total) (incf total sum) (collect i))";
    let tree = SyntaxTree::parse(input).expect("valid");
    assert_eq!(
        Formatter::new(2).format(&tree),
        "(do ((i 0 (1+ i))\n     (sum 0 (+ sum i)))\n  ((>= i limit)\n    sum\n    total)\n  (incf total sum)\n  (collect i))\n"
    );
}

#[test]
fn formats_do_star_like_do() {
    let input = "(do* ((x 0 (1+ x)) (y x (+ x y))) ((> y 10) y) (print y))";
    let tree = SyntaxTree::parse(input).expect("valid");
    assert_eq!(
        Formatter::new(2).format(&tree),
        "(do* ((x 0 (1+ x))\n      (y x (+ x y)))\n  ((> y 10) y)\n  (print y))\n"
    );
}

#[test]
fn formats_prog_bindings_and_tagbody_forms() {
    let input = "(prog ((i 0) (sum 0)) start (incf sum i) (when (> sum limit) (return sum)) (incf i) (go start))";
    let tree = SyntaxTree::parse(input).expect("valid");
    assert_eq!(
        Formatter::new(2).format(&tree),
        "(prog ((i 0)\n       (sum 0))\n  start\n  (incf sum i)\n  (when (> sum limit)\n    (return sum))\n  (incf i)\n  (go start))\n"
    );
}

#[test]
fn formats_prog_star_like_prog() {
    let input = "(prog* ((x 1) (y x)) done (return y))";
    let tree = SyntaxTree::parse(input).expect("valid");
    assert_eq!(
        Formatter::new(2).format(&tree),
        "(prog* ((x 1)\n        (y x))\n  done\n  (return y))\n"
    );
}

#[test]
fn formats_common_lisp_prefix_body_forms() {
    let input =
        "(block done (catch 'retry (unwind-protect (run job) (cleanup job) (release job))))";
    let tree = SyntaxTree::parse(input).expect("valid");
    assert_eq!(
        Formatter::new(2).format(&tree),
        "(block done\n  (catch 'retry\n    (unwind-protect (run job)\n      (cleanup job)\n      (release job))))\n"
    );
}

#[test]
fn formats_eval_when_body_after_situation_list() {
    let input = "(eval-when (:compile-toplevel :load-toplevel :execute) (declaim (optimize speed)) (defun boot () (start)))";
    let tree = SyntaxTree::parse(input).expect("valid");
    assert_eq!(
        Formatter::new(2).format(&tree),
        "(eval-when (:compile-toplevel :load-toplevel :execute)\n  (declaim (optimize speed))\n  (defun boot ()\n    (start)))\n"
    );
}

#[test]
fn formats_lambda_body_after_lambda_list() {
    let input = "(lambda (value) (validate value) (render value))";
    let tree = SyntaxTree::parse(input).expect("valid");
    assert_eq!(
        Formatter::new(2).format(&tree),
        "(lambda (value)\n  (validate value)\n  (render value))\n"
    );
}

#[test]
fn formats_when_body_after_condition() {
    let input = "(when ready-p (prepare) (run))";
    let tree = SyntaxTree::parse(input).expect("valid");
    assert_eq!(
        Formatter::new(2).format(&tree),
        "(when ready-p\n  (prepare)\n  (run))\n"
    );
}

#[test]
fn formats_destructuring_bind_with_two_prefix_forms() {
    let input = "(destructuring-bind (value other) (parse value) (list value other) (finish))";
    let tree = SyntaxTree::parse(input).expect("valid");
    assert_eq!(
        Formatter::new(2).format(&tree),
        "(destructuring-bind (value other) (parse value)\n  (list value other)\n  (finish))\n"
    );
}

#[test]
fn formats_multiple_value_bind_with_two_prefix_forms() {
    let input = "(multiple-value-bind (value foundp) (gethash key table) (list value foundp))";
    let tree = SyntaxTree::parse(input).expect("valid");
    assert_eq!(
        Formatter::new(2).format(&tree),
        "(multiple-value-bind (value foundp) (gethash key table)\n  (list value foundp))\n"
    );
}

#[test]
fn formats_handler_case_clauses_after_protected_form() {
    let input = "(handler-case (risky) (error (condition) (recover condition) (log condition)) (:no-error (value) value))";
    let tree = SyntaxTree::parse(input).expect("valid");
    assert_eq!(
        Formatter::new(2).format(&tree),
        "(handler-case (risky)\n  (error (condition)\n    (recover condition)\n    (log condition))\n  (:no-error (value)\n    value))\n"
    );
}

#[test]
fn formats_restart_case_clauses_after_protected_form() {
    let input = "(restart-case (risky) (retry () (prepare) (risky)) (skip () nil))";
    let tree = SyntaxTree::parse(input).expect("valid");
    assert_eq!(
        Formatter::new(2).format(&tree),
        "(restart-case (risky)\n  (retry ()\n    (prepare)\n    (risky))\n  (skip ()\n    nil))\n"
    );
}

#[test]
fn formats_define_compiler_macro_like_a_definition() {
    let input = "(define-compiler-macro fast-add (x y) (list '+ x y))";
    let tree = SyntaxTree::parse(input).expect("valid");
    assert_eq!(
        Formatter::new(2).format(&tree),
        "(define-compiler-macro fast-add (x y)\n  (list '+ x y))\n"
    );
}

#[test]
fn formats_setf_definition_forms_like_definitions() {
    let input = "(define-setf-expander place (env) (values) (list place env))";
    let tree = SyntaxTree::parse(input).expect("valid");
    assert_eq!(
        Formatter::new(2).format(&tree),
        "(define-setf-expander place (env)\n  (values)\n  (list place env))\n"
    );
}

#[test]
fn formats_macrolet_like_local_functions() {
    let input = "(macrolet ((with-x (x) (list x outer))) (with-x 1) (with-x 2))";
    let tree = SyntaxTree::parse(input).expect("valid");
    assert_eq!(
        Formatter::new(2).format(&tree),
        "(macrolet ((with-x (x) (list x outer)))\n  (with-x 1)\n  (with-x 2))\n"
    );
}

#[test]
fn formats_compiler_macrolet_like_local_functions() {
    let input = "(compiler-macrolet ((with-x (x) (list x outer))) (with-x 1) (with-x 2))";
    let tree = SyntaxTree::parse(input).expect("valid");
    assert_eq!(
        Formatter::new(2).format(&tree),
        "(compiler-macrolet ((with-x (x) (list x outer)))\n  (with-x 1)\n  (with-x 2))\n"
    );
}

#[test]
fn formats_multiple_local_callable_bindings_with_aligned_bindings() {
    let input = "(macrolet ((with-a (x) (list x outer)) (with-b (y) (list y outer))) (with-a 1) (with-b 2))";
    let tree = SyntaxTree::parse(input).expect("valid");
    assert_eq!(
        Formatter::new(2).format(&tree),
        "(macrolet ((with-a (x) (list x outer))\n           (with-b (y) (list y outer)))\n  (with-a 1)\n  (with-b 2))\n"
    );
}

#[test]
fn formats_declarations_with_inline_specs() {
    let input =
        "(locally (declare (optimize speed)) (declaim (inline f)) (proclaim (special x)) (f))";
    let tree = SyntaxTree::parse(input).expect("valid");
    assert_eq!(
        Formatter::new(2).format(&tree),
        "(locally\n  (declare (optimize speed))\n  (declaim (inline f))\n  (proclaim (special x))\n  (f))\n"
    );
}

#[test]
fn formats_multiple_declaration_specs_with_alignment() {
    let input = "(declare (optimize speed) (type fixnum index) (ignorable scratch))";
    let tree = SyntaxTree::parse(input).expect("valid");
    assert_eq!(
        Formatter::new(2).format(&tree),
        "(declare (optimize speed)\n         (type fixnum index)\n         (ignorable scratch))\n"
    );
}
