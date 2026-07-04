use std::path::PathBuf;

use crate::application::usecase::rename::{
    RenameFunctionOccurrence, ReplaceFunctionCallSite, UnwrapFunctionCallSite, WrapFunctionCallSite,
};
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::ByteSpan;

#[derive(Debug)]
pub(super) struct RenameFileReport {
    pub(super) path: PathBuf,
    pub(super) dialect: Dialect,
    pub(super) occurrences: Vec<ByteSpan>,
    pub(super) changed: bool,
    pub(super) written: bool,
    pub(super) rewritten: String,
}

#[derive(Debug)]
pub(super) struct RenameFunctionFileReport {
    pub(super) path: PathBuf,
    pub(super) dialect: Dialect,
    pub(super) definitions: Vec<RenameFunctionOccurrence>,
    pub(super) calls: Vec<RenameFunctionOccurrence>,
    pub(super) changed: bool,
    pub(super) written: bool,
    pub(super) rewritten: String,
}

#[derive(Debug)]
pub(super) struct RenameMacroletFileReport {
    pub(super) path: PathBuf,
    pub(super) dialect: Dialect,
    pub(super) definitions: Vec<RenameFunctionOccurrence>,
    pub(super) calls: Vec<RenameFunctionOccurrence>,
    pub(super) changed: bool,
    pub(super) written: bool,
    pub(super) rewritten: String,
}

#[derive(Debug)]
pub(super) struct RenameLocalFunctionFileReport {
    pub(super) path: PathBuf,
    pub(super) dialect: Dialect,
    pub(super) definitions: Vec<RenameFunctionOccurrence>,
    pub(super) calls: Vec<RenameFunctionOccurrence>,
    pub(super) changed: bool,
    pub(super) written: bool,
    pub(super) rewritten: String,
}

#[derive(Debug)]
pub(super) struct PendingRenameMacroletFile {
    pub(super) path: PathBuf,
    pub(super) dialect: Dialect,
    pub(super) definitions: Vec<RenameFunctionOccurrence>,
    pub(super) calls: Vec<RenameFunctionOccurrence>,
    pub(super) rewritten: String,
    pub(super) changed: bool,
}

#[derive(Debug)]
pub(super) struct PendingRenameLocalFunctionFile {
    pub(super) path: PathBuf,
    pub(super) dialect: Dialect,
    pub(super) definitions: Vec<RenameFunctionOccurrence>,
    pub(super) calls: Vec<RenameFunctionOccurrence>,
    pub(super) rewritten: String,
    pub(super) changed: bool,
}

#[derive(Debug)]
pub(super) struct PendingRenameFunctionFile {
    pub(super) path: PathBuf,
    pub(super) dialect: Dialect,
    pub(super) definitions: Vec<RenameFunctionOccurrence>,
    pub(super) calls: Vec<RenameFunctionOccurrence>,
    pub(super) rewritten: String,
    pub(super) changed: bool,
}

#[derive(Debug)]
pub(super) struct WrapFunctionCallsFileReport {
    pub(super) path: PathBuf,
    pub(super) dialect: Dialect,
    pub(super) calls: Vec<WrapFunctionCallSite>,
    pub(super) skipped_already_wrapped: Vec<WrapFunctionCallSite>,
    pub(super) skipped_nested: Vec<WrapFunctionCallSite>,
    pub(super) changed: bool,
    pub(super) written: bool,
    pub(super) rewritten: String,
}

#[derive(Debug)]
pub(super) struct PendingWrapFunctionCallsFile {
    pub(super) path: PathBuf,
    pub(super) dialect: Dialect,
    pub(super) calls: Vec<WrapFunctionCallSite>,
    pub(super) skipped_already_wrapped: Vec<WrapFunctionCallSite>,
    pub(super) skipped_nested: Vec<WrapFunctionCallSite>,
    pub(super) rewritten: String,
    pub(super) changed: bool,
}

#[derive(Debug)]
pub(super) struct WrapFunctionCallsPolicy {
    pub(super) fail_on_no_change: bool,
    pub(super) require_calls: Option<usize>,
    pub(super) passed: bool,
    pub(super) violations: Vec<String>,
}

#[derive(Debug)]
pub(super) struct ReplaceFunctionCallsFileReport {
    pub(super) path: PathBuf,
    pub(super) dialect: Dialect,
    pub(super) calls: Vec<ReplaceFunctionCallSite>,
    pub(super) changed: bool,
    pub(super) written: bool,
    pub(super) rewritten: String,
}

#[derive(Debug)]
pub(super) struct PendingReplaceFunctionCallsFile {
    pub(super) path: PathBuf,
    pub(super) dialect: Dialect,
    pub(super) calls: Vec<ReplaceFunctionCallSite>,
    pub(super) rewritten: String,
    pub(super) changed: bool,
}

#[derive(Debug)]
pub(super) struct ReplaceFunctionCallsPolicy {
    pub(super) fail_on_no_change: bool,
    pub(super) require_calls: Option<usize>,
    pub(super) passed: bool,
    pub(super) violations: Vec<String>,
}

#[derive(Debug)]
pub(super) struct UnwrapFunctionCallsFileReport {
    pub(super) path: PathBuf,
    pub(super) dialect: Dialect,
    pub(super) calls: Vec<UnwrapFunctionCallSite>,
    pub(super) skipped_non_unary_wrapper: Vec<UnwrapFunctionCallSite>,
    pub(super) skipped_nested: Vec<UnwrapFunctionCallSite>,
    pub(super) changed: bool,
    pub(super) written: bool,
    pub(super) rewritten: String,
}

#[derive(Debug)]
pub(super) struct PendingUnwrapFunctionCallsFile {
    pub(super) path: PathBuf,
    pub(super) dialect: Dialect,
    pub(super) calls: Vec<UnwrapFunctionCallSite>,
    pub(super) skipped_non_unary_wrapper: Vec<UnwrapFunctionCallSite>,
    pub(super) skipped_nested: Vec<UnwrapFunctionCallSite>,
    pub(super) rewritten: String,
    pub(super) changed: bool,
}

#[derive(Debug)]
pub(super) struct UnwrapFunctionCallsPolicy {
    pub(super) fail_on_no_change: bool,
    pub(super) require_calls: Option<usize>,
    pub(super) passed: bool,
    pub(super) violations: Vec<String>,
}
