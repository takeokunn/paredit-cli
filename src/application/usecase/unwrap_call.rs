use anyhow::{Context, Result};

use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{
    ByteSpan, Delimiter, ExpressionKind, ExpressionView, Path, SymbolName, SyntaxTree,
};

#[derive(Debug, Clone)]
pub struct UnwrapCallRequest<'a> {
    pub input: &'a str,
    pub dialect: Dialect,
    pub path: Option<Path>,
    pub target: ExpressionView,
    pub expected_function: Option<SymbolName>,
    pub argument_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnwrapCallPlan {
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

pub fn plan_unwrap_call(request: UnwrapCallRequest<'_>) -> Result<UnwrapCallPlan> {
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
        .context("argument index overflow")?;
    let argument = request.target.children.get(child_index).with_context(|| {
        format!(
            "argument index {} is out of range for {} argument(s)",
            request.argument_index,
            request.target.children.len().saturating_sub(1)
        )
    })?;
    let replacement = argument.span.slice(request.input).to_owned();
    SyntaxTree::parse(&replacement).context("unwrap-call replacement is not parseable")?;

    let rewritten = replace_span(request.input, request.target.span, &replacement);
    SyntaxTree::parse(&rewritten).context("unwrap-call rewritten output is not parseable")?;

    Ok(UnwrapCallPlan {
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

fn replace_span(input: &str, span: ByteSpan, replacement: &str) -> String {
    let mut output = input.to_owned();
    output.replace_range(span.as_range(), replacement);
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::sexpr::Path;
    use proptest::prelude::*;

    fn target(input: &str) -> ExpressionView {
        let tree = SyntaxTree::parse(input).expect("parse");
        tree.select_path(&"0".parse::<Path>().expect("path"))
            .expect("select")
            .view()
    }

    #[test]
    fn unwraps_first_argument_from_guarded_call() {
        let input = "(with-cache (fetch-user id) :ttl 60)";
        let plan = plan_unwrap_call(UnwrapCallRequest {
            input,
            dialect: Dialect::CommonLisp,
            path: Some("0".parse().expect("path")),
            target: target(input),
            expected_function: Some(SymbolName::new("with-cache").expect("symbol")),
            argument_index: 0,
        })
        .expect("plan");

        assert_eq!(plan.function.as_str(), "with-cache");
        assert_eq!(plan.call_argument_count, 3);
        assert_eq!(plan.replacement, "(fetch-user id)");
        assert_eq!(plan.rewritten, "(fetch-user id)");
        assert!(plan.changed);
    }

    #[test]
    fn unwraps_selected_argument() {
        let input = "(choose old-value new-value)";
        let plan = plan_unwrap_call(UnwrapCallRequest {
            input,
            dialect: Dialect::EmacsLisp,
            path: Some("0".parse().expect("path")),
            target: target(input),
            expected_function: None,
            argument_index: 1,
        })
        .expect("plan");

        assert_eq!(plan.function.as_str(), "choose");
        assert_eq!(plan.replacement, "new-value");
        assert_eq!(plan.rewritten, "new-value");
    }

    #[test]
    fn rejects_mismatched_function_guard() {
        let input = "(with-cache (fetch-user id))";
        let err = plan_unwrap_call(UnwrapCallRequest {
            input,
            dialect: Dialect::CommonLisp,
            path: Some("0".parse().expect("path")),
            target: target(input),
            expected_function: Some(SymbolName::new("with-transaction").expect("symbol")),
            argument_index: 0,
        })
        .expect_err("mismatch");

        assert!(err.to_string().contains("expected function"));
    }

    fn symbol_strategy() -> impl Strategy<Value = String> {
        "[a-z][a-z0-9-]{0,8}".prop_filter("reserved symbol", |name| {
            !matches!(
                name.as_str(),
                "defun" | "false" | "fn" | "lambda" | "let" | "nil" | "t" | "true"
            )
        })
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        #[test]
        fn pbt_unwrapped_argument_output_is_parseable_and_stable(
            wrapper in symbol_strategy(),
            callee in symbol_strategy(),
            value in symbol_strategy(),
            trailing in symbol_strategy(),
        ) {
            prop_assume!(wrapper != callee);

            let input = format!("({wrapper} ({callee} {value}) {trailing})");
            let plan = plan_unwrap_call(UnwrapCallRequest {
                input: &input,
                dialect: Dialect::Clojure,
                path: Some("0".parse().expect("path")),
                target: target(&input),
                expected_function: Some(SymbolName::new(&wrapper).expect("symbol")),
                argument_index: 0,
            })
            .expect("plan");

            prop_assert_eq!(plan.replacement, format!("({callee} {value})"));
            prop_assert_eq!(&plan.rewritten, &format!("({callee} {value})"));
            prop_assert!(SyntaxTree::parse(&plan.rewritten).is_ok());
            prop_assert!(plan.changed);
        }
    }
}
