use anyhow::Result;

use crate::application::call_report::syntax::{
    definition_body_start_index, definition_name, list_head,
};
use crate::application::call_report::types::CallReportItem;
use crate::domain::definition::classify_definition_head;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{
    Delimiter, ExpressionKind, ExpressionView, Path, SymbolName, SyntaxTree,
};

pub fn build_call_report(
    tree: &SyntaxTree,
    dialect: Dialect,
    symbol: Option<&SymbolName>,
    include_definitions: bool,
) -> Result<Vec<CallReportItem>> {
    let mut calls = Vec::new();

    for index in 0..tree.root_children().len() {
        let path_indexes = vec![index];
        let path = Path::from_indexes(path_indexes.clone());
        let view = tree.select_path(&path)?.view();
        collect_call_report_items_from_view(
            &view,
            dialect,
            path_indexes,
            symbol,
            include_definitions,
            None,
            &mut calls,
        );
    }

    calls.sort_by_key(|call| call.span.start());
    Ok(calls)
}

fn collect_call_report_items_from_view(
    view: &ExpressionView,
    dialect: Dialect,
    path_indexes: Vec<usize>,
    symbol: Option<&SymbolName>,
    include_definitions: bool,
    enclosing_definition: Option<String>,
    calls: &mut Vec<CallReportItem>,
) {
    let mut child_enclosing_definition = enclosing_definition.clone();
    let mut first_callable_child_index = 0;

    if view.kind == ExpressionKind::List
        && view.delimiter == Some(Delimiter::Paren)
        && let Some(head) = list_head(view)
    {
        let category = classify_definition_head(dialect, head);
        let matches_symbol = symbol.is_none_or(|target| head == target.as_str());

        if matches_symbol && (include_definitions || category.is_none()) {
            calls.push(CallReportItem {
                path: Path::from_indexes(path_indexes.clone()).to_string(),
                span: view.span,
                head: head.to_owned(),
                argument_count: view.children.len().saturating_sub(1),
                category,
                enclosing_definition: enclosing_definition.clone(),
            });
        }

        if category.is_some() {
            child_enclosing_definition = definition_name(view, head).map(ToOwned::to_owned);
            first_callable_child_index = definition_body_start_index(category);
        }
    }

    for (index, child) in view.children.iter().enumerate() {
        if index < first_callable_child_index {
            continue;
        }
        let mut child_path = path_indexes.clone();
        child_path.push(index);
        collect_call_report_items_from_view(
            child,
            dialect,
            child_path,
            symbol,
            include_definitions,
            child_enclosing_definition.clone(),
            calls,
        );
    }
}
