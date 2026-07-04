use std::collections::BTreeMap;

use anyhow::Result;

use crate::application::call_report::build_call_report;
use crate::application::signature_report::calls::classify_signature_call;
use crate::application::signature_report::syntax::{
    count_lambda_parameters, definition_name, lambda_list_index, list_head,
};
use crate::application::signature_report::types::{
    SignatureCallItem, SignatureDefinitionItem, SignatureReportFile, SignatureReportSource,
};
use crate::domain::definition::classify_definition_head;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{Path, SymbolName, SyntaxTree};

pub fn build_signature_reports(
    sources: Vec<SignatureReportSource>,
    symbol: Option<&SymbolName>,
) -> Result<Vec<SignatureReportFile>> {
    let mut parsed = Vec::with_capacity(sources.len());
    let mut definitions_by_name = BTreeMap::<String, Vec<usize>>::new();

    for source in sources {
        let definitions = collect_signature_definitions(&source.tree, source.dialect, symbol)?;
        let calls = build_call_report(&source.tree, source.dialect, symbol, false)?;

        for definition in &definitions {
            let Some(name) = &definition.name else {
                continue;
            };
            let Some(parameter_count) = definition.parameter_count else {
                continue;
            };
            definitions_by_name
                .entry(name.clone())
                .or_default()
                .push(parameter_count);
        }

        parsed.push((source.path, source.dialect, definitions, calls));
    }

    Ok(parsed
        .into_iter()
        .map(|(path, dialect, definitions, calls)| SignatureReportFile {
            path,
            dialect,
            definitions,
            calls: calls
                .into_iter()
                .map(|call| {
                    let (expected_parameter_count, status) =
                        classify_signature_call(&definitions_by_name, &call);
                    SignatureCallItem {
                        call,
                        expected_parameter_count,
                        status,
                    }
                })
                .collect(),
        })
        .collect())
}

pub(super) fn collect_signature_definitions(
    tree: &SyntaxTree,
    dialect: Dialect,
    symbol: Option<&SymbolName>,
) -> Result<Vec<SignatureDefinitionItem>> {
    let mut definitions = Vec::new();

    for index in 0..tree.root_children().len() {
        let path_indexes = vec![index];
        let path = Path::from_indexes(path_indexes.clone());
        let view = tree.select_path(&path)?.view();
        let Some(head) = list_head(&view) else {
            continue;
        };
        let Some(category) = classify_definition_head(dialect, head) else {
            continue;
        };
        if !category.is_callable() {
            continue;
        }

        let Some(lambda_index) = lambda_list_index(&view, head) else {
            continue;
        };
        let name = definition_name(&view, head).map(ToOwned::to_owned);
        if !name
            .as_deref()
            .is_some_and(|name| symbol.is_none_or(|target| name == target.as_str()))
        {
            continue;
        }

        definitions.push(SignatureDefinitionItem {
            path: Path::from_indexes(path_indexes).to_string(),
            span: view.span,
            head: head.to_owned(),
            name,
            category,
            parameter_count: Some(count_lambda_parameters(&view.children[lambda_index])),
        });
    }

    Ok(definitions)
}
