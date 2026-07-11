use super::*;
use proptest::test_runner::TestCaseError;

fn assert_cli_wrap_function_calls_property(
    function: String,
    wrapper: String,
    arg: String,
) -> Result<(), TestCaseError> {
    prop_assume!(function != wrapper);
    prop_assume!(function != arg);
    prop_assume!(wrapper != arg);

    let dir = fresh_temp_dir("wrap-function-calls-cli-pbt");
    let lisp_file = dir.join("service.lisp");
    let input = format!("(defun render () ({function} {arg}) ({wrapper} ({function} cached)))\n");
    fs::write(&lisp_file, &input)
        .map_err(|err| TestCaseError::fail(format!("write lisp fixture: {err}")))?;

    let output = paredit()
        .arg("wrap-function-calls")
        .arg(&lisp_file)
        .arg("--function")
        .arg(&function)
        .arg("--wrapper")
        .arg(&wrapper)
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
    let report = parse_wrap_report(&output.stdout)?;
    prop_assert_eq!(report.call_count, 1);
    prop_assert_eq!(report.skipped_already_wrapped_count, 1);
    prop_assert_eq!(report.skipped_nested_count, 0);
    prop_assert_eq!(report.files.first().map(|file| file.written), Some(true));

    let rewritten = fs::read_to_string(&lisp_file)
        .map_err(|err| TestCaseError::fail(format!("read rewritten fixture: {err}")))?;
    let expected = format!(
        "(defun render () ({wrapper} ({function} {arg})) ({wrapper} ({function} cached)))\n"
    );
    prop_assert_eq!(rewritten, expected);
    assert_cli_check_succeeds(&lisp_file)?;

    Ok(())
}

proptest! {
    #![proptest_config(cli_proptest_config(24))]

    #[test]
    fn pbt_cli_wrap_function_calls_output_remains_parseable_and_skips_already_wrapped_calls(
        function in "[a-z][a-z0-9-]{0,8}",
        wrapper in "[a-z][a-z0-9-]{0,8}",
        arg in "[a-z][a-z0-9-]{0,8}",
    ) {
        assert_cli_wrap_function_calls_property(function, wrapper, arg)?;
    }
}
