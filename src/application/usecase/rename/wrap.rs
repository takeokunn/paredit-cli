use anyhow::{Context, Result};

use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{ByteSpan, Path, SymbolName, SyntaxTree};

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

pub fn plan_wrap_function_calls(
    request: WrapFunctionCallsRequest<'_>,
) -> Result<WrapFunctionCallsPlan> {
    let tree = SyntaxTree::parse(request.input).context("failed to parse input")?;
    let (calls, skipped_already_wrapped, skipped_nested) = match &request.scope {
        WrapFunctionCallsScope::AllCalls => collect_wrap_all_call_sites(
            &tree,
            request.dialect,
            request.input,
            &request.function,
            &request.wrapper,
        )?,
        WrapFunctionCallsScope::ExplicitPaths(paths) => collect_wrap_explicit_call_sites(
            &tree,
            request.dialect,
            request.input,
            paths,
            &request.function,
            &request.wrapper,
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
