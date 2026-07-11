#[test]
fn readme_contributing_and_release_keep_verification_boundary_aligned() {
    let readme = std::fs::read_to_string("README.md").expect("read README");
    let contributing = std::fs::read_to_string("CONTRIBUTING.md").expect("read CONTRIBUTING");
    let release = std::fs::read_to_string("RELEASE.md").expect("read RELEASE");

    assert!(
        normalize_whitespace(&readme).contains(
            "Release readiness still requires the broader local verification loop in \
             [CONTRIBUTING.md](CONTRIBUTING.md) and the maintainer checklist in \
             [RELEASE.md](RELEASE.md), including tests, docs, packaging, and smoke checks."
        ),
        "README must route local verification and release review to CONTRIBUTING and RELEASE"
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
        normalize_whitespace(&readme).contains(
            "The declared MSRV is part of the public contract. Until CI grows a dedicated \
             MSRV lane, verify it locally with `cargo +1.85 test --locked` before release or \
             when changing parser, refactor, packaging, or public API surfaces."
        ),
        "README must document how the declared MSRV is verified before CI adds an MSRV lane"
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
