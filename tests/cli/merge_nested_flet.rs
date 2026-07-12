use super::*;

#[test]
fn cli_plans_independent_nested_flet_merge() {
    paredit()
        .args([
            "refactor",
            "merge-nested-flet",
            "--dialect",
            "common-lisp",
            "--path",
            "0",
        ])
        .write_stdin(
            "(flet ((parse (x) (list x))) (flet ((emit (x) (print x))) (emit (parse value))))",
        )
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"rewritten\": \"(flet ((parse (x) (list x)) (emit (x) (print x))) (emit (parse value)))\"",
        ));
}

#[test]
fn cli_rejects_inner_definition_reference_to_outer_function() {
    paredit()
        .args([
            "refactor",
            "merge-nested-flet",
            "--dialect",
            "common-lisp",
            "--path",
            "0",
        ])
        .write_stdin("(flet ((parse (x) x)) (flet ((emit (x) (parse x))) (emit value)))")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "cannot move an inner definition outside the scope",
        ));
}

#[test]
fn cli_writes_merged_flet() {
    let dir = fresh_temp_dir("merge-nested-flet-write");
    let file = dir.join("input.lisp");
    fs::write(
        &file,
        "(flet ((left () 1)) (flet ((right () 2)) (+ (left) (right))))\n",
    )
    .expect("write fixture");

    paredit()
        .args([
            "refactor",
            "merge-nested-flet",
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
        "(flet ((left () 1) (right () 2)) (+ (left) (right)))\n"
    );
}
