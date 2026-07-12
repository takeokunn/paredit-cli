use super::*;

/// Extract runnable examples: `<command>...</command>` tags plus the command
/// lines of multi-line `<example>` blocks. Prose that merely mentions
/// `paredit ...` in backticks is deliberately excluded.
fn skill_command_lines() -> Vec<String> {
    let skill = fs::read_to_string("skills/paredit-cli/SKILL.md")
        .expect("read skills/paredit-cli/SKILL.md");

    let mut commands = Vec::new();
    let mut in_example_block = false;
    for line in skill.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("<example") {
            in_example_block = !trimmed.contains("</example>");
            continue;
        }
        if trimmed.contains("</example>") {
            in_example_block = false;
            continue;
        }

        if let Some(start) = trimmed.find("<command>paredit ") {
            let rest = &trimmed[start + "<command>".len()..];
            let end = rest.find("</command>").unwrap_or(rest.len());
            commands.push(rest[..end].trim().to_owned());
        } else if in_example_block {
            if let Some(start) = trimmed.find("paredit ") {
                commands.push(trimmed[start..].trim().to_owned());
            }
        }
    }
    commands
}

#[test]
fn skill_examples_reference_real_commands_and_flags() {
    let capabilities = capability_map();
    let lines = skill_command_lines();
    assert!(
        lines.len() > 40,
        "expected the skill to contain many paredit examples, found {}",
        lines.len()
    );

    let problems = validate_paredit_command_strings(&lines, &capabilities);
    assert!(
        problems.is_empty(),
        "SKILL.md drifted from the real CLI:\n{}",
        problems.join("\n")
    );
}
