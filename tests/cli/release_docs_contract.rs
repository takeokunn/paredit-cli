use std::collections::BTreeSet;

#[test]
fn contributing_and_release_document_the_same_development_loop() {
    let contributing = std::fs::read_to_string("CONTRIBUTING.md").expect("read CONTRIBUTING");
    let release = std::fs::read_to_string("RELEASE.md").expect("read RELEASE");

    assert_eq!(
        command_block_after(&contributing, "## Development Loop"),
        command_block_after(&release, "1. Run the documented development loop:"),
        "CONTRIBUTING and RELEASE drifted on the documented verification loop"
    );
}

#[test]
fn contributing_and_release_document_the_same_required_archive_docs() {
    let contributing = std::fs::read_to_string("CONTRIBUTING.md").expect("read CONTRIBUTING");
    let release = std::fs::read_to_string("RELEASE.md").expect("read RELEASE");

    let expected = required_archive_docs();
    assert_eq!(
        backticked_tokens_in_section(
            &contributing,
            "## Release Checklist",
            "- Confirm generated archives include ",
        ),
        expected,
        "CONTRIBUTING release archive contract drifted"
    );
    assert_eq!(
        backticked_tokens_in_section(
            &release,
            "Confirm the release archive includes the expected policy and support",
            "documents that users rely on: ",
        ),
        expected,
        "RELEASE release archive contract drifted"
    );
}

fn command_block_after(markdown: &str, marker: &str) -> Vec<String> {
    let start = markdown.find(marker).expect("section marker") + marker.len();
    let mut in_block = false;
    let mut commands = Vec::new();

    for line in markdown[start..].lines() {
        let trimmed = line.trim();
        if !in_block {
            if trimmed == "```sh" {
                in_block = true;
            }
            continue;
        }

        if trimmed == "```" {
            return commands;
        }

        if !trimmed.is_empty() {
            commands.push(trimmed.to_owned());
        }
    }

    panic!("code fence end");
}

fn backticked_tokens_in_section(
    markdown: &str,
    section_marker: &str,
    line_marker: &str,
) -> BTreeSet<String> {
    let section_start = markdown.find(section_marker).expect("section marker");
    let section = &markdown[section_start..];
    let line_start = section.find(line_marker).expect("line marker") + line_marker.len();
    let after_marker = &section[line_start..];
    let paragraph_end = after_marker.find(".\n").expect("paragraph end");
    backticked_tokens(&after_marker[..paragraph_end])
}

fn backticked_tokens(text: &str) -> BTreeSet<String> {
    let mut tokens = BTreeSet::new();
    let mut remainder = text;

    while let Some(start) = remainder.find('`') {
        let token_start = start + 1;
        let tail = &remainder[token_start..];
        let token_end = tail.find('`').expect("closing backtick");
        tokens.insert(tail[..token_end].to_owned());
        remainder = &tail[token_end + 1..];
    }

    tokens
}

fn required_archive_docs() -> BTreeSet<String> {
    [
        "CHANGELOG.md",
        "CODE_OF_CONDUCT.md",
        "COMPATIBILITY.md",
        "CONTRIBUTING.md",
        "GOVERNANCE.md",
        "LICENSE",
        "MAINTAINERS.md",
        "README.md",
        "RELEASE.md",
        "ROADMAP.md",
        "SECURITY.md",
        "SKILLS.md",
        "SUPPORT.md",
    ]
    .into_iter()
    .map(ToOwned::to_owned)
    .collect()
}
