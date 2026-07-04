use super::*;

#[test]
fn cli_plans_thread_first_expression_without_writing() {
    let mut cmd = paredit();
    cmd.args([
        "thread-expression",
        "--path",
        "0",
        "--style",
        "first",
        "--output",
        "json",
    ])
    .write_stdin("(format-name (normalize-user (fetch-user id)) :short)")
    .assert()
    .success()
    .stdout(predicate::str::contains("\"style\": \"first\""))
    .stdout(predicate::str::contains("\"base\": \"id\""))
    .stdout(predicate::str::contains(
        "\"replacement\": \"(-> id fetch-user normalize-user (format-name :short))\"",
    ));
}

#[test]
fn cli_writes_thread_last_expression_for_clojure_file() {
    let dir = fresh_temp_dir("thread-expression-write");
    let clj_file = dir.join("pipeline.clj");
    fs::write(&clj_file, "(sum (map score users))\n").expect("write clojure fixture");

    let mut cmd = paredit();
    cmd.arg("thread-expression")
        .arg("--file")
        .arg(&clj_file)
        .arg("--path")
        .arg("0")
        .arg("--style")
        .arg("last")
        .arg("--write")
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"dialect\": \"clojure\""))
        .stdout(predicate::str::contains("\"style\": \"last\""))
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(clj_file).expect("read threaded clojure"),
        "(->> users (map score) sum)\n"
    );
}

#[test]
fn cli_rejects_thread_expression_write_without_file() {
    let mut cmd = paredit();
    cmd.args([
        "thread-expression",
        "--path",
        "0",
        "--style",
        "first",
        "--write",
    ])
    .write_stdin("(display (render x))")
    .assert()
    .failure()
    .stderr(predicate::str::contains("--write requires --file"));
}

#[test]
fn cli_rejects_already_threaded_expression() {
    let mut cmd = paredit();
    cmd.args(["thread-expression", "--path", "0", "--style", "first"])
        .write_stdin("(-> x f)")
        .assert()
        .failure()
        .stderr(predicate::str::contains("already threaded"));
}
