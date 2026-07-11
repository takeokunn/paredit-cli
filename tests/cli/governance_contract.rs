#[test]
fn governance_keeps_decision_and_escalation_routes_explicit() {
    let governance = std::fs::read_to_string("GOVERNANCE.md").expect("read GOVERNANCE");
    let normalized = normalize_whitespace(&governance);

    for required in [
        "Decisions that change stable CLI, JSON, or `--write` behavior must reference \
         [COMPATIBILITY.md](COMPATIBILITY.md) and document the user-visible effect in \
         [CHANGELOG.md](CHANGELOG.md).",
        "Security-sensitive decisions follow [SECURITY.md](SECURITY.md) even when the \
         final fix lands through a normal pull request later.",
        "Feature or refactor proposals that claim current-project priority should point to \
         [ROADMAP.md](ROADMAP.md) instead of relying on review-thread interpretation alone.",
        "When maintainership changes, update [MAINTAINERS.md](MAINTAINERS.md) with the new \
         scope of authority.",
        "Usage questions and reproducible bugs belong in the support path documented in \
         [SUPPORT.md](SUPPORT.md).",
        "Conduct issues follow [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).",
        "Security-sensitive reports follow [SECURITY.md](SECURITY.md).",
    ] {
        assert!(
            normalized.contains(&normalize_whitespace(required)),
            "GOVERNANCE routing or decision contract drifted for: {required}"
        );
    }
}

#[test]
fn governance_preserves_project_principles_and_contribution_expectations() {
    let governance = std::fs::read_to_string("GOVERNANCE.md").expect("read GOVERNANCE");
    let normalized = normalize_whitespace(&governance);

    for required in [
        "Preserve structural correctness before convenience. Refactors must keep edits \
         balanced, syntax-aware, and reviewable.",
        "Prefer explicit plans, previews, and verification over hidden mutation.",
        "Treat machine-facing CLI and JSON output as public contracts once released.",
        "Keep bug reports, feature requests, and design discussion grounded in minimal \
         reproductions, fixtures, or measurable behavior.",
        "New commands, flags, or behavior changes need tests that prove the intended \
         contract.",
        "Refactors should preserve the architecture boundary described in \
         [README.md](README.md): syntax and typed domain rules stay out of CLI and terminal \
         formatting layers.",
        "including scope-aware handling of Common Lisp callable and macro bindings in \
         Common Lisp macro binding scopes.",
        "Macro expander bodies are treated as their own reviewable scopes rather than as a \
         license for unrestricted rewriting.",
        "Process or policy changes should update the relevant public document instead of \
         living only in review comments.",
    ] {
        assert!(
            normalized.contains(&normalize_whitespace(required)),
            "GOVERNANCE principle or contribution expectation drifted for: {required}"
        );
    }
}

fn normalize_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
