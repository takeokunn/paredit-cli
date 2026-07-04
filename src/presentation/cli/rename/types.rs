use std::path::PathBuf;

use crate::application::usecase::rename::{RenameFunctionOccurrence, WrapFunctionCallSite};
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
