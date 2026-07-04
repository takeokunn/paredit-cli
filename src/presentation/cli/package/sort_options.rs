use std::fs;

use anyhow::{Context, Result};

use crate::application::usecase::package as package_usecase;

use super::super::{detect_dialect, read_input};
use super::{
    render::print_sort_package_options_plan,
    types::{SortPackageOptionsArgs, SortPackageOptionsPlan},
};

pub(in crate::presentation::cli) fn sort_package_options(
    args: SortPackageOptionsArgs,
) -> Result<()> {
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
