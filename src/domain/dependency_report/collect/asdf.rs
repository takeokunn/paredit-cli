use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{Delimiter, ExpressionKind, ExpressionView, Path};

use crate::domain::dependency_report::syntax::{
    atom_child, atom_text, dependency_designator_text, list_head,
};
use crate::domain::dependency_report::types::{DependencyKind, DependencyReportItem};

pub(super) fn collect_system_dependency_items(
    view: &ExpressionView,
    dialect: Dialect,
    path: &Path,
    dependencies: &mut Vec<DependencyReportItem>,
) {
    let Some(head) = list_head(view) else {
        return;
    };
    if !dialect.is_common_lisp_asdf_system_definition_head(head) {
        return;
    }

    for index in 1..view.children.len().saturating_sub(1) {
        let Some(option) = atom_text(&view.children[index]) else {
            continue;
        };
        let option_value_path = path.child(index + 1);

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
    path: Path,
    kind: DependencyKind,
    source: Option<String>,
    dependencies: &mut Vec<DependencyReportItem>,
) {
    if let Some(target) = dependency_designator_text(view) {
        if !target.starts_with(':') {
            dependencies.push(DependencyReportItem::new(
                kind,
                target,
                path.to_string(),
                view.span,
                source,
            ));
        }
        return;
    }

    for (index, child) in view.children.iter().enumerate() {
        let child_path = path.child(index);
        collect_dependency_designators(child, child_path, kind, source.clone(), dependencies);
    }
}

fn collect_component_items(
    view: &ExpressionView,
    path: Path,
    dependencies: &mut Vec<DependencyReportItem>,
) {
    if view.kind == ExpressionKind::List
        && view.delimiter == Some(Delimiter::Paren)
        && view.children.len() >= 2
        && atom_child(view, 0).is_some_and(|kind| kind.starts_with(':'))
    {
        if let Some(name) = atom_child(view, 1) {
            let component_kind = atom_child(view, 0).unwrap_or(":component");
            dependencies.push(DependencyReportItem::new(
                DependencyKind::AsdfComponent,
                format!("{component_kind} {name}"),
                path.to_string(),
                view.span,
                Some(":components".to_owned()),
            ));
        }
    }

    for (index, child) in view.children.iter().enumerate() {
        let child_path = path.child(index);
        collect_component_items(child, child_path, dependencies);
    }
}
