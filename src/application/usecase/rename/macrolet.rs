use anyhow::Result;

use crate::application::usecase::callable_scope::{
    LocalCallableForm, common_lisp_local_callable_form, is_macro_callable_form,
    local_callable_names,
};
use crate::domain::definition::classify_definition_head;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{
    Delimiter, ExpressionKind, ExpressionView, Path, SymbolName, SyntaxTree,
};

use super::RenameFunctionOccurrence;
use super::selection::list_head;

#[derive(Debug, Clone, Copy, Default)]
struct MacroletRenameScope {
    active_target_depth: usize,
    shadowed_depth: usize,
}

#[derive(Debug, Clone, Copy)]
enum LocalCallableRenameKind {
    Macro,
    Function,
}

impl LocalCallableRenameKind {
    fn matches_target_form(self, form: LocalCallableForm) -> bool {
        match self {
            Self::Macro => is_macro_callable_form(form),
            Self::Function => matches!(form, LocalCallableForm::Flet | LocalCallableForm::Labels),
        }
    }
}

impl MacroletRenameScope {
    fn is_target_active(self) -> bool {
        self.active_target_depth > 0
    }

    fn is_shadowed(self) -> bool {
        self.shadowed_depth > 0
    }

    fn enter_active_target(mut self) -> Self {
        self.active_target_depth += 1;
        self
    }

    fn enter_shadowed(mut self) -> Self {
        self.shadowed_depth += 1;
        self
    }
}

pub fn collect_macrolet_binding_renames(
    tree: &SyntaxTree,
    dialect: Dialect,
    from: &SymbolName,
    to: &SymbolName,
) -> Result<Vec<RenameFunctionOccurrence>> {
    let mut renames = Vec::new();

    for (index, _) in tree.root_children().iter().enumerate() {
        let path_indexes = vec![index];
        let path = Path::from_indexes(path_indexes.clone());
        let view = tree.select_path(&path)?.view();
        collect_macrolet_binding_renames_from_view(
            &view,
            path_indexes,
            dialect,
            from,
            to,
            LocalCallableRenameKind::Macro,
            MacroletRenameScope::default(),
            &mut renames,
        );
    }

    renames.sort_by_key(|rename| rename.span.start());
    Ok(renames)
}

pub fn collect_macrolet_call_head_renames(
    tree: &SyntaxTree,
    dialect: Dialect,
    from: &SymbolName,
    to: &SymbolName,
) -> Result<Vec<RenameFunctionOccurrence>> {
    let mut renames = Vec::new();

    for (index, _) in tree.root_children().iter().enumerate() {
        let path_indexes = vec![index];
        let path = Path::from_indexes(path_indexes.clone());
        let view = tree.select_path(&path)?.view();
        collect_macrolet_call_head_renames_from_view(
            &view,
            path_indexes,
            dialect,
            from,
            to,
            LocalCallableRenameKind::Macro,
            MacroletRenameScope::default(),
            &mut renames,
        );
    }

    renames.sort_by_key(|rename| rename.span.start());
    Ok(renames)
}

pub fn collect_local_function_binding_renames(
    tree: &SyntaxTree,
    dialect: Dialect,
    from: &SymbolName,
    to: &SymbolName,
) -> Result<Vec<RenameFunctionOccurrence>> {
    let mut renames = Vec::new();

    for (index, _) in tree.root_children().iter().enumerate() {
        let path_indexes = vec![index];
        let path = Path::from_indexes(path_indexes.clone());
        let view = tree.select_path(&path)?.view();
        collect_macrolet_binding_renames_from_view(
            &view,
            path_indexes,
            dialect,
            from,
            to,
            LocalCallableRenameKind::Function,
            MacroletRenameScope::default(),
            &mut renames,
        );
    }

    renames.sort_by_key(|rename| rename.span.start());
    Ok(renames)
}

pub fn collect_local_function_call_head_renames(
    tree: &SyntaxTree,
    dialect: Dialect,
    from: &SymbolName,
    to: &SymbolName,
) -> Result<Vec<RenameFunctionOccurrence>> {
    let mut renames = Vec::new();

    for (index, _) in tree.root_children().iter().enumerate() {
        let path_indexes = vec![index];
        let path = Path::from_indexes(path_indexes.clone());
        let view = tree.select_path(&path)?.view();
        collect_macrolet_call_head_renames_from_view(
            &view,
            path_indexes,
            dialect,
            from,
            to,
            LocalCallableRenameKind::Function,
            MacroletRenameScope::default(),
            &mut renames,
        );
    }

    renames.sort_by_key(|rename| rename.span.start());
    Ok(renames)
}

fn collect_macrolet_binding_renames_from_view(
    view: &ExpressionView,
    path_indexes: Vec<usize>,
    dialect: Dialect,
    from: &SymbolName,
    to: &SymbolName,
    kind: LocalCallableRenameKind,
    scope: MacroletRenameScope,
    renames: &mut Vec<RenameFunctionOccurrence>,
) {
    let mut first_callable_child_index = 0;

    if view.kind == ExpressionKind::List
        && view.delimiter == Some(Delimiter::Paren)
        && let Some(head) = list_head(view)
    {
        if let Some(form) = common_lisp_local_callable_form(dialect, head) {
            collect_local_callable_form_binding_renames(
                view,
                path_indexes,
                dialect,
                from,
                to,
                kind,
                scope,
                form,
                renames,
            );
            return;
        }

        let category = classify_definition_head(dialect, head);
        if category.is_some() {
            first_callable_child_index = definition_body_start_index(category);
        }
    }

    for (index, child) in view.children.iter().enumerate() {
        if index < first_callable_child_index {
            continue;
        }
        let mut child_path = path_indexes.clone();
        child_path.push(index);
        collect_macrolet_binding_renames_from_view(
            child, child_path, dialect, from, to, kind, scope, renames,
        );
    }
}

fn collect_macrolet_call_head_renames_from_view(
    view: &ExpressionView,
    path_indexes: Vec<usize>,
    dialect: Dialect,
    from: &SymbolName,
    to: &SymbolName,
    kind: LocalCallableRenameKind,
    scope: MacroletRenameScope,
    renames: &mut Vec<RenameFunctionOccurrence>,
) {
    let mut first_callable_child_index = 0;

    if view.kind == ExpressionKind::List
        && view.delimiter == Some(Delimiter::Paren)
        && let Some(head) = list_head(view)
    {
        if let Some(form) = common_lisp_local_callable_form(dialect, head) {
            collect_local_callable_form_call_renames(
                view,
                path_indexes,
                dialect,
                from,
                to,
                kind,
                scope,
                form,
                renames,
            );
            return;
        }

        let category = classify_definition_head(dialect, head);
        if head == from.as_str()
            && category.is_none()
            && scope.is_target_active()
            && !scope.is_shadowed()
            && let Some(head_view) = view.children.first()
        {
            let mut head_path = path_indexes.clone();
            head_path.push(0);
            renames.push(RenameFunctionOccurrence {
                path: Path::from_indexes(head_path).to_string(),
                span: head_view.span,
                text: head.to_owned(),
                replacement: to.as_str().to_owned(),
            });
        }
        if category.is_some() {
            first_callable_child_index = definition_body_start_index(category);
        }
    }

    for (index, child) in view.children.iter().enumerate() {
        if index < first_callable_child_index {
            continue;
        }
        let mut child_path = path_indexes.clone();
        child_path.push(index);
        collect_macrolet_call_head_renames_from_view(
            child, child_path, dialect, from, to, kind, scope, renames,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn collect_local_callable_form_binding_renames(
    view: &ExpressionView,
    path_indexes: Vec<usize>,
    dialect: Dialect,
    from: &SymbolName,
    to: &SymbolName,
    kind: LocalCallableRenameKind,
    scope: MacroletRenameScope,
    form: LocalCallableForm,
    renames: &mut Vec<RenameFunctionOccurrence>,
) {
    let local_names = local_callable_names(view);
    let binds_from = local_names.iter().any(|name| name == from.as_str());
    let body_scope = local_callable_body_scope(scope, kind, form, binds_from);

    if let Some(bindings) = view.children.get(1) {
        let binding_body_scope = local_callable_binding_body_scope(scope, kind, form, binds_from);
        for (binding_index, binding) in bindings.children.iter().enumerate() {
            if kind.matches_target_form(form)
                && !scope.is_target_active()
                && !scope.is_shadowed()
                && binding
                    .children
                    .first()
                    .and_then(|child| child.text.as_deref())
                    == Some(from.as_str())
                && let Some(name_view) = binding.children.first()
            {
                let mut name_path = path_indexes.clone();
                name_path.extend([1, binding_index, 0]);
                renames.push(RenameFunctionOccurrence {
                    path: Path::from_indexes(name_path).to_string(),
                    span: name_view.span,
                    text: from.as_str().to_owned(),
                    replacement: to.as_str().to_owned(),
                });
            }

            if is_macro_callable_form(form) {
                continue;
            }

            for (child_index, child) in binding.children.iter().enumerate().skip(2) {
                let mut child_path = path_indexes.clone();
                child_path.extend([1, binding_index, child_index]);
                collect_macrolet_binding_renames_from_view(
                    child,
                    child_path,
                    dialect,
                    from,
                    to,
                    kind,
                    binding_body_scope,
                    renames,
                );
            }
        }
    }

    for (index, child) in view.children.iter().enumerate().skip(2) {
        let mut child_path = path_indexes.clone();
        child_path.push(index);
        collect_macrolet_binding_renames_from_view(
            child, child_path, dialect, from, to, kind, body_scope, renames,
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn collect_local_callable_form_call_renames(
    view: &ExpressionView,
    path_indexes: Vec<usize>,
    dialect: Dialect,
    from: &SymbolName,
    to: &SymbolName,
    kind: LocalCallableRenameKind,
    scope: MacroletRenameScope,
    form: LocalCallableForm,
    renames: &mut Vec<RenameFunctionOccurrence>,
) {
    let local_names = local_callable_names(view);
    let binds_from = local_names.iter().any(|name| name == from.as_str());
    let body_scope = local_callable_body_scope(scope, kind, form, binds_from);

    if let Some(bindings) = view.children.get(1) {
        let binding_body_scope = local_callable_binding_body_scope(scope, kind, form, binds_from);
        for (binding_index, binding) in bindings.children.iter().enumerate() {
            if is_macro_callable_form(form) {
                continue;
            }
            for (child_index, child) in binding.children.iter().enumerate().skip(2) {
                let mut child_path = path_indexes.clone();
                child_path.extend([1, binding_index, child_index]);
                collect_macrolet_call_head_renames_from_view(
                    child,
                    child_path,
                    dialect,
                    from,
                    to,
                    kind,
                    binding_body_scope,
                    renames,
                );
            }
        }
    }

    for (index, child) in view.children.iter().enumerate().skip(2) {
        let mut child_path = path_indexes.clone();
        child_path.push(index);
        collect_macrolet_call_head_renames_from_view(
            child, child_path, dialect, from, to, kind, body_scope, renames,
        );
    }
}

fn local_callable_body_scope(
    scope: MacroletRenameScope,
    kind: LocalCallableRenameKind,
    form: LocalCallableForm,
    binds_from: bool,
) -> MacroletRenameScope {
    if !binds_from {
        return scope;
    }

    match kind {
        LocalCallableRenameKind::Macro
            if is_macro_callable_form(form)
                && !scope.is_target_active()
                && !scope.is_shadowed() =>
        {
            scope.enter_active_target()
        }
        LocalCallableRenameKind::Function
            if matches!(form, LocalCallableForm::Flet | LocalCallableForm::Labels)
                && !scope.is_target_active()
                && !scope.is_shadowed() =>
        {
            scope.enter_active_target()
        }
        LocalCallableRenameKind::Macro | LocalCallableRenameKind::Function => scope.enter_shadowed(),
    }
}

fn local_callable_binding_body_scope(
    scope: MacroletRenameScope,
    kind: LocalCallableRenameKind,
    form: LocalCallableForm,
    binds_from: bool,
) -> MacroletRenameScope {
    if !binds_from {
        return scope;
    }

    match (kind, form) {
        (LocalCallableRenameKind::Macro, LocalCallableForm::Labels) => scope.enter_shadowed(),
        (LocalCallableRenameKind::Function, LocalCallableForm::Labels)
            if !scope.is_target_active() && !scope.is_shadowed() =>
        {
            scope.enter_active_target()
        }
        (LocalCallableRenameKind::Function, LocalCallableForm::Labels) => scope.enter_shadowed(),
        (
            LocalCallableRenameKind::Macro | LocalCallableRenameKind::Function,
            LocalCallableForm::Flet
            | LocalCallableForm::Macrolet
            | LocalCallableForm::CompilerMacrolet,
        ) => scope,
    }
}

fn definition_body_start_index(
    category: Option<crate::domain::definition::DefinitionCategory>,
) -> usize {
    match category {
        Some(category) if category.is_callable() => 3,
        Some(_) => 2,
        None => 0,
    }
}
