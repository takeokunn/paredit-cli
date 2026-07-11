use super::*;

pub(super) fn dispatch(command: Command) -> Result<()> {
    match command {
        Command::Check(args) => analysis_report::workflow::check(args)?,
        Command::Dialect(args) => analysis_report::workflow::dialect(args)?,
        Command::Stats(args) => analysis_report::workflow::stats(args)?,
        Command::AgentReport(args) => analysis_report::workflow::agent_report(args)?,
        Command::Outline(args) => analysis_report::workflow::outline(args)?,
        Command::FormReport(args) => form_report::workflow::form_report(args)?,
        Command::FindSymbol(args) => symbol_report::workflow::find_symbol(args)?,
        Command::SymbolReport(args) => symbol_report::workflow::symbol_report(args)?,
        Command::CallReport(args) => call_report::workflow::call_report(args)?,
        Command::SignatureReport(args) => signature_report::workflow::signature_report(args)?,
        Command::CallGraph(args) => call_graph_report::workflow::call_graph(args)?,
        Command::ImpactReport(args) => impact_report::workflow::impact_report(args)?,
        Command::Refactor { command } => match command {
            command::RefactorCommand::Plan(args) => refactor::workflow::refactor_plan(args)?,
            command::RefactorCommand::Verify(args) => refactor::workflow::verify_refactor(args)?,
            command::RefactorCommand::Preview(args) => refactor::workflow::refactor_preview(args)?,
            command::RefactorCommand::Check(args) => refactor::workflow::refactor_check(args)?,
            command::RefactorCommand::Status(args) => refactor::workflow::refactor_status(args)?,
            command::RefactorCommand::Apply(args) => refactor::workflow::refactor_apply(args)?,
            command::RefactorCommand::Diff(args) => refactor::workflow::refactor_diff(args)?,
            command::RefactorCommand::WorkspacePlan(args) => {
                refactor::workflow::workspace_refactor_plan(args)?
            }
            command::RefactorCommand::WorkspacePreview(args) => {
                refactor::workflow::workspace_refactor_preview(args)?
            }
            command::RefactorCommand::WorkspaceExecute(args) => {
                refactor::workflow::workspace_refactor_execute(args)?
            }
        },
        Command::Workspace { command } => match command {
            command::WorkspaceCommand::Report(args) => {
                workspace_report::workflow::workspace_report(args)?
            }
        },
        Command::DependencyReport(args) => dependency_report::workflow::dependency_report(args)?,
        Command::PackageReport(args) => package::report::package_report(args)?,
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
        Command::AddExport(args) => package::add_export::add_export(args)?,
        Command::SortPackageExports(args) => package::sort_exports::sort_package_exports(args)?,
        Command::SortPackageOptions(args) => package::sort_options::sort_package_options(args)?,
        Command::MergePackageOptions(args) => package::merge_options::merge_package_options(args)?,
        Command::RenamePackage(args) => package::rename::rename_package(args)?,
        Command::RenameSymbol(args) => rename::rename_symbol::rename_symbol(args)?,
        Command::RenameInForm(args) => rename::rename_in_form::rename_in_form(args)?,
        Command::RenameBinding(args) => rename::rename_binding::rename_binding(args)?,
        Command::RenameSymbols(args) => rename::rename_symbols::rename_symbols(args)?,
        Command::RenameFunction(args) => rename::rename_function::rename_function(args)?,
        Command::RenameMacrolet(args) => rename::rename_macrolet::rename_macrolet(args)?,
        Command::RenameSymbolMacro(args) => rename::rename_symbol_macro::rename_symbol_macro(args)?,
        Command::RenameLocalFunction(args) => {
            rename::rename_local_function::rename_local_function(args)?
        }
        Command::ReplaceFunctionCalls(args) => {
            rename::replace_function_calls::replace_function_calls(args)?
        }
        Command::WrapFunctionCalls(args) => rename::wrap_function_calls::wrap_function_calls(args)?,
        Command::UnwrapFunctionCalls(args) => {
            rename::unwrap_function_calls::unwrap_function_calls(args)?
        }
        Command::UnwrapCall(args) => unwrap_call::unwrap_call(args)?,
        Command::ThreadExpression(args) => thread_expression::thread_expression(args)?,
        Command::UnthreadExpression(args) => unthread_expression::unthread_expression(args)?,
        Command::ExtractFunction(args) => extract_function::extract_function(args)?,
        Command::ExtractConstant(args) => extract_constant::extract_constant(args)?,
        Command::InlineFunction(args) => inline_function::inline_function(args)?,
        Command::AddFunctionParameter(args) => {
            function_parameter::add::add_function_parameter(args)?
        }
        Command::MoveFunctionParameter(args) => {
            function_parameter::move_parameter::move_function_parameter(args)?
        }
        Command::SwapFunctionParameters(args) => {
            function_parameter::swap::swap_function_parameters(args)?
        }
        Command::ReorderFunctionParameters(args) => {
            function_parameter::reorder::reorder_function_parameters(args)?
        }
        Command::RemoveFunctionParameter(args) => {
            function_parameter::remove::remove_function_parameter(args)?
        }
        Command::IntroduceLet(args) => introduce_let::introduce_let(args)?,
        Command::InlineLet(args) => inline_let::inline_let(args)?,
        Command::RemoveUnusedBinding(args) => remove_unused_binding::remove_unused_binding(args)?,
        Command::LetReport(args) => let_report::let_report(args)?,
        Command::Format(args) => basic_edit::workflow::format(args)?,
        Command::Select(args) => basic_edit::workflow::select(args)?,
        Command::Replace(args) => basic_edit::workflow::replace(args)?,
        Command::Kill(args) => basic_edit::workflow::kill(args)?,
        Command::Wrap(args) => basic_edit::workflow::wrap(args)?,
        Command::Splice(args) => basic_edit::workflow::splice(args)?,
        Command::Raise(args) => basic_edit::workflow::raise(args)?,
        Command::SlurpForward(args) => basic_edit::workflow::slurp_forward(args)?,
        Command::SlurpBackward(args) => basic_edit::workflow::slurp_backward(args)?,
        Command::BarfForward(args) => basic_edit::workflow::barf_forward(args)?,
        Command::BarfBackward(args) => basic_edit::workflow::barf_backward(args)?,
    }
    Ok(())
}
