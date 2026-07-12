use std::path::PathBuf;

use crate::domain::definition::DefinitionCategory;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{AtomOccurrence, ByteSpan};

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
