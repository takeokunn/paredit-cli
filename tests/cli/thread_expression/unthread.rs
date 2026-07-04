use super::*;

#[test]
fn cli_plans_unthread_first_expression_without_writing() {
    let mut cmd = paredit();
    cmd.args(["unthread-expression", "--path", "0", "--output", "json"])
        .write_stdin("(-> id fetch-user normalize-user (format-name :short))")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"style\": \"first\""))
        .stdout(predicate::str::contains("\"operator\": \"->\""))
        .stdout(predicate::str::contains("\"base\": \"id\""))
        .stdout(predicate::str::contains(
            "\"replacement\": \"(format-name (normalize-user (fetch-user id)) :short)\"",
        ))
        .stdout(predicate::str::contains("\"written\": false"));
}

#[test]
fn cli_writes_unthread_last_expression_for_clojure_file() {
    let dir = fresh_temp_dir("unthread-expression-write");
    let clj_file = dir.join("pipeline.clj");
    fs::write(
        &clj_file,
        "(->> users (map score) (filter positive?) sum)\n",
    )
    .expect("write clojure fixture");

    let mut cmd = paredit();
    cmd.arg("unthread-expression")
        .arg("--file")
        .arg(&clj_file)
        .arg("--path")
        .arg("0")
        .arg("--write")
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"dialect\": \"clojure\""))
        .stdout(predicate::str::contains("\"style\": \"last\""))
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(clj_file).expect("read unthreaded clojure"),
        "(sum (filter positive? (map score users)))\n"
    );
}

#[test]
fn cli_rejects_unthread_custom_operator_without_style() {
    let mut cmd = paredit();
    cmd.args(["unthread-expression", "--path", "0"])
        .write_stdin("(my-> value step)")
        .assert()
        .failure()
        .stderr(predicate::str::contains("requires --style"));
}
