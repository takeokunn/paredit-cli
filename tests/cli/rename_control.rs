use super::*;

#[test]
fn cli_renames_block_and_matching_returns() {
    paredit()
        .args([
            "refactor",
            "rename-block",
            "--dialect",
            "common-lisp",
            "--path",
            "0",
            "--from",
            "out",
            "--to",
            "done",
        ])
        .write_stdin("(block out (return-from out 1) (block out (return-from out 2)))")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"rewritten\": \"(block done (return-from done 1) (block out (return-from out 2)))\"",
        ));
}

#[test]
fn cli_renames_tag_and_matching_go_forms() {
    paredit()
        .args([
            "refactor",
            "rename-tag",
            "--dialect",
            "common-lisp",
            "--path",
            "0",
            "--from",
            "start",
            "--to",
            "next",
        ])
        .write_stdin("(tagbody start (go start) (tagbody start (go start)))")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"rewritten\": \"(tagbody next (go next) (tagbody start (go start)))\"",
        ));
}

#[test]
fn cli_rejects_tag_capture() {
    paredit()
        .args([
            "refactor",
            "rename-tag",
            "--dialect",
            "common-lisp",
            "--path",
            "0",
            "--from",
            "start",
            "--to",
            "done",
        ])
        .write_stdin("(tagbody start (go done))")
        .assert()
        .failure()
        .stderr(predicate::str::contains("capture an existing go"));
}
