fn normalize_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[test]
fn security_docs_keep_private_reporting_routing_aligned() {
    let readme = std::fs::read_to_string("README.md").expect("read README");
    let support = std::fs::read_to_string("SUPPORT.md").expect("read SUPPORT");
    let security = std::fs::read_to_string("SECURITY.md").expect("read SECURITY");
    let issue_config =
        std::fs::read_to_string(".github/ISSUE_TEMPLATE/config.yml").expect("read issue config");

    let readme = normalize_whitespace(&readme);
    let support = normalize_whitespace(&support);
    let security = normalize_whitespace(&security);

    assert!(
        readme.contains(
            "Security-sensitive reports and the currently supported release line are defined in [SECURITY.md](SECURITY.md)."
        ),
        "README must send security reporting and support-line expectations to SECURITY.md"
    );
    assert!(
        support.contains(
            "Security-sensitive reports must follow [SECURITY.md](SECURITY.md) instead of public issues or issue forms."
        ),
        "support doc must keep security reports off the public issue path"
    );
    assert!(
        security.contains("Do not open public GitHub issues for vulnerabilities"),
        "security policy must keep vulnerabilities off public issues"
    );
    assert!(
        security.contains(
            "Do not use public GitHub issues, pull requests, or issue forms for vulnerability reports before a maintainer confirms the issue is safe to disclose."
        ),
        "security policy must forbid public disclosure before maintainer confirmation"
    );
    assert!(
        issue_config
            .contains("private reporting path for vulnerabilities or security-sensitive behavior"),
        "issue template config must expose the private reporting path"
    );
}

#[test]
fn security_supported_line_and_acknowledgement_policy_match_public_docs() {
    let readme = std::fs::read_to_string("README.md").expect("read README");
    let security = std::fs::read_to_string("SECURITY.md").expect("read SECURITY");
    let maintainers = std::fs::read_to_string("MAINTAINERS.md").expect("read MAINTAINERS");

    let readme = normalize_whitespace(&readme);
    let security = normalize_whitespace(&security);

    assert!(
        readme.contains("`main` is the active development line."),
        "README must describe main as the active development line"
    );
    assert!(
        security.contains(
            "Security fixes are applied on the active development line only. There is no supported historical release branch today."
        ),
        "security policy must define the active support line"
    );
    for needle in [
        "| Unreleased `main` | Yes |",
        "| First tagged release line after publication | Yes, until superseded by a newer supported line |",
        "| Released versions older than `main` | No |",
    ] {
        assert!(
            security.contains(needle),
            "security supported-version table must keep {needle}"
        );
    }
    assert!(
        security.contains("The project aims to acknowledge valid reports within 7 days"),
        "security policy must publish the acknowledgement target"
    );
    assert!(
        maintainers.contains("acknowledge security reports within 7 days;"),
        "maintainer response policy must match the public security acknowledgement target"
    );
}
