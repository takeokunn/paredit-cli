use super::*;
use std::path::PathBuf;

#[test]
fn remove_plan_uses_unused_definition_cleanup_usecase() {
    let files = vec![PathBuf::from("src/core.lisp"), PathBuf::from("src/ui.el")];
    let steps = refactor_plan_steps(RefactorOperation::Remove, "stale-helper", &files, &[]);

    let apply = steps
        .iter()
        .find(|step| step.order == 3)
        .expect("apply step");
    assert_eq!(apply.action, "apply-unused-definition-removal");
    let apply_command = apply.command.as_deref().expect("apply command");
    assert!(apply_command.contains("paredit remove-unused-definitions --output json"));
    assert!(apply_command.contains("'src/core.lisp'"));
    assert!(apply_command.contains("'src/ui.el'"));

    let verify = steps
        .iter()
        .find(|step| step.order == 4)
        .expect("verify step");
    let verify_command = verify.command.as_deref().expect("verify command");
    assert!(verify_command.contains(
        "paredit verify-refactor --symbol 'stale-helper' --operation remove --phase post --output json"
    ));
    assert!(!verify_command.contains("--require-definitions 1"));
    assert!(!verify_command.contains("--require-references 1"));
}
