use anyhow::Result;

use crate::application::usecase::callable_scope::{
    LocalCallableForm, common_lisp_local_callable_form, is_local_callable_bound,
    local_callable_names,
};
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{
    ByteSpan, Delimiter, ExpressionKind, ExpressionView, Path, SymbolName, SyntaxTree,
};

use crate::application::usecase::function_parameter::list_edit::{list_head, spans_overlap};

struct FunctionCallTraversal<'a> {
    dialect: Dialect,
    definition_span: ByteSpan,
    function_name: &'a SymbolName,
}

pub(in crate::application::usecase::function_parameter) fn resolve_function_call_paths(
    tree: &SyntaxTree,
    dialect: Dialect,
    explicit_call_paths: Vec<Path>,
    all_calls: bool,
    definition_span: ByteSpan,
    function_name: &SymbolName,
    command: &str,
) -> Result<Vec<Path>> {
    if all_calls && !explicit_call_paths.is_empty() {
        anyhow::bail!("{command} accepts either --all-calls or repeated --call-path, not both");
    }

    if all_calls {
        let call_paths =
            discover_function_call_paths(tree, dialect, definition_span, function_name)?;
        if call_paths.is_empty() {
            anyhow::bail!(
                "{command} --all-calls found no same-file calls for {}",
                function_name
            );
        }
        return Ok(call_paths);
    }

    if explicit_call_paths.is_empty() {
        anyhow::bail!("{command} requires at least one --call-path or --all-calls");
    }

    Ok(explicit_call_paths)
}

fn discover_function_call_paths(
    tree: &SyntaxTree,
    dialect: Dialect,
    definition_span: ByteSpan,
    function_name: &SymbolName,
) -> Result<Vec<Path>> {
    let mut call_paths = Vec::new();
    let context = FunctionCallTraversal {
        dialect,
        definition_span,
        function_name,
    };

    for index in 0..tree.root_children().len() {
        let mut indexes = vec![index];
        let path = Path::from_indexes(indexes.clone());
        let selection = tree.select_path(&path)?;
        let view = selection.view();
        collect_function_call_paths(&view, &mut indexes, &context, &[], &mut call_paths);
    }

    call_paths.sort_by_key(|path| {
        tree.select_path(path)
            .map(|selection| selection.span().start().get())
            .unwrap_or(usize::MAX)
    });
    Ok(call_paths)
}

fn collect_function_call_paths(
    view: &ExpressionView,
    indexes: &mut Vec<usize>,
    context: &FunctionCallTraversal<'_>,
    local_callables: &[String],
    output: &mut Vec<Path>,
) {
    if let Some(head) = list_head(view)
        && let Some(form) = common_lisp_local_callable_form(context.dialect, head)
    {
        collect_local_callable_function_call_paths(
            view,
            indexes,
            context,
            local_callables,
            output,
            form,
        );
        return;
    }

    if view.kind == ExpressionKind::List
        && view.delimiter == Some(Delimiter::Paren)
        && !spans_overlap(context.definition_span, view.span)
        && list_head(view).is_some_and(|head| head == context.function_name.as_str())
        && !is_local_callable_bound(local_callables, context.function_name.as_str())
    {
        output.push(Path::from_indexes(indexes.clone()));
    }

    for (index, child) in view.children.iter().enumerate() {
        indexes.push(index);
        collect_function_call_paths(child, indexes, context, local_callables, output);
        indexes.pop();
    }
}

fn collect_local_callable_function_call_paths(
    view: &ExpressionView,
    indexes: &mut Vec<usize>,
    context: &FunctionCallTraversal<'_>,
    local_callables: &[String],
    output: &mut Vec<Path>,
    form: LocalCallableForm,
) {
    let local_names = local_callable_names(view);
    let mut body_scope = local_callables.to_vec();
    body_scope.extend(local_names);

    let binding_body_scope = match form {
        LocalCallableForm::Labels => body_scope.as_slice(),
        LocalCallableForm::Flet
        | LocalCallableForm::Macrolet
        | LocalCallableForm::CompilerMacrolet => local_callables,
    };

    if let Some(bindings) = view.children.get(1) {
        indexes.push(1);
        for (binding_index, binding) in bindings.children.iter().enumerate() {
            indexes.push(binding_index);
            for (binding_child_index, binding_child) in binding.children.iter().enumerate().skip(2)
            {
                indexes.push(binding_child_index);
                collect_function_call_paths(
                    binding_child,
                    indexes,
                    context,
                    binding_body_scope,
                    output,
                );
                indexes.pop();
            }
            indexes.pop();
        }
        indexes.pop();
    }

    for (index, child) in view.children.iter().enumerate().skip(2) {
        indexes.push(index);
        collect_function_call_paths(child, indexes, context, &body_scope, output);
        indexes.pop();
    }
}
