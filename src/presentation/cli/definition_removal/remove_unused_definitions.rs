use std::fs;

use anyhow::{Context, Result};

use super::super::shared::{detect_dialect, read_input};
use super::args::RemoveUnusedDefinitionsArgs;
use super::render::print_remove_unused_definitions_plan;
use crate::application::definition_report::{DefinitionReportItem, collect_definition_forms};
use crate::application::usecase::package_report::build_package_report;
use crate::application::usecase::remove_unused_definition::{
    RemoveUnusedDefinitionInputFile, RemoveUnusedDefinitionsRequest, UnusedDefinitionDefinition,
    plan_remove_unused_definitions,
};
use crate::domain::sexpr::SyntaxTree;

pub(in crate::presentation::cli) fn remove_unused_definitions(
    args: RemoveUnusedDefinitionsArgs,
) -> Result<()> {
    let mut input_files = Vec::with_capacity(args.files.len());
    let mut package_definitions = Vec::new();

    for file in &args.files {
        let input = read_input(Some(file.clone()))?;
        let dialect = detect_dialect(&input, args.dialect);
        let tree = SyntaxTree::parse(&input.text)
            .with_context(|| format!("failed to parse {}", file.display()))?;
        let (package, definitions) = collect_definition_forms(&tree, dialect)?;
        let package_report = build_package_report(&tree)
            .with_context(|| format!("failed to inspect packages in {}", file.display()))?;
        package_definitions.extend(package_report.defpackages);

        input_files.push(RemoveUnusedDefinitionInputFile {
            path: file.clone(),
            dialect,
            package,
            definitions: definitions
                .iter()
                .map(to_unused_definition_definition)
                .collect(),
            atoms: tree.atom_occurrences(),
            text: input.text,
        });
    }

    let plan = plan_remove_unused_definitions(RemoveUnusedDefinitionsRequest {
        files: input_files,
        package_definitions,
        include_protected: args.include_protected,
        include_exported: args.include_exported,
    })?;

    let written = args.write && plan.changed;
    if written {
        for file in &plan.files {
            if file.changed {
                fs::write(&file.path, &file.rewritten)
                    .with_context(|| format!("failed to write {}", file.path.display()))?;
            }
        }
    }

    print_remove_unused_definitions_plan(&plan, written, args.output)
}

fn to_unused_definition_definition(
    definition: &DefinitionReportItem,
) -> UnusedDefinitionDefinition {
    UnusedDefinitionDefinition {
        path: definition.path.clone(),
        span: definition.span,
        head: definition.head.clone(),
        name: definition.name.clone(),
        category: definition.category,
        parameter_count: definition.parameter_count,
        body_form_count: definition.body_form_count,
        package: definition.package.clone(),
    }
}
