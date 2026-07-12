use crate::domain::sexpr::{ByteSpan, SyntaxTree};

pub(super) fn is_common_lisp_value_position(tree: &SyntaxTree, span: ByteSpan) -> bool {
    tree.atom_occurrences()
        .into_iter()
        .find(|occurrence| occurrence.span == span)
        .is_some_and(|occurrence| occurrence.path.to_raw_indexes().last() != Some(&0))
}
