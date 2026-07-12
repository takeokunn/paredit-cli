use anyhow::Result;

use crate::application::usecase::callable_scope::{
    LocalCallableName, common_lisp_local_callable_form, is_local_callable_bound,
    local_callable_binding_body_scope, local_callable_body_scope, local_callable_scope_at_path,
};
use crate::application::usecase::rename::reader::{
    apply_reader_prefix_context, executable_reader_context_at_path,
};
use crate::application::usecase::rename::selection::list_head;
use crate::domain::common_lisp::CommonLispLocalCallableForm;
use crate::domain::definition::macro_expander_body_range;
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
        let path = Path::root_child(index);
        let view = tree.select_path(&path)?.view();
        collection.collect_from_view(&view, path, 0, false);
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
        if !executable_reader_context_at_path(tree, dialect, path)? {
            anyhow::bail!("call-path {path} is not in an executable reader context");
        }
        let local_callables = local_callable_scope_at_path(tree, dialect, path)?;
        if is_local_callable_bound(&local_callables, function.as_str()) {
            anyhow::bail!("call-path {path} is shadowed by a local callable named {function}");
        }
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

    fn collect_from_view(
        &mut self,
        view: &ExpressionView,
        path: Path,
        quasiquote_depth: usize,
        in_macro_expander: bool,
    ) {
        self.collect_from_view_with_scope(view, path, &[], quasiquote_depth, in_macro_expander);
    }

    fn collect_from_view_with_scope(
        &mut self,
        view: &ExpressionView,
        path: Path,
        local_callables: &[LocalCallableName],
        quasiquote_depth: usize,
        in_macro_expander: bool,
    ) {
        let Some(quasiquote_depth) = apply_reader_prefix_context(view, quasiquote_depth) else {
            return;
        };
        if quasiquote_depth > 0 && !in_macro_expander {
            for (index, child) in view.children.iter().enumerate() {
                self.collect_from_view_with_scope(
                    child,
                    path.child(index),
                    local_callables,
                    quasiquote_depth,
                    in_macro_expander,
                );
            }
            return;
        }

        if let Some(head) = list_head(view) {
            if let Some(form) = common_lisp_local_callable_form(self.dialect, head) {
                self.collect_local_callable_from_view(
                    view,
                    path,
                    local_callables,
                    form,
                    quasiquote_depth,
                    in_macro_expander,
                );
                return;
            }
        }

        if !is_local_callable_bound(local_callables, self.function.as_str()) {
            match unwrap_call_site_from_view(
                view,
                self.dialect,
                self.input,
                path.to_string(),
                self.function,
                self.wrapper,
            ) {
                UnwrapCandidate::Selected(site) => self.candidates.push(site),
                UnwrapCandidate::NonUnaryWrapper(site) => {
                    self.skipped_non_unary_wrapper.push(site);
                }
                UnwrapCandidate::NotMatched => {}
            }
        }

        let macro_expander_body =
            list_head(view).and_then(|head| macro_expander_body_range(self.dialect, view, head));

        for (index, child) in view.children.iter().enumerate() {
            self.collect_from_view_with_scope(
                child,
                path.child(index),
                local_callables,
                quasiquote_depth,
                in_macro_expander
                    || macro_expander_body
                        .is_some_and(|body_range| body_range.contains_child(index)),
            );
        }
    }

    fn collect_local_callable_from_view(
        &mut self,
        view: &ExpressionView,
        path: Path,
        local_callables: &[LocalCallableName],
        form: CommonLispLocalCallableForm,
        quasiquote_depth: usize,
        in_macro_expander: bool,
    ) {
        let body_scope = local_callable_body_scope(local_callables, view);

        if let Some(bindings) = view.children.get(1) {
            let binding_body_scope =
                local_callable_binding_body_scope(form, local_callables, &body_scope);
            for (binding_index, binding) in bindings.children.iter().enumerate() {
                for (child_index, child) in binding.children.iter().enumerate().skip(2) {
                    self.collect_from_view_with_scope(
                        child,
                        path.descendant([1, binding_index, child_index]),
                        binding_body_scope,
                        quasiquote_depth,
                        in_macro_expander || form.is_macro(),
                    );
                }
            }
        }

        for (index, child) in view.children.iter().enumerate().skip(2) {
            self.collect_from_view_with_scope(
                child,
                path.child(index),
                &body_scope,
                quasiquote_depth,
                in_macro_expander,
            );
        }
    }
}
