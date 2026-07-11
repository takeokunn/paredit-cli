use super::*;

#[test]
fn refactor_namespace_help_lists_refactor_workflow_commands() {
    paredit()
        .args(["refactor", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("plan"))
        .stdout(predicate::str::contains("preview"))
        .stdout(predicate::str::contains("check"))
        .stdout(predicate::str::contains("status"))
        .stdout(predicate::str::contains("apply"))
        .stdout(predicate::str::contains("diff"))
        .stdout(predicate::str::contains("verify"))
        .stdout(predicate::str::contains("workspace-plan"))
        .stdout(predicate::str::contains("workspace-preview"))
        .stdout(predicate::str::contains("workspace-execute"))
        .stdout(predicate::str::contains("Examples:"))
        .stdout(predicate::str::contains(
            "paredit refactor plan --symbol old-name src/foo.lisp src/bar.lisp",
        ))
        .stdout(predicate::str::contains(
            "paredit refactor preview --from old-name --to new-name src/foo.lisp src/bar.lisp",
        ))
        .stdout(predicate::str::contains(
            "paredit refactor verify --symbol old-name --new-symbol new-name --phase post src/foo.lisp src/bar.lisp",
        ));
}

#[test]
fn refactor_namespace_subcommand_help_is_routable() {
    paredit()
        .args(["refactor", "plan", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Examples:"))
        .stdout(predicate::str::contains(
            "paredit refactor plan --symbol old-name src/foo.lisp src/bar.lisp",
        ));
    paredit()
        .args(["refactor", "preview", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Examples:"))
        .stdout(predicate::str::contains(
            "paredit refactor preview --from old-name --to new-name src/foo.lisp src/bar.lisp",
        ));
    paredit()
        .args(["refactor", "verify", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Examples:"))
        .stdout(predicate::str::contains(
            "paredit refactor verify --symbol old-name --new-symbol new-name --phase post src/foo.lisp src/bar.lisp",
        ));
    paredit()
        .args(["refactor", "workspace-plan", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Examples:"))
        .stdout(predicate::str::contains(
            "paredit refactor workspace-plan --symbol old-name .",
        ));
}
