#[test]
fn support_and_feature_request_form_collect_scoping_details_for_new_capabilities() {
    let support = std::fs::read_to_string("SUPPORT.md").expect("read support doc");
    let feature_request = std::fs::read_to_string(".github/ISSUE_TEMPLATE/feature_request.yml")
        .expect("read feature request template");
    let normalized_support = normalize_whitespace(&support);

    for needle in [
        "structural editing value",
        "expected CLI or JSON contract",
        "current roadmap",
    ] {
        assert!(
            normalized_support.contains(needle),
            "SUPPORT must document feature request expectation: {needle}"
        );
    }

    for field in [
        "label: Problem Statement",
        "label: Proposed Change",
        "label: Why This Fits `paredit-cli`",
        "label: Alternatives Considered",
        "label: Policy Check",
    ] {
        assert!(
            feature_request.contains(field),
            "feature request form must collect {field}"
        );
    }
}

fn normalize_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[test]
fn feature_request_form_matches_public_triage_policy_for_enhancements() {
    let maintainers = std::fs::read_to_string("MAINTAINERS.md").expect("read MAINTAINERS");
    let feature_request = std::fs::read_to_string(".github/ISSUE_TEMPLATE/feature_request.yml")
        .expect("read feature request template");

    assert!(
        maintainers.contains("Apply `enhancement` only when the request matches"),
        "MAINTAINERS must define when enhancement triage is allowed"
    );
    assert!(
        maintainers.contains("has a clear structural-editing payoff."),
        "MAINTAINERS must require structural-editing payoff for enhancements"
    );
    assert!(
        feature_request.contains("labels:\n  - enhancement"),
        "feature request form must default to the enhancement label"
    );
    assert!(
        feature_request.contains(
            "Requests that expand surface area without clear structural-editing value are unlikely to be accepted."
        ),
        "feature request form must warn about structural-editing scope limits"
    );

    for checklist in [
        "I reviewed `ROADMAP.md` and believe this request matches current priorities.",
        "I reviewed `COMPATIBILITY.md` and understand this may require a public contract decision.",
    ] {
        assert!(
            feature_request.contains(checklist),
            "feature request form must enforce policy acknowledgment: {checklist}"
        );
    }
}
