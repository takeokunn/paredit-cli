use super::*;

#[test]
fn cli_plans_labels_to_flet_and_allows_body_calls() {
    paredit()
        .args([
            "refactor",
            "convert-labels-to-flet",
            "--dialect",
            "common-lisp",
            "--path",
            "0",
        ])
        .write_stdin("(labels ((work (x) (list x))) (work value))")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"rewritten\": \"(flet ((work (x) (list x))) (work value))\"",
        ));
}

#[test]
fn cli_rejects_recursive_definition_body_call() {
    paredit()
        .args([
            "refactor",
            "convert-labels-to-flet",
            "--dialect",
            "common-lisp",
            "--path",
            "0",
        ])
        .write_stdin("(labels ((work (x) (work x))) (work value))")
        .assert()
        .failure()
        .stderr(predicate::str::contains("recursive or mutually recursive"));
}

#[test]
fn cli_writes_converted_form() {
    let dir = fresh_temp_dir("convert-labels-to-flet-write");
    let file = dir.join("input.lisp");
    fs::write(&file, "(labels ((work () 1)) (work))\n").expect("write fixture");

    paredit()
        .args([
            "refactor",
            "convert-labels-to-flet",
            "--file",
            file.to_str().expect("utf8 path"),
            "--path",
            "0",
            "--write",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(file).expect("read output"),
        "(flet ((work () 1)) (work))\n"
    );
}
