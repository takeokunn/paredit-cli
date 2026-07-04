use crate::domain::sexpr::{ExpressionView, Path};

use crate::application::dependency_report::syntax::{dependency_designator_text, list_head};
use crate::application::dependency_report::types::{DependencyKind, DependencyReportItem};

pub(super) fn collect_list_dependency_items(
    view: &ExpressionView,
    path_indexes: &[usize],
    dependencies: &mut Vec<DependencyReportItem>,
) {
    let Some(head) = list_head(view) else {
        return;
    };
    let normalized_head = head.rsplit(':').next().unwrap_or(head);

    let Some(kind) = runtime_dependency_kind(normalized_head) else {
        return;
    };

    push_dependency_from_child(
        view,
        path_indexes,
        1,
        kind,
        Some(head.to_owned()),
        dependencies,
    );
}

fn runtime_dependency_kind(head: &str) -> Option<DependencyKind> {
    match head {
        "require" => Some(DependencyKind::Require),
        "provide" => Some(DependencyKind::Provide),
        "load" => Some(DependencyKind::Load),
        "load-file" => Some(DependencyKind::LoadFile),
        "load-library" => Some(DependencyKind::LoadLibrary),
        "use-package" => Some(DependencyKind::UsePackage),
        "import" => Some(DependencyKind::Import),
        _ => None,
    }
}

fn push_dependency_from_child(
    view: &ExpressionView,
    path_indexes: &[usize],
    child_index: usize,
    kind: DependencyKind,
    source: Option<String>,
    dependencies: &mut Vec<DependencyReportItem>,
) {
    let Some(child) = view.children.get(child_index) else {
        return;
    };
    let Some(target) = dependency_designator_text(child) else {
        return;
    };

    let mut child_path = path_indexes.to_vec();
    child_path.push(child_index);
    dependencies.push(DependencyReportItem {
        kind,
        target,
        path: Path::from_indexes(child_path).to_string(),
        span: child.span,
        source,
    });
}
