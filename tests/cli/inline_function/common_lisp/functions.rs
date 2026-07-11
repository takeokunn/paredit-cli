use super::super::*;

#[test]
fn cli_plans_inline_function_with_common_lisp_key_parameter() {
    assert_inline_success(
        &[
            "--dialect",
            "common-lisp",
            "--definition-path",
            "0",
            "--call-path",
            "1.3",
            "--output",
            "json",
        ],
        "(defun render-one (x &key (style :plain)) (list x style))\n\
         (defun render () (render-one 1 :style :bold))",
        &[
            "\"function_name\": \"render-one\"",
            "\"replacement\": \"(list 1 :bold)\"",
            "\"name\": \"style\"",
            "\"argument\": \":bold\"",
            "(defun render () (list 1 :bold))",
        ],
        &[],
    );
}

#[test]
fn cli_plans_inline_function_with_common_lisp_allow_other_keys_and_rest() {
    assert_common_lisp_inline_success(
        "(defun render-one (x &rest rest &key (style :plain) &allow-other-keys) (list x style rest))\n\
         (print (render-one 1 :style :bold :size 10))",
        &[
            "\"function_name\": \"render-one\"",
            "\"replacement\": \"(list 1 :bold (:style :bold :size 10))\"",
            "\"name\": \"rest\"",
            "\"argument\": \"(:style :bold :size 10)\"",
            "(print (list 1 :bold (:style :bold :size 10)))",
        ],
        &[],
    );
}

#[test]
fn cli_plans_inline_function_with_common_lisp_allow_other_keys_without_rest_when_dropping_is_allowed()
 {
    assert_common_lisp_inline_success_with_args(
        &["--allow-drop-arguments"],
        "(defun render-one (x &key (style :plain) &allow-other-keys) (list x style))\n\
         (print (render-one 1 :style :bold :size 10))",
        &[
            "\"function_name\": \"render-one\"",
            "\"replacement\": \"(list 1 :bold)\"",
            "\"name\": \"style\"",
            "\"argument\": \":bold\"",
            "(print (list 1 :bold))",
        ],
        &[],
    );
}

#[test]
fn cli_plans_inline_function_with_common_lisp_optional_supplied_p_parameter() {
    assert_common_lisp_inline_success(
        "(defun maybe (x &optional (y 10 y-p)) (if y-p y x))\n\
         (print (maybe 1 2))",
        &[
            "\"function_name\": \"maybe\"",
            "\"replacement\": \"(if t 2 1)\"",
            "\"name\": \"y-p\"",
            "\"argument\": \"t\"",
            "(print (if t 2 1))",
        ],
        &[],
    );
}

#[test]
fn cli_plans_inline_function_with_common_lisp_aux_binding_when_duplicate_evaluation_is_allowed() {
    assert_common_lisp_inline_success_with_args(
        &["--allow-duplicate-evaluation"],
        "(defun render-one (x &aux (y x) (z y)) (list y z))\n\
         (print (render-one 1))",
        &[
            "\"function_name\": \"render-one\"",
            "\"replacement\": \"(list 1 1)\"",
            "\"name\": \"y\"",
            "\"argument\": \"x\"",
            "\"name\": \"z\"",
            "\"argument\": \"x\"",
            "(print (list 1 1))",
        ],
        &[],
    );
}
