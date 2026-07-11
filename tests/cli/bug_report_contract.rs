#[test]
fn support_and_bug_report_form_collect_repro_details_needed_for_triage() {
    let support = std::fs::read_to_string("SUPPORT.md").expect("read support doc");
    let bug_report =
        std::fs::read_to_string(".github/ISSUE_TEMPLATE/bug_report.yml").expect("read bug report");
    let normalized_support = normalize_whitespace(&support);
    let normalized_bug_report = normalize_whitespace(&bug_report);

    for needle in [
        "exact command line",
        "actual result",
        "expected result",
        "Whether the command was run with `--write`.",
        "JSON output when the command supports `--output json`.",
        "The current commit or release version.",
    ] {
        assert!(
            normalized_support.contains(needle),
            "SUPPORT must document bug report requirement: {needle}"
        );
    }

    for needle in [
        "labels: - bug",
        "label: Reproduction",
        "whether `--write` was used",
        "dialect",
        "`--output json`",
        "label: Environment",
        "current release version or commit",
    ] {
        assert!(
            normalized_bug_report.contains(needle),
            "bug report form must collect bug triage detail: {needle}"
        );
    }
}

#[test]
fn bug_report_form_matches_public_triage_and_contract_routing_policy() {
    let maintainers = std::fs::read_to_string("MAINTAINERS.md").expect("read MAINTAINERS");
    let support = std::fs::read_to_string("SUPPORT.md").expect("read support doc");
    let bug_report =
        std::fs::read_to_string(".github/ISSUE_TEMPLATE/bug_report.yml").expect("read bug report");
    let normalized_bug_report = normalize_whitespace(&bug_report);

    assert!(
        maintainers
            .contains("Apply `bug` to reproducible correctness, safety, or regression reports."),
        "MAINTAINERS must define bug triage eligibility"
    );
    assert!(
        support.contains("[COMPATIBILITY.md](COMPATIBILITY.md)"),
        "SUPPORT must route stable-surface questions through COMPATIBILITY.md"
    );

    for needle in [
        "label: Contract Surface",
        "stable contract covered by `COMPATIBILITY.md`",
        "Preview or unstable behavior",
        "correctness, safety, CLI contract, or documentation defect",
    ] {
        assert!(
            normalized_bug_report.contains(needle),
            "bug report form must expose public contract routing detail: {needle}"
        );
    }
}

fn normalize_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
