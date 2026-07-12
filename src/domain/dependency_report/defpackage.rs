use crate::domain::package_report::PackageDefinitionReport;

use super::types::{DependencyKind, DependencyReportItem};

pub(super) fn defpackage_dependency_items(
    defpackages: &[PackageDefinitionReport],
) -> Vec<DependencyReportItem> {
    let mut dependencies = Vec::new();

    for defpackage in defpackages {
        for target in &defpackage.uses {
            dependencies.push(DependencyReportItem {
                kind: DependencyKind::DefpackageUse,
                target: target.clone(),
                path: defpackage.path.clone(),
                span: defpackage.span,
                source: Some(defpackage.name.clone()),
            });
        }
        for import in &defpackage.imports {
            dependencies.push(DependencyReportItem {
                kind: DependencyKind::DefpackageImportFrom,
                target: import.package.clone(),
                path: defpackage.path.clone(),
                span: defpackage.span,
                source: Some(defpackage.name.clone()),
            });
        }
    }

    dependencies
}
