//! Domain planning for replacing a call with one of its arguments.

use anyhow::{Context, Result};

use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{
    ByteSpan, Delimiter, ExpressionKind, ExpressionView, Path, SymbolName, SyntaxTree,
};

#[derive(Debug, Clone)]
pub(crate) struct Request<'a> {
    pub input: &'a str,
    pub dialect: Dialect,
    pub path: Option<Path>,
    pub target: ExpressionView,
    pub expected_function: Option<SymbolName>,
    pub argument_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Plan {
    pub dialect: Dialect,
    pub path: Option<Path>,
    pub function: SymbolName,
    pub span: ByteSpan,
    pub argument_index: usize,
    pub argument_span: ByteSpan,
    pub call_argument_count: usize,
    pub replacement: String,
    pub rewritten: String,
    pub changed: bool,
}

pub(crate) fn plan(request: Request<'_>) -> Result<Plan> {
    SyntaxTree::parse(request.input).context("unwrap-call input does not parse")?;

    if request.target.kind != ExpressionKind::List
        || request.target.delimiter != Some(Delimiter::Paren)
    {
        anyhow::bail!("unwrap-call target must be a parenthesized call");
    }

    let head = request
        .target
        .children
        .first()
        .and_then(|child| child.text.as_deref())
        .context("unwrap-call target must have an atom function head")?;
    let function = SymbolName::new(head)?;

    if let Some(expected) = &request.expected_function {
        if expected.as_str() != function.as_str() {
            anyhow::bail!(
                "unwrap-call expected function {}, found {}",
                expected.as_str(),
                function.as_str()
            );
        }
    }

    let child_index = request
        .argument_index
        .checked_add(1)
        .context("--argument-index is too large to address any call argument")?;
    let argument = request.target.children.get(child_index).with_context(|| {
        format!(
            "argument index {} is out of range for {} argument(s)",
            request.argument_index,
            request.target.children.len().saturating_sub(1)
        )
    })?;
    let replacement = argument.span.slice(request.input).to_owned();
    SyntaxTree::parse(&replacement).context("unwrap-call replacement is not parseable")?;

    let mut rewritten = request.input.to_owned();
    rewritten.replace_range(request.target.span.as_range(), &replacement);
    SyntaxTree::parse(&rewritten).context("unwrap-call rewritten output is not parseable")?;

    Ok(Plan {
        dialect: request.dialect,
        path: request.path,
        function,
        span: request.target.span,
        argument_index: request.argument_index,
        argument_span: argument.span,
        call_argument_count: request.target.children.len().saturating_sub(1),
        changed: request.target.span.slice(request.input) != replacement,
        replacement,
        rewritten,
    })
}
