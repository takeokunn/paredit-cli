use super::*;

#[test]
fn plans_single_common_lisp_call() {
    let input = "(defun area (w h) (* w h))\n(print (area 3 4))";
    let plan = inline_plan(input);

    assert_eq!(plan.function_name.as_str(), "area");
    assert_eq!(plan.calls[0].replacement, "(* 3 4)");
    assert_eq!(
        plan.rewritten,
        "(defun area (w h) (* w h))\n(print (* 3 4))"
    );
    assert!(plan.changed);
}

#[test]
fn discovers_all_calls_and_removes_definition() {
    let input = "(defun inc (x) (+ x 1))\n(print (inc 1))\n(print (inc 2))";
    let plan = remove_definition_plan(input, Dialect::CommonLisp);

    assert_eq!(plan.calls.len(), 2);
    assert_eq!(plan.rewritten, "(print (+ 1 1))\n(print (+ 2 1))");
    assert!(plan.definition_removed);
}

#[test]
fn inlines_common_lisp_optional_parameter_when_argument_is_supplied() {
    let input = "(defun add-default (x &optional (y 10)) (+ x y))\n(print (add-default 1 2))";
    let plan = inline_plan(input);

    assert_eq!(plan.calls[0].replacement, "(+ 1 2)");
    assert_eq!(
        plan.rewritten,
        "(defun add-default (x &optional (y 10)) (+ x y))\n(print (+ 1 2))"
    );
}

#[test]
fn inlines_common_lisp_optional_parameter_default_when_argument_is_missing() {
    let input = "(defun add-default (x &optional (y 10)) (+ x y))\n(print (add-default 1))";
    let plan = inline_plan(input);

    assert_eq!(parameter(&plan.calls[0], "y").argument, "10");
    assert_eq!(plan.calls[0].replacement, "(+ 1 10)");
    assert_eq!(
        plan.rewritten,
        "(defun add-default (x &optional (y 10)) (+ x y))\n(print (+ 1 10))"
    );
}

#[test]
fn inlines_common_lisp_optional_supplied_p_parameter() {
    let input = "(defun maybe (x &optional (y 10 y-p)) (if y-p y x))\n(print (maybe 1 2))";
    let plan = inline_plan(input);

    assert_eq!(parameter(&plan.calls[0], "y-p").argument, "t");
    assert_eq!(plan.calls[0].replacement, "(if t 2 1)");
    assert_eq!(
        plan.rewritten,
        "(defun maybe (x &optional (y 10 y-p)) (if y-p y x))\n(print (if t 2 1))"
    );
}

#[test]
fn inlines_common_lisp_key_parameter_when_argument_is_supplied() {
    let input =
        "(defun render (x &key (style :plain)) (list x style))\n(print (render 1 :style :bold))";
    let plan = inline_plan(input);

    assert_eq!(plan.calls[0].replacement, "(list 1 :bold)");
    assert_eq!(
        plan.rewritten,
        "(defun render (x &key (style :plain)) (list x style))\n(print (list 1 :bold))"
    );
    assert_eq!(plan.calls[0].parameters[1].name, "style");
    assert_eq!(plan.calls[0].parameters[1].argument, ":bold");
}

#[test]
fn inlines_common_lisp_external_key_parameter_designator() {
    let input = "(defun render (x &key ((:external internal) 10)) (list x internal))\n(print (render 1 :external 20))";
    let plan = inline_plan(input);

    assert_eq!(plan.calls[0].replacement, "(list 1 20)");
    assert_eq!(
        plan.rewritten,
        "(defun render (x &key ((:external internal) 10)) (list x internal))\n(print (list 1 20))"
    );
}

#[test]
fn inlines_common_lisp_key_parameter_default_when_argument_is_missing() {
    let input = "(defun render (x &key (style :plain)) (list x style))\n(print (render 1))";
    let plan = inline_plan(input);

    assert_eq!(parameter(&plan.calls[0], "style").argument, ":plain");
    assert_eq!(plan.calls[0].replacement, "(list 1 :plain)");
    assert_eq!(
        plan.rewritten,
        "(defun render (x &key (style :plain)) (list x style))\n(print (list 1 :plain))"
    );
}

#[test]
fn rejects_common_lisp_key_parameter_with_duplicate_argument() {
    let input = "(defun render (x &key style) (list x style))\n(print (render 1 :style :bold :style :plain))";
    let error = inline_error(input, "duplicate key argument must fail");

    assert!(error.to_string().contains("duplicate keyword :style"));
}

#[test]
fn inlines_common_lisp_key_supplied_p_parameter() {
    let input = "(defun render (x &key (style :plain style-p)) (if style-p style x))\n(print (render 1 :style :bold))";
    let plan = inline_plan(input);

    assert_eq!(parameter(&plan.calls[0], "style-p").argument, "t");
    assert_eq!(plan.calls[0].replacement, "(if t :bold 1)");
    assert_eq!(
        plan.rewritten,
        "(defun render (x &key (style :plain style-p)) (if style-p style x))\n(print (if t :bold 1))"
    );
}

#[test]
fn inlines_common_lisp_defmacro_quasiquote_with_unquote_splicing() {
    let input = "(defmacro collect (&rest values) `(list ,@values))\n(print (collect 1 2 3))";
    let plan = inline_plan(input);

    assert_eq!(plan.calls[0].replacement, "(list 1 2 3)");
    assert_eq!(
        plan.rewritten,
        "(defmacro collect (&rest values) `(list ,@values))\n(print (list 1 2 3))"
    );
}

#[test]
fn inlines_common_lisp_defmacro_with_whole_parameter() {
    let input = "(defmacro identity-form (&whole whole value) value)\n(print (identity-form 42))";
    let plan = inline_plan(input);

    assert_eq!(plan.calls[0].replacement, "42");
    assert_eq!(
        plan.rewritten,
        "(defmacro identity-form (&whole whole value) value)\n(print 42)"
    );
}

#[test]
fn inlines_common_lisp_defmacro_with_whole_and_destructuring_parameter() {
    let input = "(defmacro inspect (&whole form (left right)) (list form left right))\n(print (inspect (a b)))";
    let plan = inline_plan(input);

    assert_eq!(plan.calls[0].replacement, "(list (inspect (a b)) a b)");
    assert_eq!(
        plan.rewritten,
        "(defmacro inspect (&whole form (left right)) (list form left right))\n(print (list (inspect (a b)) a b))"
    );
}

#[test]
fn inlines_common_lisp_defmacro_with_whole_and_aux_parameter() {
    let input = "(defmacro inspect (&whole form (value &aux (tag :seen))) (list form value tag))\n(print (inspect (a)))";
    let plan = inline_plan(input);

    assert_eq!(plan.calls[0].replacement, "(list (inspect (a)) a :seen)");
    assert_eq!(
        plan.rewritten,
        "(defmacro inspect (&whole form (value &aux (tag :seen))) (list form value tag))\n(print (list (inspect (a)) a :seen))"
    );
}

#[test]
fn inlines_common_lisp_defmacro_with_body_parameter() {
    let input = "(defmacro collect-forms (&body forms) `(progn ,@forms))\n(print (collect-forms (foo) (bar 1)))";
    let plan = inline_plan(input);

    assert_eq!(plan.calls[0].replacement, "(progn (foo) (bar 1))");
    assert_eq!(
        plan.rewritten,
        "(defmacro collect-forms (&body forms) `(progn ,@forms))\n(print (progn (foo) (bar 1)))"
    );
}

#[test]
fn inlines_common_lisp_defmacro_with_aux_dependency_chain() {
    let input = "(defmacro render-one (x &aux (y x) (z y)) (list y z))\n(print (render-one 1))";
    let plan = duplicate_evaluation_plan(input);

    assert_eq!(plan.calls[0].replacement, "(list 1 1)");
    assert_eq!(
        plan.rewritten,
        "(defmacro render-one (x &aux (y x) (z y)) (list y z))\n(print (list 1 1))"
    );
}

#[test]
fn rejects_common_lisp_defmacro_that_references_environment_parameter() {
    let input = "(defmacro inspect (&environment env value) env)\n(print (inspect 42))";
    let error = inline_error(input, "environment-sensitive macro must fail");

    assert!(
        error
            .to_string()
            .contains("cannot inline macros that reference &environment parameter 'env'")
    );
}

#[test]
fn rejects_common_lisp_defmacro_that_references_environment_parameter_in_aux() {
    let input =
        "(defmacro inspect (&environment env value &aux (tag env)) tag)\n(print (inspect 42))";
    let error = inline_error(
        input,
        "environment-sensitive macro aux initializer must fail",
    );

    assert!(error.to_string().contains(
        "cannot inline macros that reference &environment parameter 'env' in the &aux initializer"
    ));
}

#[test]
fn rejects_common_lisp_defmacro_that_references_environment_parameter_in_nested_optional_default() {
    let input = "(defmacro inspect (&environment env ((value &optional (tag env)))) tag)\n(print (inspect (a)))";
    let error = inline_error(
        input,
        "environment-sensitive macro nested optional initializer must fail",
    );

    assert!(error
        .to_string()
        .contains("cannot inline macros that reference &environment parameter 'env' in the nested &optional default value"));
}

#[test]
fn rejects_common_lisp_defmacro_with_top_level_unquote_splicing() {
    let input = "(defmacro collect (&rest values) `,@values)\n(print (collect 1 2 3))";
    let error = inline_error(input, "top-level unquote-splicing must fail");

    assert!(
        error
            .to_string()
            .contains("unsupported top-level ,@expr in defmacro body")
    );
}

#[test]
fn discovers_all_calls_skips_common_lisp_flet_body_local_callable_calls() {
    let input = "(defun helper (x) (+ x 1))\n(defun render () (flet ((helper (x) (helper x))) (helper 2)) (helper 3))";
    let plan = all_calls_plan(input, Dialect::CommonLisp);

    assert_eq!(
        plan.call_paths
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        vec!["1.3.1.0.2", "1.4"]
    );
    assert_eq!(
        plan.rewritten,
        "(defun helper (x) (+ x 1))\n(defun render () (flet ((helper (x) (+ x 1))) (helper 2)) (+ 3 1))"
    );
}

#[test]
fn discovers_all_calls_skips_common_lisp_labels_local_callable_calls() {
    let input = "(defun helper (x) (+ x 1))\n(defun render () (labels ((helper (x) (if x (helper nil) 0))) (helper t)) (helper 3))";
    let plan = all_calls_plan(input, Dialect::CommonLisp);

    assert_eq!(
        plan.call_paths
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        vec!["1.4"]
    );
    assert_eq!(
        plan.rewritten,
        "(defun helper (x) (+ x 1))\n(defun render () (labels ((helper (x) (if x (helper nil) 0))) (helper t)) (+ 3 1))"
    );
}

#[test]
fn rejects_duplicate_evaluation_by_default() {
    let input = "(defun twice (x) (+ x x))\n(print (twice (next)))";
    let error = inline_error(input, "duplicate evaluation");

    assert!(error.to_string().contains("duplicate argument"));
}

#[test]
fn ignores_shadowed_parameter_references() {
    let input = "(defun outer (x) (let ((x 10)) x))\n(print (outer (next)))";
    let error = inline_error(input, "dropped shadowed argument");

    assert!(error.to_string().contains("drop argument"));
}
