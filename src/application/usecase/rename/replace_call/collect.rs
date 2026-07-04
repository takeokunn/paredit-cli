use anyhow::{Context, Result};

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
    for (index, _) in tree.root_children().iter().enumerate() {
        let path_indexes = vec![index];
        let path = Path::from_indexes(path_indexes.clone());
        let view = tree.select_path(&path)?.view();
        collect_replace_call_sites_from_view(
            &view,
            path_indexes,
            dialect,
            input,
            from,
            to,
            &mut calls,
        )?;
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

fn collect_replace_call_sites_from_view(
    view: &ExpressionView,
    path_indexes: Vec<usize>,
    dialect: Dialect,
    input: &str,
    from: &SymbolName,
    to: &SymbolName,
    calls: &mut Vec<ReplaceFunctionCallSite>,
) -> Result<()> {
    if let Some(site) = replace_call_site_from_view(
        view,
        dialect,
        input,
        Path::from_indexes(path_indexes.clone()).to_string(),
        from,
        to,
    ) {
        calls.push(site);
    }

    for (index, child) in view.children.iter().enumerate() {
        let mut child_path = path_indexes.clone();
        child_path.push(index);
        collect_replace_call_sites_from_view(child, child_path, dialect, input, from, to, calls)?;
    }
    Ok(())
}
