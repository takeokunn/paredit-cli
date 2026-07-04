use std::fs;

use anyhow::{Context, Result};

use crate::application::usecase::package as package_usecase;

use super::super::{detect_dialect, read_input};
use super::{
    render::print_rename_package_plan,
    types::{RenamePackageArgs, RenamePackageFilePlan},
};

pub(in crate::presentation::cli) fn rename_package(args: RenamePackageArgs) -> Result<()> {
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
