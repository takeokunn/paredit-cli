use crate::domain::sexpr::{Delimiter, ExpressionKind, ExpressionView, ReaderPrefix, SyntaxTree};

use super::{CommonLispReaderLiteral, CommonLispReaderLiteralKind};

/// Returns reader-created literal datums that must remain opaque to refactors.
///
/// The parser represents `#(...)` as a parenthesized list with a hash prefix,
/// but Common Lisp reads it as a vector. Its children are data, not calls or
/// symbol references, so semantic transformations must not rewrite them.
pub fn common_lisp_reader_literals(tree: &SyntaxTree) -> Vec<CommonLispReaderLiteral> {
    let mut literals = Vec::new();
    collect_reader_literals(&tree.root_view(), &mut literals);
    literals
}

fn collect_reader_literals(view: &ExpressionView, literals: &mut Vec<CommonLispReaderLiteral>) {
    if view.kind == ExpressionKind::List
        && view.delimiter == Some(Delimiter::Paren)
        && view.reader_prefixes.contains(&ReaderPrefix::HashLiteral)
    {
        literals.push(CommonLispReaderLiteral {
            kind: CommonLispReaderLiteralKind::Vector,
            span: view.span,
        });
    }

    for child in &view.children {
        collect_reader_literals(child, literals);
    }
}
