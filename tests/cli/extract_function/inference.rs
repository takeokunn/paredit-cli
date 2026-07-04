use super::*;

#[test]
fn cli_infers_extract_function_params() {
    let mut cmd = paredit();
    cmd.args([
        "extract-function",
        "--path",
        "0.3",
        "--name",
        "area-with-margin",
        "--infer-params",
        "--output",
        "json",
    ])
    .write_stdin("(defun render (width height margin) (+ (* width height) margin))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"inferred_params\": ["))
    .stdout(predicate::str::contains("\"width\""))
    .stdout(predicate::str::contains("\"height\""))
    .stdout(predicate::str::contains("\"margin\""))
    .stdout(predicate::str::contains(
        "\"call\": \"(area-with-margin width height margin)\"",
    ))
    .stdout(predicate::str::contains(
        "\"definition\": \"(defun area-with-margin (width height margin) (+ (* width height) margin))\"",
    ));
}

#[test]
fn cli_infers_extract_function_params_without_call_heads_or_literals() {
    let mut cmd = paredit();
    cmd.args([
        "extract-function",
        "--path",
        "0.3",
        "--name",
        "measure",
        "--infer-params",
        "--output",
        "json",
    ])
    .write_stdin("(defun render (width) (+ width 1 :px nil))")
    .assert()
    .success()
    .stdout(predicate::str::contains(
        "\"params\": [\n    \"width\"\n  ]",
    ))
    .stdout(predicate::str::contains(
        "\"inferred_params\": [\n    \"width\"\n  ]",
    ))
    .stdout(predicate::str::contains("\"call\": \"(measure width)\""))
    .stdout(predicate::str::contains(
        "\"definition\": \"(defun measure (width) (+ width 1 :px nil))\"",
    ));
}

#[test]
fn cli_infers_extract_function_params_without_local_let_bindings() {
    let mut cmd = paredit();
    cmd.args([
        "extract-function",
        "--path",
        "0.3",
        "--name",
        "compute",
        "--infer-params",
        "--output",
        "json",
    ])
    .write_stdin("(defun render (y z) (let ((x y)) (+ x z)))")
    .assert()
    .success()
    .stdout(predicate::str::contains(
        "\"params\": [\n    \"y\",\n    \"z\"\n  ]",
    ))
    .stdout(predicate::str::contains(
        "\"inferred_params\": [\n    \"y\",\n    \"z\"\n  ]",
    ))
    .stdout(predicate::str::contains("\"call\": \"(compute y z)\""))
    .stdout(predicate::str::contains(
        "\"definition\": \"(defun compute (y z) (let ((x y)) (+ x z)))\"",
    ));
}

#[test]
fn cli_infers_extract_function_params_without_sequential_let_bindings() {
    let mut cmd = paredit();
    cmd.args([
        "extract-function",
        "--path",
        "0.3",
        "--name",
        "compute",
        "--infer-params",
        "--output",
        "json",
    ])
    .write_stdin("(defun render (y) (let* ((x y) (z (+ x 1))) (+ z y)))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"params\": [\n    \"y\"\n  ]"))
    .stdout(predicate::str::contains(
        "\"inferred_params\": [\n    \"y\"\n  ]",
    ))
    .stdout(predicate::str::contains("\"call\": \"(compute y)\""))
    .stdout(predicate::str::contains(
        "\"definition\": \"(defun compute (y) (let* ((x y) (z (+ x 1))) (+ z y)))\"",
    ));
}

#[test]
fn cli_infers_extract_function_params_without_symbol_macrolet_bindings() {
    let mut cmd = paredit();
    cmd.args([
        "extract-function",
        "--path",
        "0.3",
        "--name",
        "build",
        "--infer-params",
        "--output",
        "json",
    ])
    .write_stdin(
        "(defun render (outer) (symbol-macrolet ((local (compute outer))) (list local outer)))",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains(
        "\"params\": [\n    \"outer\"\n  ]",
    ))
    .stdout(predicate::str::contains(
        "\"inferred_params\": [\n    \"outer\"\n  ]",
    ))
    .stdout(predicate::str::contains("\"call\": \"(build outer)\""))
    .stdout(predicate::str::contains(
        "\"definition\": \"(defun build (outer) (symbol-macrolet ((local (compute outer))) (list local outer)))\"",
    ));
}

#[test]
fn cli_infers_extract_function_params_without_clojure_destructuring_let_bindings() {
    let mut cmd = paredit();
    cmd.args([
        "extract-function",
        "--dialect",
        "clojure",
        "--path",
        "0.3",
        "--name",
        "compute",
        "--infer-params",
        "--output",
        "json",
    ])
    .write_stdin("(defn render [point scale] (let [[x y] point total (+ x y)] (* total scale)))")
    .assert()
    .success()
    .stdout(predicate::str::contains(
        "\"params\": [\n    \"point\",\n    \"scale\"\n  ]",
    ))
    .stdout(predicate::str::contains(
        "\"inferred_params\": [\n    \"point\",\n    \"scale\"\n  ]",
    ))
    .stdout(predicate::str::contains("\"call\": \"(compute point scale)\""))
    .stdout(predicate::str::contains(
        "\"definition\": \"(defn compute [point scale] (let [[x y] point total (+ x y)] (* total scale)))\"",
    ));
}

#[test]
fn cli_infers_extract_function_params_without_clojure_destructuring_lambda_bindings() {
    let mut cmd = paredit();
    cmd.args([
        "extract-function",
        "--dialect",
        "clojure",
        "--path",
        "0.3",
        "--name",
        "compute",
        "--infer-params",
        "--output",
        "json",
    ])
    .write_stdin("(defn render [scale] (fn [[x y]] (+ x y scale)))")
    .assert()
    .success()
    .stdout(predicate::str::contains(
        "\"params\": [\n    \"scale\"\n  ]",
    ))
    .stdout(predicate::str::contains(
        "\"inferred_params\": [\n    \"scale\"\n  ]",
    ))
    .stdout(predicate::str::contains("\"call\": \"(compute scale)\""))
    .stdout(predicate::str::contains(
        "\"definition\": \"(defn compute [scale] (fn [[x y]] (+ x y scale)))\"",
    ));
}

#[test]
fn cli_infers_extract_function_params_without_common_lisp_lambda_list_init_bindings() {
    let mut cmd = paredit();
    cmd.args([
        "extract-function",
        "--path",
        "0.3",
        "--name",
        "build",
        "--infer-params",
        "--output",
        "json",
    ])
    .write_stdin(
        "(defun render (fallback) (lambda (&optional (value (fallback value) supplied)) (list value supplied fallback)))",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains(
        "\"params\": [\n    \"fallback\"\n  ]",
    ))
    .stdout(predicate::str::contains(
        "\"inferred_params\": [\n    \"fallback\"\n  ]",
    ))
    .stdout(predicate::str::contains("\"call\": \"(build fallback)\""))
    .stdout(predicate::str::contains(
        "\"definition\": \"(defun build (fallback) (lambda (&optional (value (fallback value) supplied)) (list value supplied fallback)))\"",
    ));
}

#[test]
fn cli_infers_extract_function_params_without_define_setf_expander_macro_lambda_list_bindings() {
    let mut cmd = paredit();
    cmd.args([
        "extract-function",
        "--path",
        "0",
        "--name",
        "wrap-expander",
        "--infer-params",
        "--output",
        "json",
    ])
    .write_stdin(
        "(define-setf-expander slot (&whole whole &environment env target) (list whole env target outer))",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains(
        "\"params\": [\n    \"outer\"\n  ]",
    ))
    .stdout(predicate::str::contains(
        "\"inferred_params\": [\n    \"outer\"\n  ]",
    ))
    .stdout(predicate::str::contains("\"call\": \"(wrap-expander outer)\""))
    .stdout(predicate::str::contains(
        "\"definition\": \"(defun wrap-expander (outer) (define-setf-expander slot (&whole whole &environment env target) (list whole env target outer)))\"",
    ));
}

#[test]
fn cli_infers_extract_function_params_without_define_compiler_macro_lambda_list_bindings() {
    let mut cmd = paredit();
    cmd.args([
        "extract-function",
        "--path",
        "0",
        "--name",
        "wrap-compiler-macro",
        "--infer-params",
        "--output",
        "json",
    ])
    .write_stdin(
        "(define-compiler-macro render (&whole whole &environment env target) (list whole env target outer))",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains(
        "\"params\": [\n    \"outer\"\n  ]",
    ))
    .stdout(predicate::str::contains(
        "\"inferred_params\": [\n    \"outer\"\n  ]",
    ))
    .stdout(predicate::str::contains(
        "\"call\": \"(wrap-compiler-macro outer)\"",
    ))
    .stdout(predicate::str::contains(
        "\"definition\": \"(defun wrap-compiler-macro (outer) (define-compiler-macro render (&whole whole &environment env target) (list whole env target outer)))\"",
    ));
}
