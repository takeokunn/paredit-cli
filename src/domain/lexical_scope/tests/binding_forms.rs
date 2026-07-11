use super::*;

#[test]
fn destructuring_bind_checks_value_before_body_shadowing() {
    let input = "(list x (destructuring-bind (x) x x) x)";

    assert_eq!(reference_texts(input, "x"), vec!["x", "x", "x"]);
}

#[test]
fn multiple_value_bind_checks_value_before_body_shadowing() {
    let input = "(list x (multiple-value-bind (x) x x) x)";

    assert_eq!(reference_texts(input, "x"), vec!["x", "x", "x"]);
}

#[test]
fn qualified_common_lisp_binding_heads_check_value_before_body_shadowing() {
    let input = "(list x (cl:destructuring-bind (x) x x) (cl-user:multiple-value-bind (x) x x) x)";

    assert_eq!(reference_texts(input, "x"), vec!["x", "x", "x", "x"]);
}

#[test]
fn handler_case_clause_parameters_shadow_only_clause_body() {
    let input = "(list condition (handler-case (risky condition) (error (condition) condition) (:no-error (value) condition)) condition)";

    assert_eq!(
        reference_texts(input, "condition"),
        vec!["condition", "condition", "condition", "condition"]
    );
}

#[test]
fn restart_case_clause_parameters_shadow_only_clause_body() {
    let input = "(list condition (restart-case (risky condition) (retry (condition) condition) (skip () condition)) condition)";

    assert_eq!(
        reference_texts(input, "condition"),
        vec!["condition", "condition", "condition", "condition"]
    );
}

#[test]
fn handler_bind_function_lambda_parameters_shadow_only_handler_body() {
    let input = "(list condition (handler-bind ((error (lambda (condition) condition))) condition) condition)";

    assert_eq!(
        reference_texts(input, "condition"),
        vec!["condition", "condition", "condition"]
    );
}

#[test]
fn restart_bind_scans_restart_function_and_option_values_with_local_lambda_shadowing() {
    let input = "(list stream (restart-bind ((retry (lambda () stream) :report (lambda (stream) stream))) stream) stream)";

    assert_eq!(
        reference_texts(input, "stream"),
        vec!["stream", "stream", "stream", "stream"]
    );
}

#[test]
fn dolist_iteration_variable_shadows_body_and_result() {
    let input = "(list value (dolist (value values value) value) value)";

    assert_eq!(reference_texts(input, "value"), vec!["value", "value"]);
}

#[test]
fn dotimes_iteration_variable_shadows_body_and_result() {
    let input = "(list limit (dotimes (limit limit limit) limit) limit)";

    assert_eq!(
        reference_texts(input, "limit"),
        vec!["limit", "limit", "limit"]
    );
}

#[test]
fn do_variables_shadow_steps_end_clause_and_body_but_not_inits() {
    let input = "(list i (do ((i i (1+ i)) (sum i (+ sum i))) ((>= i limit) i) i) i)";

    assert_eq!(reference_texts(input, "i"), vec!["i", "i", "i", "i"]);
}

#[test]
fn do_star_variables_shadow_later_inits_and_body() {
    let input = "(list i (do* ((i i (1+ i)) (sum i (+ sum i))) ((>= sum limit) i) sum) i)";

    assert_eq!(reference_texts(input, "i"), vec!["i", "i", "i"]);
}

#[test]
fn prog_variables_shadow_body_but_not_inits() {
    let input = "(list value (prog ((value value) (copy value)) value (return value)) value)";

    assert_eq!(
        reference_texts(input, "value"),
        vec!["value", "value", "value", "value"]
    );
}

#[test]
fn prog_star_variables_shadow_later_inits_and_body() {
    let input = "(list value (prog* ((value value) (copy value)) (return value)) value)";

    assert_eq!(
        reference_texts(input, "value"),
        vec!["value", "value", "value"]
    );
}

#[test]
fn with_slots_bindings_shadow_body_but_not_instance_form() {
    let input = "(list slot (with-slots (slot (alias slot)) slot (list slot alias)) slot)";

    assert_eq!(reference_texts(input, "slot"), vec!["slot", "slot", "slot"]);
}

#[test]
fn with_accessors_bindings_shadow_body_but_not_instance_form() {
    let input = "(list value (with-accessors ((value get-value) (alias value)) value (list value alias)) value)";

    assert_eq!(
        reference_texts(input, "value"),
        vec!["value", "value", "value"]
    );
}
