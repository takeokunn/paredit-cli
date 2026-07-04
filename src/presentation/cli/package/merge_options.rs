use std::fs;

use anyhow::{Context, Result};

use crate::application::usecase::package as package_usecase;

use super::super::{detect_dialect, read_input};
use super::{
    render::print_merge_package_options_plan,
    types::{MergePackageOptionsArgs, MergePackageOptionsPlan},
};

pub(in crate::presentation::cli) fn merge_package_options(
    args: MergePackageOptionsArgs,
) -> Result<()> {
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
