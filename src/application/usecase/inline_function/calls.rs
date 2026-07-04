use anyhow::{Context, Result};

use crate::application::usecase::callable_scope::{
    LocalCallableForm, common_lisp_local_callable_form, is_local_callable_bound,
    local_callable_names,
};
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{
    ByteSpan, Delimiter, ExpressionKind, ExpressionView, Path, SymbolName, SyntaxTree,
};

use super::definition::InlineParameter;
use super::syntax::{atom_text, list_head, spans_overlap};

struct InlineCallTraversal<'a> {
    dialect: Dialect,
    definition_span: ByteSpan,
    function_name: &'a SymbolName,
}

pub(super) fn resolve_function_call_paths(
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

pub(super) fn parse_inline_function_call(
    view: ExpressionView,
    function_name: &SymbolName,
    input: &str,
) -> Result<Vec<String>> {
    if view.kind != ExpressionKind::List || view.delimiter != Some(Delimiter::Paren) {
        anyhow::bail!("inline-function call selection must be a function call list");
    }
    let head = atom_text(
        view.children
            .first()
            .context("inline-function call must not be empty")?,
    )
    .context("inline-function call must start with an atom")?;
    if head != function_name.as_str() {
        anyhow::bail!(
            "inline-function call head '{}' does not match selected definition '{}'",
            head,
            function_name
        );
    }

    Ok(view.children[1..]
        .iter()
        .map(|child| child.span.slice(input).to_owned())
        .collect())
}

pub(super) fn bind_inline_function_arguments(
    params: &[InlineParameter],
    raw_args: Vec<String>,
    function_name: &SymbolName,
) -> Result<Vec<String>> {
    let positional_count = params
        .iter()
        .take_while(|param| param.keyword.is_none())
        .count();
    if raw_args.len() < positional_count {
        anyhow::bail!(
            "inline-function arity mismatch for {}: definition requires {} positional argument(s), call has {} argument(s)",
            function_name,
            positional_count,
            raw_args.len()
        );
    }

    let keyword_params = &params[positional_count..];
    if keyword_params.is_empty() {
        if raw_args.len() != positional_count {
            anyhow::bail!(
                "inline-function arity mismatch for {}: definition has {} parameter(s), call has {} argument(s)",
                function_name,
                params.len(),
                raw_args.len()
            );
        }
        return Ok(raw_args);
    }

    let keyword_arg_count = raw_args.len() - positional_count;
    if keyword_arg_count % 2 != 0 {
        anyhow::bail!(
            "inline-function keyword arguments for {} must be supplied as keyword/value pairs",
            function_name
        );
    }

    let mut bound = raw_args[..positional_count].to_vec();
    for param in keyword_params {
        let keyword = param
            .keyword
            .as_deref()
            .context("inline-function internal error: keyword parameter missing keyword")?;
        let mut matched = None;
        for pair in raw_args[positional_count..].chunks_exact(2) {
            let key = &pair[0];
            let value = &pair[1];
            if !key.starts_with(':') {
                anyhow::bail!(
                    "inline-function expected keyword argument for {}, found {}",
                    function_name,
                    key
                );
            }
            if key == keyword {
                if matched.is_some() {
                    anyhow::bail!(
                        "inline-function call for {} supplies duplicate keyword {}",
                        function_name,
                        keyword
                    );
                }
                matched = Some(value.clone());
            }
        }
        let value = matched.with_context(|| {
            format!(
                "inline-function call for {} must explicitly supply keyword {}",
                function_name, keyword
            )
        })?;
        bound.push(value);
    }

    for pair in raw_args[positional_count..].chunks_exact(2) {
        let key = &pair[0];
        if !keyword_params
            .iter()
            .any(|param| param.keyword.as_deref() == Some(key.as_str()))
        {
            anyhow::bail!(
                "inline-function call for {} supplies unsupported keyword {}",
                function_name,
                key
            );
        }
    }

    Ok(bound)
}

fn discover_function_call_paths(
    tree: &SyntaxTree,
    dialect: Dialect,
    definition_span: ByteSpan,
    function_name: &SymbolName,
) -> Result<Vec<Path>> {
    let context = InlineCallTraversal {
        dialect,
        definition_span,
        function_name,
    };
    let mut call_paths = Vec::new();
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
    context: &InlineCallTraversal,
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
            form,
            output,
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
    context: &InlineCallTraversal,
    local_callables: &[String],
    form: LocalCallableForm,
    output: &mut Vec<Path>,
) {
    let local_names = local_callable_names(view);
    let mut body_scope = local_callables.to_vec();
    body_scope.extend(local_names.iter().cloned());

    if let Some(bindings) = view.children.get(1) {
        let binding_body_scope = match form {
            LocalCallableForm::Labels => body_scope.as_slice(),
            LocalCallableForm::Flet
            | LocalCallableForm::Macrolet
            | LocalCallableForm::CompilerMacrolet => local_callables,
        };
        for (binding_index, binding) in bindings.children.iter().enumerate() {
            indexes.extend([1, binding_index]);
            for (child_index, child) in binding.children.iter().enumerate().skip(2) {
                indexes.push(child_index);
                collect_function_call_paths(child, indexes, context, binding_body_scope, output);
                indexes.pop();
            }
            indexes.truncate(indexes.len().saturating_sub(2));
        }
    }

    for (index, child) in view.children.iter().enumerate().skip(2) {
        indexes.push(index);
        collect_function_call_paths(child, indexes, context, &body_scope, output);
        indexes.pop();
    }
}
