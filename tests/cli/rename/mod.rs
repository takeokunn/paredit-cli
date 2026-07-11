pub(super) use super::*;

use proptest::test_runner::TestCaseError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CliWrittenFileReport {
    pub(super) written: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CliDefinitionCallReport {
    pub(super) definition_count: u64,
    pub(super) call_count: u64,
    pub(super) files: Vec<CliWrittenFileReport>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CliDefinitionReferenceReport {
    pub(super) definition_count: u64,
    pub(super) reference_count: u64,
    pub(super) files: Vec<CliWrittenFileReport>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CliBindingWriteReport {
    pub(super) changed: bool,
    pub(super) form: String,
    pub(super) path: String,
    pub(super) reference_count: u64,
    pub(super) shadowed_scope_count: u64,
    pub(super) written: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CliUnwrapReport {
    pub(super) call_count: u64,
    pub(super) skipped_non_unary_wrapper_count: u64,
    pub(super) skipped_nested_count: u64,
    pub(super) files: Vec<CliWrittenFileReport>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CliWrapReport {
    pub(super) call_count: u64,
    pub(super) skipped_already_wrapped_count: u64,
    pub(super) skipped_nested_count: u64,
    pub(super) wrapper_template: Option<String>,
    pub(super) files: Vec<CliWrittenFileReport>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CliReplaceCallReport {
    pub(super) call_count: u64,
    pub(super) files: Vec<CliWrittenFileReport>,
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

fn json_written_files(
    value: &serde_json::Value,
) -> Result<Vec<CliWrittenFileReport>, TestCaseError> {
    let files = value["files"]
        .as_array()
        .ok_or_else(|| TestCaseError::fail("missing files array".to_owned()))?;

    files
        .iter()
        .map(|file| {
            Ok(CliWrittenFileReport {
                written: json_bool(file, "written")?,
            })
        })
        .collect()
}

pub(super) fn parse_definition_call_report(
    stdout: &[u8],
) -> Result<CliDefinitionCallReport, TestCaseError> {
    let value = parse_cli_json(stdout)?;
    Ok(CliDefinitionCallReport {
        definition_count: json_u64(&value, "definitionCount")?,
        call_count: json_u64(&value, "callCount")?,
        files: json_written_files(&value)?,
    })
}

pub(super) fn parse_definition_reference_report(
    stdout: &[u8],
) -> Result<CliDefinitionReferenceReport, TestCaseError> {
    let value = parse_cli_json(stdout)?;
    Ok(CliDefinitionReferenceReport {
        definition_count: json_u64(&value, "definitionCount")?,
        reference_count: json_u64(&value, "referenceCount")?,
        files: json_written_files(&value)?,
    })
}

pub(super) fn parse_binding_write_report(
    stdout: &[u8],
) -> Result<CliBindingWriteReport, TestCaseError> {
    let value = parse_cli_json(stdout)?;
    Ok(CliBindingWriteReport {
        changed: json_bool(&value, "changed")?,
        form: json_string(&value, "form")?,
        path: json_string(&value, "path")?,
        reference_count: json_u64(&value, "reference_count")?,
        shadowed_scope_count: json_u64(&value, "shadowed_scope_count")?,
        written: json_bool(&value, "written")?,
    })
}

pub(super) fn parse_unwrap_report(stdout: &[u8]) -> Result<CliUnwrapReport, TestCaseError> {
    let value = parse_cli_json(stdout)?;
    Ok(CliUnwrapReport {
        call_count: json_u64(&value, "callCount")?,
        skipped_non_unary_wrapper_count: json_u64(&value, "skippedNonUnaryWrapperCount")?,
        skipped_nested_count: json_u64(&value, "skippedNestedCount")?,
        files: json_written_files(&value)?,
    })
}

pub(super) fn parse_wrap_report(stdout: &[u8]) -> Result<CliWrapReport, TestCaseError> {
    let value = parse_cli_json(stdout)?;
    Ok(CliWrapReport {
        call_count: json_u64(&value, "callCount")?,
        skipped_already_wrapped_count: json_u64(&value, "skippedAlreadyWrappedCount")?,
        skipped_nested_count: json_u64(&value, "skippedNestedCount")?,
        wrapper_template: value["wrapperTemplate"].as_str().map(ToOwned::to_owned),
        files: json_written_files(&value)?,
    })
}

pub(super) fn parse_replace_call_report(
    stdout: &[u8],
) -> Result<CliReplaceCallReport, TestCaseError> {
    let value = parse_cli_json(stdout)?;
    Ok(CliReplaceCallReport {
        call_count: json_u64(&value, "callCount")?,
        files: json_written_files(&value)?,
    })
}

pub(super) fn assert_cli_check_succeeds(path: &std::path::Path) -> Result<(), TestCaseError> {
    let check_output = paredit()
        .arg("inspect")
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

mod binding;
mod function;
mod local_function;
mod macrolet;
mod replace_call;
mod scoped_form;
mod symbol;
mod symbol_macro;
mod unwrap;
mod wrap;
