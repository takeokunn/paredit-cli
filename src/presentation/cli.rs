mod analysis_report;
mod args;
mod call_graph_report;
mod call_report;
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
use clap::{Args, Parser, Subcommand, ValueEnum};
use serde_json::{Value, json};

use args::*;
use shared::*;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Validate that input is a balanced S-expression document.
    Check(InputArgs),
    /// Detect Lisp dialect from --file extension or explicit --dialect.
    Dialect(AnalyzeArgs),
    /// Print parse, dialect, and structural metrics for agent planning.
    Stats(AnalyzeArgs),
    /// Print a complete JSON report for AI coding agent refactor planning.
    AgentReport(AnalyzeArgs),
    /// Discover Lisp sources under roots and report parse/refactor inventory.
    WorkspaceReport(workspace_report::args::WorkspaceReportArgs),
    /// Discover Lisp sources under roots and build a gated refactor plan.
    WorkspaceRefactorPlan(refactor::args::WorkspaceRefactorPlanArgs),
    /// Discover Lisp sources under roots and preview exact refactoring rewrites.
    WorkspaceRefactorPreview(refactor::args::WorkspaceRefactorPreviewArgs),
    /// Execute a workspace refactor with preview gates and post-write verification.
    WorkspaceRefactorExecute(refactor::args::WorkspaceRefactorExecuteArgs),
    /// Print top-level forms with paths, spans, and definition hints.
    Outline(AnalyzeArgs),
    /// Report one selected form with local structure for agent refactor planning.
    FormReport(FormReportArgs),
    /// Find exact atom occurrences without touching strings or comments.
    FindSymbol(symbol_report::args::SymbolQueryArgs),
    /// Report exact atom occurrences across explicit files for rename planning.
    SymbolReport(symbol_report::args::SymbolReportArgs),
    /// Report list-head call sites across explicit files for arity refactor planning.
    CallReport(call_report::CallReportArgs),
    /// Compare callable definitions and call-site arity across explicit files.
    SignatureReport(signature_report::SignatureReportArgs),
    /// Report internal and optional external call graph edges across explicit files.
    CallGraph(call_graph_report::args::CallGraphArgs),
    /// Report refactoring impact risks for one symbol across explicit files.
    ImpactReport(impact_report::args::ImpactReportArgs),
    /// Produce an ordered, gated refactoring plan for AI coding agents.
    RefactorPlan(refactor::args::RefactorPlanArgs),
    /// Verify pre/post refactoring invariants for AI coding agents and CI gates.
    VerifyRefactor(refactor::args::VerifyRefactorArgs),
    /// Preview exact refactoring rewrites without modifying files.
    RefactorPreview(refactor::args::RefactorPreviewArgs),
    /// Validate a refactor preview manifest without writing files or rendering diffs.
    RefactorCheck(refactor::args::RefactorCheckArgs),
    /// Summarize a refactor preview manifest into agent-safe next actions.
    RefactorStatus(refactor::args::RefactorStatusArgs),
    /// Apply a previously generated refactor preview manifest with hash guards.
    RefactorApply(refactor::args::RefactorApplyArgs),
    /// Render a verified diff from a refactor preview manifest without writing files.
    RefactorDiff(refactor::args::RefactorDiffArgs),
    /// Report package, system, load, and qualified-symbol dependencies across explicit files.
    DependencyReport(dependency_report::DependencyReportArgs),
    /// Report Common Lisp package declarations across explicit files.
    PackageReport(package::PackageReportArgs),
    /// Report definition-like top-level forms across explicit files.
    DefinitionReport(definition_report::args::DefinitionReportArgs),
    /// Report definition-like top-level forms with no external exact atom references.
    UnusedDefinitionReport(definition_report::args::UnusedDefinitionReportArgs),
    /// Plan or remove a top-level definition from one file.
    RemoveDefinition(definition_removal::args::RemoveDefinitionArgs),
    /// Plan or remove unused top-level definitions across explicit files.
    RemoveUnusedDefinitions(definition_removal::args::RemoveUnusedDefinitionsArgs),
    /// Plan or move a top-level definition between files.
    MoveDefinition(definition_movement::args::MoveDefinitionArgs),
    /// Plan or split multiple top-level definitions into another file.
    SplitFile(definition_movement::args::SplitFileArgs),
    /// Plan or sort contiguous top-level definition blocks inside one file.
    SortDefinitions(definition_movement::args::SortDefinitionsArgs),
    /// Plan or move any top-level form between files.
    MoveForm(definition_movement::args::MoveFormArgs),
    /// Report repeated structural S-expression shapes across explicit files.
    DuplicateReport(duplicate_report::args::DuplicateReportArgs),
    /// Convert duplicate groups into reviewed replace-forms batches.
    ReplacementPlan(duplicate_report::args::ReplacementPlanArgs),
    /// Plan or replace multiple reviewed forms in one file.
    ReplaceForms(replace_forms::ReplaceFormsArgs),
    /// Plan or add a symbol to a Common Lisp defpackage :export option.
    AddExport(package::AddExportArgs),
    /// Plan or sort Common Lisp defpackage :export symbol designators.
    SortPackageExports(package::SortPackageExportsArgs),
    /// Plan or sort Common Lisp defpackage option forms.
    SortPackageOptions(package::SortPackageOptionsArgs),
    /// Plan or merge duplicate Common Lisp defpackage option forms.
    MergePackageOptions(package::MergePackageOptionsArgs),
    /// Plan or rename Common Lisp package designators and qualified prefixes.
    RenamePackage(package::RenamePackageArgs),
    /// Rename exact atom occurrences without touching strings or comments.
    RenameSymbol(rename::args::RenameSymbolArgs),
    /// Rename exact atom occurrences inside one selected form.
    RenameInForm(rename::args::RenameInFormArgs),
    /// Rename one local binding and only the references in its lexical scope.
    RenameBinding(rename::args::RenameBindingArgs),
    /// Plan or apply an exact atom rename across explicit files.
    RenameSymbols(rename::args::RenameSymbolsArgs),
    /// Plan or apply a callable definition and call-site rename across explicit files.
    RenameFunction(rename::args::RenameFunctionArgs),
    /// Plan or wrap callable call sites in another function or macro call.
    WrapFunctionCalls(rename::args::WrapFunctionCallsArgs),
    /// Replace one selected wrapper call with one selected argument.
    UnwrapCall(unwrap_call::UnwrapCallArgs),
    /// Convert a selected nested call chain into a thread-first or thread-last pipeline.
    ThreadExpression(thread_expression::ThreadExpressionArgs),
    /// Convert a selected thread-first or thread-last pipeline into nested calls.
    UnthreadExpression(unthread_expression::UnthreadExpressionArgs),
    /// Extract the selected expression into a zero-argument top-level function.
    ExtractFunction(extract_function::ExtractFunctionArgs),
    /// Inline one selected function call using a selected function definition.
    InlineFunction(inline_function::InlineFunctionArgs),
    /// Add a required parameter to a selected function and explicit call sites.
    AddFunctionParameter(function_parameter::args::AddFunctionParameterArgs),
    /// Move one required parameter in a selected function and explicit call sites.
    MoveFunctionParameter(function_parameter::args::MoveFunctionParameterArgs),
    /// Remove one required parameter from a selected function and explicit call sites.
    RemoveFunctionParameter(function_parameter::args::RemoveFunctionParameterArgs),
    /// Replace the selected expression with a local binding in the enclosing list.
    IntroduceLet(introduce_let::IntroduceLetArgs),
    /// Inline a single local let binding into its body.
    InlineLet(inline_let::InlineLetArgs),
    /// Plan or remove one unused local let binding.
    RemoveUnusedBinding(remove_unused_binding::RemoveUnusedBindingArgs),
    /// Report local let bindings and inline safety for agent refactor planning.
    LetReport(let_report::LetReportArgs),
    /// Print a canonical, indentation-based rendering.
    Format(FormatArgs),
    /// Print the S-expression selected by --path or --at.
    Select(TargetArgs),
    /// Replace the selected S-expression with replacement text.
    Replace(ReplaceArgs),
    /// Remove the selected S-expression.
    Kill(TargetArgs),
    /// Wrap the selected S-expression in a new list.
    Wrap(TargetArgs),
    /// Remove one list pair while keeping its children.
    Splice(TargetArgs),
    /// Replace the selected expression's parent list with the selected expression.
    Raise(TargetArgs),
    /// Pull the next sibling into the selected list.
    SlurpForward(TargetArgs),
    /// Pull the previous sibling into the selected list.
    SlurpBackward(TargetArgs),
    /// Push the last child out of the selected list.
    BarfForward(TargetArgs),
    /// Push the first child out of the selected list.
    BarfBackward(TargetArgs),
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    dispatch::dispatch(cli.command)
}
