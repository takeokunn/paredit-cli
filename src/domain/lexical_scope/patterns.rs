use crate::domain::sexpr::{Delimiter, ExpressionKind, ExpressionView};

use super::syntax::atom_text;

pub(super) fn binding_pattern_names(pattern: &ExpressionView) -> Vec<String> {
    let mut names = Vec::new();
    collect_binding_pattern_names(pattern, &mut names);
    names
}

fn collect_binding_pattern_names(pattern: &ExpressionView, output: &mut Vec<String>) {
    if let Some(name) = atom_text(pattern) {
        if !is_binding_pattern_marker(name) {
            output.push(name.to_owned());
        }
        return;
    }

    let mut index = 0usize;
    while index < pattern.children.len() {
        let child = &pattern.children[index];
        if let Some(marker) = atom_text(child) {
            if marker == ":keys" {
                if let Some(keys_form) = pattern.children.get(index + 1) {
                    collect_clojure_keys_shorthand_names(keys_form, output);
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
                    collect_binding_pattern_names(next, output);
                    index += 2;
                    continue;
                }
            }
        }
        collect_binding_pattern_names(child, output);
        index += 1;
    }
}

fn collect_clojure_keys_shorthand_names(keys_form: &ExpressionView, output: &mut Vec<String>) {
    if keys_form.kind != ExpressionKind::List || keys_form.delimiter != Some(Delimiter::Bracket) {
        return;
    }

    for key in &keys_form.children {
        let Some(name) = atom_text(key) else {
            continue;
        };
        if !is_binding_pattern_marker(name) {
            output.push(name.to_owned());
        }
    }
}

fn is_binding_pattern_marker(name: &str) -> bool {
    name == "_" || name.starts_with('&') || name.starts_with(':')
}
