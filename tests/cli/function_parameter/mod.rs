use super::*;

use proptest::test_runner::TestCaseError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CliAddFunctionParameterReport {
    pub(super) all_calls: bool,
    pub(super) function_name: String,
    pub(super) parameter_name: String,
    pub(super) argument: String,
    pub(super) insert: String,
    pub(super) parameter_section: String,
    pub(super) changed: bool,
    pub(super) written: bool,
    pub(super) rewritten: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CliRemoveFunctionParameterReport {
    pub(super) all_calls: bool,
    pub(super) function_name: String,
    pub(super) parameter_name: String,
    pub(super) parameter_index: u64,
    pub(super) parameter_keyword: Option<String>,
    pub(super) removed_arguments: Vec<Option<String>>,
    pub(super) changed: bool,
    pub(super) written: bool,
    pub(super) rewritten: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CliSwappedArgumentReport {
    pub(super) left: String,
    pub(super) right: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CliSwapFunctionParametersReport {
    pub(super) all_calls: bool,
    pub(super) function_name: String,
    pub(super) left_name: String,
    pub(super) right_name: String,
    pub(super) left_index: u64,
    pub(super) right_index: u64,
    pub(super) swapped_arguments: Vec<CliSwappedArgumentReport>,
    pub(super) changed: bool,
    pub(super) written: bool,
    pub(super) rewritten: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CliReorderFunctionParametersReport {
    pub(super) all_calls: bool,
    pub(super) function_name: String,
    pub(super) old_parameter_order: Vec<String>,
    pub(super) new_parameter_order: Vec<String>,
    pub(super) reordered_arguments: Vec<Vec<String>>,
    pub(super) changed: bool,
    pub(super) written: bool,
    pub(super) rewritten: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CliMoveFunctionParameterReport {
    pub(super) all_calls: bool,
    pub(super) function_name: String,
    pub(super) parameter_name: String,
    pub(super) from_index: u64,
    pub(super) to_index: u64,
    pub(super) moved_arguments: Vec<String>,
    pub(super) changed: bool,
    pub(super) written: bool,
    pub(super) rewritten: String,
}

fn parse_cli_json(stdout: &[u8]) -> Result<serde_json::Value, TestCaseError> {
    serde_json::from_slice(stdout).map_err(|err| TestCaseError::fail(format!("parse json: {err}")))
}

fn json_u64(value: &serde_json::Value, key: &'static str) -> Result<u64, TestCaseError> {
    value[key]
        .as_u64()
        .ok_or_else(|| TestCaseError::fail(format!("missing u64 field: {key}")))
}

fn json_bool(value: &serde_json::Value, key: &'static str) -> Result<bool, TestCaseError> {
    value[key]
        .as_bool()
        .ok_or_else(|| TestCaseError::fail(format!("missing bool field: {key}")))
}

fn json_string(value: &serde_json::Value, key: &'static str) -> Result<String, TestCaseError> {
    value[key]
        .as_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| TestCaseError::fail(format!("missing string field: {key}")))
}

fn json_optional_string(
    value: &serde_json::Value,
    key: &'static str,
) -> Result<Option<String>, TestCaseError> {
    match &value[key] {
        serde_json::Value::Null => Ok(None),
        serde_json::Value::String(text) => Ok(Some(text.clone())),
        _ => Err(TestCaseError::fail(format!(
            "missing optional string field: {key}"
        ))),
    }
}

fn json_string_array(
    value: &serde_json::Value,
    key: &'static str,
) -> Result<Vec<String>, TestCaseError> {
    value[key]
        .as_array()
        .ok_or_else(|| TestCaseError::fail(format!("missing array field: {key}")))?
        .iter()
        .map(|item| {
            item.as_str()
                .map(ToOwned::to_owned)
                .ok_or_else(|| TestCaseError::fail(format!("non-string array item: {key}")))
        })
        .collect()
}

fn json_optional_string_array(
    value: &serde_json::Value,
    key: &'static str,
) -> Result<Vec<Option<String>>, TestCaseError> {
    value[key]
        .as_array()
        .ok_or_else(|| TestCaseError::fail(format!("missing array field: {key}")))?
        .iter()
        .map(|item| match item {
            serde_json::Value::Null => Ok(None),
            serde_json::Value::String(text) => Ok(Some(text.clone())),
            _ => Err(TestCaseError::fail(format!(
                "non-optional-string array item: {key}"
            ))),
        })
        .collect()
}

pub(super) fn parse_add_function_parameter_report(
    stdout: &[u8],
) -> Result<CliAddFunctionParameterReport, TestCaseError> {
    let value = parse_cli_json(stdout)?;

    Ok(CliAddFunctionParameterReport {
        all_calls: json_bool(&value, "all_calls")?,
        function_name: json_string(&value, "function_name")?,
        parameter_name: json_string(&value, "parameter_name")?,
        argument: json_string(&value, "argument")?,
        insert: json_string(&value, "insert")?,
        parameter_section: json_string(&value, "parameter_section")?,
        changed: json_bool(&value, "changed")?,
        written: json_bool(&value, "written")?,
        rewritten: json_string(&value, "rewritten")?,
    })
}

pub(super) fn parse_remove_function_parameter_report(
    stdout: &[u8],
) -> Result<CliRemoveFunctionParameterReport, TestCaseError> {
    let value = parse_cli_json(stdout)?;

    Ok(CliRemoveFunctionParameterReport {
        all_calls: json_bool(&value, "all_calls")?,
        function_name: json_string(&value, "function_name")?,
        parameter_name: json_string(&value, "parameter_name")?,
        parameter_index: json_u64(&value, "parameter_index")?,
        parameter_keyword: json_optional_string(&value, "parameter_keyword")?,
        removed_arguments: json_optional_string_array(&value, "removed_arguments")?,
        changed: json_bool(&value, "changed")?,
        written: json_bool(&value, "written")?,
        rewritten: json_string(&value, "rewritten")?,
    })
}

pub(super) fn parse_swap_function_parameters_report(
    stdout: &[u8],
) -> Result<CliSwapFunctionParametersReport, TestCaseError> {
    let value = parse_cli_json(stdout)?;
    let swapped_arguments = value["swapped_arguments"]
        .as_array()
        .ok_or_else(|| TestCaseError::fail("missing swapped_arguments array".to_owned()))?
        .iter()
        .map(|argument| {
            Ok::<CliSwappedArgumentReport, TestCaseError>(CliSwappedArgumentReport {
                left: json_string(argument, "left")?,
                right: json_string(argument, "right")?,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(CliSwapFunctionParametersReport {
        all_calls: json_bool(&value, "all_calls")?,
        function_name: json_string(&value, "function_name")?,
        left_name: json_string(&value, "left_name")?,
        right_name: json_string(&value, "right_name")?,
        left_index: json_u64(&value, "left_index")?,
        right_index: json_u64(&value, "right_index")?,
        swapped_arguments,
        changed: json_bool(&value, "changed")?,
        written: json_bool(&value, "written")?,
        rewritten: json_string(&value, "rewritten")?,
    })
}

pub(super) fn parse_reorder_function_parameters_report(
    stdout: &[u8],
) -> Result<CliReorderFunctionParametersReport, TestCaseError> {
    let value = parse_cli_json(stdout)?;
    let reordered_arguments = value["reordered_arguments"]
        .as_array()
        .ok_or_else(|| TestCaseError::fail("missing reordered_arguments array".to_owned()))?
        .iter()
        .map(|arguments| {
            arguments
                .as_array()
                .ok_or_else(|| {
                    TestCaseError::fail("non-array reordered_arguments item".to_owned())
                })?
                .iter()
                .map(|argument| {
                    argument.as_str().map(ToOwned::to_owned).ok_or_else(|| {
                        TestCaseError::fail("non-string reordered_arguments entry".to_owned())
                    })
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(CliReorderFunctionParametersReport {
        all_calls: json_bool(&value, "all_calls")?,
        function_name: json_string(&value, "function_name")?,
        old_parameter_order: json_string_array(&value, "old_parameter_order")?,
        new_parameter_order: json_string_array(&value, "new_parameter_order")?,
        reordered_arguments,
        changed: json_bool(&value, "changed")?,
        written: json_bool(&value, "written")?,
        rewritten: json_string(&value, "rewritten")?,
    })
}

pub(super) fn parse_move_function_parameter_report(
    stdout: &[u8],
) -> Result<CliMoveFunctionParameterReport, TestCaseError> {
    let value = parse_cli_json(stdout)?;

    Ok(CliMoveFunctionParameterReport {
        all_calls: json_bool(&value, "all_calls")?,
        function_name: json_string(&value, "function_name")?,
        parameter_name: json_string(&value, "parameter_name")?,
        from_index: json_u64(&value, "from_index")?,
        to_index: json_u64(&value, "to_index")?,
        moved_arguments: json_string_array(&value, "moved_arguments")?,
        changed: json_bool(&value, "changed")?,
        written: json_bool(&value, "written")?,
        rewritten: json_string(&value, "rewritten")?,
    })
}

pub(super) fn assert_cli_check_succeeds(path: &std::path::Path) -> Result<(), TestCaseError> {
    let check_output = paredit()
        .arg("check")
        .arg("--file")
        .arg(path)
        .output()
        .map_err(|err| TestCaseError::fail(format!("run check: {err}")))?;
    prop_assert!(
        check_output.status.success(),
        "check stderr={}",
        String::from_utf8_lossy(&check_output.stderr)
    );
    Ok(())
}

mod add;
mod move_parameter;
mod policy;
mod remove;
mod reorder;
mod swap;
