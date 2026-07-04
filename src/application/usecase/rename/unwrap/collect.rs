use anyhow::Result;

use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{ExpressionView, Path, SymbolName, SyntaxTree};

use super::UnwrapFunctionCallSite;
use super::call_site::{UnwrapCandidate, unwrap_call_site_from_view};
use super::choose::select_outermost_unwrap_call_sites;

pub(super) fn collect_unwrap_all_call_sites(
    tree: &SyntaxTree,
    dialect: Dialect,
    input: &str,
    function: &SymbolName,
    wrapper: &SymbolName,
) -> Result<(
    Vec<UnwrapFunctionCallSite>,
    Vec<UnwrapFunctionCallSite>,
    Vec<UnwrapFunctionCallSite>,
)> {
    let mut collection = UnwrapCollection::new(dialect, input, function, wrapper);

    for (index, _) in tree.root_children().iter().enumerate() {
        let path_indexes = vec![index];
        let path = Path::from_indexes(path_indexes.clone());
        let view = tree.select_path(&path)?.view();
        collection.collect_from_view(&view, path_indexes);
    }

    let (calls, skipped_nested) = select_outermost_unwrap_call_sites(collection.candidates);
    Ok((calls, collection.skipped_non_unary_wrapper, skipped_nested))
}

pub(super) fn collect_unwrap_explicit_call_sites(
    tree: &SyntaxTree,
    dialect: Dialect,
    input: &str,
    paths: &[Path],
    function: &SymbolName,
    wrapper: &SymbolName,
) -> Result<(
    Vec<UnwrapFunctionCallSite>,
    Vec<UnwrapFunctionCallSite>,
    Vec<UnwrapFunctionCallSite>,
)> {
    let mut calls = Vec::new();
    let mut skipped_non_unary_wrapper = Vec::new();

    for path in paths {
        let view = tree.select_path(path)?.view();
        match unwrap_call_site_from_view(&view, dialect, input, path.to_string(), function, wrapper)
        {
            UnwrapCandidate::Selected(site) => calls.push(site),
            UnwrapCandidate::NonUnaryWrapper(site) => skipped_non_unary_wrapper.push(site),
            UnwrapCandidate::NotMatched => {
                anyhow::bail!("call-path {path} is not a unary {wrapper} wrapper around {function}")
            }
        }
    }

    Ok((calls, skipped_non_unary_wrapper, Vec::new()))
}

struct UnwrapCollection<'a> {
    dialect: Dialect,
    input: &'a str,
    function: &'a SymbolName,
    wrapper: &'a SymbolName,
    candidates: Vec<UnwrapFunctionCallSite>,
    skipped_non_unary_wrapper: Vec<UnwrapFunctionCallSite>,
}

impl<'a> UnwrapCollection<'a> {
    fn new(
        dialect: Dialect,
        input: &'a str,
        function: &'a SymbolName,
        wrapper: &'a SymbolName,
    ) -> Self {
        Self {
            dialect,
            input,
            function,
            wrapper,
            candidates: Vec::new(),
            skipped_non_unary_wrapper: Vec::new(),
        }
    }

    fn collect_from_view(&mut self, view: &ExpressionView, path_indexes: Vec<usize>) {
        let path = Path::from_indexes(path_indexes.clone()).to_string();
        match unwrap_call_site_from_view(
            view,
            self.dialect,
            self.input,
            path,
            self.function,
            self.wrapper,
        ) {
            UnwrapCandidate::Selected(site) => self.candidates.push(site),
            UnwrapCandidate::NonUnaryWrapper(site) => self.skipped_non_unary_wrapper.push(site),
            UnwrapCandidate::NotMatched => {}
        }

        for (index, child) in view.children.iter().enumerate() {
            let mut child_path = path_indexes.clone();
            child_path.push(index);
            self.collect_from_view(child, child_path);
        }
    }
}
