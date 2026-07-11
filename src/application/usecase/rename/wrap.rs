use anyhow::{Context, Result, ensure};

use crate::domain::common_lisp::common_lisp_symbol_name_eq;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{
    ByteSpan, ExpressionKind, ExpressionView, Path, SymbolName, SyntaxTree,
};

mod call_site;
mod choose;
mod collect;

use collect::{collect_wrap_all_call_sites, collect_wrap_explicit_call_sites};

use super::selection::apply_byte_span_edits;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WrapFunctionCallSite {
    pub path: String,
    pub span: ByteSpan,
    pub replacement: String,
    pub text: String,
}

#[derive(Debug, Clone)]
pub enum WrapFunctionCallsScope {
    AllCalls,
    ExplicitPaths(Vec<Path>),
}

#[derive(Debug, Clone)]
pub struct WrapFunctionCallsRequest<'a> {
    pub input: &'a str,
    pub dialect: Dialect,
    pub function: SymbolName,
    pub wrapper: SymbolName,
    pub wrapper_template: Option<String>,
    pub scope: WrapFunctionCallsScope,
}

#[derive(Debug, Clone)]
pub struct WrapFunctionCallsPlan {
    pub dialect: Dialect,
    pub calls: Vec<WrapFunctionCallSite>,
    pub skipped_already_wrapped: Vec<WrapFunctionCallSite>,
    pub skipped_nested: Vec<WrapFunctionCallSite>,
    pub rewritten: String,
    pub changed: bool,
}

#[derive(Debug, Clone)]
pub(super) struct WrapFunctionCallTemplate {
    source: String,
    placeholder_span: ByteSpan,
}

impl WrapFunctionCallTemplate {
    fn parse(source: String, wrapper: &SymbolName) -> Result<Self> {
        let tree = SyntaxTree::parse(&source).context("failed to parse wrapper template")?;
        ensure!(
            tree.root_children().len() == 1,
            "wrapper template must contain exactly one root form"
        );

        let root = tree.select_path(&Path::root_child(0))?.view();
        let head = crate::application::usecase::rename::selection::list_head(&root)
            .context("wrapper template root form must be a parenthesized list")?;
        ensure!(
            common_lisp_symbol_name_eq(head, wrapper.as_str()),
            "wrapper template head must match --wrapper ({})",
            wrapper.as_str()
        );

        let mut placeholders = Vec::new();
        collect_template_placeholders(&root, &mut placeholders);
        ensure!(
            placeholders.len() == 1,
            "wrapper template must contain exactly one _ placeholder atom"
        );

        Ok(Self {
            source,
            placeholder_span: placeholders[0],
        })
    }

    pub(super) fn apply(&self, call_text: &str) -> Result<String> {
        apply_byte_span_edits(
            &self.source,
            vec![(self.placeholder_span, call_text.to_owned())],
        )
    }
}

fn collect_template_placeholders(view: &ExpressionView, output: &mut Vec<ByteSpan>) {
    if view.kind == ExpressionKind::Atom && view.text.as_deref() == Some("_") {
        output.push(view.span);
        return;
    }
    for child in &view.children {
        collect_template_placeholders(child, output);
    }
}

pub fn plan_wrap_function_calls(
    request: WrapFunctionCallsRequest<'_>,
) -> Result<WrapFunctionCallsPlan> {
    let tree = SyntaxTree::parse(request.input).context("failed to parse input")?;
    let template = request
        .wrapper_template
        .map(|source| WrapFunctionCallTemplate::parse(source, &request.wrapper))
        .transpose()?;
    let (calls, skipped_already_wrapped, skipped_nested) = match &request.scope {
        WrapFunctionCallsScope::AllCalls => collect_wrap_all_call_sites(
            &tree,
            request.dialect,
            request.input,
            &request.function,
            &request.wrapper,
            template.as_ref(),
        )?,
        WrapFunctionCallsScope::ExplicitPaths(paths) => collect_wrap_explicit_call_sites(
            &tree,
            request.dialect,
            request.input,
            paths,
            &request.function,
            &request.wrapper,
            template.as_ref(),
        )?,
    };
    let edits = calls
        .iter()
        .map(|site| (site.span, site.replacement.clone()))
        .collect::<Vec<_>>();
    let rewritten = apply_byte_span_edits(request.input, edits)?;
    SyntaxTree::parse(&rewritten).context("wrapped output is not a valid S-expression document")?;

    Ok(WrapFunctionCallsPlan {
        dialect: request.dialect,
        calls,
        skipped_already_wrapped,
        skipped_nested,
        changed: rewritten != request.input,
        rewritten,
    })
}
