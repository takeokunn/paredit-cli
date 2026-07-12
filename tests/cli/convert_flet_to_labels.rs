use super::*;

#[test]
fn cli_plans_capture_free_flet_to_labels() {
    paredit()
        .args([
            "refactor",
            "convert-flet-to-labels",
            "--dialect",
            "common-lisp",
            "--path",
            "0",
        ])
        .write_stdin("(flet ((work (x) (list x))) (work value))")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"rewritten\": \"(labels ((work (x) (list x))) (work value))\"",
        ));
}

#[test]
fn cli_rejects_definition_body_reference_capture() {
    paredit()
        .args([
            "refactor",
            "convert-flet-to-labels",
            "--dialect",
            "common-lisp",
            "--path",
            "0",
        ])
        .write_stdin("(flet ((work (x) (work x))) (work value))")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "cannot capture local function references",
        ));
}

#[test]
fn cli_writes_converted_form() {
    let dir = fresh_temp_dir("convert-flet-to-labels-write");
    let file = dir.join("input.lisp");
    fs::write(&file, "(flet ((work () 1)) (work))\n").expect("write fixture");

    paredit()
        .args([
            "refactor",
            "convert-flet-to-labels",
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
        "(labels ((work () 1)) (work))\n"
    );
}
