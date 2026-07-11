use crate::domain::sexpr::ByteSpan;

use super::{OptionReplacement, OptionSlot, PackageOptionSortOrder};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::application::usecase::package) struct OptionSortKey {
    rank: usize,
    name: String,
    payload: String,
    text: String,
}

/// Separator used when the option list's original first slot (which has no
/// leading trivia of its own) is reordered away from the front.
const DEFAULT_OPTION_SEPARATOR: &str = "\n  ";

pub(super) fn sort_slots(slots: &[OptionSlot]) -> (Vec<String>, Vec<OptionReplacement>) {
    let mut order = (0..slots.len()).collect::<Vec<_>>();
    order.sort_by(|&left, &right| {
        slots[left]
            .sort_key
            .cmp(&slots[right].sort_key)
            .then_with(|| slots[left].full_text.cmp(&slots[right].full_text))
    });

    let new_options = order
        .iter()
        .map(|&index| slots[index].label.clone())
        .collect::<Vec<_>>();

    let replacements = if is_identity(&order) {
        Vec::new()
    } else {
        let region = ByteSpan::new(
            slots[0].full_span.start(),
            slots[slots.len() - 1].full_span.end(),
        );

        let mut replacement = String::new();
        for (target_offset, &source_index) in order.iter().enumerate() {
            let slot = &slots[source_index];
            // The option list's original first slot carries no leading
            // trivia of its own. If it lands anywhere but the front after
            // reordering, it needs a separator so it doesn't glue onto the
            // previous option's closing delimiter.
            if target_offset > 0 && !slot.has_leading_trivia {
                replacement.push_str(DEFAULT_OPTION_SEPARATOR);
            }
            replacement.push_str(&slot.full_text);
        }
        // When some other slot displaces the option list's original first
        // slot from the front, that slot's own leading trivia (a blank run,
        // possibly with a comment) ends up at the top of the region, where
        // there is nothing left for it to separate from. Drop just the
        // blank-line run while leaving a leading comment in place.
        let replacement = if order[0] == 0 {
            replacement
        } else {
            strip_leading_blank_lines(&replacement)
        };

        vec![OptionReplacement {
            span: region,
            replacement,
        }]
    };

    (new_options, replacements)
}

fn is_identity(order: &[usize]) -> bool {
    order
        .iter()
        .enumerate()
        .all(|(index, &value)| index == value)
}

/// Drops leading lines that are empty or all-whitespace, returning the
/// remainder starting at the first line with content (a comment or an
/// option form). A comment's own indentation is left untouched.
///
/// A leading newline is always a slot-boundary marker (the newline that used
/// to end the previous option's line), not a blank line by itself, so it is
/// dropped unconditionally before the remainder is scanned for genuine blank
/// lines.
fn strip_leading_blank_lines(text: &str) -> String {
    let text = text.strip_prefix('\n').unwrap_or(text);
    let bytes = text.as_bytes();
    let mut cursor = 0;
    loop {
        let line_end = bytes[cursor..]
            .iter()
            .position(|&byte| byte == b'\n')
            .map(|offset| cursor + offset);
        match line_end {
            Some(end) if text[cursor..end].trim().is_empty() => cursor = end + 1,
            _ => break,
        }
    }
    text[cursor..].to_owned()
}

pub(super) fn option_sort_key(
    name: &str,
    payload: &str,
    text: &str,
    order: PackageOptionSortOrder,
) -> OptionSortKey {
    OptionSortKey {
        rank: option_rank(name, order),
        name: name.to_owned(),
        payload: normalized_atom(payload),
        text: text.to_ascii_lowercase(),
    }
}

fn option_rank(name: &str, order: PackageOptionSortOrder) -> usize {
    if order == PackageOptionSortOrder::Name {
        return 0;
    }

    match name {
        "nicknames" | "documentation" => 0,
        "use" => 10,
        "shadow" => 20,
        "shadowing-import-from" => 30,
        "import-from" => 40,
        "local-nicknames" => 50,
        "export" => 60,
        "intern" => 70,
        _ => 100,
    }
}

fn normalized_atom(value: &str) -> String {
    value
        .strip_prefix("#:")
        .or_else(|| value.strip_prefix(':'))
        .unwrap_or(value)
        .to_ascii_lowercase()
}
