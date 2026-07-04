use super::*;

#[test]
fn infers_free_variables_from_selected_expression() {
    let params = infer_at(
        "(defun render (width height margin) (+ (* width height) margin))",
        &[0, 3],
        &[],
    );

    assert_eq!(params, vec!["width", "height", "margin"]);
}

#[test]
fn excludes_local_let_bindings_from_body() {
    let params = infer_at("(let ((local input)) (+ local outer))", &[0], &[]);

    assert_eq!(params, vec!["input", "outer"]);
}

#[test]
fn treats_let_star_bindings_as_sequential() {
    let params = infer_at(
        "(let* ((first input) (second first)) (+ first second outer))",
        &[0],
        &[],
    );

    assert_eq!(params, vec!["input", "outer"]);
}

#[test]
fn treats_symbol_macrolet_body_names_as_local() {
    let params = infer_at(
        "(symbol-macrolet ((local (compute outer))) (list local outer))",
        &[0],
        &[],
    );

    assert_eq!(params, vec!["outer"]);
}

#[test]
fn treats_flet_lambda_list_as_local_to_function_body() {
    let params = infer_at(
        "(flet ((helper (local) (+ local outer))) (helper input))",
        &[0],
        &[],
    );

    assert_eq!(params, vec!["outer", "input"]);
}

#[test]
fn treats_labels_recursive_callable_names_as_non_params() {
    let params = infer_at(
        "(labels ((helper (local) (if local (helper outer) outer))) (helper input))",
        &[0],
        &[],
    );

    assert_eq!(params, vec!["outer", "input"]);
}

#[test]
fn treats_macrolet_lambda_list_as_local_to_expander_body() {
    let params = infer_at(
        "(macrolet ((with-local (local) (list local outer))) (with-local input))",
        &[0],
        &[],
    );

    assert_eq!(params, vec!["outer", "input"]);
}

#[test]
fn treats_define_setf_expander_macro_lambda_list_as_local_to_expander_body() {
    let params = infer_at(
        "(define-setf-expander slot (&whole whole &environment env target) (list whole env target outer))",
        &[0],
        &[],
    );

    assert_eq!(params, vec!["outer"]);
}

#[test]
fn treats_define_compiler_macro_lambda_list_as_local_to_expander_body() {
    let params = infer_at(
        "(define-compiler-macro render (&whole whole &environment env target) (list whole env target outer))",
        &[0],
        &[],
    );

    assert_eq!(params, vec!["outer"]);
}

#[test]
fn keeps_flet_callable_name_as_free_value_reference() {
    let params = infer_at("(flet ((helper () outer)) (list helper))", &[0], &[]);

    assert_eq!(params, vec!["outer", "helper"]);
}

#[test]
fn excludes_destructured_lambda_parameters_and_explicit_params() {
    let params = infer_at(
        "(lambda [{:keys [inner]}] (+ inner outer ignored))",
        &[0],
        &["ignored"],
    );

    assert_eq!(params, vec!["outer"]);
}
