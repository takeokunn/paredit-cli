use std::collections::{HashMap, HashSet};

use crate::application::usecase::package_report::PackageDefinitionReport;
use crate::application::usecase::remove_unused_definition::types::UnusedDefinitionDefinition;
use crate::domain::definition::DefinitionCategory;

/// `DefinitionCategory::UnknownMacro` covers a `define-*`-prefixed macro
/// this tool does not recognize, whose expansion is unknown. Such a macro
/// commonly derives *other* symbol names from its argument via string
/// concatenation (for example a strategy DSL where `(define-strategy foo
/// ...)` generates and exports `make-foo-strategy`), so "is the argument
/// symbol referenced elsewhere" is not a safe proxy for "is this definition
/// unused": the argument symbol itself may legitimately have zero direct
/// references while the code it defines is very much in use. Bulk removal
/// therefore requires the same explicit `--include-protected` opt-in as
/// other categories this tool cannot fully verify. This is distinct from
/// `Other`, which covers a dialect's own recognized definition forms (for
/// example Emacs Lisp `defun`/`defvar` or Clojure `defn`) that are not
/// broken out into a more specific category but are still known,
/// non-generative shapes.
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
        let normalized_exports: Vec<String> = package
            .exports
            .iter()
            .map(|symbol| normalize_symbol_key(symbol))
            .collect();

        for package_key in package_export_keys(package) {
            let symbols = exported.entry(package_key).or_insert_with(HashSet::new);
            symbols.extend(normalized_exports.iter().cloned());
        }
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

fn package_export_keys(package: &PackageDefinitionReport) -> Vec<String> {
    let mut keys = Vec::with_capacity(1 + package.nicknames.len());
    keys.push(normalize_package_key(&package.name));
    keys.extend(
        package
            .nicknames
            .iter()
            .map(|nickname| normalize_package_key(nickname)),
    );
    keys.sort();
    keys.dedup();
    keys
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
