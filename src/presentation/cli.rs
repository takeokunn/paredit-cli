mod analysis_report;
mod args;
mod call_graph_report;
mod call_report;
mod definition_movement;
mod definition_removal;
mod definition_report;
mod dependency_report;
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

use crate::application::definition_report::{
    DefinitionReportFile, UnusedDefinitionFile, UnusedDefinitionPolicy,
    UnusedDefinitionPolicyOptions, build_definition_report, build_parsed_definition_file,
    collect_definition_forms, collect_unused_definition_candidates,
    evaluate_unused_definition_policy,
};
use crate::application::impact_report::{
    ImpactReportFile, ImpactRiskLevel as ApplicationImpactRiskLevel, raw_refactor_risks,
    summarize_impact_reports,
};
use crate::application::refactor::execute::{
    RefactorWriteCandidate, RefactorWriteRefusal, build_refactor_write_plan,
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
    WorkspaceReport(workspace_report::WorkspaceReportArgs),
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
    FindSymbol(symbol_report::SymbolQueryArgs),
    /// Report exact atom occurrences across explicit files for rename planning.
    SymbolReport(symbol_report::SymbolReportArgs),
    /// Report list-head call sites across explicit files for arity refactor planning.
    CallReport(call_report::CallReportArgs),
    /// Compare callable definitions and call-site arity across explicit files.
    SignatureReport(signature_report::SignatureReportArgs),
    /// Report internal and optional external call graph edges across explicit files.
    CallGraph(call_graph_report::CallGraphArgs),
    /// Report refactoring impact risks for one symbol across explicit files.
    ImpactReport(impact_report::ImpactReportArgs),
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
    DefinitionReport(definition_report::DefinitionReportArgs),
    /// Report definition-like top-level forms with no external exact atom references.
    UnusedDefinitionReport(definition_report::UnusedDefinitionReportArgs),
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
    DuplicateReport(duplicate_report::DuplicateReportArgs),
    /// Convert duplicate groups into reviewed replace-forms batches.
    ReplacementPlan(duplicate_report::ReplacementPlanArgs),
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
    match cli.command {
        Command::Check(args) => analysis_report::check(args)?,
        Command::Dialect(args) => analysis_report::dialect(args)?,
        Command::Stats(args) => analysis_report::stats(args)?,
        Command::AgentReport(args) => analysis_report::agent_report(args)?,
        Command::WorkspaceReport(args) => workspace_report::workspace_report(args)?,
        Command::WorkspaceRefactorPlan(args) => refactor::workflow::workspace_refactor_plan(args)?,
        Command::WorkspaceRefactorPreview(args) => {
            refactor::workflow::workspace_refactor_preview(args)?
        }
        Command::WorkspaceRefactorExecute(args) => {
            refactor::workflow::workspace_refactor_execute(args)?
        }
        Command::Outline(args) => analysis_report::outline(args)?,
        Command::FormReport(args) => form_report::form_report(args)?,
        Command::FindSymbol(args) => symbol_report::find_symbol(args)?,
        Command::SymbolReport(args) => symbol_report::symbol_report(args)?,
        Command::CallReport(args) => call_report::call_report(args)?,
        Command::SignatureReport(args) => signature_report::signature_report(args)?,
        Command::CallGraph(args) => call_graph_report::call_graph(args)?,
        Command::ImpactReport(args) => impact_report::impact_report(args)?,
        Command::RefactorPlan(args) => refactor::workflow::refactor_plan(args)?,
        Command::VerifyRefactor(args) => refactor::workflow::verify_refactor(args)?,
        Command::RefactorPreview(args) => refactor::workflow::refactor_preview(args)?,
        Command::RefactorCheck(args) => refactor::workflow::refactor_check(args)?,
        Command::RefactorStatus(args) => refactor::workflow::refactor_status(args)?,
        Command::RefactorApply(args) => refactor::workflow::refactor_apply(args)?,
        Command::RefactorDiff(args) => refactor::workflow::refactor_diff(args)?,
        Command::DependencyReport(args) => dependency_report::dependency_report(args)?,
        Command::PackageReport(args) => package::package_report(args)?,
        Command::DefinitionReport(args) => definition_report::definition_report(args)?,
        Command::UnusedDefinitionReport(args) => definition_report::unused_definition_report(args)?,
        Command::RemoveDefinition(args) => {
            definition_removal::remove_definition::remove_definition(args)?
        }
        Command::RemoveUnusedDefinitions(args) => {
            definition_removal::remove_unused_definitions::remove_unused_definitions(args)?
        }
        Command::MoveDefinition(args) => {
            definition_movement::move_definition::move_definition(args)?
        }
        Command::SplitFile(args) => definition_movement::split_file::split_file(args)?,
        Command::SortDefinitions(args) => {
            definition_movement::sort_definitions::sort_definitions(args)?
        }
        Command::MoveForm(args) => definition_movement::move_form::move_form(args)?,
        Command::DuplicateReport(args) => duplicate_report::duplicate_report(args)?,
        Command::ReplacementPlan(args) => duplicate_report::replacement_plan(args)?,
        Command::ReplaceForms(args) => replace_forms::replace_forms(args)?,
        Command::AddExport(args) => package::add_export(args)?,
        Command::SortPackageExports(args) => package::sort_package_exports(args)?,
        Command::SortPackageOptions(args) => package::sort_package_options(args)?,
        Command::MergePackageOptions(args) => package::merge_package_options(args)?,
        Command::RenamePackage(args) => package::rename_package(args)?,
        Command::RenameSymbol(args) => rename::rename_symbol::rename_symbol(args)?,
        Command::RenameInForm(args) => rename::rename_in_form::rename_in_form(args)?,
        Command::RenameBinding(args) => rename::rename_binding::rename_binding(args)?,
        Command::RenameSymbols(args) => rename::rename_symbols::rename_symbols(args)?,
        Command::RenameFunction(args) => rename::rename_function::rename_function(args)?,
        Command::WrapFunctionCalls(args) => rename::wrap_function_calls::wrap_function_calls(args)?,
        Command::UnwrapCall(args) => unwrap_call::unwrap_call(args)?,
        Command::ThreadExpression(args) => thread_expression::thread_expression(args)?,
        Command::UnthreadExpression(args) => unthread_expression::unthread_expression(args)?,
        Command::ExtractFunction(args) => extract_function::extract_function(args)?,
        Command::InlineFunction(args) => inline_function::inline_function(args)?,
        Command::AddFunctionParameter(args) => {
            function_parameter::add::add_function_parameter(args)?
        }
        Command::MoveFunctionParameter(args) => {
            function_parameter::move_parameter::move_function_parameter(args)?
        }
        Command::RemoveFunctionParameter(args) => {
            function_parameter::remove::remove_function_parameter(args)?
        }
        Command::IntroduceLet(args) => introduce_let::introduce_let(args)?,
        Command::InlineLet(args) => inline_let::inline_let(args)?,
        Command::RemoveUnusedBinding(args) => remove_unused_binding::remove_unused_binding(args)?,
        Command::LetReport(args) => let_report::let_report(args)?,
        Command::Format(args) => {
            let input = read_input(args.file)?;
            let _dialect = detect_dialect(&input, args.dialect);
            let tree = SyntaxTree::parse(&input.text)?;
            print!("{}", Formatter::new(args.indent).format(&tree));
        }
        Command::Select(args) => {
            let input = read_input(args.file)?;
            let tree = SyntaxTree::parse(&input.text)?;
            let selection = resolve_target(&tree, args.path.as_ref(), args.at)?;
            print!("{}", selection.text(&input.text));
        }
        Command::Replace(args) => {
            let input = read_input(args.file)?;
            SyntaxTree::parse(&args.with)
                .context("replacement is not a valid S-expression document")?;
            let tree = SyntaxTree::parse(&input.text)?;
            let selection = resolve_target(&tree, args.path.as_ref(), args.at)?;
            print!("{}", Edit::replace(&input.text, selection, &args.with));
        }
        Command::Kill(args) => edit_target(args, Edit::kill)?,
        Command::Wrap(args) => edit_target(args, Edit::wrap)?,
        Command::Splice(args) => edit_target(args, Edit::splice)?,
        Command::Raise(args) => edit_target(args, Edit::raise)?,
        Command::SlurpForward(args) => edit_target(args, Edit::slurp_forward)?,
        Command::SlurpBackward(args) => edit_target(args, Edit::slurp_backward)?,
        Command::BarfForward(args) => edit_target(args, Edit::barf_forward)?,
        Command::BarfBackward(args) => edit_target(args, Edit::barf_backward)?,
    }
    Ok(())
}
