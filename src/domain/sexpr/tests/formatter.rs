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
fn formats_declarations_as_head_body_forms() {
    let input =
        "(locally (declare (optimize speed)) (declaim (inline f)) (proclaim (special x)) (f))";
    let tree = SyntaxTree::parse(input).expect("valid");
    assert_eq!(
        Formatter::new(2).format(&tree),
        "(locally\n  (declare\n    (optimize speed))\n  (declaim\n    (inline f))\n  (proclaim\n    (special x))\n  (f))\n"
    );
}
