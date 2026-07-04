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
fn treats_destructuring_bind_names_as_local_to_body() {
    let params = infer_at(
        "(destructuring-bind (local other) row (list local other outer))",
        &[0],
        &[],
    );

    assert_eq!(params, vec!["row", "outer"]);
}

#[test]
fn treats_multiple_value_bind_names_as_local_to_body() {
    let params = infer_at(
        "(multiple-value-bind (value foundp) (lookup key table) (list value foundp outer))",
        &[0],
        &[],
    );

    assert_eq!(params, vec!["key", "table", "outer"]);
}

#[test]
fn treats_handler_case_clause_lambda_lists_as_local() {
    let params = infer_at(
        "(handler-case (risky input) (error (condition) (recover condition outer)) (:no-error (value) (finish value done)))",
        &[0],
        &[],
    );

    assert_eq!(params, vec!["input", "outer", "done"]);
}

#[test]
fn treats_restart_case_clause_lambda_lists_as_local() {
    let params = infer_at(
        "(restart-case (risky input) (retry (condition) (recover condition outer)) (skip () fallback))",
        &[0],
        &[],
    );

    assert_eq!(params, vec!["input", "outer", "fallback"]);
}

#[test]
fn treats_dolist_iteration_variable_as_local_to_result_and_body() {
    let params = infer_at(
        "(dolist (item items (finish item done)) (render item outer))",
        &[0],
        &[],
    );

    assert_eq!(params, vec!["items", "done", "outer"]);
}

#[test]
fn treats_dotimes_iteration_variable_as_local_to_result_and_body() {
    let params = infer_at(
        "(dotimes (index count (finish index done)) (render index outer))",
        &[0],
        &[],
    );

    assert_eq!(params, vec!["count", "done", "outer"]);
}

#[test]
fn treats_with_slots_names_as_local_to_body() {
    let params = infer_at(
        "(with-slots (width (height slot-height)) panel (list width height outer))",
        &[0],
        &[],
    );

    assert_eq!(params, vec!["panel", "outer"]);
}

#[test]
fn treats_with_accessors_names_as_local_to_body() {
    let params = infer_at(
        "(with-accessors ((width panel-width) height) panel (list width height outer))",
        &[0],
        &[],
    );

    assert_eq!(params, vec!["panel", "outer"]);
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
