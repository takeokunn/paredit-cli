use super::*;

#[test]
fn similarity_help_surfaces_comparison_scope_aliases() {
    paredit()
        .args(["inspect", "similarity", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Use `file` as a shorthand for same-file.",
        ))
        .stdout(predicate::str::contains(
            "Restrict comparisons by file relationship.",
        ));
}
