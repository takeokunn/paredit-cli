use crate::domain::sexpr::{ExpressionView, Path};

use crate::application::usecase::dependency_report::syntax::{
    atom_text, package_qualified_dependency_target,
};
use crate::application::usecase::dependency_report::types::{DependencyKind, DependencyReportItem};

pub(super) fn collect_qualified_symbol_dependency(
    view: &ExpressionView,
    path_indexes: &[usize],
    dependencies: &mut Vec<DependencyReportItem>,
) {
    let Some(atom) = atom_text(view) else {
        return;
    };
    let Some(target) = package_qualified_dependency_target(atom) else {
        return;
    };

    dependencies.push(DependencyReportItem {
        kind: DependencyKind::QualifiedSymbol,
        target,
        path: Path::from_indexes(path_indexes.to_vec()).to_string(),
        span: view.span,
        source: Some(atom.to_owned()),
    });
}
