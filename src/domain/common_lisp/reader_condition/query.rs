use crate::domain::sexpr::{ByteSpan, ExpressionKind, ExpressionPath, ExpressionView, SyntaxTree};

use super::{
    CommonLispReaderConditionalDispatch, CommonLispReaderConditionalForm,
    CommonLispReaderConditionalKind,
};

/// Returns every Common Lisp `#+` or `#-` dispatch atom in source order.
///
/// The parser keeps the dispatch, feature expression, and guarded datum as
/// sibling expressions. A bare dispatch is still reported so callers can
/// reject incomplete input safely before attempting a structural refactor.
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

/// Returns whether a parsed document contains a Common Lisp `#+` or `#-` dispatch.
pub fn contains_common_lisp_reader_conditional(tree: &SyntaxTree) -> bool {
    !common_lisp_reader_conditional_dispatches(tree).is_empty()
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
    for (index, child) in view.children.iter().enumerate() {
        if let Some(kind) = reader_conditional_kind(child) {
            let span = view
                .children
                .get(index + 2)
                .map_or(child.content_span, |guarded| {
                    ByteSpan::new(child.content_span.start(), guarded.span.end())
                });
            forms.push(CommonLispReaderConditionalForm {
                kind,
                dispatch_span: child.content_span,
                span,
            });
        }
        collect_forms(child, forms);
    }
}

fn reader_conditional_kind(view: &ExpressionView) -> Option<CommonLispReaderConditionalKind> {
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
