#[test]
fn governance_and_conduct_docs_keep_public_escalation_routes_aligned() {
    let governance = std::fs::read_to_string("GOVERNANCE.md").expect("read GOVERNANCE");
    let conduct = std::fs::read_to_string("CODE_OF_CONDUCT.md").expect("read CODE_OF_CONDUCT");
    let maintainers = std::fs::read_to_string("MAINTAINERS.md").expect("read MAINTAINERS");

    for needle in [
        "Usage questions and reproducible bugs belong in the support path documented in",
        "Conduct issues follow [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).",
        "Security-sensitive reports follow [SECURITY.md](SECURITY.md).",
    ] {
        assert!(
            governance.contains(needle),
            "governance escalation path must keep {needle}"
        );
    }

    assert!(
        conduct.contains("security-sensitive problems, use [SECURITY.md](SECURITY.md) instead."),
        "code of conduct reporting must route security-sensitive problems to SECURITY.md"
    );

    for needle in [
        "Route conduct issues through [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).",
        "Route security-sensitive reports through [SECURITY.md](SECURITY.md).",
    ] {
        assert!(
            maintainers.contains(needle),
            "maintainers responsibilities must keep {needle}"
        );
    }
}

#[test]
fn security_acknowledgement_target_matches_maintainer_operating_policy() {
    let security = std::fs::read_to_string("SECURITY.md").expect("read SECURITY");
    let maintainers = std::fs::read_to_string("MAINTAINERS.md").expect("read MAINTAINERS");

    let target = "7 days";
    assert!(
        security.contains(target),
        "security policy must publish the acknowledgement target"
    );
    assert!(
        maintainers.contains(target),
        "maintainer response expectations must match the security acknowledgement target"
    );

    assert!(
        security.contains("Do not use public GitHub issues, pull requests, or issue forms for"),
        "security policy must keep vulnerability disclosure off public issue surfaces"
    );
    assert!(
        maintainers
            .contains("Keep reproducible bug reports and feature requests moving through public"),
        "maintainer policy must distinguish low-risk public triage from private security handling"
    );
}
