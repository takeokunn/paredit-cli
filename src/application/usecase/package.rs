//! Use-case helpers for Common Lisp package refactorings.

use anyhow::{Context, Result};

use crate::domain::sexpr::SyntaxTree;

mod export;
mod merge_options;
mod rename;
mod rewrite;
mod sort_exports;
mod sort_options;
mod syntax;
mod types;

use export::find_defpackage_export_edit;
use merge_options::defpackage_option_merge_edits;
use rename::package_rename_occurrences;
use rewrite::{SpanReplacement, replace_span, rewrite_package_occurrences, rewrite_spans};
use sort_exports::defpackage_export_sort_edits;
use sort_options::defpackage_option_sort_edits;

pub use sort_options::PackageOptionSortOrder;
pub use types::*;

pub fn plan_add_export(request: AddExportRequest<'_>) -> Result<AddExportPlan> {
    let tree = SyntaxTree::parse(request.input).context("failed to parse input")?;
    let edit =
        find_defpackage_export_edit(&tree, request.dialect, request.package, request.symbol)?;
    let rewritten = if edit.already_exported {
        request.input.to_owned()
    } else {
        replace_span(request.input, edit.insertion_span, &edit.replacement)
    };
    SyntaxTree::parse(&rewritten)
        .context("add-export output is not a valid S-expression document")?;

    Ok(AddExportPlan {
        package: edit.package_name,
        symbol: request.symbol.clone(),
        defpackage_path: edit.defpackage_path,
        defpackage_span: edit.defpackage_span,
        export_span: edit.export_span,
        insertion_span: edit.insertion_span,
        already_exported: edit.already_exported,
        changed: rewritten != request.input,
        rewritten,
    })
}

pub fn plan_rename_package(request: RenamePackageRequest<'_>) -> Result<RenamePackagePlan> {
    let tree = SyntaxTree::parse(request.input).context("failed to parse input")?;
    let occurrences = package_rename_occurrences(&tree, request.dialect, request.from, request.to)?;
    let rewritten = rewrite_package_occurrences(request.input, &occurrences);
    SyntaxTree::parse(&rewritten)
        .context("rename-package output is not a valid S-expression document")?;

    Ok(RenamePackagePlan {
        changed: rewritten != request.input,
        occurrences,
        rewritten,
    })
}

pub fn plan_sort_package_exports(
    request: SortPackageExportsRequest<'_>,
) -> Result<SortPackageExportsPlan> {
    let tree = SyntaxTree::parse(request.input).context("failed to parse input")?;
    let edits = defpackage_export_sort_edits(&tree, request.dialect, request.package)?;
    let replacements = edits
        .iter()
        .flat_map(|edit| {
            edit.replacements.iter().map(|replacement| SpanReplacement {
                span: replacement.span,
                replacement: replacement.replacement.clone(),
            })
        })
        .collect::<Vec<_>>();
    let rewritten = rewrite_spans(request.input, &replacements);
    SyntaxTree::parse(&rewritten)
        .context("sort-package-exports output is not a valid S-expression document")?;

    let exports = edits
        .into_iter()
        .map(|edit| PackageExportSort {
            package: edit.package_name,
            defpackage_path: edit.defpackage_path,
            defpackage_span: edit.defpackage_span,
            export_path: edit.export_path,
            export_span: edit.export_span,
            changed: edit.old_symbols != edit.new_symbols,
            old_symbols: edit.old_symbols,
            new_symbols: edit.new_symbols,
        })
        .collect();

    Ok(SortPackageExportsPlan {
        changed: rewritten != request.input,
        exports,
        rewritten,
    })
}

pub fn plan_sort_package_options(
    request: SortPackageOptionsRequest<'_>,
) -> Result<SortPackageOptionsPlan> {
    let tree = SyntaxTree::parse(request.input).context("failed to parse input")?;
    let edits = defpackage_option_sort_edits(
        request.input,
        &tree,
        request.dialect,
        request.package,
        request.order,
    )?;
    let replacements = edits
        .iter()
        .flat_map(|edit| {
            edit.replacements.iter().map(|replacement| SpanReplacement {
                span: replacement.span,
                replacement: replacement.replacement.clone(),
            })
        })
        .collect::<Vec<_>>();
    let rewritten = rewrite_spans(request.input, &replacements);
    SyntaxTree::parse(&rewritten)
        .context("sort-package-options output is not a valid S-expression document")?;

    let packages = edits
        .into_iter()
        .map(|edit| PackageOptionSort {
            package: edit.package_name,
            defpackage_path: edit.defpackage_path,
            defpackage_span: edit.defpackage_span,
            changed: edit.old_options != edit.new_options,
            old_options: edit.old_options,
            new_options: edit.new_options,
        })
        .collect();

    Ok(SortPackageOptionsPlan {
        changed: rewritten != request.input,
        packages,
        rewritten,
    })
}

pub fn plan_merge_package_options(
    request: MergePackageOptionsRequest<'_>,
) -> Result<MergePackageOptionsPlan> {
    let tree = SyntaxTree::parse(request.input).context("failed to parse input")?;
    let edits =
        defpackage_option_merge_edits(request.input, &tree, request.dialect, request.package)?;
    let replacements = edits
        .iter()
        .flat_map(|edit| {
            edit.replacements.iter().map(|replacement| SpanReplacement {
                span: replacement.span,
                replacement: replacement.replacement.clone(),
            })
        })
        .collect::<Vec<_>>();
    let rewritten = rewrite_spans(request.input, &replacements);
    SyntaxTree::parse(&rewritten)
        .context("merge-package-options output is not a valid S-expression document")?;

    let merges = edits
        .into_iter()
        .flat_map(|edit| {
            edit.merges.into_iter().map(move |merge| {
                let changed = !merge.removed_spans.is_empty() || merge.old_atoms != merge.new_atoms;
                PackageOptionMerge {
                    package: edit.package_name.clone(),
                    defpackage_path: edit.defpackage_path.clone(),
                    defpackage_span: edit.defpackage_span,
                    head: merge.head,
                    key: merge.key,
                    kept_path: merge.kept_path,
                    kept_span: merge.kept_span,
                    removed_paths: merge.removed_paths,
                    removed_spans: merge.removed_spans,
                    changed,
                    old_atoms: merge.old_atoms,
                    new_atoms: merge.new_atoms,
                }
            })
        })
        .collect::<Vec<_>>();

    Ok(MergePackageOptionsPlan {
        changed: rewritten != request.input,
        merges,
        rewritten,
    })
}

#[cfg(test)]
mod tests;
