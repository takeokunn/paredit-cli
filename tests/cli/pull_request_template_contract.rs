use std::collections::BTreeSet;

#[test]
fn pull_request_template_covers_documented_development_loop() {
    let contributing = std::fs::read_to_string("CONTRIBUTING.md").expect("read CONTRIBUTING");
    let pull_request =
        std::fs::read_to_string(".github/PULL_REQUEST_TEMPLATE.md").expect("read pr template");

    let expected = command_block_after(&contributing, "## Development Loop")
        .into_iter()
        .filter(|command| command != "nix develop")
        .collect::<BTreeSet<_>>();

    assert_eq!(
        verification_checkboxes(&pull_request),
        expected,
        "pull request verification checklist drifted from CONTRIBUTING development loop"
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

fn verification_checkboxes(markdown: &str) -> BTreeSet<String> {
    let verification_start = markdown
        .find("## Verification")
        .expect("verification heading");
    let policy_start = markdown
        .find("## Policy Review")
        .expect("policy review heading");

    markdown[verification_start..policy_start]
        .lines()
        .filter_map(|line| checkbox_command(line.trim()))
        .collect()
}

fn checkbox_command(line: &str) -> Option<String> {
    let prefix = "- [ ] `";
    if !line.starts_with(prefix) {
        return None;
    }

    let tail = &line[prefix.len()..];
    let end = tail.find('`').expect("closing backtick");
    Some(tail[..end].to_owned())
}
