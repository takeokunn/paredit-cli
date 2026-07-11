use anyhow::{Context, Result};

use crate::application::usecase::definition_report::collect_definition_forms;
use crate::application::usecase::dependency_report::build_dependency_report;
use crate::domain::sexpr::SyntaxTree;
use crate::presentation::cli::dependency_report::{
    args::DependencyReportArgs, render::print_dependency_report, types::DependencyReportFile,
};
use crate::presentation::cli::shared::{detect_dialect, read_input};

pub(in crate::presentation::cli) fn dependency_report(args: DependencyReportArgs) -> Result<()> {
    let mut reports = Vec::with_capacity(args.files.len());

    for file in &args.files {
        let input = read_input(Some(file.clone()))?;
        let dialect = detect_dialect(&input, args.dialect);
        let tree = SyntaxTree::parse(&input.text)
            .with_context(|| format!("failed to parse {}", file.display()))?;
        let (package, _) = collect_definition_forms(&tree, dialect)?;
        let dependency_report = build_dependency_report(&tree, dialect)?;

        reports.push(DependencyReportFile {
            path: file.clone(),
            dialect,
            package,
            dependencies: dependency_report.dependencies,
        });
    }

    print_dependency_report(&reports, args.output)
}
