fn normalize_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[test]
fn conduct_docs_remain_discoverable_and_routed_through_public_policy_docs() {
    let readme = std::fs::read_to_string("README.md").expect("read README");
    let governance = std::fs::read_to_string("GOVERNANCE.md").expect("read GOVERNANCE");
    let maintainers = std::fs::read_to_string("MAINTAINERS.md").expect("read MAINTAINERS");
    let conduct = std::fs::read_to_string("CODE_OF_CONDUCT.md").expect("read CODE_OF_CONDUCT");

    let readme = normalize_whitespace(&readme);
    let governance = normalize_whitespace(&governance);
    let conduct = normalize_whitespace(&conduct);

    assert!(
        readme.contains(
            "See [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) for collaboration and community expectations."
        ),
        "README must keep the code of conduct discoverable"
    );
    assert!(
        governance.contains("Conduct issues follow [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)."),
        "governance must route conduct escalation through CODE_OF_CONDUCT.md"
    );
    assert!(
        maintainers
            .contains("Route conduct issues through [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)."),
        "maintainers policy must preserve the conduct escalation path"
    );
    assert!(
        conduct.contains(
            "For conduct problems in public project spaces, contact the maintainer through GitHub and include links, screenshots, or other concrete evidence."
        ),
        "code of conduct reporting must require concrete evidence"
    );
    assert!(
        conduct
            .contains("For security-sensitive problems, use [SECURITY.md](SECURITY.md) instead."),
        "code of conduct must keep security-sensitive reports on the security path"
    );
}

#[test]
fn code_of_conduct_keeps_behavior_and_moderation_contract_explicit() {
    let conduct = std::fs::read_to_string("CODE_OF_CONDUCT.md").expect("read CODE_OF_CONDUCT");
    let conduct = normalize_whitespace(&conduct);

    for needle in [
        "technical discussion direct, respectful, and evidence-based.",
        "Focus on the code, behavior, and reproducible evidence instead of personal attacks.",
        "Respect different experience levels and communication styles while maintaining a high technical bar.",
        "Harassment, intimidation, or discriminatory language.",
        "Publishing private information about other participants without permission.",
        "Project maintainers may edit, hide, lock, or remove comments, issues, pull requests, and other contributions that violate this policy.",
        "Repeated or severe violations may lead to temporary or permanent exclusion from project spaces.",
    ] {
        assert!(
            conduct.contains(needle),
            "code of conduct must keep {needle}"
        );
    }
}
