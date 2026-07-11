use anyhow::Result;

use crate::domain::common_lisp::{common_lisp_operator_head_eq, common_lisp_symbol_name_eq};
use crate::domain::definition::{DefinitionCategory, definition_shape};
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{ExpressionKind, ExpressionView, Path, SymbolName, SyntaxTree};

use super::super::RenameFunctionOccurrence;
use super::super::reader::{
    apply_reader_prefix_context, explicit_reader_form_kind,
    explicit_reader_function_lambda_body_children,
};
use super::shared::list_head;

pub fn collect_define_symbol_macro_definition_renames(
    tree: &SyntaxTree,
    dialect: Dialect,
    from: &SymbolName,
    to: &SymbolName,
) -> Result<Vec<RenameFunctionOccurrence>> {
    let mut renames = Vec::new();

    for (top_index, _) in tree.root_children().iter().enumerate() {
        collect_definition_renames_from_path(
            tree,
            Path::root_child(top_index),
            dialect,
            from,
            to,
            &mut renames,
        )?;
    }

    Ok(renames)
}

fn collect_definition_renames_from_path(
    tree: &SyntaxTree,
    path: Path,
    dialect: Dialect,
    from: &SymbolName,
    to: &SymbolName,
    renames: &mut Vec<RenameFunctionOccurrence>,
) -> Result<()> {
    let view = tree.select_path(&path)?.view();
    collect_definition_renames_from_view(&view, path, dialect, from, to, 0, renames)
}

#[allow(clippy::too_many_arguments)]
fn collect_definition_renames_from_view(
    view: &ExpressionView,
    path: Path,
    dialect: Dialect,
    from: &SymbolName,
    to: &SymbolName,
    quasiquote_depth: usize,
    renames: &mut Vec<RenameFunctionOccurrence>,
) -> Result<()> {
    let Some(quasiquote_depth) = apply_reader_prefix_context(view, quasiquote_depth) else {
        return Ok(());
    };

    if collect_explicit_reader_form_definition_renames(
        view,
        &path,
        dialect,
        from,
        to,
        quasiquote_depth,
        renames,
    )? {
        return Ok(());
    }

    if quasiquote_depth == 0 {
        collect_target_definition_rename(view, &path, dialect, from, to, renames);
    }

    for (child_index, child) in view.children.iter().enumerate() {
        collect_definition_renames_from_view(
            child,
            path.child(child_index),
            dialect,
            from,
            to,
            quasiquote_depth,
            renames,
        )?;
    }

    Ok(())
}

fn collect_target_definition_rename(
    view: &ExpressionView,
    path: &Path,
    dialect: Dialect,
    from: &SymbolName,
    to: &SymbolName,
    renames: &mut Vec<RenameFunctionOccurrence>,
) {
    let Some(head) = list_head(view) else {
        return;
    };
    if !common_lisp_operator_head_eq(head, "define-symbol-macro") {
        return;
    }
    let Some(shape) = definition_shape(dialect, view, head)
        .filter(|shape| shape.category == DefinitionCategory::Variable)
    else {
        return;
    };
    let Some(name_target) = shape.name_target(view, path) else {
        return;
    };
    if !common_lisp_symbol_name_eq(name_target.text, from.as_str()) {
        return;
    }
    renames.push(RenameFunctionOccurrence {
        path: name_target.path.to_string(),
        span: name_target.span,
        text: from.as_str().to_owned(),
        replacement: to.as_str().to_owned(),
    });
}

#[allow(clippy::too_many_arguments)]
fn collect_explicit_reader_form_definition_renames(
    view: &ExpressionView,
    path: &Path,
    dialect: Dialect,
    from: &SymbolName,
    to: &SymbolName,
    quasiquote_depth: usize,
    renames: &mut Vec<RenameFunctionOccurrence>,
) -> Result<bool> {
    if view.kind != ExpressionKind::List || view.children.len() < 2 {
        return Ok(false);
    }

    let Some(kind_name) = explicit_reader_form_kind(view) else {
        return Ok(false);
    };

    match kind_name.as_str() {
        "quote" => Ok(true),
        "function" if quasiquote_depth == 0 => {
            if let Some(children) = explicit_reader_function_lambda_body_children(view) {
                for (child_index, child) in children {
                    collect_definition_renames_from_view(
                        child,
                        path.child(1).child(child_index),
                        dialect,
                        from,
                        to,
                        quasiquote_depth,
                        renames,
                    )?;
                }
            }
            Ok(true)
        }
        "function" => Ok(true),
        "quasiquote" => {
            for (child_index, child) in view.children.iter().enumerate().skip(1) {
                collect_definition_renames_from_view(
                    child,
                    path.child(child_index),
                    dialect,
                    from,
                    to,
                    quasiquote_depth + 1,
                    renames,
                )?;
            }
            Ok(true)
        }
        "unquote" | "unquote-splicing" if quasiquote_depth > 0 => {
            for (child_index, child) in view.children.iter().enumerate().skip(1) {
                collect_definition_renames_from_view(
                    child,
                    path.child(child_index),
                    dialect,
                    from,
                    to,
                    quasiquote_depth - 1,
                    renames,
                )?;
            }
            Ok(true)
        }
        _ => Ok(false),
    }
}
