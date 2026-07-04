use super::*;

#[test]
fn cli_plans_binding_rename_without_shadowed_inner_binding() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--path",
        "0.3",
        "--from",
        "value",
        "--to",
        "product",
        "--output",
        "json",
    ])
    .write_stdin("(defun render () (let ((value 1)) (+ value (let ((value 2)) value) value)))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"let\""))
    .stdout(predicate::str::contains("\"binding_span\""))
    .stdout(predicate::str::contains("\"reference_count\": 2"))
    .stdout(predicate::str::contains("\"shadowed_scope_count\": 1"))
    .stdout(predicate::str::contains(
        "(defun render () (let ((product 1)) (+ product (let ((value 2)) value) product)))",
    ));
}

#[test]
fn cli_plans_let_star_binding_rename_through_later_binding_values() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--path",
        "0",
        "--from",
        "value",
        "--to",
        "seed",
        "--output",
        "json",
    ])
    .write_stdin("(let* ((value 1) (next (+ value 1))) (+ next value))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"let*\""))
    .stdout(predicate::str::contains("\"reference_count\": 2"))
    .stdout(predicate::str::contains(
        "(let* ((seed 1) (next (+ seed 1))) (+ next seed))",
    ));
}

#[test]
fn cli_plans_common_lisp_bare_let_binding_rename_without_touching_later_init() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--path",
        "0",
        "--from",
        "value",
        "--to",
        "seed",
        "--output",
        "json",
    ])
    .write_stdin("(let (value (next value)) (list value next))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"let\""))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains(
        "(let (seed (next value)) (list seed next))",
    ));
}

#[test]
fn cli_plans_common_lisp_bare_let_star_binding_rename_through_later_init() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--path",
        "0",
        "--from",
        "value",
        "--to",
        "seed",
        "--output",
        "json",
    ])
    .write_stdin("(let* (value (next value)) (list value next))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"let*\""))
    .stdout(predicate::str::contains("\"reference_count\": 2"))
    .stdout(predicate::str::contains(
        "(let* (seed (next seed)) (list seed next))",
    ));
}

#[test]
fn cli_plans_outer_let_binding_rename_without_touching_inner_bare_binding() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--path",
        "0",
        "--from",
        "value",
        "--to",
        "seed",
        "--output",
        "json",
    ])
    .write_stdin("(let ((value 1)) (let (value) value) value)")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"let\""))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains("\"shadowed_scope_count\": 1"))
    .stdout(predicate::str::contains(
        "(let ((seed 1)) (let (value) value) seed)",
    ));
}

#[test]
fn cli_plans_symbol_macrolet_binding_rename_without_touching_expansion_reference() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--path",
        "0",
        "--from",
        "value",
        "--to",
        "slot",
        "--output",
        "json",
    ])
    .write_stdin("(symbol-macrolet ((value (compute value))) (list value value))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"symbol-macrolet\""))
    .stdout(predicate::str::contains("\"reference_count\": 2"))
    .stdout(predicate::str::contains(
        "(symbol-macrolet ((slot (compute value))) (list slot slot))",
    ));
}

#[test]
fn cli_plans_dolist_iteration_binding_rename_without_touching_source() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--path",
        "0",
        "--from",
        "value",
        "--to",
        "item",
        "--output",
        "json",
    ])
    .write_stdin("(dolist (value items value) (collect value) items)")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"dolist\""))
    .stdout(predicate::str::contains("\"reference_count\": 2"))
    .stdout(predicate::str::contains(
        "(dolist (item items item) (collect item) items)",
    ));
}

#[test]
fn cli_plans_dotimes_iteration_binding_rename_without_touching_count() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--path",
        "0",
        "--from",
        "index",
        "--to",
        "i",
        "--output",
        "json",
    ])
    .write_stdin("(dotimes (index limit index) (push index result) limit)")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"dotimes\""))
    .stdout(predicate::str::contains("\"reference_count\": 2"))
    .stdout(predicate::str::contains(
        "(dotimes (i limit i) (push i result) limit)",
    ));
}

#[test]
fn cli_plans_do_binding_rename_across_steps_end_clause_and_body() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--path",
        "0",
        "--from",
        "value",
        "--to",
        "item",
        "--output",
        "json",
    ])
    .write_stdin("(do ((value seed (1+ value))) ((done value) value) (collect value seed))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"do\""))
    .stdout(predicate::str::contains("\"reference_count\": 4"))
    .stdout(predicate::str::contains(
        "(do ((item seed (1+ item))) ((done item) item) (collect item seed))",
    ));
}

#[test]
fn cli_plans_do_star_binding_rename_across_later_inits_steps_end_clause_and_body() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--path",
        "0",
        "--from",
        "value",
        "--to",
        "item",
        "--output",
        "json",
    ])
    .write_stdin(
        "(do* ((value seed (1+ value)) (copy value)) ((done value) (list value copy)) (collect value copy))",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"do*\""))
    .stdout(predicate::str::contains("\"reference_count\": 5"))
    .stdout(predicate::str::contains(
        "(do* ((item seed (1+ item)) (copy item)) ((done item) (list item copy)) (collect item copy))",
    ));
}

#[test]
fn cli_plans_prog_star_binding_rename_across_later_inits_and_body() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--path",
        "0",
        "--from",
        "value",
        "--to",
        "item",
        "--output",
        "json",
    ])
    .write_stdin("(prog* ((value seed) (copy value)) (return (list value copy)))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"prog*\""))
    .stdout(predicate::str::contains("\"reference_count\": 2"))
    .stdout(predicate::str::contains(
        "(prog* ((item seed) (copy item)) (return (list item copy)))",
    ));
}

#[test]
fn cli_plans_lambda_parameter_rename_without_shadow_capture() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--path",
        "0",
        "--from",
        "value",
        "--to",
        "product",
        "--output",
        "json",
    ])
    .write_stdin("(lambda (value) (list value (lambda (value) value) value))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"lambda\""))
    .stdout(predicate::str::contains("\"reference_count\": 2"))
    .stdout(predicate::str::contains("\"shadowed_scope_count\": 1"))
    .stdout(predicate::str::contains(
        "(lambda (product) (list product (lambda (value) value) product))",
    ));
}

#[test]
fn cli_plans_defun_parameter_rename() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--path",
        "0",
        "--from",
        "value",
        "--to",
        "product",
        "--output",
        "json",
    ])
    .write_stdin("(defun render (value other) (list value other))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"defun\""))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains(
        "(defun render (product other) (list product other))",
    ));
}

#[test]
fn cli_plans_defmacro_optional_parameter_rename_without_touching_default_form() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--path",
        "0",
        "--from",
        "value",
        "--to",
        "form",
        "--output",
        "json",
    ])
    .write_stdin(
        "(defmacro wrap (&optional (value (default value) supplied)) (list value supplied))",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"defmacro\""))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains(
        "(defmacro wrap (&optional (form (default value) supplied)) (list form supplied))",
    ));
}

#[test]
fn cli_plans_defmethod_specialized_parameter_rename_without_touching_specializer() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--path",
        "0",
        "--from",
        "node",
        "--to",
        "widget-node",
        "--output",
        "json",
    ])
    .write_stdin("(defmethod render ((node widget) stream) (list node stream widget))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"defmethod\""))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains(
        "(defmethod render ((widget-node widget) stream) (list widget-node stream widget))",
    ));
}

#[test]
fn cli_plans_defmethod_qualifier_parameter_rename() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--path",
        "0",
        "--from",
        "node",
        "--to",
        "widget-node",
        "--output",
        "json",
    ])
    .write_stdin(
        "(defmethod render :around ((node widget) stream) (call-next-method) (list node stream))",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"defmethod\""))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains(
        "(defmethod render :around ((widget-node widget) stream) (call-next-method) (list widget-node stream))",
    ));
}

#[test]
fn cli_plans_cl_defmethod_optional_parameter_rename_without_touching_default_form() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--path",
        "0",
        "--from",
        "stream",
        "--to",
        "out",
        "--output",
        "json",
    ])
    .write_stdin(
        "(cl-defmethod render ((node widget) &optional (stream (default-stream node) stream-p)) (list node stream stream-p))",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"cl-defmethod\""))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains(
        "(cl-defmethod render ((node widget) &optional (out (default-stream node) stream-p)) (list node out stream-p))",
    ));
}

#[test]
fn cli_plans_define_setf_expander_environment_parameter_rename() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--path",
        "0",
        "--from",
        "env",
        "--to",
        "macro-env",
        "--output",
        "json",
    ])
    .write_stdin(
        "(define-setf-expander slot (&whole whole &environment env target) (list whole env target))",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains(
        "\"form\": \"define-setf-expander\"",
    ))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains(
        "(define-setf-expander slot (&whole whole &environment macro-env target) (list whole macro-env target))",
    ));
}

#[test]
fn cli_plans_define_compiler_macro_environment_parameter_rename() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--path",
        "0",
        "--from",
        "env",
        "--to",
        "macro-env",
        "--output",
        "json",
    ])
    .write_stdin(
        "(define-compiler-macro render (&whole whole &environment env target) (list whole env target))",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains(
        "\"form\": \"define-compiler-macro\"",
    ))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains(
        "(define-compiler-macro render (&whole whole &environment macro-env target) (list whole macro-env target))",
    ));
}

#[test]
fn cli_plans_with_slots_binding_rename_preserving_slot_name() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--path",
        "0",
        "--from",
        "value",
        "--to",
        "slot-value",
        "--output",
        "json",
    ])
    .write_stdin("(with-slots (value (alias slot-name)) object (list value alias object))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"with-slots\""))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains(
        "(with-slots ((slot-value value) (alias slot-name)) object (list slot-value alias object))",
    ));
}

#[test]
fn cli_plans_loop_for_in_binding_rename_without_touching_source() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--path",
        "0",
        "--from",
        "value",
        "--to",
        "item",
        "--output",
        "json",
    ])
    .write_stdin("(loop for value in values collect value finally (return value))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"loop\""))
    .stdout(predicate::str::contains("\"reference_count\": 2"))
    .stdout(predicate::str::contains(
        "(loop for item in values collect item finally (return item))",
    ));
}

#[test]
fn cli_plans_loop_with_binding_rename_without_touching_init() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--path",
        "0",
        "--from",
        "value",
        "--to",
        "acc",
        "--output",
        "json",
    ])
    .write_stdin("(loop with value = (seed value) collect value)")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"loop\""))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains(
        "(loop with acc = (seed value) collect acc)",
    ));
}

#[test]
fn cli_plans_outer_binding_rename_without_touching_loop_shadow() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--path",
        "0",
        "--from",
        "value",
        "--to",
        "outer",
        "--output",
        "json",
    ])
    .write_stdin("(let ((value 1)) (loop for value in values collect value) value)")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"let\""))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains(
        "(let ((outer 1)) (loop for value in values collect value) outer)",
    ));
}
