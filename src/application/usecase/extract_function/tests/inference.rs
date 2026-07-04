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
fn excludes_destructured_lambda_parameters_and_explicit_params() {
    let params = infer_at(
        "(lambda [{:keys [inner]}] (+ inner outer ignored))",
        &[0],
        &["ignored"],
    );

    assert_eq!(params, vec!["outer"]);
}
