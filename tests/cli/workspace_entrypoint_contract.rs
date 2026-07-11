use super::*;

#[test]
fn workspace_namespace_help_lists_workspace_workflow_commands() {
    paredit()
        .args(["workspace", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("report"))
        .stdout(predicate::str::contains("refactor-plan").not())
        .stdout(predicate::str::contains("refactor-preview").not())
        .stdout(predicate::str::contains("refactor-execute").not())
        .stdout(predicate::str::contains("Examples:"))
        .stdout(predicate::str::contains("paredit workspace report ."))
        .stdout(predicate::str::contains(
            "paredit refactor workspace-plan --symbol old-name .",
        ))
        .stdout(predicate::str::contains(
            "paredit refactor workspace-execute --from old-name --to new-name --write .",
        ));
}

#[test]
fn workspace_namespace_subcommand_help_is_routable() {
    paredit()
        .args(["workspace", "report", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Examples:"))
        .stdout(predicate::str::contains("paredit workspace report ."));
}
