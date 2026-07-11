use anyhow::Result;

use crate::domain::sexpr::{ExpressionKind, ExpressionView, Path};

use super::{OptionSlot, PackageOptionSortOrder, ordering};
use crate::application::usecase::package::syntax::{atom_text, package_option_name};

pub(super) fn collect_option_slots(
    input: &str,
    view: &ExpressionView,
    defpackage_path: &Path,
    order: PackageOptionSortOrder,
) -> Result<Vec<OptionSlot>> {
    view.children
        .iter()
        .enumerate()
        .skip(2)
        .map(|(option_index, option)| {
            analyze_option_slot(input, option, defpackage_path, option_index, order)
        })
        .collect()
}

fn analyze_option_slot(
    input: &str,
    option: &ExpressionView,
    defpackage_path: &Path,
    option_index: usize,
    order: PackageOptionSortOrder,
) -> Result<OptionSlot> {
    if option.kind != ExpressionKind::List || option.children.is_empty() {
        anyhow::bail!(
            "cannot sort defpackage options at {}; only direct option lists are supported",
            defpackage_path
        );
    }
    let Some(option_head) = atom_text(&option.children[0]) else {
        anyhow::bail!(
            "cannot sort defpackage option at {}; option head must be an atom",
            defpackage_path.child(option_index)
        );
    };

    let name = package_option_name(option_head);
    let payload = option
        .children
        .iter()
        .skip(1)
        .find_map(atom_text)
        .unwrap_or("")
        .to_owned();
    let label = if payload.is_empty() {
        option_head.to_owned()
    } else {
        format!("{option_head} {payload}")
    };
    let text = option.span.slice(input).to_owned();
    let sort_key = ordering::option_sort_key(&name, &payload, &text, order);

    Ok(OptionSlot {
        span: option.span,
        text,
        label,
        sort_key,
    })
}
