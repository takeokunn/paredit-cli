use std::collections::BTreeSet;

#[test]
fn pull_request_template_policy_review_matches_maintainer_minimum() {
    let maintainers = std::fs::read_to_string("MAINTAINERS.md").expect("read MAINTAINERS");
    let pull_request =
        std::fs::read_to_string(".github/PULL_REQUEST_TEMPLATE.md").expect("read pr template");

    let expected = maintainer_review_documents(&maintainers);
    let actual = policy_review_documents(&pull_request);

    for doc in expected {
        assert!(
            actual.contains(&doc),
            "pull request policy review checklist must include MAINTAINERS review minimum for {doc}"
        );
    }
}

#[test]
fn pull_request_template_requires_verification_when_maintainers_do() {
    let maintainers = std::fs::read_to_string("MAINTAINERS.md").expect("read MAINTAINERS");
    let pull_request =
        std::fs::read_to_string(".github/PULL_REQUEST_TEMPLATE.md").expect("read pr template");

    assert!(
        maintainers
            .contains("Require an explicit verification list for every user-visible change."),
        "MAINTAINERS must document explicit verification requirements"
    );
    assert!(
        pull_request.contains("## Verification"),
        "pull request template must expose a verification checklist"
    );
}

fn maintainer_review_documents(markdown: &str) -> BTreeSet<String> {
    let pull_request_review_minimum = section(markdown, "## Pull Request Review Minimum");
    let mut documents = BTreeSet::new();

    for doc in ["COMPATIBILITY.md", "CHANGELOG.md", "SECURITY.md"] {
        assert!(
            pull_request_review_minimum.contains(doc),
            "MAINTAINERS review minimum must mention {doc}"
        );
        documents.insert(doc.to_owned());
    }

    documents
}

fn policy_review_documents(markdown: &str) -> BTreeSet<String> {
    section(markdown, "## Policy Review")
        .lines()
        .filter_map(backticked_document)
        .collect()
}

fn section<'a>(markdown: &'a str, heading: &str) -> &'a str {
    let start = markdown.find(heading).expect("heading");
    let tail = &markdown[start + heading.len()..];
    let next_heading = tail.find("\n## ").unwrap_or(tail.len());
    &tail[..next_heading]
}

fn backticked_document(line: &str) -> Option<String> {
    let first_tick = line.find('`')?;
    let tail = &line[first_tick + 1..];
    let second_tick = tail.find('`')?;
    let candidate = &tail[..second_tick];
    candidate.ends_with(".md").then(|| candidate.to_owned())
}
