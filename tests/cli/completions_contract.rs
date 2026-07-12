use super::*;

#[test]
fn completions_generates_scripts_for_supported_shells() {
    for shell in ["bash", "zsh", "fish", "elvish", "powershell"] {
        paredit()
            .args(["completions", shell])
            .assert()
            .success()
            .stdout(predicate::str::contains("paredit"));
    }
}

#[test]
fn completions_rejects_unknown_shell() {
    paredit()
        .args(["completions", "tcsh"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value"));
}
