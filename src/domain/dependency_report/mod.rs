//! Dependency inventory analysis for Lisp source forms and package declarations.

use anyhow::Result;

use crate::domain::dialect::Dialect;
use crate::domain::package_report::build_package_report;
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
    dependencies.sort_by(DependencyReportItem::cmp_position);

    Ok(DependencyReport::new(dependencies))
}
