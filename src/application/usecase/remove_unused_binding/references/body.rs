use crate::domain::dialect::Dialect;
use crate::domain::lexical_scope::collect_unshadowed_symbol_references;
use crate::domain::sexpr::{ByteSpan, ExpressionView, SymbolName};

pub(super) fn body_binding_reference_spans(
    dialect: Dialect,
    input: &str,
    target: &ExpressionView,
    name: &SymbolName,
    body_start_index: usize,
) -> Vec<ByteSpan> {
    let mut reference_spans = Vec::new();
    for body in target.children.iter().skip(body_start_index) {
        collect_unshadowed_symbol_references(dialect, body, name, input, &mut reference_spans);
    }
    reference_spans
}
