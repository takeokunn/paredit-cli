use super::*;

#[test]
fn cli_plans_inline_function_with_all_calls() {
    assert_inline_success(
        &["--definition-path", "0", "--all-calls", "--output", "json"],
        "(defun area (width height) (* width height))\n\
         (defun render () (area 10 20))\n\
         (defun summarize () (+ (area 3 4) 1))",
        &[
            "\"all_calls\": true",
            "\"call_paths\": [",
            "\"1.3\"",
            "\"2.3.1\"",
            "\"replacement\": \"(* 10 20)\"",
            "\"replacement\": \"(* 3 4)\"",
            "(defun render () (* 10 20))",
            "(defun summarize () (+ (* 3 4) 1))",
        ],
        &[],
    );
}

#[test]
fn cli_plans_inline_function_all_calls_without_common_lisp_labels_shadowed_calls() {
    assert_inline_success(
        &[
            "--dialect",
            "common-lisp",
            "--definition-path",
            "0",
            "--all-calls",
            "--output",
            "json",
        ],
        "(defun helper (x) (+ x 1))\n\
         (defun render () (labels ((helper (x) (if x (helper nil) 0))) (helper t)) (helper 3))",
        &[
            "\"all_calls\": true",
            "\"call_paths\": [",
            "\"1.4\"",
            "\"replacement\": \"(+ 3 1)\"",
            "(labels ((helper (x) (if x (helper nil) 0))) (helper t)) (+ 3 1)",
        ],
        &["\"1.3.1.0.2.2\""],
    );
}

#[test]
fn cli_plans_inline_function_all_calls_with_emacs_lisp_cl_flet_global_references() {
    assert_inline_success(
        &[
            "--dialect",
            "emacs-lisp",
            "--definition-path",
            "0",
            "--all-calls",
            "--output",
            "json",
        ],
        "(defun helper (x) (+ x 1))\n\
         (defun render () (cl-flet ((helper (x) (helper x))) (helper 2)) (helper 3))",
        &[
            "\"dialect\": \"emacs-lisp\"",
            "\"all_calls\": true",
            "\"call_paths\": [",
            "\"1.3.1.0.2\"",
            "\"1.4\"",
            "\"replacement\": \"(+ x 1)\"",
            "\"replacement\": \"(+ 3 1)\"",
            "(cl-flet ((helper (x) (+ x 1))) (helper 2)) (+ 3 1)",
        ],
        &[],
    );
}

#[test]
fn cli_plans_inline_function_all_calls_without_emacs_lisp_cl_labels_shadowed_calls() {
    assert_inline_success(
        &[
            "--dialect",
            "emacs-lisp",
            "--definition-path",
            "0",
            "--all-calls",
            "--output",
            "json",
        ],
        "(defun helper (x) (+ x 1))\n\
         (defun render () (cl-labels ((helper (x) (if x (helper nil) 0))) (helper t)) (helper 3))",
        &[
            "\"dialect\": \"emacs-lisp\"",
            "\"all_calls\": true",
            "\"call_paths\": [",
            "\"1.4\"",
            "\"replacement\": \"(+ 3 1)\"",
            "(cl-labels ((helper (x) (if x (helper nil) 0))) (helper t)) (+ 3 1)",
        ],
        &["\"1.3.1.0.2.2\""],
    );
}

#[test]
fn cli_plans_inline_function_all_calls_respects_common_lisp_macrolet_shadowing() {
    assert_inline_success(
        &[
            "--dialect",
            "common-lisp",
            "--definition-path",
            "0",
            "--all-calls",
            "--output",
            "json",
        ],
        "(defun helper (x) (+ x 1))\n\
         (defun render () (macrolet ((helper (x) (helper x))) (helper 2)) (helper 3))",
        &[
            "\"all_calls\": true",
            "\"call_paths\": [",
            "\"1.3.1.0.2\"",
            "\"1.4\"",
            "\"replacement\": \"(+ x 1)\"",
            "\"replacement\": \"(+ 3 1)\"",
            "(macrolet ((helper (x) (+ x 1))) (helper 2)) (+ 3 1)",
        ],
        &[],
    );
}

#[test]
fn cli_plans_inline_function_all_calls_respects_common_lisp_cl_macrolet_shadowing() {
    assert_inline_success(
        &[
            "--dialect",
            "common-lisp",
            "--definition-path",
            "0",
            "--all-calls",
            "--output",
            "json",
        ],
        "(defun helper (x) (+ x 1))\n\
         (defun render () (cl:macrolet ((helper (x) (helper x))) (helper 2)) (helper 3))",
        &[
            "\"function_name\": \"helper\"",
            "\"call_paths\": [",
            "\"1.3.1.0.2\"",
            "\"1.4\"",
            "(cl:macrolet ((helper (x) (+ x 1))) (helper 2)) (+ 3 1)",
        ],
        &[],
    );
}

#[test]
fn cli_plans_inline_function_all_calls_respects_common_lisp_cl_user_macrolet_shadowing() {
    assert_inline_success(
        &[
            "--dialect",
            "common-lisp",
            "--definition-path",
            "0",
            "--all-calls",
            "--output",
            "json",
        ],
        "(defun helper (x) (+ x 1))\n\
         (defun render () (cl-user:macrolet ((helper (x) (helper x))) (helper 2)) (helper 3))",
        &[
            "\"function_name\": \"helper\"",
            "\"call_paths\": [",
            "\"1.3.1.0.2\"",
            "\"1.4\"",
            "(cl-user:macrolet ((helper (x) (+ x 1))) (helper 2)) (+ 3 1)",
        ],
        &[],
    );
}

#[test]
fn cli_plans_inline_function_all_calls_respects_common_lisp_compiler_macrolet_shadowing() {
    assert_inline_success(
        &[
            "--dialect",
            "common-lisp",
            "--definition-path",
            "0",
            "--all-calls",
            "--output",
            "json",
        ],
        "(defun helper (x) (+ x 1))\n\
         (defun render () (compiler-macrolet ((helper (x) (helper x))) (helper 2)) (helper 3))",
        &[
            "\"all_calls\": true",
            "\"call_paths\": [",
            "\"1.3.1.0.2\"",
            "\"1.4\"",
            "\"replacement\": \"(+ x 1)\"",
            "\"replacement\": \"(+ 3 1)\"",
            "(compiler-macrolet ((helper (x) (+ x 1))) (helper 2)) (+ 3 1)",
        ],
        &[],
    );
}

#[test]
fn cli_plans_inline_function_all_calls_respects_common_lisp_cl_user_compiler_macrolet_shadowing() {
    assert_inline_success(
        &[
            "--dialect",
            "common-lisp",
            "--definition-path",
            "0",
            "--all-calls",
            "--output",
            "json",
        ],
        "(defun helper (x) (+ x 1))\n\
         (defun render () (cl-user:compiler-macrolet ((helper (x) (helper x))) (helper 2)) (helper 3))",
        &[
            "\"function_name\": \"helper\"",
            "\"call_paths\": [",
            "\"1.3.1.0.2\"",
            "\"1.4\"",
            "(cl-user:compiler-macrolet ((helper (x) (+ x 1))) (helper 2)) (+ 3 1)",
        ],
        &[],
    );
}

#[test]
fn cli_plans_inline_function_all_calls_respects_emacs_lisp_cl_macrolet_shadowing() {
    assert_inline_success(
        &[
            "--dialect",
            "emacs-lisp",
            "--definition-path",
            "0",
            "--all-calls",
            "--output",
            "json",
        ],
        "(defun helper (x) (+ x 1))\n\
         (defun render () (cl-macrolet ((helper (x) (helper x))) (helper 2)) (helper 3))",
        &[
            "\"dialect\": \"emacs-lisp\"",
            "\"all_calls\": true",
            "\"call_paths\": [",
            "\"1.3.1.0.2\"",
            "\"1.4\"",
            "\"replacement\": \"(+ x 1)\"",
            "\"replacement\": \"(+ 3 1)\"",
            "(cl-macrolet ((helper (x) (+ x 1))) (helper 2)) (+ 3 1)",
        ],
        &[],
    );
}

#[test]
fn cli_rejects_inline_function_explicit_labels_shadowed_call_path() {
    assert_inline_failure(
        &[
            "--dialect",
            "common-lisp",
            "--definition-path",
            "0",
            "--call-path",
            "1.3.1.0.2.2",
        ],
        Some(
            "(defun helper (x) (+ x 1))\n\
             (defun render () (labels ((helper (x) (if x (helper nil) 0))) (helper t)) (helper 3))",
        ),
        &["shadowed by a local callable binding"],
    );
}

#[test]
fn cli_rejects_inline_function_explicit_emacs_lisp_cl_labels_shadowed_call_path() {
    assert_inline_failure(
        &[
            "--dialect",
            "emacs-lisp",
            "--definition-path",
            "0",
            "--call-path",
            "1.3.1.0.2.2",
        ],
        Some(
            "(defun helper (x) (+ x 1))\n\
             (defun render () (cl-labels ((helper (x) (if x (helper nil) 0))) (helper t)) (helper 3))",
        ),
        &["shadowed by a local callable binding"],
    );
}

#[test]
fn cli_rejects_inline_function_explicit_macrolet_shadowed_call_path() {
    assert_inline_failure(
        &[
            "--dialect",
            "common-lisp",
            "--definition-path",
            "0",
            "--call-path",
            "1.3.2",
        ],
        Some(
            "(defun helper (x) (+ x 1))\n\
             (defun render () (macrolet ((helper (x) (helper x))) (helper 2)) (helper 3))",
        ),
        &["shadowed by a local callable binding"],
    );
}

#[test]
fn cli_rejects_inline_function_explicit_cl_user_macrolet_shadowed_call_path() {
    assert_inline_failure(
        &[
            "--dialect",
            "common-lisp",
            "--definition-path",
            "0",
            "--call-path",
            "1.3.2",
        ],
        Some(
            "(defun helper (x) (+ x 1))\n\
             (defun render () (cl-user:macrolet ((helper (x) (helper x))) (helper 2)) (helper 3))",
        ),
        &["shadowed by a local callable binding"],
    );
}

#[test]
fn cli_rejects_inline_function_explicit_compiler_macrolet_shadowed_call_path() {
    assert_inline_failure(
        &[
            "--dialect",
            "common-lisp",
            "--definition-path",
            "0",
            "--call-path",
            "1.3.2",
        ],
        Some(
            "(defun helper (x) (+ x 1))\n\
             (defun render () (compiler-macrolet ((helper (x) (helper x))) (helper 2)) (helper 3))",
        ),
        &["shadowed by a local callable binding"],
    );
}

#[test]
fn cli_rejects_inline_function_explicit_cl_user_compiler_macrolet_shadowed_call_path() {
    assert_inline_failure(
        &[
            "--dialect",
            "common-lisp",
            "--definition-path",
            "0",
            "--call-path",
            "1.3.2",
        ],
        Some(
            "(defun helper (x) (+ x 1))\n\
             (defun render () (cl-user:compiler-macrolet ((helper (x) (helper x))) (helper 2)) (helper 3))",
        ),
        &["shadowed by a local callable binding"],
    );
}

#[test]
fn cli_rejects_inline_function_explicit_emacs_lisp_cl_macrolet_shadowed_call_path() {
    assert_inline_failure(
        &[
            "--dialect",
            "emacs-lisp",
            "--definition-path",
            "0",
            "--call-path",
            "1.3.2",
        ],
        Some(
            "(defun helper (x) (+ x 1))\n\
             (defun render () (cl-macrolet ((helper (x) (helper x))) (helper 2)) (helper 3))",
        ),
        &["shadowed by a local callable binding"],
    );
}
