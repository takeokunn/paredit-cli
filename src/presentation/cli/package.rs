use super::*;
use crate::application::package_report::build_package_report;
use crate::application::usecase::package as package_usecase;

mod render;
mod types;

use render::{
    print_add_export_plan, print_merge_package_options_plan, print_package_report,
    print_rename_package_plan, print_sort_package_exports_plan, print_sort_package_options_plan,
};
pub(super) use types::{
    AddExportArgs, MergePackageOptionsArgs, PackageReportArgs, RenamePackageArgs,
    SortPackageExportsArgs, SortPackageOptionsArgs,
};
use types::{
    AddExportPlan, MergePackageOptionsPlan, PackageReportFile, RenamePackageFilePlan,
    SortPackageExportsPlan, SortPackageOptionsPlan,
};

pub(super) fn package_report(args: PackageReportArgs) -> Result<()> {
    let mut reports = Vec::with_capacity(args.files.len());

    for file in &args.files {
        let input = read_input(Some(file.clone()))?;
        let dialect = detect_dialect(&input, args.dialect);
        let tree = SyntaxTree::parse(&input.text)
            .with_context(|| format!("failed to parse {}", file.display()))?;
        let report = build_package_report(&tree)
            .with_context(|| format!("failed to inspect packages in {}", file.display()))?;

        reports.push(PackageReportFile {
            path: file.clone(),
            dialect,
            report,
        });
    }

    print_package_report(&reports, args.output)
}

pub(super) fn add_export(args: AddExportArgs) -> Result<()> {
    let input = read_input(Some(args.file.clone()))?;
    let dialect = detect_dialect(&input, args.dialect);
    let usecase_plan = package_usecase::plan_add_export(package_usecase::AddExportRequest {
        input: &input.text,
        package: args.package.as_ref(),
        symbol: &args.symbol,
    })
    .with_context(|| format!("failed to plan add-export for {}", args.file.display()))?;
    let changed = usecase_plan.changed;
    let written = args.write && changed;

    if written {
        fs::write(&args.file, &usecase_plan.rewritten)
            .with_context(|| format!("failed to write {}", args.file.display()))?;
    }

    let plan = AddExportPlan {
        path: args.file,
        dialect,
        package: usecase_plan.package,
        symbol: usecase_plan.symbol,
        defpackage_path: usecase_plan.defpackage_path,
        defpackage_span: usecase_plan.defpackage_span,
        export_span: usecase_plan.export_span,
        insertion_span: usecase_plan.insertion_span,
        already_exported: usecase_plan.already_exported,
        changed,
        written,
        rewritten: usecase_plan.rewritten,
    };

    print_add_export_plan(&plan, args.output)
}

pub(super) fn rename_package(args: RenamePackageArgs) -> Result<()> {
    let mut plans = Vec::with_capacity(args.files.len());

    for file in &args.files {
        let input = read_input(Some(file.clone()))?;
        let dialect = detect_dialect(&input, args.dialect);
        let usecase_plan =
            package_usecase::plan_rename_package(package_usecase::RenamePackageRequest {
                input: &input.text,
                from: &args.from,
                to: &args.to,
            })
            .with_context(|| format!("failed to plan rename-package for {}", file.display()))?;
        let changed = usecase_plan.changed;
        let written = args.write && changed;

        if written {
            fs::write(file, &usecase_plan.rewritten)
                .with_context(|| format!("failed to write {}", file.display()))?;
        }

        plans.push(RenamePackageFilePlan {
            path: file.clone(),
            dialect,
            occurrences: usecase_plan.occurrences,
            changed,
            written,
        });
    }

    print_rename_package_plan(&plans, &args.from, &args.to, args.write, args.output)
}

pub(super) fn sort_package_exports(args: SortPackageExportsArgs) -> Result<()> {
    let input = read_input(Some(args.file.clone()))?;
    let dialect = detect_dialect(&input, args.dialect);
    let usecase_plan =
        package_usecase::plan_sort_package_exports(package_usecase::SortPackageExportsRequest {
            input: &input.text,
            package: args.package.as_ref(),
        })
        .with_context(|| {
            format!(
                "failed to plan sort-package-exports for {}",
                args.file.display()
            )
        })?;
    let changed = usecase_plan.changed;
    let written = args.write && changed;

    if written {
        fs::write(&args.file, &usecase_plan.rewritten)
            .with_context(|| format!("failed to write {}", args.file.display()))?;
    }

    let plan = SortPackageExportsPlan {
        path: args.file,
        dialect,
        exports: usecase_plan.exports,
        changed,
        written,
        rewritten: usecase_plan.rewritten,
    };

    print_sort_package_exports_plan(&plan, args.output)
}

pub(super) fn sort_package_options(args: SortPackageOptionsArgs) -> Result<()> {
    let input = read_input(Some(args.file.clone()))?;
    let dialect = detect_dialect(&input, args.dialect);
    let usecase_plan =
        package_usecase::plan_sort_package_options(package_usecase::SortPackageOptionsRequest {
            input: &input.text,
            package: args.package.as_ref(),
            order: args.order.into(),
        })
        .with_context(|| {
            format!(
                "failed to plan sort-package-options for {}",
                args.file.display()
            )
        })?;
    let changed = usecase_plan.changed;
    let written = args.write && changed;

    if written {
        fs::write(&args.file, &usecase_plan.rewritten)
            .with_context(|| format!("failed to write {}", args.file.display()))?;
    }

    let plan = SortPackageOptionsPlan {
        path: args.file,
        dialect,
        packages: usecase_plan.packages,
        changed,
        written,
        rewritten: usecase_plan.rewritten,
    };

    print_sort_package_options_plan(&plan, args.output)
}

pub(super) fn merge_package_options(args: MergePackageOptionsArgs) -> Result<()> {
    let input = read_input(Some(args.file.clone()))?;
    let dialect = detect_dialect(&input, args.dialect);
    let usecase_plan =
        package_usecase::plan_merge_package_options(package_usecase::MergePackageOptionsRequest {
            input: &input.text,
            package: args.package.as_ref(),
        })
        .with_context(|| {
            format!(
                "failed to plan merge-package-options for {}",
                args.file.display()
            )
        })?;
    let changed = usecase_plan.changed;
    let written = args.write && changed;

    if written {
        fs::write(&args.file, &usecase_plan.rewritten)
            .with_context(|| format!("failed to write {}", args.file.display()))?;
    }

    let plan = MergePackageOptionsPlan {
        path: args.file,
        dialect,
        merges: usecase_plan.merges,
        changed,
        written,
        rewritten: usecase_plan.rewritten,
    };

    print_merge_package_options_plan(&plan, args.output)
}
