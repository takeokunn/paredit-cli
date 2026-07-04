use anyhow::Result;

use crate::domain::sexpr::{Delimiter, ExpressionKind, ExpressionView, Path, SyntaxTree};

use super::syntax::{
    atom_child, atom_text, dependency_designator_text, list_head,
    package_qualified_dependency_target,
};
use super::types::{DependencyKind, DependencyReportItem};

pub(super) fn collect_dependency_items(tree: &SyntaxTree) -> Result<Vec<DependencyReportItem>> {
    let mut dependencies = Vec::new();

    for index in 0..tree.root_children().len() {
        let path_indexes = vec![index];
        let path = Path::from_indexes(path_indexes.clone());
        let view = tree.select_path(&path)?.view();
        collect_dependency_items_from_view(&view, path_indexes, &mut dependencies);
    }

    Ok(dependencies)
}

fn collect_dependency_items_from_view(
    view: &ExpressionView,
    path_indexes: Vec<usize>,
    dependencies: &mut Vec<DependencyReportItem>,
) {
    if view.kind == ExpressionKind::List && view.delimiter == Some(Delimiter::Paren) {
        collect_list_dependency_items(view, &path_indexes, dependencies);
    }

    if let Some(atom) = atom_text(view)
        && let Some(target) = package_qualified_dependency_target(atom)
    {
        dependencies.push(DependencyReportItem {
            kind: DependencyKind::QualifiedSymbol,
            target,
            path: Path::from_indexes(path_indexes.clone()).to_string(),
            span: view.span,
            source: Some(atom.to_owned()),
        });
    }

    for (index, child) in view.children.iter().enumerate() {
        let mut child_path = path_indexes.clone();
        child_path.push(index);
        collect_dependency_items_from_view(child, child_path, dependencies);
    }
}

fn collect_list_dependency_items(
    view: &ExpressionView,
    path_indexes: &[usize],
    dependencies: &mut Vec<DependencyReportItem>,
) {
    let Some(head) = list_head(view) else {
        return;
    };
    let normalized_head = head.rsplit(':').next().unwrap_or(head);

    match normalized_head {
        "require" => push_dependency_from_child(
            view,
            path_indexes,
            1,
            DependencyKind::Require,
            Some(head.to_owned()),
            dependencies,
        ),
        "provide" => push_dependency_from_child(
            view,
            path_indexes,
            1,
            DependencyKind::Provide,
            Some(head.to_owned()),
            dependencies,
        ),
        "load" => push_dependency_from_child(
            view,
            path_indexes,
            1,
            DependencyKind::Load,
            Some(head.to_owned()),
            dependencies,
        ),
        "load-file" => push_dependency_from_child(
            view,
            path_indexes,
            1,
            DependencyKind::LoadFile,
            Some(head.to_owned()),
            dependencies,
        ),
        "load-library" => push_dependency_from_child(
            view,
            path_indexes,
            1,
            DependencyKind::LoadLibrary,
            Some(head.to_owned()),
            dependencies,
        ),
        "use-package" => push_dependency_from_child(
            view,
            path_indexes,
            1,
            DependencyKind::UsePackage,
            Some(head.to_owned()),
            dependencies,
        ),
        "import" => push_dependency_from_child(
            view,
            path_indexes,
            1,
            DependencyKind::Import,
            Some(head.to_owned()),
            dependencies,
        ),
        "defsystem" => collect_asdf_system_dependency_items(view, path_indexes, dependencies),
        _ => {}
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

fn collect_asdf_system_dependency_items(
    view: &ExpressionView,
    path_indexes: &[usize],
    dependencies: &mut Vec<DependencyReportItem>,
) {
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
            ":components" => collect_asdf_component_items(
                &view.children[index + 1],
                option_value_path,
                dependencies,
            ),
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

fn collect_asdf_component_items(
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
        collect_asdf_component_items(child, child_path, dependencies);
    }
}
