use std::collections::{HashMap, HashSet};

use crate::application::usecase::package_report::PackageDefinitionReport;
use crate::application::usecase::remove_unused_definition::types::UnusedDefinitionDefinition;
use crate::domain::definition::DefinitionCategory;

pub(super) fn definition_is_bulk_removable(category: DefinitionCategory) -> bool {
    matches!(
        category,
        DefinitionCategory::Function
            | DefinitionCategory::Macro
            | DefinitionCategory::GenericFunction
            | DefinitionCategory::Method
            | DefinitionCategory::Class
            | DefinitionCategory::Struct
            | DefinitionCategory::Condition
            | DefinitionCategory::Variable
            | DefinitionCategory::Constant
            | DefinitionCategory::Parameter
            | DefinitionCategory::Other
    )
}

pub(super) fn collect_exported_symbol_index(
    packages: &[PackageDefinitionReport],
) -> HashMap<String, HashSet<String>> {
    let mut exported = HashMap::new();

    for package in packages {
        let symbols = exported
            .entry(normalize_package_key(&package.name))
            .or_insert_with(HashSet::new);
        symbols.extend(
            package
                .exports
                .iter()
                .map(|symbol| normalize_symbol_key(symbol)),
        );
    }

    exported
}

pub(super) fn definition_is_exported(
    definition: &UnusedDefinitionDefinition,
    exported_symbols: &HashMap<String, HashSet<String>>,
) -> bool {
    let (Some(package), Some(name)) = (&definition.package, &definition.name) else {
        return false;
    };
    let Some(exports) = exported_symbols.get(&normalize_package_key(package)) else {
        return false;
    };
    exports.contains(&normalize_symbol_key(name))
}

fn normalize_package_key(value: &str) -> String {
    normalize_keyword_prefix(value).to_ascii_lowercase()
}

fn normalize_symbol_key(value: &str) -> String {
    let normalized = normalize_keyword_prefix(value);
    let symbol = normalized
        .rsplit_once("::")
        .map(|(_, symbol)| symbol)
        .or_else(|| normalized.rsplit_once(':').map(|(_, symbol)| symbol))
        .unwrap_or(normalized);

    normalize_keyword_prefix(symbol).to_ascii_lowercase()
}

fn normalize_keyword_prefix(value: &str) -> &str {
    value
        .strip_prefix("#:")
        .or_else(|| value.strip_prefix(':'))
        .unwrap_or(value)
}
