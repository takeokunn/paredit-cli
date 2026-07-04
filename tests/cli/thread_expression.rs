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

fn generated_thread_first_chain(depth: usize) -> String {
    let mut expression = "seed".to_owned();
    for index in 0..depth {
        expression = format!("(f{index} {expression})");
    }
    expression
}

fn generated_thread_last_chain(depth: usize) -> String {
    let mut expression = "seed".to_owned();
    for index in 0..depth {
        expression = format!("(g{index} arg{index} {expression})");
    }
    expression
}

fn generated_thread_first_pipeline(depth: usize) -> String {
    let mut forms = vec!["->".to_owned(), "seed".to_owned()];
    for index in 0..depth {
        if index % 2 == 0 {
            forms.push(format!("f{index}"));
        } else {
            forms.push(format!("(f{index} arg{index})"));
        }
    }
    format!("({})", forms.join(" "))
}

fn generated_thread_last_pipeline(depth: usize) -> String {
    let mut forms = vec!["->>".to_owned(), "seed".to_owned()];
    for index in 0..depth {
        if index % 2 == 0 {
            forms.push(format!("g{index}"));
        } else {
            forms.push(format!("(g{index} arg{index})"));
        }
    }
    format!("({})", forms.join(" "))
}

fn assert_thread_expression_property(
    input: String,
    style: &str,
    expected_operator: &str,
) -> Result<(), TestCaseError> {
    let output = paredit()
        .args([
            "thread-expression",
            "--path",
            "0",
            "--style",
            style,
            "--output",
            "json",
        ])
        .write_stdin(input)
        .output()
        .map_err(|err| TestCaseError::fail(format!("run paredit: {err}")))?;

    prop_assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report = serde_json::from_slice::<serde_json::Value>(&output.stdout)
        .map_err(|err| TestCaseError::fail(format!("parse json: {err}")))?;
    prop_assert_eq!(report["changed"].as_bool(), Some(true));
    let replacement = report["replacement"].as_str().unwrap_or_default();
    let expected_prefix = format!("({} seed", expected_operator);
    prop_assert!(replacement.starts_with(&expected_prefix));

    let rewritten = report["rewritten"].as_str().unwrap_or_default();
    let check_output = paredit()
        .arg("check")
        .write_stdin(rewritten.to_owned())
        .output()
        .map_err(|err| TestCaseError::fail(format!("run check: {err}")))?;
    prop_assert!(
        check_output.status.success(),
        "check stderr={}",
        String::from_utf8_lossy(&check_output.stderr)
    );
    Ok(())
}

fn assert_unthread_expression_property(input: String) -> Result<(), TestCaseError> {
    let output = paredit()
        .args(["unthread-expression", "--path", "0", "--output", "json"])
        .write_stdin(input)
        .output()
        .map_err(|err| TestCaseError::fail(format!("run paredit: {err}")))?;

    prop_assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report = serde_json::from_slice::<serde_json::Value>(&output.stdout)
        .map_err(|err| TestCaseError::fail(format!("parse json: {err}")))?;
    prop_assert_eq!(report["changed"].as_bool(), Some(true));
    let replacement = report["replacement"].as_str().unwrap_or_default();
    prop_assert!(replacement.starts_with('('));

    let check_output = paredit()
        .arg("check")
        .write_stdin(replacement.to_owned())
        .output()
        .map_err(|err| TestCaseError::fail(format!("run check: {err}")))?;
    prop_assert!(
        check_output.status.success(),
        "check stderr={}",
        String::from_utf8_lossy(&check_output.stderr)
    );
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(12))]

    #[test]
    fn cli_thread_expression_preserves_parseability_for_generated_chains(depth in 1usize..8) {
        assert_thread_expression_property(generated_thread_first_chain(depth), "first", "->")?;
        assert_thread_expression_property(generated_thread_last_chain(depth), "last", "->>")?;
    }

    #[test]
    fn cli_unthread_expression_preserves_parseability_for_generated_pipelines(depth in 1usize..8) {
        assert_unthread_expression_property(generated_thread_first_pipeline(depth))?;
        assert_unthread_expression_property(generated_thread_last_pipeline(depth))?;
    }
}
