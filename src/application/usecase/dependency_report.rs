use anyhow::Result;

use crate::application::usecase::package_report::build_package_report;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::SyntaxTree;

mod collect;
mod defpackage;
mod syntax;
#[cfg(test)]
mod tests;
mod types;

pub use types::{DependencyKind, DependencyReport, DependencyReportItem};

use collect::collect_dependency_items;
use defpackage::defpackage_dependency_items;

pub fn build_dependency_report(tree: &SyntaxTree, dialect: Dialect) -> Result<DependencyReport> {
    let package_report = build_package_report(tree, dialect)?;
    let mut dependencies = collect_dependency_items(tree, dialect)?;
    dependencies.extend(defpackage_dependency_items(&package_report.defpackages));
    dependencies.sort_by(|left, right| {
        left.span
            .start()
            .cmp(&right.span.start())
            .then_with(|| left.kind.cmp(&right.kind))
            .then_with(|| left.target.cmp(&right.target))
    });

    Ok(DependencyReport { dependencies })
}
