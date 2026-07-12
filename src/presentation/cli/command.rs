use super::{
    args::{AnalyzeArgs, FormatArgs, InputArgs, ReplaceArgs, TargetArgs},
    call_graph_report, call_report, convert_let_star_to_let, definition_movement,
    definition_removal, definition_report, dependency_report, duplicate_report, extract_constant,
    extract_function, extract_local_function, form_report, function_parameter, impact_report,
    inline_function, inline_lambda, inline_let, inline_local_function, introduce_let, let_report,
    package, refactor, remove_unused_binding, rename, replace_forms, signature_report,
    similarity_report, symbol_report, thread_expression, unthread_expression, unwrap_call,
    workspace_report,
};
use clap::Subcommand;

/// Read-only inventory and analysis commands.
#[derive(Debug, Subcommand)]
pub(super) enum InspectCommand {
    /// Validate that input is a balanced S-expression document.
    Check(InputArgs),
    /// Detect Lisp dialect from --file extension or explicit --dialect.
    Dialect(AnalyzeArgs),
    /// Print parse, dialect, and structural metrics for agent planning.
    Stats(AnalyzeArgs),
    /// Print a complete JSON report for AI coding agent refactor planning.
    AgentReport(AnalyzeArgs),
    /// Print top-level forms with paths, spans, and definition hints.
    Outline(AnalyzeArgs),
    /// Report one selected form with local structure for agent refactor planning.
    Form(form_report::args::FormReportArgs),
    /// Find exact atom occurrences without touching strings or comments.
    FindSymbol(symbol_report::args::SymbolQueryArgs),
    /// Report exact atom occurrences across explicit files for rename planning.
    Symbols(symbol_report::args::SymbolReportArgs),
    /// Report list-head call sites across explicit files for arity refactor planning.
    Calls(call_report::args::CallReportArgs),
    /// Compare callable definitions and call-site arity across explicit files.
    Signature(signature_report::args::SignatureReportArgs),
    /// Report internal and optional external call graph edges across explicit files.
    CallGraph(call_graph_report::args::CallGraphArgs),
    /// Report refactoring impact risks for one symbol across explicit files.
    Impact(impact_report::args::ImpactReportArgs),
    /// Discover Lisp sources under roots and report parse/refactor inventory.
    Workspace(workspace_report::args::WorkspaceReportArgs),
    /// Report package, system, load, and qualified-symbol dependencies across explicit files.
    Dependencies(dependency_report::args::DependencyReportArgs),
    /// Report Common Lisp package declarations across explicit files.
    Packages(package::types::PackageReportArgs),
    /// Report definition-like top-level forms across explicit files.
    Definitions(definition_report::args::DefinitionReportArgs),
    /// Report definition-like top-level forms with no external exact atom references.
    UnusedDefinitions(definition_report::args::UnusedDefinitionReportArgs),
    /// Report repeated structural S-expression shapes across explicit files.
    Duplicates(duplicate_report::args::DuplicateReportArgs),
    /// Report structurally similar S-expression forms across explicit files.
    Similarity(similarity_report::args::SimilarityReportArgs),
    /// Report local let bindings and inline safety for agent refactor planning.
    Lets(let_report::LetReportArgs),
}

/// Single-document structural editing commands. These print rewritten source to stdout.
#[derive(Debug, Subcommand)]
pub(super) enum EditCommand {
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
    /// Exchange the selected expression with its next sibling.
    TransposeForward(TargetArgs),
    /// Exchange the selected expression with its previous sibling.
    TransposeBackward(TargetArgs),
    /// Pull the next sibling into the selected list.
    SlurpForward(TargetArgs),
    /// Pull the previous sibling into the selected list.
    SlurpBackward(TargetArgs),
    /// Push the last child out of the selected list.
    BarfForward(TargetArgs),
    /// Push the first child out of the selected list.
    BarfBackward(TargetArgs),
}

#[derive(Debug, Subcommand)]
#[command(
    after_help = "Examples:\n  paredit refactor plan --symbol old-name src/foo.lisp src/bar.lisp\n  paredit refactor preview --from old-name --to new-name src/foo.lisp src/bar.lisp\n  paredit refactor verify --symbol old-name --new-symbol new-name --phase post src/foo.lisp src/bar.lisp"
)]
pub(super) enum RefactorCommand {
    /// Produce an ordered, gated refactoring plan for AI coding agents.
    Plan(refactor::args::RefactorPlanArgs),
    /// Verify pre/post refactoring invariants for AI coding agents and CI gates.
    Verify(refactor::args::VerifyRefactorArgs),
    /// Preview exact refactoring rewrites without modifying files.
    Preview(refactor::args::RefactorPreviewArgs),
    /// Validate a refactor preview manifest without writing files or rendering diffs.
    Check(refactor::args::RefactorCheckArgs),
    /// Summarize a refactor preview manifest into agent-safe next actions.
    Status(refactor::args::RefactorStatusArgs),
    /// Apply a previously generated refactor preview manifest with hash guards.
    Apply(refactor::args::RefactorApplyArgs),
    /// Render a verified diff from a refactor preview manifest without writing files.
    Diff(refactor::args::RefactorDiffArgs),
    /// Discover Lisp sources under roots and build a gated refactor plan.
    WorkspacePlan(refactor::args::WorkspaceRefactorPlanArgs),
    /// Discover Lisp sources under roots and preview exact refactoring rewrites.
    WorkspacePreview(refactor::args::WorkspaceRefactorPreviewArgs),
    /// Execute a workspace refactor with preview gates and post-write verification.
    WorkspaceExecute(refactor::args::WorkspaceRefactorExecuteArgs),
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
    /// Convert duplicate groups into reviewed replace-forms batches.
    ReplacementPlan(duplicate_report::args::ReplacementPlanArgs),
    /// Plan or replace multiple reviewed forms in one file.
    ReplaceForms(replace_forms::ReplaceFormsArgs),
    /// Plan or add a symbol to a Common Lisp defpackage :export option.
    AddExport(package::types::AddExportArgs),
    /// Plan or sort Common Lisp defpackage :export symbol designators.
    SortPackageExports(package::types::SortPackageExportsArgs),
    /// Plan or sort Common Lisp defpackage option forms.
    SortPackageOptions(package::types::SortPackageOptionsArgs),
    /// Plan or merge duplicate Common Lisp defpackage option forms.
    MergePackageOptions(package::types::MergePackageOptionsArgs),
    /// Plan or rename Common Lisp package designators and qualified prefixes.
    RenamePackage(package::types::RenamePackageArgs),
    /// Rename whatever symbol occupies a byte offset, dispatching to
    /// whichever namespace and lexical scope actually own it.
    RenameAt(rename::args::RenameAtArgs),
    /// Rename exact atom occurrences without touching strings or comments.
    RenameSymbol(rename::args::RenameSymbolArgs),
    /// Rename exact atom occurrences inside one selected form.
    RenameInForm(rename::args::RenameInFormArgs),
    /// Rename one local binding and only the references in its lexical scope.
    RenameBinding(rename::args::RenameBindingArgs),
    /// Plan or apply an exact atom rename across explicit files.
    RenameSymbols(rename::args::RenameSymbolsArgs),
    /// Plan or apply a Common Lisp callable definition and callable-designator rename across explicit files, including function, macro-function, compiler-macro-function, symbol-function, fdefinition, setf names, and definition forms such as define-method-combination.
    RenameFunction(rename::args::RenameFunctionArgs),
    /// Plan or apply a Common Lisp macrolet/compiler-macrolet binding and call-site rename across explicit files while keeping expander bodies out of scope.
    RenameMacrolet(rename::args::RenameMacroletArgs),
    /// Plan or apply a Common Lisp define-symbol-macro binding and value-reference rename across explicit files while keeping expansion and lexical shadowing boundaries separate.
    RenameSymbolMacro(rename::args::RenameSymbolMacroArgs),
    /// Plan or apply a Common Lisp flet/labels local function binding and call-site rename across explicit files, preserving the difference between non-recursive flet bodies and recursive labels bodies.
    RenameLocalFunction(rename::args::RenameLocalFunctionArgs),
    /// Plan or replace callable call-site heads across explicit files.
    ReplaceFunctionCalls(rename::args::ReplaceFunctionCallsArgs),
    /// Plan or wrap callable call sites in another function or macro call.
    WrapFunctionCalls(rename::args::WrapFunctionCallsArgs),
    /// Plan or remove a unary wrapper around callable call sites.
    UnwrapFunctionCalls(rename::args::UnwrapFunctionCallsArgs),
    /// Replace one selected wrapper call with one selected argument.
    UnwrapCall(unwrap_call::UnwrapCallArgs),
    /// Convert a selected nested call chain into a thread-first or thread-last pipeline.
    ThreadExpression(thread_expression::ThreadExpressionArgs),
    /// Convert a selected thread-first or thread-last pipeline into nested calls.
    UnthreadExpression(unthread_expression::UnthreadExpressionArgs),
    /// Extract the selected expression into a top-level function with inferred parameters.
    ExtractFunction(extract_function::ExtractFunctionArgs),
    /// Extract the selected expression into a local flet or labels function.
    ExtractLocalFunction(extract_local_function::ExtractLocalFunctionArgs),
    /// Extract the selected expression into a top-level constant.
    ExtractConstant(extract_constant::ExtractConstantArgs),
    /// Inline one selected function call using a selected function definition.
    InlineFunction(inline_function::InlineFunctionArgs),
    /// Replace an immediately invoked Common Lisp lambda with a parallel let.
    InlineLambda(inline_lambda::InlineLambdaArgs),
    /// Inline the sole direct call in a single-binding Common Lisp flet form.
    InlineLocalFunction(inline_local_function::InlineLocalFunctionArgs),
    /// Add a parameter to a selected function and explicit call sites.
    AddFunctionParameter(function_parameter::args::AddFunctionParameterArgs),
    /// Move one positional parameter in a selected function and explicit call sites.
    MoveFunctionParameter(function_parameter::args::MoveFunctionParameterArgs),
    /// Swap two positional parameters in a selected function and explicit call sites.
    SwapFunctionParameters(function_parameter::args::SwapFunctionParametersArgs),
    /// Reorder all positional parameters in a selected function and explicit call sites.
    ReorderFunctionParameters(function_parameter::args::ReorderFunctionParametersArgs),
    /// Remove one positional parameter from a selected function and explicit call sites.
    RemoveFunctionParameter(function_parameter::args::RemoveFunctionParameterArgs),
    /// Replace the selected expression with a local binding in the enclosing list.
    IntroduceLet(introduce_let::IntroduceLetArgs),
    /// Inline a single local let binding into its body.
    InlineLet(inline_let::InlineLetArgs),
    /// Convert an independent Common Lisp let* form into let.
    ConvertLetStarToLet(convert_let_star_to_let::ConvertLetStarToLetArgs),
    /// Plan or remove one unused local let binding.
    RemoveUnusedBinding(remove_unused_binding::RemoveUnusedBindingArgs),
}

#[derive(Debug, Subcommand)]
pub(super) enum Command {
    /// Read-only inventory, validation, and analysis.
    Inspect {
        #[command(subcommand)]
        command: InspectCommand,
    },
    /// Structural edits on one selected form. Rewritten source is printed to stdout.
    Edit {
        #[command(subcommand)]
        command: EditCommand,
    },
    /// Semantic refactors, including planning, previews, verification, and apply flows.
    Refactor {
        #[command(subcommand)]
        command: RefactorCommand,
    },
}
