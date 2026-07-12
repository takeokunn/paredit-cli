use std::error::Error;
use std::fmt;

use crate::domain::common_lisp::{
    CommonLispReaderConditionalKind, CommonLispReaderLabelKind, CommonLispReaderLiteralKind,
    common_lisp_reader_conditional_dispatches, common_lisp_reader_conditional_forms,
    common_lisp_reader_label_dispatches, common_lisp_reader_label_forms,
    common_lisp_reader_literals,
};
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{ByteSpan, ExpressionView, ReaderPrefix, SyntaxTree};

/// Rejects mutations whose meaning depends on Common Lisp reader-time behavior.
///
/// Reader conditionals need a feature profile, reader labels preserve object
/// identity, and `#.` needs an evaluation environment. None is available to a
/// structural source transformation, so changing any part of such a document
/// would make a partially rewritten result unsafe to apply.
pub(crate) fn reject_common_lisp_reader_conditionals(
    tree: &SyntaxTree,
    dialect: Dialect,
) -> Result<(), ReaderConditionalSafetyError> {
    if dialect != Dialect::CommonLisp {
        return Ok(());
    }

    let Some(dispatch) = common_lisp_reader_conditional_dispatches(tree)
        .into_iter()
        .next()
    else {
        if let Some(dispatch) = common_lisp_reader_label_dispatches(tree).into_iter().next() {
            return Err(ReaderConditionalSafetyError::CommonLispReaderLabel {
                kind: dispatch.kind,
                span: dispatch.span,
            });
        }
        if let Some(literal) = common_lisp_reader_literals(tree).into_iter().next() {
            return Err(ReaderConditionalSafetyError::CommonLispReaderLiteral {
                kind: literal.kind,
                span: literal.span,
            });
        }
        if let Some(span) = first_read_time_evaluation(&tree.root_view()) {
            return Err(ReaderConditionalSafetyError::CommonLispReadTimeEvaluation { span });
        }
        return Ok(());
    };

    Err(ReaderConditionalSafetyError::CommonLispReaderConditional {
        kind: dispatch.kind,
        span: dispatch.span,
    })
}

/// Rejects only edits that overlap a Common Lisp reader-time form.
///
/// This is for transformations whose behavior depends solely on their explicit
/// target spans. Semantic transformations must keep using the document-wide
/// guard above because a reader conditional can change their lexical context.
pub(crate) fn reject_overlapping_common_lisp_reader_time_forms(
    tree: &SyntaxTree,
    dialect: Dialect,
    mutation_spans: impl IntoIterator<Item = ByteSpan>,
) -> Result<(), ReaderConditionalSafetyError> {
    if dialect != Dialect::CommonLisp {
        return Ok(());
    }

    let mutation_spans = mutation_spans.into_iter().collect::<Vec<_>>();
    for form in common_lisp_reader_conditional_forms(tree) {
        if mutation_spans
            .iter()
            .copied()
            .any(|span| spans_overlap(span, form.span))
        {
            return Err(ReaderConditionalSafetyError::CommonLispReaderConditional {
                kind: form.kind,
                span: form.dispatch_span,
            });
        }
    }

    for form in common_lisp_reader_label_forms(tree) {
        if mutation_spans
            .iter()
            .copied()
            .any(|span| spans_overlap(span, form.span))
        {
            return Err(ReaderConditionalSafetyError::CommonLispReaderLabel {
                kind: form.kind,
                span: form.dispatch_span,
            });
        }
    }

    for literal in common_lisp_reader_literals(tree) {
        if mutation_spans
            .iter()
            .copied()
            .any(|span| spans_overlap(span, literal.span))
        {
            return Err(ReaderConditionalSafetyError::CommonLispReaderLiteral {
                kind: literal.kind,
                span: literal.span,
            });
        }
    }

    if let Some(span) = first_overlapping_read_time_evaluation(&tree.root_view(), &mutation_spans) {
        return Err(ReaderConditionalSafetyError::CommonLispReadTimeEvaluation { span });
    }

    Ok(())
}

/// A mutation cannot safely proceed without a reader feature profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// Variants identify the Common Lisp reader construct that blocked the mutation.
#[allow(clippy::enum_variant_names)]
pub enum ReaderConditionalSafetyError {
    CommonLispReaderConditional {
        kind: CommonLispReaderConditionalKind,
        span: ByteSpan,
    },
    CommonLispReadTimeEvaluation {
        span: ByteSpan,
    },
    CommonLispReaderLabel {
        kind: CommonLispReaderLabelKind,
        span: ByteSpan,
    },
    CommonLispReaderLiteral {
        kind: CommonLispReaderLiteralKind,
        span: ByteSpan,
    },
}

impl fmt::Display for ReaderConditionalSafetyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CommonLispReaderConditional { kind, span } => write!(
                formatter,
                "cannot safely modify Common Lisp source containing reader conditional {} at byte {}",
                match kind {
                    CommonLispReaderConditionalKind::Include => "#+",
                    CommonLispReaderConditionalKind::Exclude => "#-",
                },
                span.start().get(),
            ),
            Self::CommonLispReadTimeEvaluation { span } => write!(
                formatter,
                "cannot safely modify Common Lisp source containing #. read-time evaluation at byte {}",
                span.start().get(),
            ),
            Self::CommonLispReaderLabel { kind, span } => write!(
                formatter,
                "cannot safely modify Common Lisp source containing reader label {} at byte {}",
                match kind {
                    CommonLispReaderLabelKind::Definition => "#n=",
                    CommonLispReaderLabelKind::Reference => "#n#",
                },
                span.start().get(),
            ),
            Self::CommonLispReaderLiteral { kind, span } => write!(
                formatter,
                "cannot safely modify Common Lisp source containing reader literal {} at byte {}",
                match kind {
                    CommonLispReaderLiteralKind::Vector => "#(...)",
                },
                span.start().get(),
            ),
        }
    }
}

impl Error for ReaderConditionalSafetyError {}

fn first_read_time_evaluation(view: &ExpressionView) -> Option<ByteSpan> {
    if view.reader_prefixes.contains(&ReaderPrefix::ReadEval) {
        return Some(view.span);
    }

    view.children.iter().find_map(first_read_time_evaluation)
}

fn first_overlapping_read_time_evaluation(
    view: &ExpressionView,
    mutation_spans: &[ByteSpan],
) -> Option<ByteSpan> {
    if view.reader_prefixes.contains(&ReaderPrefix::ReadEval)
        && mutation_spans
            .iter()
            .copied()
            .any(|span| spans_overlap(span, view.span))
    {
        return Some(view.span);
    }

    view.children
        .iter()
        .find_map(|child| first_overlapping_read_time_evaluation(child, mutation_spans))
}

fn spans_overlap(left: ByteSpan, right: ByteSpan) -> bool {
    left.start().get() < right.end().get() && right.start().get() < left.end().get()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_common_lisp_vector_literals_for_semantic_mutations() {
        let tree = SyntaxTree::parse("(render #(helper value))").expect("parse succeeds");

        let error = reject_common_lisp_reader_conditionals(&tree, Dialect::CommonLisp)
            .expect_err("vector literals must reject semantic mutations");

        assert!(matches!(
            error,
            ReaderConditionalSafetyError::CommonLispReaderLiteral {
                kind: CommonLispReaderLiteralKind::Vector,
                ..
            }
        ));
    }
}
