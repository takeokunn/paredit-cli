use anyhow::Result;

use super::super::RenameAtNamespace;
use super::super::safety::ensure_binding_target_is_available;
use super::super::selection::is_common_lisp_value_position;
use super::Candidate;
use crate::domain::dialect::Dialect;
use crate::domain::rename::{binding_rename_parts, selection::apply_byte_span_edits};
use crate::domain::sexpr::{Path, SymbolName, SyntaxTree};

pub(in crate::domain::rename::at) fn binding_candidates(
    tree: &SyntaxTree,
    input: &str,
    path: &Path,
    from: &SymbolName,
    to: &SymbolName,
) -> Result<Vec<Candidate>> {
    let indexes = path.to_raw_indexes();
    let selected_span = tree.select_path(path)?.span();
    let mut candidates = Vec::new();
    for end in (1..indexes.len()).rev() {
        let ancestor = Path::from_indexes(indexes[..end].to_vec());
        let view = tree.select_path(&ancestor)?.view();
        let Ok(parts) = binding_rename_parts(Dialect::CommonLisp, &view, from, input) else {
            continue;
        };
        let reference_spans: Vec<_> = parts
            .reference_spans
            .iter()
            .copied()
            .filter(|span| is_common_lisp_value_position(tree, *span))
            .collect();
        if parts.binding_span != selected_span && !reference_spans.contains(&selected_span) {
            continue;
        }
        ensure_binding_target_is_available(&view, from, to, parts.binding_span, input)?;
        let mut occurrences = vec![parts.binding_span];
        occurrences.extend(reference_spans.iter().copied());
        let mut edits = vec![(
            parts.binding_edit.span,
            parts.binding_edit.replacement(input, to),
        )];
        edits.extend(
            reference_spans
                .iter()
                .map(|span| (*span, to.as_str().to_owned())),
        );
        candidates.push(Candidate {
            namespace: RenameAtNamespace::Value,
            occurrences,
            rewritten: apply_byte_span_edits(input, edits)?,
        });
        break;
    }
    Ok(candidates)
}
