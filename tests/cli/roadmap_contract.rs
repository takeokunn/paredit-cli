#[test]
fn contributor_and_maintainer_docs_route_priority_decisions_through_roadmap() {
    let contributing = std::fs::read_to_string("CONTRIBUTING.md").expect("read CONTRIBUTING");
    let governance = std::fs::read_to_string("GOVERNANCE.md").expect("read GOVERNANCE");
    let maintainers = std::fs::read_to_string("MAINTAINERS.md").expect("read MAINTAINERS");

    assert!(
        contributing.contains(
            "Use [ROADMAP.md](ROADMAP.md) to check whether a proposed feature or refactor"
        ),
        "CONTRIBUTING must route proposed features and refactors through ROADMAP.md"
    );
    assert!(
        governance.contains("point to [ROADMAP.md](ROADMAP.md)"),
        "GOVERNANCE must require roadmap evidence for priority claims"
    );
    assert!(
        maintainers.contains("Apply `enhancement` only when the request matches"),
        "MAINTAINERS must scope enhancement triage through the roadmap"
    );
    assert!(
        maintainers.contains("[ROADMAP.md](ROADMAP.md)"),
        "MAINTAINERS must cite ROADMAP.md for enhancement triage"
    );
}

#[test]
fn roadmap_non_goals_and_governance_scope_control_keep_the_same_boundaries() {
    let governance = std::fs::read_to_string("GOVERNANCE.md").expect("read GOVERNANCE");
    let roadmap = std::fs::read_to_string("ROADMAP.md").expect("read ROADMAP");
    let normalized_governance = normalize_whitespace(&governance);
    let normalized_roadmap = normalize_whitespace(&roadmap);

    for shared_boundary in [
        "evaluating Lisp code",
        "comments or strings",
        "implicit traversal",
        "Common Lisp macro binding scopes",
    ] {
        assert!(
            normalized_governance.contains(shared_boundary),
            "GOVERNANCE scope control must mention boundary: {shared_boundary}"
        );
        assert!(
            normalized_roadmap.contains(shared_boundary),
            "ROADMAP non-goals must mention boundary: {shared_boundary}"
        );
    }

    for prioritized_scope in ["macrolet", "compiler-macrolet", "symbol-macrolet"] {
        assert!(
            normalized_roadmap.contains(prioritized_scope),
            "ROADMAP contribution focus must mention Common Lisp scope: {prioritized_scope}"
        );
    }
}

#[test]
fn roadmap_usage_section_points_scope_and_release_decisions_to_the_right_docs() {
    let roadmap = std::fs::read_to_string("ROADMAP.md").expect("read ROADMAP");

    for linked_doc in ["GOVERNANCE.md", "COMPATIBILITY.md", "RELEASE.md"] {
        assert!(
            roadmap.contains(&format!("[{linked_doc}]({linked_doc})")),
            "ROADMAP usage section must link {linked_doc}"
        );
    }
}

fn normalize_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
