use super::*;

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
fn cli_plans_common_lisp_destructuring_bind_rename_without_touching_value_form() {
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
        "slot",
        "--output",
        "json",
    ])
    .write_stdin("(destructuring-bind (value other) (parse value) (list value other))")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"destructuring-bind\""))
    .stdout(predicate::str::contains("\"reference_count\": 1"))
    .stdout(predicate::str::contains(
        "(destructuring-bind (slot other) (parse value) (list slot other))",
    ));
}

#[test]
fn cli_plans_common_lisp_multiple_value_bind_rename_without_shadow_capture() {
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
        "slot",
        "--output",
        "json",
    ])
    .write_stdin(
        "(multiple-value-bind (value other) (compute) (list value (destructuring-bind (value) row value) other value))",
    )
    .assert()
    .success()
    .stdout(predicate::str::contains("\"form\": \"multiple-value-bind\""))
    .stdout(predicate::str::contains("\"reference_count\": 2"))
    .stdout(predicate::str::contains("\"shadowed_scope_count\": 1"))
    .stdout(predicate::str::contains(
        "(multiple-value-bind (slot other) (compute) (list slot (destructuring-bind (value) row value) other slot))",
    ));
}
