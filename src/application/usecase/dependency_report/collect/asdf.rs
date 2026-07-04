use crate::domain::sexpr::{Delimiter, ExpressionKind, ExpressionView, Path};

use crate::application::usecase::dependency_report::syntax::{
    atom_child, atom_text, dependency_designator_text, list_head,
};
use crate::application::usecase::dependency_report::types::{DependencyKind, DependencyReportItem};

pub(super) fn collect_system_dependency_items(
    view: &ExpressionView,
    path_indexes: &[usize],
    dependencies: &mut Vec<DependencyReportItem>,
) {
    if list_head(view)
        .and_then(|head| head.rsplit(':').next())
        .is_none_or(|head| head != "defsystem")
    {
        return;
    }

    for index in 1..view.children.len().saturating_sub(1) {
        let Some(option) = atom_text(&view.children[index]) else {
            continue;
        };
        let mut option_value_path = path_indexes.to_vec();
        option_value_path.push(index + 1);

        match option.to_ascii_lowercase().as_str() {
            ":depends-on" => collect_dependency_designators(
                &view.children[index + 1],
                option_value_path,
                DependencyKind::AsdfDependsOn,
                Some(option.to_owned()),
                dependencies,
            ),
            ":components" => {
                collect_component_items(&view.children[index + 1], option_value_path, dependencies);
            }
            _ => {}
        }
    }
}

fn collect_dependency_designators(
    view: &ExpressionView,
    path_indexes: Vec<usize>,
    kind: DependencyKind,
    source: Option<String>,
    dependencies: &mut Vec<DependencyReportItem>,
) {
    if let Some(target) = dependency_designator_text(view) {
        if !target.starts_with(':') {
            dependencies.push(DependencyReportItem {
                kind,
                target,
                path: Path::from_indexes(path_indexes.clone()).to_string(),
                span: view.span,
                source,
            });
        }
        return;
    }

    for (index, child) in view.children.iter().enumerate() {
        let mut child_path = path_indexes.clone();
        child_path.push(index);
        collect_dependency_designators(child, child_path, kind, source.clone(), dependencies);
    }
}

fn collect_component_items(
    view: &ExpressionView,
    path_indexes: Vec<usize>,
    dependencies: &mut Vec<DependencyReportItem>,
) {
    if view.kind == ExpressionKind::List
        && view.delimiter == Some(Delimiter::Paren)
        && view.children.len() >= 2
        && atom_child(view, 0).is_some_and(|kind| kind.starts_with(':'))
        && let Some(name) = atom_child(view, 1)
    {
        let component_kind = atom_child(view, 0).unwrap_or(":component");
        dependencies.push(DependencyReportItem {
            kind: DependencyKind::AsdfComponent,
            target: format!("{component_kind} {name}"),
            path: Path::from_indexes(path_indexes.clone()).to_string(),
            span: view.span,
            source: Some(":components".to_owned()),
        });
    }

    for (index, child) in view.children.iter().enumerate() {
        let mut child_path = path_indexes.clone();
        child_path.push(index);
        collect_component_items(child, child_path, dependencies);
    }
}
