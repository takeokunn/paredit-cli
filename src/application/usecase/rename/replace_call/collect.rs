use anyhow::{Context, Result};

use crate::application::usecase::callable_scope::{
    LocalCallableForm, common_lisp_local_callable_form, local_callable_names,
};
use crate::application::usecase::rename::selection::list_head;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{ExpressionView, Path, SymbolName, SyntaxTree};

use super::ReplaceFunctionCallSite;
use super::call_site::replace_call_site_from_view;

pub(super) fn collect_all_replace_call_sites(
    tree: &SyntaxTree,
    dialect: Dialect,
    input: &str,
    from: &SymbolName,
    to: &SymbolName,
) -> Result<Vec<ReplaceFunctionCallSite>> {
    let mut calls = Vec::new();
    let ctx = ReplaceCallTraversal {
        dialect,
        input,
        from,
        to,
    };
    for (index, _) in tree.root_children().iter().enumerate() {
        let path_indexes = vec![index];
        let path = Path::from_indexes(path_indexes.clone());
        let view = tree.select_path(&path)?.view();
        collect_replace_call_sites_from_view(&view, path_indexes, &ctx, &[], &mut calls)?;
    }
    calls.sort_by_key(|site| site.head_span.start());
    Ok(calls)
}

pub(super) fn collect_explicit_replace_call_sites(
    tree: &SyntaxTree,
    dialect: Dialect,
    input: &str,
    paths: &[Path],
    from: &SymbolName,
    to: &SymbolName,
) -> Result<Vec<ReplaceFunctionCallSite>> {
    let mut calls = Vec::new();
    for path in paths {
        let view = tree.select_path(path)?.view();
        let site = replace_call_site_from_view(&view, dialect, input, path.to_string(), from, to)
            .with_context(|| format!("call-path {path} is not a call to {from}"))?;
        calls.push(site);
    }
    calls.sort_by_key(|site| site.head_span.start());
    Ok(calls)
}

struct ReplaceCallTraversal<'a> {
    dialect: Dialect,
    input: &'a str,
    from: &'a SymbolName,
    to: &'a SymbolName,
}

fn collect_replace_call_sites_from_view(
    view: &ExpressionView,
    path_indexes: Vec<usize>,
    ctx: &ReplaceCallTraversal<'_>,
    local_callables: &[String],
    calls: &mut Vec<ReplaceFunctionCallSite>,
) -> Result<()> {
    if let Some(head) = list_head(view)
        && let Some(form) = common_lisp_local_callable_form(ctx.dialect, head)
    {
        collect_local_callable_replace_call_sites(
            view,
            path_indexes,
            ctx,
            local_callables,
            form,
            calls,
        )?;
        return Ok(());
    }

    if !crate::application::usecase::callable_scope::is_local_callable_bound(
        local_callables,
        ctx.from.as_str(),
    ) && let Some(site) = replace_call_site_from_view(
        view,
        ctx.dialect,
        ctx.input,
        Path::from_indexes(path_indexes.clone()).to_string(),
        ctx.from,
        ctx.to,
    ) {
        calls.push(site);
    }

    for (index, child) in view.children.iter().enumerate() {
        let mut child_path = path_indexes.clone();
        child_path.push(index);
        collect_replace_call_sites_from_view(child, child_path, ctx, local_callables, calls)?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn collect_local_callable_replace_call_sites(
    view: &ExpressionView,
    path_indexes: Vec<usize>,
    ctx: &ReplaceCallTraversal<'_>,
    local_callables: &[String],
    form: LocalCallableForm,
    calls: &mut Vec<ReplaceFunctionCallSite>,
) -> Result<()> {
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
            for (child_index, child) in binding.children.iter().enumerate().skip(2) {
                let mut child_path = path_indexes.clone();
                child_path.extend([1, binding_index, child_index]);
                collect_replace_call_sites_from_view(
                    child,
                    child_path,
                    ctx,
                    binding_body_scope,
                    calls,
                )?;
            }
        }
    }

    for (index, child) in view.children.iter().enumerate().skip(2) {
        let mut child_path = path_indexes.clone();
        child_path.push(index);
        collect_replace_call_sites_from_view(child, child_path, ctx, &body_scope, calls)?;
    }

    Ok(())
}
