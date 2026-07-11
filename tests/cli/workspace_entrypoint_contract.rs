use super::*;

#[test]
fn inspect_namespace_help_lists_workspace_analysis() {
    paredit()
        .args(["inspect", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("workspace"));
}

#[test]
fn workspace_namespace_subcommand_help_is_routable() {
    paredit()
        .args(["inspect", "workspace", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Examples:"))
        .stdout(predicate::str::contains("paredit inspect workspace ."));
}
