use super::*;

#[test]
fn cli_writes_introduce_let_for_emacs_lisp_file() {
    assert_written_file(
        "introduce-let",
        "render.el",
        "(defun render () (+ (* width height) margin))\n",
        &["--path", "0.3.1", "--name", "product", "--write"],
        &["\"dialect\": \"emacs-lisp\"", "\"written\": true"],
        "(defun render () (let ((product (* width height))) (+ product margin)))\n",
    );
}

#[test]
fn cli_writes_introduce_let_for_emacs_lisp_cl_symbol_macrolet_shadowing() {
    assert_written_file(
        "introduce-let-emacs-shadow",
        "render.el",
        "(defun render () (+ (* width height) (cl-symbol-macrolet ((product 1)) (* width height))))\n",
        &[
            "--path",
            "0.3.1",
            "--name",
            "product",
            "--all-occurrences",
            "--write",
        ],
        &[
            "\"dialect\": \"emacs-lisp\"",
            "\"occurrence_count\": 1",
            "\"skipped_shadowed_occurrence_count\": 1",
            "\"written\": true",
        ],
        "(defun render () (let ((product (* width height))) (+ product (cl-symbol-macrolet ((product 1)) (* width height)))))\n",
    );
}

#[test]
fn cli_writes_introduce_let_for_all_equivalent_occurrences() {
    assert_written_file(
        "introduce-let-all-occurrences",
        "render.lisp",
        "(defun render () (+ (* width height) margin (*  width height)))\n",
        &[
            "--path",
            "0.3.1",
            "--name",
            "product",
            "--all-occurrences",
            "--write",
        ],
        &["\"occurrence_count\": 2", "\"written\": true"],
        "(defun render () (let ((product (* width height))) (+ product margin product)))\n",
    );
}

#[test]
fn cli_writes_introduce_let_without_shadowed_occurrence_capture() {
    assert_written_file(
        "introduce-let-shadowed-all-occurrences",
        "render.lisp",
        "(defun render () (+ (* width height) (let ((product 1)) (* width height))))\n",
        &[
            "--path",
            "0.3.1",
            "--name",
            "product",
            "--all-occurrences",
            "--write",
        ],
        &[
            "\"occurrence_count\": 1",
            "\"skipped_shadowed_occurrence_count\": 1",
            "\"written\": true",
        ],
        "(defun render () (let ((product (* width height))) (+ product (let ((product 1)) (* width height)))))\n",
    );
}
