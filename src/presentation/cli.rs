mod analysis_report;
mod args;
mod call_graph_report;
mod call_report;
mod command;
mod definition_movement;
mod definition_removal;
mod definition_report;
mod dependency_report;
mod dispatch;
mod duplicate_report;
mod extract_function;
mod form_report;
mod function_parameter;
mod impact_report;
mod inline_function;
mod inline_let;
mod introduce_let;
mod let_report;
mod package;
mod refactor;
mod remove_unused_binding;
mod rename;
mod replace_forms;
mod shared;
mod signature_report;
mod symbol_report;
mod thread_expression;
mod unthread_expression;
mod unwrap_call;
mod workspace_report;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path as FsPath, PathBuf};

use crate::application::definition_report::collect_definition_forms;
use crate::application::impact_report::{
    ImpactReportFile, ImpactRiskLevel as ApplicationImpactRiskLevel, raw_refactor_risks,
    summarize_impact_reports,
};
use crate::application::refactor::execute::{
    RefactorExecuteGateInputs, RefactorWriteCandidate, RefactorWriteRefusal,
    build_refactor_execute_decision, build_refactor_write_plan,
};
use crate::application::refactor::plan::{
    RefactorOperation as ApplicationRefactorOperation, RefactorPlanGate, RefactorPlanPolicy,
    RefactorPlanPolicyRequest, RefactorPlanRequest, RefactorPlanStep, RefactorPlanSummary,
    RefactorVerificationCheck, RefactorVerificationRequest,
    VerificationPhase as ApplicationVerificationPhase, build_refactor_plan_decision,
    refactor_plan_gates as application_refactor_plan_gates,
    refactor_verification_checks as application_refactor_verification_checks,
};
use crate::application::refactor::preview::{
    RefactorPreviewEdit, RefactorPreviewPolicy, RefactorPreviewPolicyOptions,
    RefactorPreviewSummary, evaluate_refactor_preview_policy, refactor_preview_edits,
};
use crate::domain::definition::DefinitionCategory;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{
    ByteOffset, ByteSpan, Delimiter, Edit, Formatter, Path, SymbolName, SyntaxTree,
};
use crate::infrastructure::workspace::{WorkspaceDiscoveryOptions, discover_workspace_files};
use anyhow::{Context, Result};
use clap::{Args, Parser, ValueEnum};
use serde_json::{Value, json};

use args::*;
use command::Command;
use shared::*;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    dispatch::dispatch(cli.command)
}
