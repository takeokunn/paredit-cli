//! Application facade for the unwrap-call domain plan.

use anyhow::Result;

use crate::application::usecase::mutation_safety::reject_overlapping_common_lisp_reader_time_forms;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{ByteSpan, ExpressionView, Path, SymbolName, SyntaxTree};

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
    let tree = SyntaxTree::parse(request.input)?;
    reject_overlapping_common_lisp_reader_time_forms(
        &tree,
        request.dialect,
        [request.target.span],
    )?;
    let plan = crate::domain::unwrap_call::plan(crate::domain::unwrap_call::Request {
        input: request.input,
        dialect: request.dialect,
        path: request.path,
        target: request.target,
        expected_function: request.expected_function,
        argument_index: request.argument_index,
    })?;
    Ok(UnwrapCallPlan {
        dialect: plan.dialect,
        path: plan.path,
        function: plan.function,
        span: plan.span,
        argument_index: plan.argument_index,
        argument_span: plan.argument_span,
        call_argument_count: plan.call_argument_count,
        replacement: plan.replacement,
        rewritten: plan.rewritten,
        changed: plan.changed,
    })
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
