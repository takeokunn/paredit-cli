use anyhow::{Context, Result};

use crate::application::usecase::rename::selection::list_head;
use crate::domain::definition::classify_definition_head;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{ExpressionView, Path, SymbolName, SyntaxTree};

use super::WrapFunctionCallSite;
use super::call_site::wrap_call_site_from_view;
use super::choose::select_outermost_wrap_call_sites;

pub(super) fn collect_wrap_all_call_sites(
    tree: &SyntaxTree,
    dialect: Dialect,
    input: &str,
    function: &SymbolName,
    wrapper: &SymbolName,
) -> Result<(
    Vec<WrapFunctionCallSite>,
    Vec<WrapFunctionCallSite>,
    Vec<WrapFunctionCallSite>,
)> {
    let mut candidates = Vec::new();
    let mut skipped_already_wrapped = Vec::new();

    for (index, _) in tree.root_children().iter().enumerate() {
        let path_indexes = vec![index];
        let path = Path::from_indexes(path_indexes.clone());
        let view = tree.select_path(&path)?.view();
        let mut collection = WrapCallSiteCollection {
            dialect,
            input,
            function,
            wrapper,
            candidates: &mut candidates,
            skipped_already_wrapped: &mut skipped_already_wrapped,
        };
        collect_wrap_call_sites_from_view(&view, path_indexes, None, &mut collection);
    }

    let (calls, skipped_nested) = select_outermost_wrap_call_sites(candidates);
    Ok((calls, skipped_already_wrapped, skipped_nested))
}

pub(super) fn collect_wrap_explicit_call_sites(
    tree: &SyntaxTree,
    dialect: Dialect,
    input: &str,
    paths: &[Path],
    function: &SymbolName,
    wrapper: &SymbolName,
) -> Result<(
    Vec<WrapFunctionCallSite>,
    Vec<WrapFunctionCallSite>,
    Vec<WrapFunctionCallSite>,
)> {
    let mut calls = Vec::new();
    let mut skipped_already_wrapped = Vec::new();

    for path in paths {
        let view = tree.select_path(path)?.view();
        let site = wrap_call_site_from_view(&view, input, path.to_string(), function, wrapper)
            .with_context(|| format!("call-path {path} is not a call to {function}"))?;
        if call_site_is_already_wrapped(tree, dialect, path, wrapper)? {
            skipped_already_wrapped.push(site);
        } else {
            calls.push(site);
        }
    }

    Ok((calls, skipped_already_wrapped, Vec::new()))
}

struct WrapCallSiteCollection<'a> {
    dialect: Dialect,
    input: &'a str,
    function: &'a SymbolName,
    wrapper: &'a SymbolName,
    candidates: &'a mut Vec<WrapFunctionCallSite>,
    skipped_already_wrapped: &'a mut Vec<WrapFunctionCallSite>,
}

fn collect_wrap_call_sites_from_view(
    view: &ExpressionView,
    path_indexes: Vec<usize>,
    parent_head: Option<&str>,
    collection: &mut WrapCallSiteCollection<'_>,
) {
    let current_head = list_head(view);
    if let Some(site) = wrap_call_site_from_view(
        view,
        collection.input,
        Path::from_indexes(path_indexes.clone()).to_string(),
        collection.function,
        collection.wrapper,
    ) {
        if parent_head == Some(collection.wrapper.as_str()) {
            collection.skipped_already_wrapped.push(site);
        } else if classify_definition_head(collection.dialect, current_head.unwrap_or_default())
            .is_none()
        {
            collection.candidates.push(site);
        }
    }

    for (index, child) in view.children.iter().enumerate() {
        let mut child_path = path_indexes.clone();
        child_path.push(index);
        collect_wrap_call_sites_from_view(child, child_path, current_head, collection);
    }
}

fn call_site_is_already_wrapped(
    tree: &SyntaxTree,
    dialect: Dialect,
    path: &Path,
    wrapper: &SymbolName,
) -> Result<bool> {
    let indexes = path
        .indexes()
        .iter()
        .map(|index| index.get())
        .collect::<Vec<_>>();
    if indexes.is_empty() {
        return Ok(false);
    }
    let parent_path = Path::from_indexes(indexes[..indexes.len() - 1].to_vec());
    if parent_path.indexes().is_empty() {
        return Ok(false);
    }
    let parent = tree.select_path(&parent_path)?.view();
    let Some(head) = list_head(&parent) else {
        return Ok(false);
    };
    Ok(head == wrapper.as_str() && classify_definition_head(dialect, head).is_none())
}
