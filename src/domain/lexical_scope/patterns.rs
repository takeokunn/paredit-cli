use crate::domain::sexpr::{Delimiter, ExpressionKind, ExpressionView};

use super::syntax::atom_text;

pub(super) fn binding_pattern_names(pattern: &ExpressionView) -> Vec<String> {
    let mut names = Vec::new();
    collect_binding_pattern_names(pattern, &mut names);
    names
}

pub(super) fn lambda_list_names(parameter_form: &ExpressionView) -> Vec<String> {
    let mut names = Vec::new();
    collect_lambda_list_names(parameter_form, &mut names);
    names
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum LambdaListMode {
    Required,
    Optional,
    Key,
    Aux,
}

fn collect_lambda_list_names(parameter_form: &ExpressionView, output: &mut Vec<String>) {
    let mut mode = LambdaListMode::Required;
    let mut index = 0usize;

    while index < parameter_form.children.len() {
        let child = &parameter_form.children[index];
        if let Some(marker) = atom_text(child) {
            match marker {
                "&optional" => {
                    mode = LambdaListMode::Optional;
                    index += 1;
                    continue;
                }
                "&key" => {
                    mode = LambdaListMode::Key;
                    index += 1;
                    continue;
                }
                "&aux" => {
                    mode = LambdaListMode::Aux;
                    index += 1;
                    continue;
                }
                "&rest" | "&body" | "&whole" | "&environment" => {
                    if let Some(next) = parameter_form.children.get(index + 1) {
                        collect_binding_pattern_names(next, output);
                    }
                    index += 2;
                    continue;
                }
                "&allow-other-keys" => {
                    index += 1;
                    continue;
                }
                _ if marker.starts_with('&') => {
                    index += 1;
                    continue;
                }
                _ => {}
            }
        }

        collect_lambda_list_parameter_spec_names(child, mode, output);
        index += 1;
    }
}

fn collect_lambda_list_parameter_spec_names(
    spec: &ExpressionView,
    mode: LambdaListMode,
    output: &mut Vec<String>,
) {
    if atom_text(spec).is_some() || mode == LambdaListMode::Required {
        collect_binding_pattern_names(spec, output);
        return;
    }

    if spec.kind != ExpressionKind::List || spec.children.is_empty() {
        return;
    }

    match mode {
        LambdaListMode::Required => collect_binding_pattern_names(spec, output),
        LambdaListMode::Optional => {
            collect_binding_pattern_names(&spec.children[0], output);
            collect_supplied_p_name(spec, output);
        }
        LambdaListMode::Key => {
            collect_key_parameter_name(&spec.children[0], output);
            collect_supplied_p_name(spec, output);
        }
        LambdaListMode::Aux => collect_binding_pattern_names(&spec.children[0], output),
    }
}

fn collect_key_parameter_name(spec_name: &ExpressionView, output: &mut Vec<String>) {
    if spec_name.kind == ExpressionKind::List && spec_name.children.len() >= 2 {
        if let Some(designator) = atom_text(&spec_name.children[0]) {
            if designator.starts_with(':') {
                collect_binding_pattern_names(&spec_name.children[1], output);
                return;
            }
        }
    }

    collect_binding_pattern_names(spec_name, output);
}

fn collect_supplied_p_name(spec: &ExpressionView, output: &mut Vec<String>) {
    if let Some(supplied_p) = spec.children.get(2) {
        collect_binding_pattern_names(supplied_p, output);
    }
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
