use super::*;

#[test]
fn transpose_forward_selects_by_path_from_stdin_and_stays_parseable() {
    let output = paredit()
        .args(["edit", "transpose-forward", "--path", "0.0"])
        .write_stdin("(alpha  ;; slot comment\n beta gamma)")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    assert_eq!(
        String::from_utf8(output.clone()).expect("utf8"),
        "(beta  ;; slot comment\n alpha gamma)"
    );
    paredit()
        .args(["inspect", "check"])
        .write_stdin(output)
        .assert()
        .success();
}

#[test]
fn transpose_backward_selects_by_offset_from_file_without_modifying_it() {
    let dir = fresh_temp_dir("transpose");
    let file = dir.join("source.lisp");
    let input = "(alpha beta gamma)";
    fs::write(&file, input).expect("write fixture");

    paredit()
        .args([
            "edit",
            "transpose-backward",
            "--file",
            file.to_str().expect("utf8 path"),
            "--at",
            "8",
        ])
        .assert()
        .success()
        .stdout("(beta alpha gamma)");
    assert_eq!(fs::read_to_string(&file).expect("read fixture"), input);
}

#[test]
fn transpose_reports_a_boundary_failure() {
    paredit()
        .args(["edit", "transpose-forward", "--path", "0.1"])
        .write_stdin("(alpha beta)")
        .assert()
        .failure()
        .stderr(predicate::str::contains("no next sibling"));
}
