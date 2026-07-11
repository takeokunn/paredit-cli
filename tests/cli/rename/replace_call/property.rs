use super::*;
use proptest::test_runner::TestCaseError;

fn assert_cli_replace_function_calls_property(
    from: String,
    to: String,
    arg: String,
) -> Result<(), TestCaseError> {
    prop_assume!(from != to);
    prop_assume!(from != arg);
    prop_assume!(to != arg);

    let dir = fresh_temp_dir("replace-function-calls-cli-pbt");
    let lisp_file = dir.join("service.lisp");
    let input = format!("(defun keep () {from})\n({from} {arg})\n");
    fs::write(&lisp_file, &input)
        .map_err(|err| TestCaseError::fail(format!("write lisp fixture: {err}")))?;

    let output = paredit()
        .arg("replace-function-calls")
        .arg(&lisp_file)
        .arg("--from")
        .arg(&from)
        .arg("--to")
        .arg(&to)
        .arg("--all-calls")
        .arg("--write")
        .arg("--output")
        .arg("json")
        .output()
        .map_err(|err| TestCaseError::fail(format!("run paredit: {err}")))?;

    prop_assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report = parse_replace_call_report(&output.stdout)?;
    prop_assert_eq!(report.call_count, 1);
    prop_assert_eq!(report.files.first().map(|file| file.written), Some(true));

    let rewritten = fs::read_to_string(&lisp_file)
        .map_err(|err| TestCaseError::fail(format!("read rewritten fixture: {err}")))?;
    let replaced_call = format!("({to} {arg})");
    let preserved_definition = format!("(defun keep () {from})");
    prop_assert!(rewritten.contains(&replaced_call));
    prop_assert!(rewritten.contains(&preserved_definition));
    assert_cli_check_succeeds(&lisp_file)?;

    Ok(())
}

proptest! {
    #![proptest_config(cli_proptest_config(24))]

    #[test]
    fn pbt_cli_replace_function_calls_output_remains_parseable(
        from in "[a-z][a-z0-9-]{0,8}",
        to in "[a-z][a-z0-9-]{0,8}",
        arg in "[a-z][a-z0-9-]{0,8}",
    ) {
        assert_cli_replace_function_calls_property(from, to, arg)?;
    }
}
