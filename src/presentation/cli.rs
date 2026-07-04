mod analysis_report;
mod args;
mod basic_edit;
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

use crate::application::refactor::execute::{
    build_refactor_execute_decision, RefactorExecuteGateInputs, RefactorWriteRefusal,
};
use crate::application::refactor::plan::{
    build_refactor_plan_decision, refactor_plan_gates as application_refactor_plan_gates,
    refactor_verification_checks as application_refactor_verification_checks,
    RefactorOperation as ApplicationRefactorOperation, RefactorPlanGate, RefactorPlanPolicy,
    RefactorPlanPolicyRequest, RefactorPlanRequest, RefactorPlanStep, RefactorPlanSummary,
    RefactorVerificationCheck, RefactorVerificationRequest,
    VerificationPhase as ApplicationVerificationPhase,
};
use crate::application::refactor::preview::{
    evaluate_refactor_preview_policy, refactor_preview_edits, RefactorPreviewEdit,
    RefactorPreviewPolicy, RefactorPreviewPolicyOptions, RefactorPreviewSummary,
};
use crate::application::usecase::impact_report::{
    raw_refactor_risks, summarize_impact_reports, ImpactReportFile,
    ImpactRiskLevel as ApplicationImpactRiskLevel,
};
use crate::domain::definition::DefinitionCategory;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{ByteOffset, ByteSpan, Path, SymbolName, SyntaxTree};
use crate::infrastructure::workspace::{discover_workspace_files, WorkspaceDiscoveryOptions};
use anyhow::{Context, Result};
use clap::{Args, Parser, ValueEnum};
use serde_json::{json, Value};

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
