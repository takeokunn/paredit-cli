use crate::domain::sexpr::{Delimiter, ExpressionView};

pub(super) fn binding_form_binds_name(view: &ExpressionView, name: &str) -> bool {
    let Some(head) = list_head(view) else {
        return false;
    };

    match head {
        "let" | "let*" | "symbol-macrolet" => view
            .children
            .get(1)
            .is_some_and(|bindings| binding_pairs_contain_name(bindings, name)),
        "lambda" => view
            .children
            .get(1)
            .is_some_and(|parameters| pattern_contains_name(parameters, name)),
        "fn" => view
            .children
            .get(1)
            .filter(|parameters| parameters.delimiter == Some(Delimiter::Bracket))
            .is_some_and(|parameters| pattern_contains_name(parameters, name)),
        "defun" | "defmacro" | "defn" | "define-setf-expander" | "define-compiler-macro" => view
            .children
            .get(2)
            .is_some_and(|parameters| pattern_contains_name(parameters, name)),
        _ => false,
    }
}

fn binding_pairs_contain_name(bindings: &ExpressionView, name: &str) -> bool {
    if bindings.delimiter == Some(Delimiter::Bracket) {
        return bindings
            .children
            .iter()
            .step_by(2)
            .any(|binding| pattern_contains_name(binding, name));
    }

    bindings.children.iter().any(|binding| {
        binding
            .children
            .first()
            .is_some_and(|pattern| pattern_contains_name(pattern, name))
    })
}

fn pattern_contains_name(view: &ExpressionView, name: &str) -> bool {
    atom_text(view).map(|text| text == name).unwrap_or_else(|| {
        view.children
            .iter()
            .any(|child| pattern_contains_name(child, name))
    })
}

fn list_head(view: &ExpressionView) -> Option<&str> {
    view.children.first().and_then(atom_text)
}

fn atom_text(view: &ExpressionView) -> Option<&str> {
    view.text.as_deref()
}
