use crate::domain::sexpr::{Delimiter, ExpressionKind, ExpressionView};

pub fn duplicate_shape(view: &ExpressionView, preserve_list_head: bool) -> String {
    let mut output = String::new();
    write_duplicate_shape(view, preserve_list_head, &mut output);
    output
}

// Writes into one shared buffer instead of allocating a `Vec<String>` and
// joining at every recursion level, which made shape construction cost
// O(subtree size × depth) allocations for deeply nested forms.
fn write_duplicate_shape(view: &ExpressionView, preserve_list_head: bool, output: &mut String) {
    match view.kind {
        ExpressionKind::Root => {
            output.push_str("(root ");
            for (index, child) in view.children.iter().enumerate() {
                if index > 0 {
                    output.push(' ');
                }
                write_duplicate_shape(child, false, output);
            }
            output.push(')');
        }
        ExpressionKind::Atom => output.push_str("_atom"),
        ExpressionKind::List => {
            let delimiter = match view.delimiter {
                Some(Delimiter::Paren) => "paren",
                Some(Delimiter::Bracket) => "bracket",
                Some(Delimiter::Brace) => "brace",
                None => "list",
            };
            output.push('(');
            output.push_str(delimiter);
            output.push(' ');
            for (index, child) in view.children.iter().enumerate() {
                if index > 0 {
                    output.push(' ');
                }
                if preserve_list_head && index == 0 {
                    match atom_text(child) {
                        Some(head) => {
                            output.push_str("head:");
                            output.push_str(head);
                        }
                        None => write_duplicate_shape(child, false, output),
                    }
                } else {
                    write_duplicate_shape(child, false, output);
                }
            }
            output.push(')');
        }
    }
}

fn atom_text(view: &ExpressionView) -> Option<&str> {
    (view.kind == ExpressionKind::Atom)
        .then_some(view.text.as_deref())
        .flatten()
}
