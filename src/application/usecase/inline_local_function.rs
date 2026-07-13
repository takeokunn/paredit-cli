//! Application facade for inlining a single Common Lisp `flet` call.

use crate::application::usecase::mutation_safety::reject_common_lisp_reader_conditionals;
use crate::domain::dialect::Dialect;
use crate::domain::inline_local_function::{self, Request as DomainRequest};
use crate::domain::sexpr::{ByteSpan, Path, SymbolName, SyntaxTree};
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct InlineLocalFunctionRequest<'a> {
    pub input: &'a str,
    pub dialect: Dialect,
    pub path: Path,
}

#[derive(Debug, Clone)]
pub struct InlineLocalFunctionParameterPlan {
    pub name: SymbolName,
    pub argument: String,
    pub reference_count: usize,
}

#[derive(Debug, Clone)]
pub struct InlineLocalFunctionPlan {
    pub dialect: Dialect,
    pub path: Path,
    pub form_span: ByteSpan,
    pub call_span: ByteSpan,
    pub function_name: SymbolName,
    pub parameters: Vec<InlineLocalFunctionParameterPlan>,
    pub replacement: String,
    pub rewritten: String,
    pub changed: bool,
}

pub fn plan_inline_local_function(
    request: InlineLocalFunctionRequest<'_>,
) -> Result<InlineLocalFunctionPlan> {
    let tree = SyntaxTree::parse(request.input)?;
    reject_common_lisp_reader_conditionals(&tree, request.dialect)?;
    let plan = inline_local_function::plan(DomainRequest {
        input: request.input,
        dialect: request.dialect,
        path: request.path.clone(),
    })?;
    Ok(InlineLocalFunctionPlan {
        dialect: plan.dialect,
        path: plan.path,
        form_span: plan.form_span,
        call_span: plan.call_span,
        function_name: plan.function_name,
        parameters: plan
            .parameters
            .into_iter()
            .map(|parameter| InlineLocalFunctionParameterPlan {
                name: parameter.name,
                argument: parameter.argument,
                reference_count: parameter.reference_count,
            })
            .collect(),
        replacement: plan.replacement,
        rewritten: plan.rewritten,
        changed: plan.changed,
    })
}
