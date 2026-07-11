#[test]
fn bug_report_form_collects_support_required_reproduction_details() {
    let support = std::fs::read_to_string("SUPPORT.md").expect("read support doc");
    let bug_report =
        std::fs::read_to_string(".github/ISSUE_TEMPLATE/bug_report.yml").expect("read bug report");

    for needle in [
        "exact command line",
        "expected result",
        "actual result",
        "Reproduction",
        "Expected Behavior",
        "Actual Behavior",
        "Environment",
    ] {
        assert!(
            support.contains(needle) || bug_report.contains(needle),
            "support contract should keep {needle} documented"
        );
    }

    for field in [
        "id: reproduction",
        "id: expected",
        "id: actual",
        "id: environment",
    ] {
        assert!(
            bug_report.contains(field),
            "bug report form must collect {field}"
        );
    }
}

#[test]
fn support_routing_keeps_security_reports_out_of_public_issue_forms() {
    let support = std::fs::read_to_string("SUPPORT.md").expect("read support doc");
    let issue_config =
        std::fs::read_to_string(".github/ISSUE_TEMPLATE/config.yml").expect("read issue config");

    assert!(
        support.contains("Security-sensitive reports must follow [SECURITY.md](SECURITY.md)"),
        "support doc must route security-sensitive reports to SECURITY.md"
    );
    assert!(
        issue_config
            .contains("private reporting path for vulnerabilities or security-sensitive behavior"),
        "issue config must expose the private security reporting path"
    );
}
