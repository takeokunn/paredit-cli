use std::path::PathBuf;

use anyhow::Result;

use crate::domain::common_lisp::CommonLispPackageDeclarationForm;
use crate::domain::definition::{definition_shape, DefinitionCategory};
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{
    AtomOccurrence, ByteSpan, Delimiter, ExpressionKind, ExpressionView, Path, SyntaxTree,
};

fn atom_text(view: &ExpressionView) -> Option<&str> {
    (view.kind == ExpressionKind::Atom)
        .then_some(view.text.as_deref())
        .flatten()
}

fn atom_child(view: &ExpressionView, index: usize) -> Option<&str> {
    view.children.get(index).and_then(atom_text)
}

fn list_head(view: &ExpressionView) -> Option<&str> {
    if view.kind != ExpressionKind::List || view.delimiter != Some(Delimiter::Paren) {
        return None;
    }

    atom_child(view, 0)
}

#[derive(Debug, Clone, Copy)]
pub struct UnusedDefinitionPolicyOptions {
    fail_on_unused: bool,
    require_unused_definitions: Option<usize>,
}

impl UnusedDefinitionPolicyOptions {
    pub fn new(
        fail_on_unused: bool,
        require_unused_definitions: Option<usize>,
    ) -> Result<Self, String> {
        if matches!(require_unused_definitions, Some(0)) {
            return Err("require-unused-definitions must be greater than zero".to_string());
        }

        Ok(Self {
            fail_on_unused,
            require_unused_definitions,
        })
    }

    pub const fn fail_on_unused(self) -> bool {
        self.fail_on_unused
    }

    pub const fn require_unused_definitions(self) -> Option<usize> {
        self.require_unused_definitions
    }
}

#[derive(Debug, Clone)]
pub struct DefinitionReportItem {
    pub path: String,
    pub span: ByteSpan,
    pub head: String,
    pub name: Option<String>,
    pub category: DefinitionCategory,
    pub parameter_count: Option<usize>,
    pub body_form_count: Option<usize>,
    pub package: Option<String>,
}

#[derive(Debug)]
pub struct DefinitionReportFile {
    pub path: PathBuf,
    pub dialect: Dialect,
    pub package: Option<String>,
    pub definitions: Vec<DefinitionReportItem>,
}

#[derive(Debug)]
pub struct ParsedDefinitionFile {
    pub path: PathBuf,
    pub dialect: Dialect,
    pub package: Option<String>,
    pub definitions: Vec<DefinitionReportItem>,
    pub atoms: Vec<AtomOccurrence>,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct DefinitionReference {
    pub file_index: usize,
    pub path: String,
    pub span: ByteSpan,
}

#[derive(Debug)]
pub struct UnusedDefinitionItem {
    pub definition: DefinitionReportItem,
    pub references: Vec<DefinitionReference>,
}

#[derive(Debug)]
pub struct UnusedDefinitionFile {
    pub path: PathBuf,
    pub dialect: Dialect,
    pub package: Option<String>,
    pub definitions: Vec<UnusedDefinitionItem>,
}

#[derive(Debug)]
pub struct UnusedDefinitionPolicy {
    pub fail_on_unused: bool,
    pub require_unused_definitions: Option<usize>,
    pub definition_count: usize,
    pub candidate_count: usize,
    pub actionable_candidate_count: usize,
    pub passed: bool,
    pub violations: Vec<String>,
}

pub fn build_definition_report(
    path: PathBuf,
    dialect: Dialect,
    tree: &SyntaxTree,
) -> Result<DefinitionReportFile> {
    let (package, definitions) = collect_definition_forms(tree, dialect)?;

    Ok(DefinitionReportFile {
        path,
        dialect,
        package,
        definitions,
    })
}

pub fn build_parsed_definition_file(
    path: PathBuf,
    dialect: Dialect,
    tree: &SyntaxTree,
    text: &str,
) -> Result<ParsedDefinitionFile> {
    let (package, definitions) = collect_definition_forms(tree, dialect)?;

    Ok(ParsedDefinitionFile {
        path,
        dialect,
        package,
        definitions,
        atoms: tree.atom_occurrences(),
        text: text.to_owned(),
    })
}

pub fn collect_definition_forms(
    tree: &SyntaxTree,
    dialect: Dialect,
) -> Result<(Option<String>, Vec<DefinitionReportItem>)> {
    let mut current_package = None;
    let mut definitions = Vec::new();

    for index in 0..tree.root_children().len() {
        let path = Path::root_child(index);
        let view = tree.select_path(&path)?.view();
        let Some(head) = list_head(&view) else {
            continue;
        };

        if dialect.common_lisp_package_declaration_form_for_head(head)
            == Some(CommonLispPackageDeclarationForm::InPackage)
        {
            if let Some(package_name) = atom_child(&view, 1) {
                current_package = Some(package_name.to_owned());
            }
            continue;
        }

        let Some(shape) = definition_shape(dialect, &view, head) else {
            continue;
        };

        definitions.push(DefinitionReportItem {
            path: path.to_string(),
            span: view.span,
            head: head.to_owned(),
            name: shape.name(&view).map(ToOwned::to_owned),
            category: shape.category,
            parameter_count: shape.lambda_parameter_count(&view),
            body_form_count: Some(shape.body_form_count(&view)),
            package: current_package.clone(),
        });
    }

    Ok((current_package, definitions))
}

pub fn unused_definition_candidate_count(reports: &[UnusedDefinitionFile]) -> usize {
    reports
        .iter()
        .flat_map(|report| &report.definitions)
        .filter(|item| item.references.is_empty())
        .count()
}

pub fn unused_definition_actionable_candidate_count(reports: &[UnusedDefinitionFile]) -> usize {
    reports
        .iter()
        .flat_map(|report| &report.definitions)
        .filter(|item| item.references.is_empty() && item.definition.category.is_bulk_removable())
        .count()
}

pub fn evaluate_unused_definition_policy(
    options: UnusedDefinitionPolicyOptions,
    reports: &[UnusedDefinitionFile],
) -> UnusedDefinitionPolicy {
    let definition_count = reports
        .iter()
        .map(|report| report.definitions.len())
        .sum::<usize>();
    let candidate_count = unused_definition_candidate_count(reports);
    let actionable_candidate_count = unused_definition_actionable_candidate_count(reports);
    let mut violations = Vec::new();

    if options.fail_on_unused() && actionable_candidate_count > 0 {
        violations.push(format!(
            "actionable_candidate_count {actionable_candidate_count} exceeds 0"
        ));
    }
    if let Some(required) = options.require_unused_definitions() {
        if actionable_candidate_count < required {
            violations.push(format!(
                "actionable_candidate_count {actionable_candidate_count} is below required {required}"
            ));
        }
    }

    UnusedDefinitionPolicy {
        fail_on_unused: options.fail_on_unused(),
        require_unused_definitions: options.require_unused_definitions(),
        definition_count,
        candidate_count,
        actionable_candidate_count,
        passed: violations.is_empty(),
        violations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_unused_definition_threshold() {
        assert!(UnusedDefinitionPolicyOptions::new(true, Some(1)).is_ok());
        assert_eq!(
            UnusedDefinitionPolicyOptions::new(false, Some(0)).unwrap_err(),
            "require-unused-definitions must be greater than zero"
        );
    }
}
