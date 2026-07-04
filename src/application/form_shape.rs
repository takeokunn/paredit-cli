use crate::domain::sexpr::{Delimiter, ExpressionKind, ExpressionView};

pub fn duplicate_shape(view: &ExpressionView, preserve_list_head: bool) -> String {
    match view.kind {
        ExpressionKind::Root => format!(
            "(root {})",
            view.children
                .iter()
                .map(|child| duplicate_shape(child, false))
                .collect::<Vec<_>>()
                .join(" ")
        ),
        ExpressionKind::Atom => "_atom".to_owned(),
        ExpressionKind::List => {
            let delimiter = match view.delimiter {
                Some(Delimiter::Paren) => "paren",
                Some(Delimiter::Bracket) => "bracket",
                Some(Delimiter::Brace) => "brace",
                None => "list",
            };
            let children = view
                .children
                .iter()
                .enumerate()
                .map(|(index, child)| {
                    if preserve_list_head && index == 0 {
                        atom_text(child)
                            .map(|head| format!("head:{head}"))
                            .unwrap_or_else(|| duplicate_shape(child, false))
                    } else {
                        duplicate_shape(child, false)
                    }
                })
                .collect::<Vec<_>>()
                .join(" ");

            format!("({delimiter} {children})")
        }
    }
}

fn atom_text(view: &ExpressionView) -> Option<&str> {
    (view.kind == ExpressionKind::Atom)
        .then_some(view.text.as_deref())
        .flatten()
}
