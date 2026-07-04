use anyhow::Result;

use crate::domain::sexpr::{Path, SymbolName, SyntaxTree};

use super::PackageRenameOccurrence;

mod occurrences;
mod paths;
mod replacement;

use occurrences::collect_package_rename_occurrences;

pub(super) fn package_rename_occurrences(
    tree: &SyntaxTree,
    from: &SymbolName,
    to: &SymbolName,
) -> Result<Vec<PackageRenameOccurrence>> {
    let mut occurrences = Vec::new();

    for index in 0..tree.root_children().len() {
        let path_indexes = vec![index];
        let path = Path::from_indexes(path_indexes.clone());
        let view = tree.select_path(&path)?.view();
        collect_package_rename_occurrences(&view, path_indexes, from, to, &mut occurrences);
    }

    occurrences.sort_by_key(|occurrence| occurrence.span.start());
    Ok(occurrences)
}
