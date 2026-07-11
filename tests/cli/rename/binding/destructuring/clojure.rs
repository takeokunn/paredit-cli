use super::*;

#[test]
fn cli_plans_destructured_let_binding_rename() {
    let mut cmd = paredit();
    cmd.args([
        "refactor",
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
        "refactor",
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
        "refactor",
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
        "refactor",
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
