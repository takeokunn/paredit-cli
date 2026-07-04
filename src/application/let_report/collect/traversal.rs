use anyhow::Result;

use crate::domain::dialect::Dialect;
use crate::domain::sexpr::ExpressionView;

use super::super::types::LetFormReport;
use super::report::analyze_let_form;

pub(in crate::application::let_report) fn collect_let_reports_from_view(
    dialect: Dialect,
    input: &str,
    view: &ExpressionView,
    path_indexes: Vec<usize>,
    reports: &mut Vec<LetFormReport>,
) -> Result<()> {
    if let Some(report) = analyze_let_form(dialect, input, view, &path_indexes)? {
        reports.push(report);
    }

    for (index, child) in view.children.iter().enumerate() {
        let mut child_path = path_indexes.clone();
        child_path.push(index);
        collect_let_reports_from_view(dialect, input, child, child_path, reports)?;
    }

    Ok(())
}
