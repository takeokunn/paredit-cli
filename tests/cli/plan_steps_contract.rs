use super::*;

/// The step `command` strings emitted by `refactor plan` are executed
/// literally by agents; this test pins them to the real CLI surface so a
/// command or flag rename cannot silently break emitted plans.
#[test]
fn plan_step_commands_reference_real_commands_and_flags() {
    let dir = fresh_temp_dir("plan-steps-contract");
    let file = dir.join("source.lisp");
    fs::write(&file, "(defun foo (x) x)\n(defun bar () (foo 1))\n").expect("write source fixture");

    let output = paredit()
        .args(["refactor", "plan", "--symbol", "foo", "--output", "json"])
        .arg(&file)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let plan: serde_json::Value = serde_json::from_slice(&output).expect("plan emits valid JSON");

    let mut commands = Vec::new();
    for step in plan["steps"].as_array().expect("plan steps") {
        let command = step["command"].as_str().expect("step command");
        // Steps may chain several invocations with `&&`.
        for segment in command.split("&&") {
            let segment = segment.trim();
            if segment.starts_with("paredit ") {
                commands.push(segment.to_owned());
            }
        }
    }
    assert!(
        commands.len() >= 3,
        "expected several runnable step commands, found {commands:?}"
    );

    let problems = validate_paredit_command_strings(&commands, &capability_map());
    assert!(
        problems.is_empty(),
        "refactor plan emits step commands that drifted from the real CLI:\n{}",
        problems.join("\n")
    );
}
