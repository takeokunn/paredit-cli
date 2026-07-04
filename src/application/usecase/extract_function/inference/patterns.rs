use crate::domain::sexpr::ExpressionView;

use super::super::syntax::atom_text;
use super::symbols::is_extract_function_param_candidate;

pub(super) fn parameter_names(parameter_form: &ExpressionView) -> Vec<String> {
    parameter_form
        .children
        .iter()
        .flat_map(extract_function_pattern_names)
        .collect()
}

pub(super) fn extract_function_pattern_names(pattern: &ExpressionView) -> Vec<String> {
    let mut names = Vec::new();
    collect_extract_function_pattern_names(pattern, &mut names);
    names
}

fn collect_extract_function_pattern_names(pattern: &ExpressionView, names: &mut Vec<String>) {
    if let Some(text) = atom_text(pattern) {
        if text != "_"
            && is_extract_function_param_candidate(text)
            && !names.iter().any(|name| name == text)
        {
            names.push(text.to_owned());
        }
        return;
    }

    for child in &pattern.children {
        collect_extract_function_pattern_names(child, names);
    }
}
