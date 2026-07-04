use anyhow::Result;

use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{Path, SyntaxTree};

mod collect;
mod policy;
mod syntax;
#[cfg(test)]
mod tests;
mod types;

pub use policy::evaluate_let_report_policy;
pub use types::{LetBindingReport, LetFormReport, LetReportPolicy, LetReportPolicyOptions};

pub fn build_let_report(
    dialect: Dialect,
    input: &str,
    tree: &SyntaxTree,
) -> Result<Vec<LetFormReport>> {
    let mut reports = Vec::new();
    for index in 0..tree.root_children().len() {
        let path_indexes = vec![index];
        let path = Path::from_indexes(path_indexes.clone());
        let view = tree.select_path(&path)?.view();
        collect::collect_let_reports_from_view(dialect, input, &view, path_indexes, &mut reports)?;
    }
    Ok(reports)
}
