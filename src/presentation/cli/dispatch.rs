use super::*;

pub(super) fn dispatch(command: Command) -> Result<()> {
    match command {
        Command::Inspect { command } => match command {
            command::InspectCommand::Check(args) => analysis_report::workflow::check(args)?,
            command::InspectCommand::Dialect(args) => analysis_report::workflow::dialect(args)?,
            command::InspectCommand::Stats(args) => analysis_report::workflow::stats(args)?,
            command::InspectCommand::AgentReport(args) => {
                analysis_report::workflow::agent_report(args)?
            }
            command::InspectCommand::Outline(args) => analysis_report::workflow::outline(args)?,
            command::InspectCommand::Form(args) => form_report::workflow::form_report(args)?,
            command::InspectCommand::FindSymbol(args) => {
                symbol_report::workflow::find_symbol(args)?
            }
            command::InspectCommand::Symbols(args) => symbol_report::workflow::symbol_report(args)?,
            command::InspectCommand::Calls(args) => call_report::workflow::call_report(args)?,
            command::InspectCommand::Signature(args) => {
                signature_report::workflow::signature_report(args)?
            }
            command::InspectCommand::CallGraph(args) => {
                call_graph_report::workflow::call_graph(args)?
            }
            command::InspectCommand::Impact(args) => impact_report::workflow::impact_report(args)?,
            command::InspectCommand::Workspace(args) => {
                workspace_report::workflow::workspace_report(args)?
            }
            command::InspectCommand::Dependencies(args) => {
                dependency_report::workflow::dependency_report(args)?
            }
            command::InspectCommand::Packages(args) => package::report::package_report(args)?,
            command::InspectCommand::Definitions(args) => {
                definition_report::workflow::definition_report(args)?
            }
            command::InspectCommand::UnusedDefinitions(args) => {
                definition_report::workflow::unused_definition_report(args)?
            }
            command::InspectCommand::Duplicates(args) => {
                duplicate_report::workflow::duplicate_report(args)?
            }
            command::InspectCommand::Similarity(args) => {
                similarity_report::workflow::similarity_report(args)?
            }
            command::InspectCommand::Lets(args) => let_report::let_report(args)?,
        },
        Command::Edit { command } => match command {
            command::EditCommand::Format(args) => basic_edit::workflow::format(args)?,
            command::EditCommand::Select(args) => basic_edit::workflow::select(args)?,
            command::EditCommand::Replace(args) => basic_edit::workflow::replace(args)?,
            command::EditCommand::Kill(args) => basic_edit::workflow::kill(args)?,
            command::EditCommand::Wrap(args) => basic_edit::workflow::wrap(args)?,
            command::EditCommand::Splice(args) => basic_edit::workflow::splice(args)?,
            command::EditCommand::Raise(args) => basic_edit::workflow::raise(args)?,
            command::EditCommand::TransposeForward(args) => {
                basic_edit::workflow::transpose_forward(args)?
            }
            command::EditCommand::TransposeBackward(args) => {
                basic_edit::workflow::transpose_backward(args)?
            }
            command::EditCommand::SlurpForward(args) => basic_edit::workflow::slurp_forward(args)?,
            command::EditCommand::SlurpBackward(args) => {
                basic_edit::workflow::slurp_backward(args)?
            }
            command::EditCommand::BarfForward(args) => basic_edit::workflow::barf_forward(args)?,
            command::EditCommand::BarfBackward(args) => basic_edit::workflow::barf_backward(args)?,
        },
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
            command::RefactorCommand::RemoveDefinition(args) => {
                definition_removal::remove_definition::remove_definition(args)?
            }
            command::RefactorCommand::RemoveUnusedDefinitions(args) => {
                definition_removal::remove_unused_definitions::remove_unused_definitions(args)?
            }
            command::RefactorCommand::MoveDefinition(args) => {
                definition_movement::move_definition::move_definition(args)?
            }
            command::RefactorCommand::SplitFile(args) => {
                definition_movement::split_file::split_file(args)?
            }
            command::RefactorCommand::SortDefinitions(args) => {
                definition_movement::sort_definitions::sort_definitions(args)?
            }
            command::RefactorCommand::MoveForm(args) => {
                definition_movement::move_form::move_form(args)?
            }
            command::RefactorCommand::ReplacementPlan(args) => {
                duplicate_report::workflow::replacement_plan(args)?
            }
            command::RefactorCommand::ReplaceForms(args) => replace_forms::replace_forms(args)?,
            command::RefactorCommand::AddExport(args) => package::add_export::add_export(args)?,
            command::RefactorCommand::SortPackageExports(args) => {
                package::sort_exports::sort_package_exports(args)?
            }
            command::RefactorCommand::SortPackageOptions(args) => {
                package::sort_options::sort_package_options(args)?
            }
            command::RefactorCommand::MergePackageOptions(args) => {
                package::merge_options::merge_package_options(args)?
            }
            command::RefactorCommand::RenamePackage(args) => package::rename::rename_package(args)?,
            command::RefactorCommand::RenameAt(args) => rename::rename_at::rename_at(args)?,
            command::RefactorCommand::RenameSymbol(args) => {
                rename::rename_symbol::rename_symbol(args)?
            }
            command::RefactorCommand::RenameInForm(args) => {
                rename::rename_in_form::rename_in_form(args)?
            }
            command::RefactorCommand::RenameBinding(args) => {
                rename::rename_binding::rename_binding(args)?
            }
            command::RefactorCommand::RenameSymbols(args) => {
                rename::rename_symbols::rename_symbols(args)?
            }
            command::RefactorCommand::RenameFunction(args) => {
                rename::rename_function::rename_function(args)?
            }
            command::RefactorCommand::RenameMacrolet(args) => {
                rename::rename_macrolet::rename_macrolet(args)?
            }
            command::RefactorCommand::RenameSymbolMacro(args) => {
                rename::rename_symbol_macro::rename_symbol_macro(args)?
            }
            command::RefactorCommand::RenameLocalFunction(args) => {
                rename::rename_local_function::rename_local_function(args)?
            }
            command::RefactorCommand::ReplaceFunctionCalls(args) => {
                rename::replace_function_calls::replace_function_calls(args)?
            }
            command::RefactorCommand::WrapFunctionCalls(args) => {
                rename::wrap_function_calls::wrap_function_calls(args)?
            }
            command::RefactorCommand::UnwrapFunctionCalls(args) => {
                rename::unwrap_function_calls::unwrap_function_calls(args)?
            }
            command::RefactorCommand::UnwrapCall(args) => unwrap_call::unwrap_call(args)?,
            command::RefactorCommand::ThreadExpression(args) => {
                thread_expression::thread_expression(args)?
            }
            command::RefactorCommand::UnthreadExpression(args) => {
                unthread_expression::unthread_expression(args)?
            }
            command::RefactorCommand::ExtractFunction(args) => {
                extract_function::extract_function(args)?
            }
            command::RefactorCommand::ExtractLocalFunction(args) => {
                extract_local_function::extract_local_function(args)?
            }
            command::RefactorCommand::ExtractConstant(args) => {
                extract_constant::extract_constant(args)?
            }
            command::RefactorCommand::InlineFunction(args) => {
                inline_function::inline_function(args)?
            }
            command::RefactorCommand::InlineLambda(args) => inline_lambda::inline_lambda(args)?,
            command::RefactorCommand::InlineLocalFunction(args) => {
                inline_local_function::inline_local_function(args)?
            }
            command::RefactorCommand::AddFunctionParameter(args) => {
                function_parameter::add::add_function_parameter(args)?
            }
            command::RefactorCommand::MoveFunctionParameter(args) => {
                function_parameter::move_parameter::move_function_parameter(args)?
            }
            command::RefactorCommand::SwapFunctionParameters(args) => {
                function_parameter::swap::swap_function_parameters(args)?
            }
            command::RefactorCommand::ReorderFunctionParameters(args) => {
                function_parameter::reorder::reorder_function_parameters(args)?
            }
            command::RefactorCommand::RemoveFunctionParameter(args) => {
                function_parameter::remove::remove_function_parameter(args)?
            }
            command::RefactorCommand::IntroduceLet(args) => introduce_let::introduce_let(args)?,
            command::RefactorCommand::InlineLet(args) => inline_let::inline_let(args)?,
            command::RefactorCommand::RemoveUnusedBinding(args) => {
                remove_unused_binding::remove_unused_binding(args)?
            }
        },
    }
    Ok(())
}
