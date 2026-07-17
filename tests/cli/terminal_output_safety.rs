use super::*;

const HOSTILE_TEXT: &str = "line\n\t\u{1b}\u{202e}";
const ESCAPED_HOSTILE_TEXT: &str = r"line\u{a}\u{9}\u{1b}\u{202e}";

#[test]
fn text_renderer_escapes_controls_in_dynamic_source_text() {
    let input = format!("(foo \"{HOSTILE_TEXT}\")");
    let output = paredit()
        .args([
            "inspect",
            "form",
            "--path",
            "0",
            "--include-source",
            "--output",
            "text",
        ])
        .write_stdin(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let output = String::from_utf8(output).expect("text output is UTF-8");

    assert!(output.contains(&format!("source\t(foo \"{ESCAPED_HOSTILE_TEXT}\")\n")));
    assert!(!output.contains(HOSTILE_TEXT));
    assert!(!output.contains('\u{1b}'));
    assert!(!output.contains('\u{202e}'));
}

#[cfg(unix)]
#[test]
fn cli_diagnostic_escapes_controls_in_the_full_error_chain() {
    let dir = fresh_temp_dir("terminal-output-safety");
    let missing = dir.join(format!("missing-{HOSTILE_TEXT}.lisp"));
    let output = paredit()
        .args(["inspect", "form", "--file"])
        .arg(&missing)
        .assert()
        .code(1)
        .get_output()
        .stderr
        .clone();
    let output = String::from_utf8(output).expect("diagnostic output is UTF-8");

    assert!(output.starts_with("Error: "));
    assert!(output.contains(&format!("missing-{ESCAPED_HOSTILE_TEXT}.lisp")));
    assert!(
        output.contains(": "),
        "full error chain is rendered: {output:?}"
    );
    assert!(!output.contains(HOSTILE_TEXT));
    assert!(!output.contains('\u{1b}'));
    assert!(!output.contains('\u{202e}'));
}
