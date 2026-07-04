use super::*;
use serde_json::json;

pub(super) fn print_merge_package_options_plan(plan: &MergePackageOptionsPlan) -> Result<()> {
    let merge_count = plan.merges.len();
    let changed_merge_count = plan.merges.iter().filter(|merge| merge.changed).count();

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "path": plan.path.display().to_string(),
            "dialect": plan.dialect.label(),
            "merge_count": merge_count,
            "changed_merge_count": changed_merge_count,
            "changed": plan.changed,
            "written": plan.written,
            "merges": plan
                .merges
                .iter()
                .map(|merge| json!({
                    "package": merge.package.as_str(),
                    "defpackage": {
                        "path": merge.defpackage_path.as_str(),
                        "span": {
                            "start": merge.defpackage_span.start().get(),
                            "end": merge.defpackage_span.end().get(),
                        },
                    },
                    "head": merge.head.as_str(),
                    "key": merge.key.as_deref(),
                    "kept": {
                        "path": merge.kept_path.as_str(),
                        "span": {
                            "start": merge.kept_span.start().get(),
                            "end": merge.kept_span.end().get(),
                        },
                    },
                    "removed": merge
                        .removed_paths
                        .iter()
                        .zip(merge.removed_spans.iter())
                        .map(|(path, span)| json!({
                            "path": path.as_str(),
                            "span": {
                                "start": span.start().get(),
                                "end": span.end().get(),
                            },
                        }))
                        .collect::<Vec<_>>(),
                    "old_atoms": merge.old_atoms.as_slice(),
                    "new_atoms": merge.new_atoms.as_slice(),
                    "changed": merge.changed,
                }))
                .collect::<Vec<_>>(),
            "rewritten": plan.rewritten.as_str(),
        }))?
    );
    Ok(())
}

pub(super) fn print_sort_package_options_plan(plan: &SortPackageOptionsPlan) -> Result<()> {
    let package_count = plan.packages.len();
    let changed_package_count = plan
        .packages
        .iter()
        .filter(|package| package.changed)
        .count();

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "path": plan.path.display().to_string(),
            "dialect": plan.dialect.label(),
            "package_count": package_count,
            "changed_package_count": changed_package_count,
            "changed": plan.changed,
            "written": plan.written,
            "packages": plan
                .packages
                .iter()
                .map(|package| json!({
                    "package": package.package.as_str(),
                    "defpackage": {
                        "path": package.defpackage_path.as_str(),
                        "span": {
                            "start": package.defpackage_span.start().get(),
                            "end": package.defpackage_span.end().get(),
                        },
                    },
                    "old_options": package.old_options.as_slice(),
                    "new_options": package.new_options.as_slice(),
                    "changed": package.changed,
                }))
                .collect::<Vec<_>>(),
            "rewritten": plan.rewritten.as_str(),
        }))?
    );
    Ok(())
}

pub(super) fn print_sort_package_exports_plan(plan: &SortPackageExportsPlan) -> Result<()> {
    let export_count = plan.exports.len();
    let changed_export_count = plan.exports.iter().filter(|export| export.changed).count();

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "path": plan.path.display().to_string(),
            "dialect": plan.dialect.label(),
            "export_count": export_count,
            "changed_export_count": changed_export_count,
            "changed": plan.changed,
            "written": plan.written,
            "exports": plan
                .exports
                .iter()
                .map(|export| json!({
                    "package": export.package.as_str(),
                    "defpackage": {
                        "path": export.defpackage_path.as_str(),
                        "span": {
                            "start": export.defpackage_span.start().get(),
                            "end": export.defpackage_span.end().get(),
                        },
                    },
                    "export": {
                        "path": export.export_path.as_str(),
                        "span": {
                            "start": export.export_span.start().get(),
                            "end": export.export_span.end().get(),
                        },
                    },
                    "old_symbols": export.old_symbols.as_slice(),
                    "new_symbols": export.new_symbols.as_slice(),
                    "changed": export.changed,
                }))
                .collect::<Vec<_>>(),
            "rewritten": plan.rewritten.as_str(),
        }))?
    );
    Ok(())
}

pub(super) fn print_package_report(reports: &[PackageReportFile]) -> Result<()> {
    let defpackage_count = reports
        .iter()
        .map(|report| report.report.defpackages.len())
        .sum::<usize>();
    let in_package_count = reports
        .iter()
        .map(|report| report.report.in_packages.len())
        .sum::<usize>();

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "file_count": reports.len(),
            "defpackage_count": defpackage_count,
            "in_package_count": in_package_count,
            "files": reports
                .iter()
                .map(|report| json!({
                    "path": report.path.display().to_string(),
                    "dialect": report.dialect.label(),
                    "defpackages": report
                        .report
                        .defpackages
                        .iter()
                        .map(|defpackage| json!({
                            "path": defpackage.path.as_str(),
                            "span": {
                                "start": defpackage.span.start().get(),
                                "end": defpackage.span.end().get(),
                            },
                            "name": defpackage.name.as_str(),
                            "nicknames": defpackage.nicknames.as_slice(),
                            "uses": defpackage.uses.as_slice(),
                            "exports": defpackage.exports.as_slice(),
                            "imports": defpackage
                                .imports
                                .iter()
                                .map(|import| json!({
                                    "package": import.package.as_str(),
                                    "symbols": import.symbols.as_slice(),
                                }))
                                .collect::<Vec<_>>(),
                            "option_count": defpackage.option_count,
                        }))
                        .collect::<Vec<_>>(),
                    "in_packages": report
                        .report
                        .in_packages
                        .iter()
                        .map(|in_package| json!({
                            "path": in_package.path.as_str(),
                            "span": {
                                "start": in_package.span.start().get(),
                                "end": in_package.span.end().get(),
                            },
                            "name": in_package.name.as_str(),
                        }))
                        .collect::<Vec<_>>(),
                }))
                .collect::<Vec<_>>(),
        }))?
    );
    Ok(())
}

pub(super) fn print_rename_package_plan(
    plans: &[RenamePackageFilePlan],
    from: &SymbolName,
    to: &SymbolName,
    write: bool,
) -> Result<()> {
    let occurrence_count = plans
        .iter()
        .map(|plan| plan.occurrences.len())
        .sum::<usize>();
    let changed_count = plans.iter().filter(|plan| plan.changed).count();
    let written_count = plans.iter().filter(|plan| plan.written).count();

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "from": from.as_str(),
            "to": to.as_str(),
            "write": write,
            "file_count": plans.len(),
            "occurrence_count": occurrence_count,
            "changed_count": changed_count,
            "written_count": written_count,
            "files": plans
                .iter()
                .map(|plan| json!({
                    "path": plan.path.display().to_string(),
                    "dialect": plan.dialect.label(),
                    "count": plan.occurrences.len(),
                    "changed": plan.changed,
                    "written": plan.written,
                    "occurrences": plan
                        .occurrences
                        .iter()
                        .map(|occurrence| json!({
                            "kind": occurrence.kind.label(),
                            "path": occurrence.path,
                            "span": {
                                "start": occurrence.span.start().get(),
                                "end": occurrence.span.end().get(),
                            },
                            "text": occurrence.text,
                            "replacement": occurrence.replacement,
                        }))
                        .collect::<Vec<_>>(),
                }))
                .collect::<Vec<_>>(),
        }))?
    );
    Ok(())
}

pub(super) fn print_add_export_plan(plan: &AddExportPlan) -> Result<()> {
    let export_span = plan.export_span.map(|span| {
        json!({
            "start": span.start().get(),
            "end": span.end().get(),
        })
    });

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "path": plan.path.display().to_string(),
            "dialect": plan.dialect.label(),
            "package": plan.package.as_str(),
            "symbol": plan.symbol.as_str(),
            "defpackage": {
                "path": plan.defpackage_path.as_str(),
                "span": {
                    "start": plan.defpackage_span.start().get(),
                    "end": plan.defpackage_span.end().get(),
                },
            },
            "export_span": export_span,
            "insertion_span": {
                "start": plan.insertion_span.start().get(),
                "end": plan.insertion_span.end().get(),
            },
            "already_exported": plan.already_exported,
            "changed": plan.changed,
            "written": plan.written,
            "rewritten": plan.rewritten.as_str(),
        }))?
    );
    Ok(())
}
