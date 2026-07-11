use super::*;
use proptest::test_runner::TestCaseError;

fn assert_cli_rename_binding_property(
    from: String,
    to: String,
    other: String,
) -> Result<(), TestCaseError> {
    prop_assume!(from != to);
    prop_assume!(from != other);
    prop_assume!(to != other);

    let dir = fresh_temp_dir("rename-binding-cli-pbt");
    let lisp_file = dir.join("core.lisp");
    let input =
        format!("(defun render () (let (({from} 1) ({other} {from})) (list {from} {other})))\n");
    fs::write(&lisp_file, &input)
        .map_err(|err| TestCaseError::fail(format!("write lisp fixture: {err}")))?;

    let output = paredit()
        .args([
            "rename-binding",
            "--file",
            lisp_file.to_str().expect("utf-8 fixture path"),
            "--path",
            "0.3",
            "--from",
            &from,
            "--to",
            &to,
            "--write",
        ])
        .output()
        .map_err(|err| TestCaseError::fail(format!("run paredit: {err}")))?;

    prop_assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report = parse_binding_write_report(&output.stdout)?;
    prop_assert!(report.changed);
    prop_assert_eq!(report.form, "let");
    prop_assert_eq!(report.path, "0.3");
    prop_assert_eq!(report.reference_count, 1);
    prop_assert_eq!(report.shadowed_scope_count, 0);
    prop_assert!(report.written);

    let rewritten = fs::read_to_string(&lisp_file)
        .map_err(|err| TestCaseError::fail(format!("read rewritten fixture: {err}")))?;
    let expected =
        format!("(defun render () (let (({to} 1) ({other} {from})) (list {to} {other})))\n");
    prop_assert_eq!(rewritten, expected);

    assert_cli_check_succeeds(&lisp_file)?;

    Ok(())
}

proptest! {
    #![proptest_config(cli_proptest_config(24))]

    #[test]
    fn pbt_cli_rename_binding_output_remains_parseable_and_scope_aware(
        from in "[a-z][a-z0-9-]{0,8}",
        to in "[a-z][a-z0-9-]{0,8}",
        other in "[a-z][a-z0-9-]{0,8}",
    ) {
        assert_cli_rename_binding_property(from, to, other)?;
    }
}

#[test]
fn cli_writes_binding_rename_without_touching_shadowed_scope() {
    let dir = fresh_temp_dir("rename-binding");
    let lisp_file = dir.join("core.lisp");
    fs::write(
        &lisp_file,
        "(defun render () (let ((value 1)) (+ value (let ((value 2)) value) value)))\n",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("rename-binding")
        .arg("--file")
        .arg(&lisp_file)
        .arg("--path")
        .arg("0.3")
        .arg("--from")
        .arg("value")
        .arg("--to")
        .arg("product")
        .arg("--write")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(lisp_file).expect("read rewritten lisp"),
        "(defun render () (let ((product 1)) (+ product (let ((value 2)) value) product)))\n"
    );
}

#[test]
fn cli_rejects_rename_binding_write_without_file() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--path",
        "0.3",
        "--from",
        "value",
        "--to",
        "product",
        "--write",
    ])
    .write_stdin("(defun render () (let ((value 1)) value))")
    .assert()
    .failure()
    .stderr(predicate::str::contains("--write requires --file"));
}

#[test]
fn cli_rejects_missing_binding_rename_target() {
    let mut cmd = paredit();
    cmd.args([
        "rename-binding",
        "--path",
        "0",
        "--from",
        "missing",
        "--to",
        "renamed",
    ])
    .write_stdin("(let ((value 1)) value)")
    .assert()
    .failure()
    .stderr(predicate::str::contains(
        "binding 'missing' was not found in selected let",
    ));
}
