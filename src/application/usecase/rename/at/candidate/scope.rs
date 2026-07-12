use anyhow::Result;

use super::super::RenameAtNamespace;
use crate::domain::common_lisp::common_lisp_operator_head_eq;
use crate::domain::sexpr::{ByteSpan, ExpressionKind, Path, SyntaxTree};

pub(super) fn enclosing_specialized_scope(
    tree: &SyntaxTree,
    path: &Path,
    namespace: RenameAtNamespace,
) -> Result<Option<ByteSpan>> {
    let indexes = path.to_raw_indexes();
    for end in (1..indexes.len()).rev() {
        let ancestor = Path::from_indexes(indexes[..end].to_vec());
        let view = tree.select_path(&ancestor)?.view();
        if view.kind != ExpressionKind::List {
            continue;
        }
        let Some(head) = view
            .children
            .first()
            .and_then(|child| child.text.as_deref())
        else {
            continue;
        };
        let matches = match namespace {
            RenameAtNamespace::LocalFunction => {
                common_lisp_operator_head_eq(head, "flet")
                    || common_lisp_operator_head_eq(head, "labels")
            }
            RenameAtNamespace::Macro => {
                common_lisp_operator_head_eq(head, "macrolet")
                    || common_lisp_operator_head_eq(head, "compiler-macrolet")
            }
            RenameAtNamespace::SymbolMacro => common_lisp_operator_head_eq(head, "symbol-macrolet"),
            _ => false,
        };
        let descendants = &indexes[end..];
        let definition_body_without_self_scope = descendants.len() >= 3
            && descendants[0] == 1
            && descendants[2] >= 2
            && (common_lisp_operator_head_eq(head, "flet")
                || common_lisp_operator_head_eq(head, "macrolet")
                || common_lisp_operator_head_eq(head, "compiler-macrolet"));
        let symbol_macro_expansion = descendants.len() >= 3
            && descendants[0] == 1
            && descendants[2] >= 1
            && common_lisp_operator_head_eq(head, "symbol-macrolet");
        if matches && !definition_body_without_self_scope && !symbol_macro_expansion {
            return Ok(Some(view.span));
        }
    }
    Ok(None)
}

pub(super) fn occurrence_has_scope(
    tree: &SyntaxTree,
    span: ByteSpan,
    namespace: RenameAtNamespace,
    expected: Option<ByteSpan>,
) -> bool {
    let Some(path) = tree
        .atom_occurrences()
        .into_iter()
        .find(|occurrence| occurrence.span == span)
        .map(|occurrence| occurrence.path)
    else {
        return false;
    };
    enclosing_specialized_scope(tree, &path, namespace).ok() == Some(expected)
}
