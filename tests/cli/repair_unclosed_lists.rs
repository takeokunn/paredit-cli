use super::*;

#[test]
fn repair_unclosed_lists_writes_only_required_closers() {
    let dir = fresh_temp_dir("repair-unclosed-lists");
    let file = dir.join("source.lisp");
    fs::write(&file, "(outer [inner {leaf}").expect("write fixture");

    paredit()
        .args(["edit", "repair-unclosed-lists", "--write", "--file"])
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::is_empty());

    assert_eq!(
        fs::read_to_string(&file).expect("read repaired fixture"),
        "(outer [inner {leaf}])"
    );
    paredit()
        .args(["inspect", "check", "--file"])
        .arg(&file)
        .assert()
        .success();
}

#[test]
fn repair_unclosed_lists_rejects_other_parse_errors() {
    paredit()
        .args(["edit", "repair-unclosed-lists"])
        .write_stdin("(alpha]")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "repair-unclosed-lists only repairs unclosed lists",
        ));
}
