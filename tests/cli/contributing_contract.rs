#[test]
fn readme_contributing_and_release_keep_verification_boundary_aligned() {
    let readme = std::fs::read_to_string("README.md").expect("read README");
    let contributing = std::fs::read_to_string("CONTRIBUTING.md").expect("read CONTRIBUTING");
    let release = std::fs::read_to_string("RELEASE.md").expect("read RELEASE");

    assert!(
        normalize_whitespace(&readme).contains(
            "Contributions and releases follow [CONTRIBUTING.md](CONTRIBUTING.md), \
             [RELEASE.md](RELEASE.md), and [COMPATIBILITY.md](COMPATIBILITY.md)."
        ),
        "README must route contributions and releases to the policy documents"
    );
    assert!(
        normalize_whitespace(&contributing).contains(
            "`nix flake check` is the automated baseline for workflow linting, formatting, \
             clippy, nextest, package build/tests, and publish dry-run, but it does not \
             replace the full release checklist in [RELEASE.md](RELEASE.md)."
        ),
        "CONTRIBUTING must describe the CI baseline boundary"
    );
    assert!(
        normalize_whitespace(&release)
            .contains("Treat that automation as a baseline gate, not as complete release proof."),
        "RELEASE must preserve the automation boundary"
    );
    assert!(
        readme.contains("The current minimum supported Rust version is `1.85`."),
        "README must state the declared MSRV"
    );
}

#[test]
fn contributing_keeps_public_policy_routing_explicit() {
    let contributing = std::fs::read_to_string("CONTRIBUTING.md").expect("read CONTRIBUTING");
    let normalized = normalize_whitespace(&contributing);

    for required in [
        "Follow [GOVERNANCE.md](GOVERNANCE.md) when proposing scope, policy, or \
         maintainer-process changes.",
        "Follow [RELEASE.md](RELEASE.md) when preparing or reviewing a release.",
        "Follow [COMPATIBILITY.md](COMPATIBILITY.md) when changing CLI behavior, JSON \
         output, or `--write` semantics.",
        "Follow [MAINTAINERS.md](MAINTAINERS.md) for triage and response expectations \
         when proposing process changes.",
        "Follow [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) in issues, reviews, and other \
         project discussions.",
        "Follow [SECURITY.md](SECURITY.md) when handling vulnerability reports or \
         security-sensitive bug fixes.",
        "Send users to [SUPPORT.md](SUPPORT.md) for usage questions, issue reporting, and \
         reproduction expectations.",
        "Use [ROADMAP.md](ROADMAP.md) to check whether a proposed feature or refactor \
         aligns with current project priorities.",
        "Record user-visible behavior changes in [CHANGELOG.md](CHANGELOG.md).",
    ] {
        assert!(
            normalized.contains(&normalize_whitespace(required)),
            "CONTRIBUTING project policy routing drifted for: {required}"
        );
    }
}

fn normalize_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
