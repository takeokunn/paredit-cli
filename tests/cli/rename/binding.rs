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
fn cli_plans_destructured_let_binding_rename() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--dialect",
        "clojure",
        "--path",
        "0",
        "--from",
        "value",
        "--to",
        "product",
        "--output",
        "json",
    ])
    .write_stdin("(let [[value other] source next value] [value other next])")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"let\""))
    .stdout(predicate::str::contains("\"reference_count\": 2"))
    .stdout(predicate::str::contains(
        "(let [[product other] source next product] [product other next])",
    ));
}

#[test]
fn cli_plans_destructured_fn_parameter_rename_without_shadow_capture() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--dialect",
        "clojure",
        "--path",
        "0",
        "--from",
        "value",
        "--to",
        "product",
        "--output",
        "json",
    ])
    .write_stdin("(fn [{value :value row :row}] (list value (fn [value] value) row))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"fn\""))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains("\"shadowed_scope_count\": 1"))
    .stdout(predicate::str::contains(
        "(fn [{product :value row :row}] (list product (fn [value] value) row))",
    ));
}

#[test]
fn cli_plans_clojure_keys_destructured_fn_parameter_rename_preserving_lookup_key() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--dialect",
        "clojure",
        "--path",
        "0",
        "--from",
        "value",
        "--to",
        "product",
        "--output",
        "json",
    ])
    .write_stdin("(fn [{:keys [value row]}] (list value (fn [value] value) row))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"fn\""))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains("\"shadowed_scope_count\": 1"))
    .stdout(predicate::str::contains(
        "(fn [{product :value :keys [row]}] (list product (fn [value] value) row))",
    ));
}

#[test]
fn cli_plans_clojure_as_destructured_fn_parameter_rename_without_shadow_capture() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--dialect",
        "clojure",
        "--path",
        "0",
        "--from",
        "row",
        "--to",
        "record",
        "--output",
        "json",
    ])
    .write_stdin("(fn [{:keys [value] :as row}] (list value row (fn [row] row)))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"fn\""))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains("\"shadowed_scope_count\": 1"))
    .stdout(predicate::str::contains(
        "(fn [{:keys [value] :as record}] (list value record (fn [row] row)))",
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
fn cli_writes_binding_rename_without_touching_shadowed_scope() {
    let dir = fresh_temp_dir("rename-binding");
    let lisp_file = dir.join("core.lisp");
    fs::write(
        &lisp_file,
        "(defun render () (let ((value 1)) (+ value (let ((value 2)) value) value)))\n",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("rename-binding")
        .arg("--file")
        .arg(&lisp_file)
        .arg("--path")
        .arg("0.3")
        .arg("--from")
        .arg("value")
        .arg("--to")
        .arg("product")
        .arg("--write")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(lisp_file).expect("read rewritten lisp"),
        "(defun render () (let ((product 1)) (+ product (let ((value 2)) value) product)))\n"
    );
}

#[test]
fn cli_rejects_missing_binding_rename_target() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--path",
        "0",
        "--from",
        "missing",
        "--to",
        "renamed",
    ])
    .write_stdin("(let ((value 1)) value)")
    .assert()
    .failure()
    .stderr(predicate::str::contains(
        "binding 'missing' was not found in selected let",
    ));
}
