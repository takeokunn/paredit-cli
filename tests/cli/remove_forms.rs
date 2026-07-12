use super::*;

#[test]
fn cli_plans_remove_forms_from_stdin() {
    let mut cmd = paredit();
    cmd.args([
        "refactor",
        "remove-forms",
        "--path",
        "0",
        "--output",
        "json",
    ])
    .write_stdin("(keep 1)\n(drop 2)\n")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"path_count\": 1"))
    .stdout(predicate::str::contains("\"changed\": true"))
    .stdout(predicate::str::contains("\"written\": false"))
    .stdout(predicate::str::contains("\"rewritten\": \"(drop 2)\\n\""))
    .stdout(predicate::str::contains("\"text\": \"(keep 1)\""));
}

#[test]
fn cli_plans_remove_forms_from_file_without_writing() {
    let dir = fresh_temp_dir("remove-forms-plan");
    let file = dir.join("forms.lisp");
    let original = "(keep 1)\n(drop 2)\n(remove 3)\n";
    fs::write(&file, original).expect("write fixture");

    let mut cmd = paredit();
    cmd.args([
        "refactor",
        "remove-forms",
        "--file",
        file.to_str().expect("utf-8 path"),
        "--path",
        "0",
        "--path",
        "2",
        "--output",
        "json",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("\"path_count\": 2"))
    .stdout(predicate::str::contains("\"changed\": true"))
    .stdout(predicate::str::contains("\"written\": false"))
    .stdout(predicate::str::contains("\"rewritten\": \"(drop 2)\\n\""))
    .stdout(predicate::str::contains("\"path\": \"0\""))
    .stdout(predicate::str::contains("\"path\": \"2\""));

    assert_eq!(
        fs::read_to_string(&file).expect("read unchanged file"),
        original
    );
}

#[test]
fn cli_writes_removed_forms_to_file() {
    let dir = fresh_temp_dir("remove-forms-write");
    let file = dir.join("forms.lisp");
    fs::write(&file, "(keep 1)\n(drop 2)\n(remove 3)\n").expect("write fixture");

    let mut cmd = paredit();
    cmd.args([
        "refactor",
        "remove-forms",
        "--file",
        file.to_str().expect("utf-8 path"),
        "--path",
        "0",
        "--path",
        "2",
        "--write",
        "--output",
        "json",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("\"changed\": true"))
    .stdout(predicate::str::contains("\"written\": true"))
    .stdout(predicate::str::contains("\"rewritten\": \"(drop 2)\\n\""));

    assert_eq!(
        fs::read_to_string(&file).expect("read rewritten file"),
        "(drop 2)\n"
    );
}

#[test]
fn cli_rejects_remove_forms_write_without_file() {
    let mut cmd = paredit();
    cmd.args(["refactor", "remove-forms", "--path", "0", "--write"])
        .write_stdin("(drop 1)")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "remove-forms --write requires --file",
        ));
}

#[test]
fn cli_rejects_overlapping_remove_forms_paths() {
    let mut cmd = paredit();
    cmd.args(["refactor", "remove-forms", "--path", "0", "--path", "0.1"])
        .write_stdin("(outer (inner 1))")
        .assert()
        .failure()
        .stderr(predicate::str::contains("paths must not overlap"));
}
