use super::*;

#[test]
fn cli_plans_flet_lambda_list_parameter_rename_without_touching_call_args() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--dialect",
        "common-lisp",
        "--path",
        "0",
        "--from",
        "value",
        "--to",
        "item",
        "--output",
        "json",
    ])
    .write_stdin("(flet ((helper (value) (list value))) (helper value))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"flet\""))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains(
        "(flet ((helper (item) (list item))) (helper value))",
    ));
}

#[test]
fn cli_plans_macrolet_lambda_list_parameter_rename_without_touching_expansion_site() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--dialect",
        "common-lisp",
        "--path",
        "0",
        "--from",
        "value",
        "--to",
        "form",
        "--output",
        "json",
    ])
    .write_stdin("(macrolet ((wrap (value) (list value))) (wrap value) value)")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"macrolet\""))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains(
        "(macrolet ((wrap (form) (list form))) (wrap value) value)",
    ));
}

#[test]
fn cli_plans_compiler_macrolet_lambda_list_parameter_rename() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--dialect",
        "common-lisp",
        "--path",
        "0",
        "--from",
        "value",
        "--to",
        "form",
        "--output",
        "json",
    ])
    .write_stdin("(compiler-macrolet ((expand (value) (list value))) (expand value) value)")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"compiler-macrolet\""))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains(
        "(compiler-macrolet ((expand (form) (list form))) (expand value) value)",
    ));
}

#[test]
fn cli_plans_macrolet_whole_parameter_rename_without_touching_call_site_form() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--dialect",
        "common-lisp",
        "--path",
        "0",
        "--from",
        "whole",
        "--to",
        "form",
        "--output",
        "json",
    ])
    .write_stdin("(macrolet ((wrap (&whole whole value) (list whole value))) (wrap value))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"macrolet\""))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains(
        "(macrolet ((wrap (&whole form value) (list form value))) (wrap value))",
    ));
}

#[test]
fn cli_plans_compiler_macrolet_environment_parameter_rename_without_touching_body() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--dialect",
        "common-lisp",
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
        "(compiler-macrolet ((expand (&whole whole &environment env value) (list whole env value))) (expand value) env)",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"compiler-macrolet\""))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains(
        "(compiler-macrolet ((expand (&whole whole &environment macro-env value) (list whole macro-env value))) (expand value) env)",
    ));
}

#[test]
fn cli_plans_defmacro_environment_parameter_rename_without_touching_body() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--dialect",
        "common-lisp",
        "--path",
        "0",
        "--from",
        "env",
        "--to",
        "macro-env",
        "--output",
        "json",
    ])
    .write_stdin("(defmacro inspect (&environment env value) (list env value))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"defmacro\""))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains(
        "(defmacro inspect (&environment macro-env value) (list macro-env value))",
    ));
}

#[test]
fn cli_plans_defmacro_whole_and_environment_parameter_rename_without_touching_body() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--dialect",
        "common-lisp",
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
        "(defmacro inspect (&whole whole &environment env value) (list whole env value))",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"defmacro\""))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains(
        "(defmacro inspect (&whole whole &environment macro-env value) (list whole macro-env value))",
    ));
}

#[test]
fn cli_plans_defmacro_aux_parameter_rename_without_touching_aux_initializer() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--dialect",
        "common-lisp",
        "--path",
        "0",
        "--from",
        "form",
        "--to",
        "macro-form",
        "--output",
        "json",
    ])
    .write_stdin("(defmacro inspect (&whole form value &aux (tag form)) (list form value tag))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"defmacro\""))
    .stdout(predicate::str::contains("\"reference_count\": 2"))
    .stdout(predicate::str::contains(
        "(defmacro inspect (&whole macro-form value &aux (tag macro-form)) (list macro-form value tag))",
    ));
}

#[test]
fn cli_plans_macrolet_aux_parameter_rename_without_touching_aux_initializer() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--dialect",
        "common-lisp",
        "--path",
        "0",
        "--from",
        "form",
        "--to",
        "macro-form",
        "--output",
        "json",
    ])
    .write_stdin(
        "(macrolet ((inspect (&whole form value &aux (tag form)) (list form value tag))) (inspect value) form)",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"macrolet\""))
    .stdout(predicate::str::contains("\"reference_count\": 2"))
    .stdout(predicate::str::contains(
        "(macrolet ((inspect (&whole macro-form value &aux (tag macro-form)) (list macro-form value tag))) (inspect value) form)",
    ));
}

#[test]
fn cli_plans_compiler_macrolet_aux_parameter_rename_without_touching_aux_initializer() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--dialect",
        "common-lisp",
        "--path",
        "0",
        "--from",
        "form",
        "--to",
        "macro-form",
        "--output",
        "json",
    ])
    .write_stdin(
        "(compiler-macrolet ((inspect (&whole form value &aux (tag form)) (list form value tag))) (inspect value) form)",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"compiler-macrolet\""))
    .stdout(predicate::str::contains("\"reference_count\": 2"))
    .stdout(predicate::str::contains(
        "(compiler-macrolet ((inspect (&whole macro-form value &aux (tag macro-form)) (list macro-form value tag))) (inspect value) form)",
    ));
}

#[test]
fn cli_plans_macrolet_key_parameter_rename_without_touching_key_designator_or_call_site() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--dialect",
        "common-lisp",
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
        "(macrolet ((inspect (&key ((:value value) (default value) value-supplied)) (list value value-supplied))) (inspect :value value) value)",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"macrolet\""))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains(
        "(macrolet ((inspect (&key ((:value form) (default value) value-supplied)) (list form value-supplied))) (inspect :value value) value)",
    ));
}

#[test]
fn cli_plans_compiler_macrolet_key_parameter_rename_without_touching_key_designator_or_call_site() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--dialect",
        "common-lisp",
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
        "(compiler-macrolet ((inspect (&key ((:value value) (default value) value-supplied)) (list value value-supplied))) (inspect :value value) value)",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"compiler-macrolet\""))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains(
        "(compiler-macrolet ((inspect (&key ((:value form) (default value) value-supplied)) (list form value-supplied))) (inspect :value value) value)",
    ));
}

#[test]
fn cli_plans_labels_lambda_list_parameter_rename_without_touching_outer_body_call() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--dialect",
        "common-lisp",
        "--path",
        "0",
        "--from",
        "value",
        "--to",
        "node",
        "--output",
        "json",
    ])
    .write_stdin("(labels ((walk (value) (if value (walk value) value))) (walk seed) value)")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"labels\""))
    .stdout(predicate::str::contains("\"reference_count\": 3"))
    .stdout(predicate::str::contains(
        "(labels ((walk (node) (if node (walk node) node))) (walk seed) value)",
    ));
}

#[test]
fn cli_plans_outer_binding_rename_without_touching_local_callable_lambda_shadow() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--dialect",
        "common-lisp",
        "--path",
        "0",
        "--from",
        "value",
        "--to",
        "item",
        "--output",
        "json",
    ])
    .write_stdin("(let ((value 1)) (flet ((helper (value) value)) (helper value) value))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"let\""))
    .stdout(predicate::str::contains("\"reference_count\": 2"))
    .stdout(predicate::str::contains("\"shadowed_scope_count\": 1"))
    .stdout(predicate::str::contains(
        "(let ((item 1)) (flet ((helper (value) value)) (helper item) item))",
    ));
}

#[test]
fn cli_rejects_ambiguous_labels_local_callable_lambda_list_parameter_rename() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--dialect",
        "common-lisp",
        "--path",
        "0",
        "--from",
        "value",
        "--to",
        "node",
        "--output",
        "json",
    ])
    .write_stdin("(labels ((left (value) value) (right (value) value)) (left 1))")
    .assert()
    .failure()
    .stderr(predicate::str::contains(
        "multiple selected labels local callable lambda lists",
    ));
}

#[test]
fn cli_rejects_ambiguous_local_callable_lambda_list_parameter_rename() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--dialect",
        "common-lisp",
        "--path",
        "0",
        "--from",
        "value",
        "--to",
        "item",
        "--output",
        "json",
    ])
    .write_stdin("(flet ((left (value) value) (right (value) value)) (left 1))")
    .assert()
    .failure()
    .stderr(predicate::str::contains(
        "multiple selected flet local callable lambda lists",
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
fn cli_plans_emacs_lisp_lambda_parameter_rename_without_shadow_capture() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--dialect",
        "emacs-lisp",
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
fn cli_plans_emacs_lisp_defun_parameter_rename() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--dialect",
        "emacs-lisp",
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
