use super::*;

#[test]
fn cli_plans_introduce_let_for_common_lisp() {
    assert_plan_output(
        &[
            "introduce-let",
            "--path",
            "0.3.1",
            "--name",
            "product",
            "--output",
            "json",
        ],
        "(defun render () (+ (* width height) margin))",
        &[
            "\"dialect\": \"common-lisp\"",
            "\"binding_value\": \"(* width height)\"",
            "\"replacement\": \"(let ((product (* width height))) (+ product margin))\"",
            "(defun render () (let ((product (* width height))) (+ product margin)))",
        ],
    );
}

#[test]
fn cli_plans_introduce_let_for_all_equivalent_occurrences() {
    assert_plan_output(
        &[
            "introduce-let",
            "--path",
            "0.3.1",
            "--name",
            "product",
            "--all-occurrences",
            "--output",
            "json",
        ],
        "(defun render () (+ (* width height) margin (*  width height)))",
        &[
            "\"occurrence_count\": 2",
            "\"replacement\": \"(let ((product (* width height))) (+ product margin product))\"",
            "(defun render () (let ((product (* width height))) (+ product margin product)))",
        ],
    );
}

#[test]
fn cli_plans_introduce_let_skips_shadowed_all_occurrences() {
    assert_plan_output(
        &[
            "introduce-let",
            "--path",
            "0.3.1",
            "--name",
            "product",
            "--all-occurrences",
            "--output",
            "json",
        ],
        "(defun render () (+ (* width height) (let ((product 1)) (* width height))))",
        &[
            "\"occurrence_count\": 1",
            "\"skipped_shadowed_occurrence_count\": 1",
            "(defun render () (let ((product (* width height))) (+ product (let ((product 1)) (* width height)))))",
        ],
    );
}

#[test]
fn cli_plans_introduce_let_skips_symbol_macrolet_shadowed_all_occurrences() {
    assert_plan_output(
        &[
            "introduce-let",
            "--path",
            "0.3.1",
            "--name",
            "product",
            "--all-occurrences",
            "--output",
            "json",
        ],
        "(defun render () (+ (* width height) (symbol-macrolet ((product 1)) (* width height))))",
        &[
            "\"occurrence_count\": 1",
            "\"skipped_shadowed_occurrence_count\": 1",
            "(defun render () (let ((product (* width height))) (+ product (symbol-macrolet ((product 1)) (* width height)))))",
        ],
    );
}

#[test]
fn cli_plans_introduce_let_keeps_let_star_same_initializer_outer_scope() {
    assert_plan_output(
        &[
            "introduce-let",
            "--path",
            "0.3.1",
            "--name",
            "product",
            "--all-occurrences",
            "--output",
            "json",
        ],
        "(defun render () (+ (* width height) (let* ((product (* width height))) (* width height))))",
        &[
            "\"occurrence_count\": 2",
            "\"skipped_shadowed_occurrence_count\": 1",
            "(defun render () (let ((product (* width height))) (+ product (let* ((product product)) (* width height)))))",
        ],
    );
}

#[test]
fn cli_plans_introduce_let_skips_define_setf_expander_shadowed_all_occurrences() {
    assert_plan_output(
        &[
            "introduce-let",
            "--path",
            "0.3.1",
            "--name",
            "product",
            "--all-occurrences",
            "--output",
            "json",
        ],
        "(defun render () (+ (* width height) (define-setf-expander slot (&environment product place) (* width height))))",
        &[
            "\"occurrence_count\": 1",
            "\"skipped_shadowed_occurrence_count\": 1",
            "(defun render () (let ((product (* width height))) (+ product (define-setf-expander slot (&environment product place) (* width height)))))",
        ],
    );
}

#[test]
fn cli_plans_introduce_let_skips_define_compiler_macro_shadowed_all_occurrences() {
    assert_plan_output(
        &[
            "introduce-let",
            "--path",
            "0.3.1",
            "--name",
            "product",
            "--all-occurrences",
            "--output",
            "json",
        ],
        "(defun render () (+ (* width height) (define-compiler-macro slot (&environment product place) (* width height))))",
        &[
            "\"occurrence_count\": 1",
            "\"skipped_shadowed_occurrence_count\": 1",
            "(defun render () (let ((product (* width height))) (+ product (define-compiler-macro slot (&environment product place) (* width height)))))",
        ],
    );
}

#[test]
fn cli_plans_introduce_let_skips_handler_case_clause_shadow_only() {
    assert_plan_output(
        &[
            "introduce-let",
            "--path",
            "0.3.1",
            "--name",
            "product",
            "--all-occurrences",
            "--output",
            "json",
        ],
        "(defun render () (+ (* width height) (handler-case (* width height) (error (product) (* width height)) (:no-error (value) (* width height)))))",
        &[
            "\"occurrence_count\": 3",
            "\"skipped_shadowed_occurrence_count\": 1",
            "(defun render () (let ((product (* width height))) (+ product (handler-case product (error (product) (* width height)) (:no-error (value) product)))))",
        ],
    );
}

#[test]
fn cli_plans_introduce_let_skips_macrolet_lambda_body_shadow_only() {
    assert_plan_output(
        &[
            "introduce-let",
            "--path",
            "0.3.1",
            "--name",
            "product",
            "--all-occurrences",
            "--output",
            "json",
        ],
        "(defun render () (+ (* width height) (macrolet ((with-product (product) (* width height))) (* width height))))",
        &[
            "\"occurrence_count\": 2",
            "\"skipped_shadowed_occurrence_count\": 1",
            "(defun render () (let ((product (* width height))) (+ product (macrolet ((with-product (product) (* width height))) product))))",
        ],
    );
}
