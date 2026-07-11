use crate::domain::common_lisp::{
    CommonLispLambdaListShape, CommonLispOperator, normalize_common_lisp_operator_head,
};
use crate::domain::sexpr::{Delimiter, ExpressionKind, ExpressionView};

use super::DefinitionCategory;

pub(super) fn definition_lambda_list_child_index(
    view: &ExpressionView,
    head: &str,
) -> Option<usize> {
    if let Some(shape) = CommonLispOperator::from_head(head)
        .and_then(CommonLispOperator::definition_lambda_list_shape)
    {
        return common_lisp_lambda_list_child_index(view, shape);
    }

    let normalized = normalize_common_lisp_operator_head(head);

    match normalized {
        "cl-defun" | "defsubst" | "definline" | "cl-defmacro" | "define" | "lambda" | "defn"
        | "defn-" => list_child_index(view, 2),
        "cl-defmethod" => common_lisp_lambda_list_child_index(
            view,
            CommonLispLambdaListShape::FirstListAtOrAfter(2),
        ),
        "cl-defgeneric" => list_child_index(view, 2),
        "deftest" | "ert-deftest" | "define-test" | "define-ert-test" => list_child_index(view, 2),
        _ => None,
    }
}

pub(super) fn definition_body_start_child_index(
    view: &ExpressionView,
    head: &str,
    category: Option<DefinitionCategory>,
    lambda_list_index: Option<usize>,
) -> usize {
    if matches!(
        CommonLispOperator::from_head(head),
        Some(CommonLispOperator::Defsetf)
    ) {
        return match lambda_list_index {
            Some(index) => (index + 2).min(view.children.len()),
            None => view.children.len(),
        };
    }

    match (category, lambda_list_index) {
        (Some(category), Some(index)) if category.is_callable() => index + 1,
        (Some(category), None) if category.is_callable() => 3,
        (Some(_), _) => 2,
        (None, _) => 0,
    }
}

pub(super) fn definition_lambda_parameter_count(lambda_list: &ExpressionView) -> usize {
    lambda_list
        .children
        .iter()
        .filter(|child| match child.kind {
            ExpressionKind::Atom => child
                .text
                .as_deref()
                .is_some_and(|text| !text.starts_with('&')),
            ExpressionKind::List => true,
            ExpressionKind::Root => false,
        })
        .count()
}

fn common_lisp_lambda_list_child_index(
    view: &ExpressionView,
    shape: CommonLispLambdaListShape,
) -> Option<usize> {
    match shape {
        CommonLispLambdaListShape::ChildAt(index) => list_child_index(view, index),
        CommonLispLambdaListShape::FirstListAtOrAfter(start_index) => {
            (start_index..view.children.len()).find(|&index| {
                matches!(
                    view.children[index].delimiter,
                    Some(Delimiter::Paren | Delimiter::Bracket)
                )
            })
        }
    }
}

fn list_child_index(view: &ExpressionView, index: usize) -> Option<usize> {
    view.children
        .get(index)
        .and_then(|child| (child.kind == ExpressionKind::List).then_some(index))
}
