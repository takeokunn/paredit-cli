use super::assert_extract_function_inference;

#[test]
fn cli_infers_extract_function_params_without_common_lisp_flet_shadowing() {
    assert_extract_function_inference(
        &[
            "--path",
            "0.3",
            "--name",
            "build",
            "--infer-params",
            "--output",
            "json",
        ],
        "(defun render (outer input) (flet ((helper (local) (+ local outer))) (helper input)))",
        &["outer", "input"],
        &["outer", "input"],
        "(build outer input)",
        Some(
            "(defun build (outer input) (flet ((helper (local) (+ local outer))) (helper input)))",
        ),
    );
}

#[test]
fn cli_infers_extract_function_params_without_emacs_lisp_flet_shadowing() {
    assert_extract_function_inference(
        &[
            "--dialect",
            "emacs-lisp",
            "--path",
            "0.3",
            "--name",
            "build",
            "--infer-params",
            "--output",
            "json",
        ],
        "(defun render (outer input) (cl-flet ((helper (local) (+ local outer))) (helper input)))",
        &["outer", "input"],
        &["outer", "input"],
        "(build outer input)",
        None,
    );
}

#[test]
fn cli_infers_extract_function_params_without_common_lisp_labels_shadowing() {
    assert_extract_function_inference(
        &[
            "--path",
            "0.3",
            "--name",
            "build",
            "--infer-params",
            "--output",
            "json",
        ],
        "(defun render (outer input) (labels ((helper (local) (if local (helper outer) outer))) (helper input)))",
        &["outer", "input"],
        &["outer", "input"],
        "(build outer input)",
        Some(
            "(defun build (outer input) (labels ((helper (local) (if local (helper outer) outer))) (helper input)))",
        ),
    );
}

#[test]
fn cli_infers_extract_function_params_without_common_lisp_macrolet_shadowing() {
    assert_extract_function_inference(
        &[
            "--path",
            "0.3",
            "--name",
            "build",
            "--infer-params",
            "--output",
            "json",
        ],
        "(defun render (outer input) (macrolet ((with-local (local) (list local outer))) (with-local input)))",
        &["outer", "input"],
        &["outer", "input"],
        "(build outer input)",
        Some(
            "(defun build (outer input) (macrolet ((with-local (local) (list local outer))) (with-local input)))",
        ),
    );
}

#[test]
fn cli_infers_extract_function_params_without_cl_user_macrolet_shadowing() {
    assert_extract_function_inference(
        &[
            "--path",
            "0.3",
            "--name",
            "build",
            "--infer-params",
            "--output",
            "json",
        ],
        "(defun render (outer input) (cl-user:macrolet ((with-local (local) (list local outer))) (with-local input)))",
        &["outer", "input"],
        &["outer", "input"],
        "(build outer input)",
        Some(
            "(defun build (outer input) (cl-user:macrolet ((with-local (local) (list local outer))) (with-local input)))",
        ),
    );
}

#[test]
fn cli_infers_extract_function_params_without_common_lisp_package_macrolet_shadowing() {
    assert_extract_function_inference(
        &[
            "--path",
            "0.3",
            "--name",
            "build",
            "--infer-params",
            "--output",
            "json",
        ],
        "(defun render (outer input) (cl:macrolet ((with-local (local) (list local outer))) (with-local input)))",
        &["outer", "input"],
        &["outer", "input"],
        "(build outer input)",
        Some(
            "(defun build (outer input) (cl:macrolet ((with-local (local) (list local outer))) (with-local input)))",
        ),
    );
}

#[test]
fn cli_infers_extract_function_params_without_common_lisp_compiler_macrolet_shadowing() {
    assert_extract_function_inference(
        &[
            "--path",
            "0.3",
            "--name",
            "wrap-compiler-macrolet",
            "--infer-params",
            "--output",
            "json",
        ],
        "(defun render (outer) (compiler-macrolet ((with-local (local) (list local outer))) (with-local input)))",
        &["outer", "input"],
        &["outer", "input"],
        "(wrap-compiler-macrolet outer input)",
        Some(
            "(defun wrap-compiler-macrolet (outer input) (compiler-macrolet ((with-local (local) (list local outer))) (with-local input)))",
        ),
    );
}

#[test]
fn cli_infers_extract_function_params_without_common_lisp_package_compiler_macrolet_shadowing() {
    assert_extract_function_inference(
        &[
            "--path",
            "0.3",
            "--name",
            "wrap-compiler-macrolet",
            "--infer-params",
            "--output",
            "json",
        ],
        "(defun render (outer) (cl:compiler-macrolet ((with-local (local) (list local outer))) (with-local input)))",
        &["outer", "input"],
        &["outer", "input"],
        "(wrap-compiler-macrolet outer input)",
        Some(
            "(defun wrap-compiler-macrolet (outer input) (cl:compiler-macrolet ((with-local (local) (list local outer))) (with-local input)))",
        ),
    );
}

#[test]
fn cli_infers_extract_function_params_without_cl_user_compiler_macrolet_shadowing() {
    assert_extract_function_inference(
        &[
            "--path",
            "0.3",
            "--name",
            "wrap-cl-user-compiler-macrolet",
            "--infer-params",
            "--output",
            "json",
        ],
        "(defun render (outer) (cl-user:compiler-macrolet ((with-local (local) (list local outer))) (with-local input)))",
        &["outer", "input"],
        &["outer", "input"],
        "(wrap-cl-user-compiler-macrolet outer input)",
        Some(
            "(defun wrap-cl-user-compiler-macrolet (outer input) (cl-user:compiler-macrolet ((with-local (local) (list local outer))) (with-local input)))",
        ),
    );
}
