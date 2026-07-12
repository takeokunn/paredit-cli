use crate::domain::sexpr::{ByteSpan, ExpressionKind, ExpressionPath, ExpressionView, SyntaxTree};

use super::{CommonLispReaderLabelDispatch, CommonLispReaderLabelForm, CommonLispReaderLabelKind};

/// Returns every Common Lisp `#n=` or `#n#` dispatch atom in source order.
pub fn common_lisp_reader_label_dispatches(
    tree: &SyntaxTree,
) -> Vec<CommonLispReaderLabelDispatch> {
    let mut dispatches = Vec::new();
    collect_dispatches(
        &tree.root_view(),
        &ExpressionPath::from_indexes(Vec::new()),
        &mut dispatches,
    );
    dispatches
}

/// Returns the complete source region consumed by every reader-label form.
pub fn common_lisp_reader_label_forms(tree: &SyntaxTree) -> Vec<CommonLispReaderLabelForm> {
    let mut forms = Vec::new();
    collect_forms(&tree.root_view(), &mut forms);
    forms
}

fn collect_dispatches(
    view: &ExpressionView,
    path: &ExpressionPath,
    dispatches: &mut Vec<CommonLispReaderLabelDispatch>,
) {
    if let Some(kind) = reader_label_kind(view) {
        dispatches.push(CommonLispReaderLabelDispatch {
            kind,
            path: path.clone(),
            span: view.content_span,
        });
    }

    for (index, child) in view.children.iter().enumerate() {
        collect_dispatches(child, &path.child(index), dispatches);
    }
}

fn collect_forms(view: &ExpressionView, forms: &mut Vec<CommonLispReaderLabelForm>) {
    for (index, child) in view.children.iter().enumerate() {
        if let Some(kind) = reader_label_kind(child) {
            let span = match kind {
                CommonLispReaderLabelKind::Definition => view
                    .children
                    .get(index + 1)
                    .map_or(child.content_span, |datum| {
                        ByteSpan::new(child.content_span.start(), datum.span.end())
                    }),
                CommonLispReaderLabelKind::Reference => child.content_span,
            };
            forms.push(CommonLispReaderLabelForm {
                kind,
                dispatch_span: child.content_span,
                span,
            });
        }
        collect_forms(child, forms);
    }
}

fn reader_label_kind(view: &ExpressionView) -> Option<CommonLispReaderLabelKind> {
    if view.kind != ExpressionKind::Atom {
        return None;
    }

    let text = view
        .text
        .as_deref()
        .and_then(|text| text.get(view.symbol_offset..))?;
    let suffix = text.strip_prefix('#')?;
    let (digits, kind) = match suffix.strip_suffix('=') {
        Some(digits) => (digits, CommonLispReaderLabelKind::Definition),
        None => (
            suffix.strip_suffix('#')?,
            CommonLispReaderLabelKind::Reference,
        ),
    };

    (!digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit())).then_some(kind)
}
