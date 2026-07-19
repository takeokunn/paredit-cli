use super::assert_extract_function_inference;

#[test]
fn cli_infers_extract_function_params_without_symbol_macrolet_bindings() {
    assert_extract_function_inference(
        &[
            "--dialect",
            "common-lisp",
            "--path",
            "0.3",
            "--name",
            "build",
            "--infer-params",
            "--output",
            "json",
        ],
        "(defun render (outer) (symbol-macrolet ((local (compute outer))) (list local outer)))",
        &["outer"],
        &["outer"],
        "(build outer)",
        Some(
            "(defun build (outer) (symbol-macrolet ((local (compute outer))) (list local outer)))",
        ),
    );
}

#[test]
fn cli_infers_extract_function_params_without_package_qualified_symbol_macrolet_bindings() {
    assert_extract_function_inference(
        &[
            "--dialect",
            "common-lisp",
            "--path",
            "0.3",
            "--name",
            "build",
            "--infer-params",
            "--output",
            "json",
        ],
        "(defun render (outer) (symbol-macrolet ((cl:product (compute outer))) (list product outer)))",
        &["outer"],
        &["outer"],
        "(build outer)",
        Some(
            "(defun build (outer) (symbol-macrolet ((cl:product (compute outer))) (list product outer)))",
        ),
    );
}

#[test]
fn cli_infers_extract_function_params_without_emacs_lisp_symbol_macrolet_bindings() {
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
        "(defun render (outer) (cl-symbol-macrolet ((local (compute outer))) (list local outer)))",
        &["outer"],
        &["outer"],
        "(build outer)",
        None,
    );
}

#[test]
fn cli_infers_extract_function_params_without_clojure_destructuring_let_bindings() {
    assert_extract_function_inference(
        &[
            "--dialect",
            "clojure",
            "--path",
            "0.3",
            "--name",
            "compute",
            "--infer-params",
            "--output",
            "json",
        ],
        "(defn render [point scale] (let [[x y] point total (+ x y)] (* total scale)))",
        &["point", "scale"],
        &["point", "scale"],
        "(compute point scale)",
        Some("(defn compute [point scale] (let [[x y] point total (+ x y)] (* total scale)))"),
    );
}

#[test]
fn cli_infers_extract_function_params_without_clojure_destructuring_lambda_bindings() {
    assert_extract_function_inference(
        &[
            "--dialect",
            "clojure",
            "--path",
            "0.3",
            "--name",
            "compute",
            "--infer-params",
            "--output",
            "json",
        ],
        "(defn render [scale] (fn [[x y]] (+ x y scale)))",
        &["scale"],
        &["scale"],
        "(compute scale)",
        Some("(defn compute [scale] (fn [[x y]] (+ x y scale)))"),
    );
}

#[test]
fn cli_infers_extract_function_params_without_bare_symbol_let_bindings() {
    assert_extract_function_inference(
        &[
            "--dialect",
            "common-lisp",
            "--path",
            "0.3",
            "--name",
            "helper",
            "--infer-params",
            "--output",
            "json",
        ],
        "(defun caller (x) (let ((y (helper2 x)) found) (dolist (p y found) (setq found t))))",
        &["x"],
        &["x"],
        "(helper x)",
        Some(
            "(defun helper (x) (let ((y (helper2 x)) found) (dolist (p y found) (setq found t))))",
        ),
    );
}
