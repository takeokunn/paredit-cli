use super::*;

pub(super) fn print_merge_package_options_plan(plan: &MergePackageOptionsPlan) -> Result<()> {
    let merge_count = plan.merges.len();
    let changed_merge_count = plan.merges.iter().filter(|merge| merge.changed).count();

    println!("file\t{}", plan.path.display());
    println!("dialect\t{}", plan.dialect.label());
    println!("merge_count\t{merge_count}");
    println!("changed_merge_count\t{changed_merge_count}");
    println!("changed\t{}", plan.changed);
    println!("written\t{}", plan.written);
    for merge in &plan.merges {
        let key = merge.key.as_deref().unwrap_or("-");
        println!(
            "\tmerge\t{}\t{}\tkey={}\t{}..{}\tchanged={}",
            merge.package,
            merge.head,
            key,
            merge.kept_span.start().get(),
            merge.kept_span.end().get(),
            merge.changed
        );
        println!("\t\told\t{}", merge.old_atoms.join(" | "));
        println!("\t\tnew\t{}", merge.new_atoms.join(" | "));
        println!("\t\tremoved\t{}", merge.removed_paths.join(" | "));
    }
    println!("rewritten\t{}", plan.rewritten);
    Ok(())
}

pub(super) fn print_sort_package_options_plan(plan: &SortPackageOptionsPlan) -> Result<()> {
    let package_count = plan.packages.len();
    let changed_package_count = plan
        .packages
        .iter()
        .filter(|package| package.changed)
        .count();

    println!("file\t{}", plan.path.display());
    println!("dialect\t{}", plan.dialect.label());
    println!("package_count\t{package_count}");
    println!("changed_package_count\t{changed_package_count}");
    println!("changed\t{}", plan.changed);
    println!("written\t{}", plan.written);
    for package in &plan.packages {
        println!(
            "\tpackage\t{}\t{}\t{}..{}\tchanged={}",
            package.package,
            package.defpackage_path,
            package.defpackage_span.start().get(),
            package.defpackage_span.end().get(),
            package.changed
        );
        println!("\t\told\t{}", package.old_options.join(" | "));
        println!("\t\tnew\t{}", package.new_options.join(" | "));
    }
    println!("rewritten\t{}", plan.rewritten);
    Ok(())
}

pub(super) fn print_sort_package_exports_plan(plan: &SortPackageExportsPlan) -> Result<()> {
    let export_count = plan.exports.len();
    let changed_export_count = plan.exports.iter().filter(|export| export.changed).count();

    println!("file\t{}", plan.path.display());
    println!("dialect\t{}", plan.dialect.label());
    println!("export_count\t{export_count}");
    println!("changed_export_count\t{changed_export_count}");
    println!("changed\t{}", plan.changed);
    println!("written\t{}", plan.written);
    for export in &plan.exports {
        println!(
            "\texport\t{}\t{}\t{}..{}\tchanged={}",
            export.package,
            export.export_path,
            export.export_span.start().get(),
            export.export_span.end().get(),
            export.changed
        );
        println!("\t\told\t{}", export.old_symbols.join(" "));
        println!("\t\tnew\t{}", export.new_symbols.join(" "));
    }
    println!("rewritten\t{}", plan.rewritten);
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

    println!("files\t{}", reports.len());
    println!("defpackage_count\t{defpackage_count}");
    println!("in_package_count\t{in_package_count}");
    for report in reports {
        println!(
            "{}\t{}\tdefpackages={}\tin_packages={}",
            report.path.display(),
            report.dialect.label(),
            report.report.defpackages.len(),
            report.report.in_packages.len()
        );
        for defpackage in &report.report.defpackages {
            println!(
                "\tdefpackage\t{}\t{}..{}\t{}",
                defpackage.path,
                defpackage.span.start().get(),
                defpackage.span.end().get(),
                defpackage.name
            );
        }
        for in_package in &report.report.in_packages {
            println!(
                "\tin-package\t{}\t{}..{}\t{}",
                in_package.path,
                in_package.span.start().get(),
                in_package.span.end().get(),
                in_package.name
            );
        }
    }
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

    println!("from\t{from}");
    println!("to\t{to}");
    println!("write\t{write}");
    println!("file_count\t{}", plans.len());
    println!("occurrence_count\t{occurrence_count}");
    println!("changed_count\t{changed_count}");
    println!("written_count\t{written_count}");
    for plan in plans {
        println!(
            "{}\t{}\toccurrences={}\tchanged={}\twritten={}",
            plan.path.display(),
            plan.dialect.label(),
            plan.occurrences.len(),
            plan.changed,
            plan.written
        );
        for occurrence in &plan.occurrences {
            println!(
                "\t{}\t{}\t{}..{}\t{}\t=>\t{}",
                occurrence.kind.label(),
                occurrence.path,
                occurrence.span.start().get(),
                occurrence.span.end().get(),
                occurrence.text,
                occurrence.replacement
            );
        }
    }
    Ok(())
}

pub(super) fn print_add_export_plan(plan: &AddExportPlan) -> Result<()> {
    println!("file\t{}", plan.path.display());
    println!("dialect\t{}", plan.dialect.label());
    println!("package\t{}", plan.package);
    println!("symbol\t{}", plan.symbol);
    println!("defpackage_path\t{}", plan.defpackage_path);
    println!(
        "defpackage_span\t{}..{}",
        plan.defpackage_span.start().get(),
        plan.defpackage_span.end().get()
    );
    if let Some(export_span) = plan.export_span {
        println!(
            "export_span\t{}..{}",
            export_span.start().get(),
            export_span.end().get()
        );
    } else {
        println!("export_span\tmissing");
    }
    println!(
        "insertion_span\t{}..{}",
        plan.insertion_span.start().get(),
        plan.insertion_span.end().get()
    );
    println!("already_exported\t{}", plan.already_exported);
    println!("changed\t{}", plan.changed);
    println!("written\t{}", plan.written);
    println!("rewritten\t{}", plan.rewritten);
    Ok(())
}
