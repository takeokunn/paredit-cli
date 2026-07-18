#[cfg(test)]
use crate::domain::sexpr::ExpressionPath;
use crate::domain::sexpr::{ByteSpan, ExpressionKind, ExpressionView, SyntaxTree};

#[cfg(test)]
use super::CommonLispReaderConditionalDispatch;
use super::{CommonLispReaderConditionalForm, CommonLispReaderConditionalKind};

/// Returns every Common Lisp `#+` or `#-` dispatch atom in source order.
///
/// The parser keeps the dispatch, feature expression, and guarded datum as
/// sibling expressions. A bare dispatch is still reported so callers can
/// reject incomplete input safely before attempting a structural refactor.
#[cfg(test)]
pub fn common_lisp_reader_conditional_dispatches(
    tree: &SyntaxTree,
) -> Vec<CommonLispReaderConditionalDispatch> {
    let mut dispatches = Vec::new();
    collect_dispatches(
        &tree.root_view(),
        &ExpressionPath::from_indexes(Vec::new()),
        &mut dispatches,
    );
    dispatches
}

/// Returns the complete source region consumed by every reader conditional.
///
/// The parser represents a reader conditional as three sibling expressions:
/// its dispatch atom, feature expression, and guarded datum. The returned span
/// protects all three, rather than only the dispatch token.
pub fn common_lisp_reader_conditional_forms(
    tree: &SyntaxTree,
) -> Vec<CommonLispReaderConditionalForm> {
    let mut forms = Vec::new();
    collect_forms(&tree.root_view(), &mut forms);
    forms
}

#[cfg(test)]
fn collect_dispatches(
    view: &ExpressionView,
    path: &ExpressionPath,
    dispatches: &mut Vec<CommonLispReaderConditionalDispatch>,
) {
    if let Some(kind) = reader_conditional_kind(view) {
        dispatches.push(CommonLispReaderConditionalDispatch {
            kind,
            path: path.clone(),
            span: view.content_span,
        });
    }

    for (index, child) in view.children.iter().enumerate() {
        collect_dispatches(child, &path.child(index), dispatches);
    }
}

fn collect_forms(view: &ExpressionView, forms: &mut Vec<CommonLispReaderConditionalForm>) {
    let mut stack = vec![(view, 0)];
    while let Some((view, index)) = stack.pop() {
        let Some(child) = view.children.get(index) else {
            continue;
        };
        if let Some(kind) = reader_conditional_kind(child) {
            let end = view
                .children
                .get(index + 2)
                .or_else(|| view.children.get(index + 1))
                .map_or(child.span.end(), |component| component.span.end());
            forms.push(CommonLispReaderConditionalForm {
                kind,
                dispatch_span: child.content_span,
                span: ByteSpan::new(child.span.start(), end),
            });
        }

        stack.push((view, index + 1));
        stack.push((child, 0));
    }
}

pub(crate) fn reader_conditional_kind(
    view: &ExpressionView,
) -> Option<CommonLispReaderConditionalKind> {
    if view.kind != ExpressionKind::Atom {
        return None;
    }

    match view
        .text
        .as_deref()
        .and_then(|text| text.get(view.symbol_offset..))
    {
        Some("#+") => Some(CommonLispReaderConditionalKind::Include),
        Some("#-") => Some(CommonLispReaderConditionalKind::Exclude),
        _ => None,
    }
}
