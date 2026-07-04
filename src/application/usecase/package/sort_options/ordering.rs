use super::{OptionReplacement, OptionSlot, PackageOptionSortOrder};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(in crate::application::usecase::package) struct OptionSortKey {
    rank: usize,
    name: String,
    payload: String,
    text: String,
}

pub(super) fn sort_slots(slots: &[OptionSlot]) -> (Vec<String>, Vec<OptionReplacement>) {
    let mut sorted_slots = slots.to_vec();
    sorted_slots.sort_by(|left, right| {
        left.sort_key
            .cmp(&right.sort_key)
            .then_with(|| left.text.cmp(&right.text))
    });

    let new_options = sorted_slots
        .iter()
        .map(|slot| slot.label.clone())
        .collect::<Vec<_>>();
    let replacements = slots
        .iter()
        .zip(sorted_slots)
        .filter_map(|(old_slot, new_slot)| {
            (old_slot.text != new_slot.text).then(|| OptionReplacement {
                span: old_slot.span,
                replacement: new_slot.text,
            })
        })
        .collect();

    (new_options, replacements)
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
