use super::*;

pub(super) fn dispatch(command: Command) -> Result<()> {
    match command {
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
        Command::ImpactReport(args) => impact_report::workflow::impact_report(args)?,
        Command::RefactorPlan(args) => refactor::workflow::refactor_plan(args)?,
        Command::VerifyRefactor(args) => refactor::workflow::verify_refactor(args)?,
        Command::RefactorPreview(args) => refactor::workflow::refactor_preview(args)?,
        Command::RefactorCheck(args) => refactor::workflow::refactor_check(args)?,
        Command::RefactorStatus(args) => refactor::workflow::refactor_status(args)?,
        Command::RefactorApply(args) => refactor::workflow::refactor_apply(args)?,
        Command::RefactorDiff(args) => refactor::workflow::refactor_diff(args)?,
        Command::DependencyReport(args) => dependency_report::dependency_report(args)?,
        Command::PackageReport(args) => package::package_report(args)?,
        Command::DefinitionReport(args) => definition_report::workflow::definition_report(args)?,
        Command::UnusedDefinitionReport(args) => {
            definition_report::workflow::unused_definition_report(args)?
        }
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
        Command::DuplicateReport(args) => duplicate_report::workflow::duplicate_report(args)?,
        Command::ReplacementPlan(args) => duplicate_report::workflow::replacement_plan(args)?,
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
