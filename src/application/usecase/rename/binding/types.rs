use crate::domain::sexpr::{ByteSpan, ExpressionView, SymbolName};

use super::rewrite::rewrite_clojure_keys_map_pattern;

#[derive(Debug, Clone)]
pub(in crate::application::usecase::rename) struct BindingRenameParts {
    pub(in crate::application::usecase::rename) form: String,
    pub(in crate::application::usecase::rename) form_span: ByteSpan,
    pub(in crate::application::usecase::rename) binding_span: ByteSpan,
    pub(in crate::application::usecase::rename) binding_edit: BindingEdit,
    pub(in crate::application::usecase::rename) reference_spans: Vec<ByteSpan>,
    pub(in crate::application::usecase::rename) shadowed_scope_count: usize,
}

#[derive(Debug, Clone)]
pub(super) struct BindingGroup {
    pub(super) names: Vec<ParameterNameSpan>,
    pub(super) value: Option<ExpressionView>,
}

#[derive(Debug, Clone)]
pub(super) struct ParameterNameSpan {
    pub(super) name: String,
    pub(super) name_span: ByteSpan,
    pub(super) binding_edit: BindingEdit,
}

#[derive(Debug, Clone)]
pub(in crate::application::usecase::rename) struct BindingEdit {
    pub(in crate::application::usecase::rename) span: ByteSpan,
    kind: BindingEditKind,
}

#[derive(Debug, Clone)]
enum BindingEditKind {
    RenameAtom,
    RewriteBareSlotSpec {
        slot_name: String,
    },
    RewriteClojureKeysMap {
        map_pattern: ExpressionView,
        renamed_name: String,
    },
}

impl BindingEdit {
    pub(super) fn rename_atom(span: ByteSpan) -> Self {
        Self {
            span,
            kind: BindingEditKind::RenameAtom,
        }
    }

    pub(super) fn bare_slot_spec(span: ByteSpan, slot_name: String) -> Self {
        Self {
            span,
            kind: BindingEditKind::RewriteBareSlotSpec { slot_name },
        }
    }

    pub(super) fn clojure_keys_map(
        map_pattern: ExpressionView,
        span: ByteSpan,
        renamed_name: String,
    ) -> Self {
        Self {
            span,
            kind: BindingEditKind::RewriteClojureKeysMap {
                map_pattern,
                renamed_name,
            },
        }
    }

    pub(in crate::application::usecase::rename) fn replacement(
        &self,
        input: &str,
        to: &SymbolName,
    ) -> String {
        match &self.kind {
            BindingEditKind::RenameAtom => to.as_str().to_owned(),
            BindingEditKind::RewriteBareSlotSpec { slot_name } => {
                format!("({} {})", to.as_str(), slot_name)
            }
            BindingEditKind::RewriteClojureKeysMap {
                map_pattern,
                renamed_name,
            } => rewrite_clojure_keys_map_pattern(input, map_pattern, renamed_name, to),
        }
    }
}
