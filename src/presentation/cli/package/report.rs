use anyhow::{Context, Result};

use crate::application::usecase::package_report::build_package_report;
use crate::domain::sexpr::SyntaxTree;

use super::super::{detect_dialect, read_input};
use super::{
    render::print_package_report,
    types::{PackageReportArgs, PackageReportFile},
};

pub(in crate::presentation::cli) fn package_report(args: PackageReportArgs) -> Result<()> {
    let mut reports = Vec::with_capacity(args.files.len());

    for file in &args.files {
        let input = read_input(Some(file.clone()))?;
        let dialect = detect_dialect(&input, args.dialect);
        let tree = SyntaxTree::parse(&input.text)
            .with_context(|| format!("failed to parse {}", file.display()))?;
        let report = build_package_report(&tree, dialect)
            .with_context(|| format!("failed to inspect packages in {}", file.display()))?;

        reports.push(PackageReportFile {
            path: file.clone(),
            dialect,
            report,
        });
    }

    print_package_report(&reports, args.output)
}
