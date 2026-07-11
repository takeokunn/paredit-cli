use super::*;
use proptest::test_runner::TestCaseError;

#[test]
fn cli_plans_definition_sort_without_writing() {
    let dir = fresh_temp_dir("sort-definitions-plan");
    let file = dir.join("core.lisp");
    let original = "(in-package #:demo)\n\n\
                    (defun zeta () :z)\n\
                    (defmacro alpha () nil)\n\
                    (defun beta () :b)\n\
                    (define-symbol-macro current-user (session-user *session*))\n";
    fs::write(&file, original).expect("write source fixture");

    let mut cmd = paredit();
    cmd.arg("sort-definitions")
        .arg("--file")
        .arg(&file)
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"command\": \"sort-definitions\"",
        ))
        .stdout(predicate::str::contains("\"strategy\": \"name\""))
        .stdout(predicate::str::contains("\"definition_count\": 4"))
        .stdout(predicate::str::contains("\"name\": \"alpha\""))
        .stdout(predicate::str::contains("\"name\": \"beta\""))
        .stdout(predicate::str::contains("\"name\": \"current-user\""))
        .stdout(predicate::str::contains("\"name\": \"zeta\""))
        .stdout(predicate::str::contains("\"changed\": true"))
        .stdout(predicate::str::contains("\"written\": false"));

    assert_eq!(
        fs::read_to_string(&file).expect("read unchanged source"),
        original
    );
}

#[test]
fn cli_writes_sorted_definitions() {
    let dir = fresh_temp_dir("sort-definitions-write");
    let file = dir.join("core.lisp");
    fs::write(
        &file,
        "(in-package #:demo)\n\n\
         (defun zeta () :z)\n\
         (defmacro alpha () nil)\n\
         (defun beta () :b)\n",
    )
    .expect("write source fixture");

    let mut cmd = paredit();
    cmd.arg("sort-definitions")
        .arg("--file")
        .arg(&file)
        .arg("--write")
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"written\": true"));

    let rewritten = fs::read_to_string(&file).expect("read rewritten source");
    let package = rewritten.find("(in-package #:demo)").expect("package form");
    let alpha = rewritten.find("(defmacro alpha").expect("alpha");
    let beta = rewritten.find("(defun beta").expect("beta");
    let zeta = rewritten.find("(defun zeta").expect("zeta");
    assert!(package < alpha);
    assert!(alpha < beta);
    assert!(beta < zeta);
}

#[test]
fn cli_respects_non_definition_barriers() {
    let dir = fresh_temp_dir("sort-definitions-barrier");
    let file = dir.join("core.lisp");
    let original = "(defun zeta () :z)\n\
                    (print :barrier)\n\
                    (defun alpha () :a)\n";
    fs::write(&file, original).expect("write source fixture");

    let mut cmd = paredit();
    cmd.arg("sort-definitions")
        .arg("--file")
        .arg(&file)
        .arg("--write")
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"definition_count\": 0"))
        .stdout(predicate::str::contains("\"changed\": false"))
        .stdout(predicate::str::contains("\"written\": false"));

    assert_eq!(
        fs::read_to_string(&file).expect("read unchanged source"),
        original
    );
}

fn assert_sort_definitions_property(definition_count: usize) -> Result<(), TestCaseError> {
    let dir = fresh_temp_dir(&format!("sort-definitions-pbt-{definition_count}"));
    let file = dir.join("core.lisp");
    let mut source = String::from("(in-package #:demo)\n");
    for index in (0..definition_count).rev() {
        source.push_str(&format!("(defun generated-{index} () {index})\n"));
    }
    fs::write(&file, source).expect("write generated source");

    let output = paredit()
        .arg("sort-definitions")
        .arg("--file")
        .arg(&file)
        .arg("--write")
        .arg("--output")
        .arg("json")
        .output()
        .map_err(|err| TestCaseError::fail(format!("run sort-definitions: {err}")))?;
    prop_assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let report = serde_json::from_slice::<serde_json::Value>(&output.stdout)
        .map_err(|err| TestCaseError::fail(format!("parse json: {err}")))?;
    prop_assert_eq!(
        report["definition_count"].as_u64(),
        Some(definition_count as u64)
    );

    let rewritten = fs::read_to_string(&file)
        .map_err(|err| TestCaseError::fail(format!("read rewritten source: {err}")))?;
    let mut previous_position = 0;
    for index in 0..definition_count {
        let name = format!("generated-{index}");
        let position = rewritten
            .find(&name)
            .ok_or_else(|| TestCaseError::fail(format!("missing {name}")))?;
        prop_assert!(position >= previous_position);
        previous_position = position;
    }

    let check_output = paredit()
        .arg("check")
        .write_stdin(rewritten)
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
    #![proptest_config(cli_proptest_config(12))]

    #[test]
    fn cli_sort_definitions_preserves_parseability_and_name_order(definition_count in 2usize..8) {
        assert_sort_definitions_property(definition_count)?;
    }
}
