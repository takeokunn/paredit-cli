use crate::domain::sexpr::{Delimiter, ExpressionKind, ExpressionView};

use super::super::selection::atom_text;
use super::types::{BindingEdit, ParameterNameSpan};

pub(super) fn binding_pattern_name_spans(
    pattern: &ExpressionView,
    input: &str,
) -> Vec<ParameterNameSpan> {
    let mut names = Vec::new();
    collect_binding_pattern_name_spans(pattern, input, &mut names);
    names
}

fn collect_binding_pattern_name_spans(
    pattern: &ExpressionView,
    input: &str,
    output: &mut Vec<ParameterNameSpan>,
) {
    if let Some(name) = atom_text(pattern) {
        if !is_binding_pattern_marker(name) {
            output.push(ParameterNameSpan {
                name: name.to_owned(),
                name_span: pattern.span,
                binding_edit: BindingEdit::rename_atom(pattern.span),
            });
        }
        return;
    }

    let mut index = 0usize;
    while index < pattern.children.len() {
        let child = &pattern.children[index];
        if let Some(marker) = atom_text(child) {
            if marker == ":keys" {
                if let Some(keys_form) = pattern.children.get(index + 1) {
                    collect_clojure_keys_shorthand_name_spans(pattern, keys_form, output);
                }
                index += 2;
                continue;
            }
            if matches!(marker, ":strs" | ":syms") {
                index += 2;
                continue;
            }
            if marker == ":as" {
                if let Some(next) = pattern.children.get(index + 1) {
                    collect_binding_pattern_name_spans(next, input, output);
                    index += 2;
                    continue;
                }
            }
        }
        collect_binding_pattern_name_spans(child, input, output);
        index += 1;
    }
}

fn collect_clojure_keys_shorthand_name_spans(
    map_pattern: &ExpressionView,
    keys_form: &ExpressionView,
    output: &mut Vec<ParameterNameSpan>,
) {
    if keys_form.kind != ExpressionKind::List || keys_form.delimiter != Some(Delimiter::Bracket) {
        return;
    }

    for key in &keys_form.children {
        let Some(name) = atom_text(key) else {
            continue;
        };
        if is_binding_pattern_marker(name) {
            continue;
        }
        output.push(ParameterNameSpan {
            name: name.to_owned(),
            name_span: key.span,
            binding_edit: BindingEdit::clojure_keys_map(
                map_pattern.clone(),
                map_pattern.span,
                name.to_owned(),
            ),
        });
    }
}

fn is_binding_pattern_marker(name: &str) -> bool {
    name == "_" || name.starts_with('&') || name.starts_with(':')
}
