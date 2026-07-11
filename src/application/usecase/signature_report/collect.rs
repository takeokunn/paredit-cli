use std::collections::BTreeMap;

use anyhow::Result;

use crate::application::usecase::call_report::build_call_report;
use crate::application::usecase::signature_report::calls::classify_signature_call;
use crate::application::usecase::signature_report::syntax::list_head;
use crate::application::usecase::signature_report::types::{
    SignatureCallItem, SignatureDefinitionItem, SignatureReportFile, SignatureReportSource,
};
use crate::domain::common_lisp::{common_lisp_operator_head_eq, common_lisp_symbol_name_eq};
use crate::domain::definition::DefinitionCategory;
use crate::domain::definition::definition_shape;
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
        let path = Path::root_child(index);
        let view = tree.select_path(&path)?.view();
        let Some(head) = list_head(&view) else {
            continue;
        };
        let Some(shape) = definition_shape(dialect, &view, head) else {
            continue;
        };
        let is_symbol_macro_definition = shape.category == DefinitionCategory::Variable
            && common_lisp_operator_head_eq(head, "define-symbol-macro");

        if !shape.category.is_callable() && !is_symbol_macro_definition {
            continue;
        }

        let name = shape.name(&view).map(ToOwned::to_owned);
        if name.as_deref().is_none_or(|name| {
            symbol.is_some_and(|target| !common_lisp_symbol_name_eq(name, target.as_str()))
        }) {
            continue;
        }

        let parameter_count = if is_symbol_macro_definition {
            None
        } else {
            let Some(parameter_count) = shape.lambda_parameter_count(&view) else {
                continue;
            };
            Some(parameter_count)
        };

        definitions.push(SignatureDefinitionItem {
            path: path.clone(),
            span: view.span,
            head: head.to_owned(),
            name,
            category: shape.category,
            parameter_count,
        });
    }

    Ok(definitions)
}
