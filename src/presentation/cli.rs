mod analysis_report;
mod args;
mod basic_edit;
mod call_graph_report;
mod call_report;
mod capabilities;
mod command;
mod definition_movement;
mod definition_removal;
mod definition_report;
mod dependency_report;
mod dispatch;
mod duplicate_report;
mod extract_constant;
mod extract_function;
mod form_report;
mod function_parameter;
mod gate;
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
mod similarity_report;
mod symbol_report;
mod thread_expression;
mod unthread_expression;
mod unwrap_call;
mod workspace_report;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path as FsPath, PathBuf};

use crate::application::refactor::execute::{
    RefactorExecuteGateInputs, RefactorWriteRefusal, build_refactor_execute_decision,
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
use crate::application::usecase::impact_report::{
    ImpactReportFile, ImpactRiskLevel as ApplicationImpactRiskLevel, raw_refactor_risks,
    summarize_impact_reports,
};
use crate::domain::definition::DefinitionCategory;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{ByteOffset, ByteSpan, Path, SymbolName, SyntaxTree};
use crate::infrastructure::workspace::{WorkspaceDiscoveryOptions, discover_workspace_files};
use anyhow::{Context, Result};
use clap::{Args, Parser, ValueEnum};
use serde_json::{Value, json};

use args::*;
use command::Command;
pub(crate) use shared::{
    apply_byte_span_edits, bounded_preview, matching_symbol_occurrences, read_input_and_dialect,
    read_input_dialect_and_tree, require_output_file, resolve_target, stable_text_hash,
    unified_diff, write_file_with_rollback, write_files_with_rollback,
};

#[derive(Debug, Parser)]
#[command(
    name = "paredit",
    version,
    about,
    long_about = None,
    after_help = "Canonical namespaces:\n  `paredit inspect ...` reads and reports without writing.\n  `paredit edit ...` transforms one selected form; stdout by default, --write to update the file.\n  `paredit refactor ...` plans, previews, verifies, and applies semantic changes.\n\nAll source-facing commands live in these three namespaces.\n`paredit completions <shell>` prints a shell completion script.\nRun `paredit inspect capabilities --output json` for a machine-readable catalog of every command and flag."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    match dispatch::dispatch(cli.command) {
        Err(error) if error.downcast_ref::<gate::GateFailure>().is_some() => {
            eprintln!("Error: {error:#}");
            std::process::exit(gate::GATE_FAILURE_EXIT_CODE);
        }
        result => result,
    }
}
