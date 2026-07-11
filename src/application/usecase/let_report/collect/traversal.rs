use anyhow::Result;

use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{ExpressionView, Path};

use super::super::types::LetFormReport;
use super::report::analyze_let_form;

pub(in crate::application::usecase::let_report) fn collect_let_reports_from_view(
    dialect: Dialect,
    input: &str,
    view: &ExpressionView,
    path: Path,
    reports: &mut Vec<LetFormReport>,
) -> Result<()> {
    if let Some(report) = analyze_let_form(dialect, input, view, &path)? {
        reports.push(report);
    }

    for (index, child) in view.children.iter().enumerate() {
        collect_let_reports_from_view(dialect, input, child, path.child(index), reports)?;
    }

    Ok(())
}
