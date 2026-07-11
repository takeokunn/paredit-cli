fn github_blob_url(path: &str) -> String {
    format!("https://github.com/takeokunn/paredit-cli/blob/main/{path}")
}

#[test]
fn issue_template_contact_links_route_to_public_project_docs() {
    let config =
        std::fs::read_to_string(".github/ISSUE_TEMPLATE/config.yml").expect("read issue config");

    for path in ["SUPPORT.md", "SECURITY.md", "ROADMAP.md"] {
        assert!(
            config.contains(&github_blob_url(path)),
            "issue template contact links must route users to {path}"
        );
    }
}

#[test]
fn issue_and_pull_request_templates_reference_project_policy_docs() {
    let feature_request = std::fs::read_to_string(".github/ISSUE_TEMPLATE/feature_request.yml")
        .expect("read feature request template");
    let pull_request =
        std::fs::read_to_string(".github/PULL_REQUEST_TEMPLATE.md").expect("read pr template");

    for doc in ["ROADMAP.md", "GOVERNANCE.md", "COMPATIBILITY.md"] {
        assert!(
            feature_request.contains(doc),
            "feature request template must point contributors to {doc}"
        );
    }

    for doc in [
        "CHANGELOG.md",
        "COMPATIBILITY.md",
        "RELEASE.md",
        "ROADMAP.md",
        "SECURITY.md",
    ] {
        assert!(
            pull_request.contains(doc),
            "pull request template must require policy review against {doc}"
        );
    }
}
