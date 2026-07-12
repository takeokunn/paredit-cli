use crate::domain::sexpr::{ExpressionKind, ExpressionView};

pub(crate) fn ordinary_lambda_list_bindings(form: &ExpressionView) -> Vec<String> {
    form.children
        .get(1)
        .map(lambda_list_bindings)
        .unwrap_or_default()
}

pub(crate) fn macro_lambda_list_bindings(form: &ExpressionView) -> Vec<String> {
    form.children
        .get(1)
        .map(lambda_list_bindings)
        .unwrap_or_default()
}

pub(crate) fn destructuring_lambda_list_bindings(pattern: &ExpressionView) -> Vec<String> {
    lambda_list_bindings(pattern)
}

fn lambda_list_bindings(lambda_list: &ExpressionView) -> Vec<String> {
    let mut names = Vec::new();
    let mut section = LambdaListSection::Required;
    for parameter in &lambda_list.children {
        if let Some(keyword) = atom_text(parameter).filter(|name| name.starts_with('&')) {
            section = LambdaListSection::from_keyword(keyword, section);
            continue;
        }
        match section {
            LambdaListSection::Required => push_pattern_bindings(parameter, &mut names),
            LambdaListSection::Optional | LambdaListSection::Aux => {
                push_optional_bindings(parameter, &mut names)
            }
            LambdaListSection::Rest | LambdaListSection::Whole | LambdaListSection::Environment => {
                push_pattern_bindings(parameter, &mut names);
                section = LambdaListSection::Ignored;
            }
            LambdaListSection::Key => push_key_bindings(parameter, &mut names),
            LambdaListSection::Ignored => {}
        }
    }
    names
}

#[derive(Clone, Copy)]
enum LambdaListSection {
    Required,
    Optional,
    Rest,
    Key,
    Aux,
    Whole,
    Environment,
    Ignored,
}

impl LambdaListSection {
    fn from_keyword(keyword: &str, current: Self) -> Self {
        if keyword.eq_ignore_ascii_case("&optional") {
            Self::Optional
        } else if keyword.eq_ignore_ascii_case("&rest") || keyword.eq_ignore_ascii_case("&body") {
            Self::Rest
        } else if keyword.eq_ignore_ascii_case("&key") {
            Self::Key
        } else if keyword.eq_ignore_ascii_case("&aux") {
            Self::Aux
        } else if keyword.eq_ignore_ascii_case("&whole") {
            Self::Whole
        } else if keyword.eq_ignore_ascii_case("&environment") {
            Self::Environment
        } else if keyword.eq_ignore_ascii_case("&allow-other-keys") {
            current
        } else {
            Self::Ignored
        }
    }
}

fn push_optional_bindings(parameter: &ExpressionView, names: &mut Vec<String>) {
    if parameter.kind == ExpressionKind::Atom {
        push_pattern_bindings(parameter, names);
        return;
    }
    push_pattern_bindings_at(parameter, 0, names);
    push_simple_binding_at(parameter, 2, names);
}

fn push_key_bindings(parameter: &ExpressionView, names: &mut Vec<String>) {
    if parameter.kind == ExpressionKind::Atom {
        push_pattern_bindings(parameter, names);
        return;
    }
    let Some(primary) = parameter.children.first() else {
        return;
    };
    if primary.kind == ExpressionKind::List {
        push_pattern_bindings_at(primary, 1, names);
    } else {
        push_pattern_bindings(primary, names);
    }
    push_simple_binding_at(parameter, 2, names);
}

fn push_pattern_bindings_at(parameter: &ExpressionView, index: usize, names: &mut Vec<String>) {
    if let Some(binding) = parameter.children.get(index) {
        push_pattern_bindings(binding, names);
    }
}

fn push_simple_binding_at(parameter: &ExpressionView, index: usize, names: &mut Vec<String>) {
    if let Some(binding) = parameter.children.get(index) {
        push_simple_binding(binding, names);
    }
}

fn push_pattern_bindings(parameter: &ExpressionView, names: &mut Vec<String>) {
    if parameter.kind == ExpressionKind::List {
        names.extend(lambda_list_bindings(parameter));
    } else {
        push_simple_binding(parameter, names);
    }
}

fn push_simple_binding(parameter: &ExpressionView, names: &mut Vec<String>) {
    if let Some(name) = atom_text(parameter).filter(|name| !name.starts_with('&')) {
        names.push(name.to_owned());
    }
}

fn atom_text(view: &ExpressionView) -> Option<&str> {
    (view.kind == ExpressionKind::Atom)
        .then_some(view.text.as_deref())
        .flatten()
}
