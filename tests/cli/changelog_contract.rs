#[test]
fn changelog_keeps_public_change_tracking_guidance_and_unreleased_section() {
    let changelog = std::fs::read_to_string("CHANGELOG.md").expect("read CHANGELOG");

    assert!(
        changelog.contains("All notable user-visible changes to this project will be documented"),
        "CHANGELOG must state that user-visible changes are tracked publicly"
    );
    assert!(
        changelog.contains("not internal-only refactors"),
        "CHANGELOG must distinguish user-visible changes from internal-only refactors"
    );
    assert!(
        changelog.contains("## [Unreleased]"),
        "CHANGELOG must keep an [Unreleased] section for upcoming public changes"
    );
}

#[test]
fn policy_docs_and_pr_template_require_changelog_updates_for_user_visible_changes() {
    let governance = std::fs::read_to_string("GOVERNANCE.md").expect("read GOVERNANCE");
    let compatibility = std::fs::read_to_string("COMPATIBILITY.md").expect("read COMPATIBILITY");
    let maintainers = std::fs::read_to_string("MAINTAINERS.md").expect("read MAINTAINERS");
    let release = std::fs::read_to_string("RELEASE.md").expect("read RELEASE");
    let pull_request =
        std::fs::read_to_string(".github/PULL_REQUEST_TEMPLATE.md").expect("read pr template");

    assert!(
        governance.contains("document the user-visible effect in\n  [CHANGELOG.md](CHANGELOG.md)."),
        "GOVERNANCE must route stable-surface decisions through CHANGELOG.md"
    );
    assert!(
        compatibility.contains("Record the user-visible effect in [CHANGELOG.md](CHANGELOG.md)."),
        "COMPATIBILITY must require changelog entries for stable-surface changes"
    );
    assert!(
        maintainers
            .contains("Require [CHANGELOG.md](CHANGELOG.md) updates for user-visible behavior."),
        "MAINTAINERS must require changelog updates for user-visible behavior"
    );
    assert!(
        release.contains(
            "User-visible behavior changes are recorded in [CHANGELOG.md](CHANGELOG.md)."
        ),
        "RELEASE must require changelog coverage before shipping"
    );
    assert!(
        pull_request.contains("`CHANGELOG.md` updated for user-visible behavior changes"),
        "pull request template must expose a changelog review checkbox"
    );
}
