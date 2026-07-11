use super::*;
use std::collections::BTreeSet;

#[test]
fn readme_commands_match_top_level_help() {
    let readme_commands = readme_command_set();
    let help_commands = top_level_help_command_set();

    assert_eq!(
        readme_commands, help_commands,
        "README ## Commands block drifted from `paredit --help`"
    );
}

fn readme_command_set() -> BTreeSet<String> {
    let readme = std::fs::read_to_string("README.md").expect("read README");
    let marker = "## Commands\n\n```sh\n";
    let commands_start = readme.find(marker).expect("README commands section") + marker.len();
    let commands_end = readme[commands_start..]
        .find("\n```")
        .expect("README commands code fence");
    let commands_block = &readme[commands_start..commands_start + commands_end];

    commands_block
        .lines()
        .filter_map(|line| line.strip_prefix("paredit "))
        .filter_map(|line| line.split_whitespace().next())
        .map(ToOwned::to_owned)
        .collect()
}

fn top_level_help_command_set() -> BTreeSet<String> {
    let output = paredit()
        .arg("--help")
        .output()
        .expect("run paredit --help");
    assert!(output.status.success(), "paredit --help failed");

    let stdout = String::from_utf8(output.stdout).expect("help output is utf-8");
    let commands_section = stdout
        .split("Commands:\n")
        .nth(1)
        .and_then(|section| section.split("\n\nOptions:\n").next())
        .expect("help commands section");

    commands_section
        .lines()
        .filter_map(|line| {
            let command = line.split_whitespace().next()?;
            if command == "help" {
                None
            } else {
                Some(command.to_owned())
            }
        })
        .collect()
}
