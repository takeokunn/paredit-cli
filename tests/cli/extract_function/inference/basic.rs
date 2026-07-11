use super::assert_extract_function_inference;

#[test]
fn cli_infers_extract_function_params() {
    assert_extract_function_inference(
        &[
            "--path",
            "0.3",
            "--name",
            "area-with-margin",
            "--infer-params",
            "--output",
            "json",
        ],
        "(defun render (width height margin) (+ (* width height) margin))",
        &["width", "height", "margin"],
        &["width", "height", "margin"],
        "(area-with-margin width height margin)",
        Some("(defun area-with-margin (width height margin) (+ (* width height) margin))"),
    );
}

#[test]
fn cli_infers_extract_function_params_without_call_heads_or_literals() {
    assert_extract_function_inference(
        &[
            "--path",
            "0.3",
            "--name",
            "measure",
            "--infer-params",
            "--output",
            "json",
        ],
        "(defun render (width) (+ width 1 :px nil))",
        &["width"],
        &["width"],
        "(measure width)",
        Some("(defun measure (width) (+ width 1 :px nil))"),
    );
}

#[test]
fn cli_infers_extract_function_params_without_local_let_bindings() {
    assert_extract_function_inference(
        &[
            "--path",
            "0.3",
            "--name",
            "compute",
            "--infer-params",
            "--output",
            "json",
        ],
        "(defun render (y z) (let ((x y)) (+ x z)))",
        &["y", "z"],
        &["y", "z"],
        "(compute y z)",
        Some("(defun compute (y z) (let ((x y)) (+ x z)))"),
    );
}

#[test]
fn cli_infers_extract_function_params_without_emacs_lisp_local_let_bindings() {
    assert_extract_function_inference(
        &[
            "--dialect",
            "emacs-lisp",
            "--path",
            "0.3",
            "--name",
            "compute",
            "--infer-params",
            "--output",
            "json",
        ],
        "(defun render (y z) (let ((x y)) (+ x z)))",
        &["y", "z"],
        &["y", "z"],
        "(compute y z)",
        None,
    );
}

#[test]
fn cli_infers_extract_function_params_without_sequential_let_bindings() {
    assert_extract_function_inference(
        &[
            "--path",
            "0.3",
            "--name",
            "compute",
            "--infer-params",
            "--output",
            "json",
        ],
        "(defun render (y) (let* ((x y) (z (+ x 1))) (+ z y)))",
        &["y"],
        &["y"],
        "(compute y)",
        Some("(defun compute (y) (let* ((x y) (z (+ x 1))) (+ z y)))"),
    );
}
