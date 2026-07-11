use super::*;

#[test]
fn top_level_help_routes_new_automation_to_grouped_namespaces() {
    paredit()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Canonical namespaces:"))
        .stdout(predicate::str::contains(
            "`paredit inspect ...` reads and reports without writing.",
        ))
        .stdout(predicate::str::contains(
            "`paredit edit ...` transforms one selected form and writes source to stdout.",
        ))
        .stdout(predicate::str::contains(
            "`paredit refactor ...` plans, previews, verifies, and applies semantic changes.",
        ))
        .stdout(predicate::str::contains(
            "All commands are available only through these namespaces.",
        ));
}

#[test]
fn rename_function_help_surfaces_common_lisp_callable_designators() {
    paredit()
        .args(["refactor", "rename-function", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Plan or apply a Common Lisp callable definition and callable-designator rename",
        ));
}

#[test]
fn rename_macrolet_help_surfaces_expander_body_boundary() {
    paredit()
        .args(["refactor", "rename-macrolet", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Plan or apply a Common Lisp macrolet/compiler-macrolet binding and call-site rename",
        ));
}

#[test]
fn rename_symbol_macro_help_surfaces_lexical_shadowing_boundary() {
    paredit()
        .args(["refactor", "rename-symbol-macro", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Plan or apply a Common Lisp define-symbol-macro binding and value-reference rename",
        ));
}

#[test]
fn rename_local_function_help_surfaces_flet_and_labels_boundary() {
    paredit()
        .args(["refactor", "rename-local-function", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Plan or apply a Common Lisp flet/labels local function binding and call-site rename",
        ));
}
