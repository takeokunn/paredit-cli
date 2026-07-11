use super::*;

#[test]
fn plans_private_unused_definition_removal() {
    let text = "(in-package #:app)\n(defun stale-helper () 1)\n(defun live () 2)\n(live)\n";
    let stale_form = "(defun stale-helper () 1)";
    let live_form = "(defun live () 2)";
    let plan = plan_remove_unused_definitions(request_for(
        text,
        vec![
            definition(
                text,
                stale_form,
                "stale-helper",
                DefinitionCategory::Function,
            ),
            definition(text, live_form, "live", DefinitionCategory::Function),
        ],
    ))
    .expect("plan should build");

    assert_eq!(plan.candidate_count, 1);
    assert_eq!(plan.removal_count, 1);
    assert_eq!(plan.skipped_count, 0);
    assert!(!plan.files[0].rewritten.contains(stale_form));
    SyntaxTree::parse(&plan.files[0].rewritten).expect("rewrite must stay parseable");
}
