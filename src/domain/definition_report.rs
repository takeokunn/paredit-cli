use std::collections::HashSet;
use std::path::PathBuf;
use std::thread;

use anyhow::Result;

use crate::domain::common_lisp::CommonLispPackageDeclarationForm;
use crate::domain::definition::{DefinitionCategory, definition_shape};
use crate::domain::definition_reference::{
    collect_package_form_spans, collect_reference_needles, collect_symbol_references,
};
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{
    AtomOccurrence, ByteSpan, Delimiter, ExpressionKind, ExpressionView, Path, SymbolName,
    SyntaxTree,
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

pub fn collect_unused_definition_candidates(
    files: &[ParsedDefinitionFile],
) -> Vec<UnusedDefinitionFile> {
    let views: Vec<_> = files
        .iter()
        .map(|file| {
            SyntaxTree::parse(&file.text)
                .ok()
                .map(|tree| tree.root_view())
        })
        .collect();

    let package_form_spans: Vec<Vec<ByteSpan>> = files
        .iter()
        .enumerate()
        .map(|(index, file)| {
            let mut spans = Vec::new();
            if let Some(view) = &views[index] {
                collect_package_form_spans(file.dialect, view, &mut spans);
            }
            spans
        })
        .collect();
    let atom_needles: Vec<HashSet<String>> = views
        .iter()
        .map(|view| {
            let mut needles = HashSet::new();
            if let Some(view) = view {
                collect_reference_needles(view, &mut needles);
            }
            needles
        })
        .collect();

    let worker_count = thread::available_parallelism()
        .map(|parallelism| parallelism.get())
        .unwrap_or(1)
        .clamp(1, files.len().max(1));
    let mut ordered: Vec<Option<UnusedDefinitionFile>> = (0..files.len()).map(|_| None).collect();
    thread::scope(|scope| {
        let views = &views;
        let package_form_spans = &package_form_spans;
        let atom_needles = &atom_needles;
        let handles: Vec<_> = (0..worker_count)
            .map(|worker| {
                scope.spawn(move || {
                    files
                        .iter()
                        .enumerate()
                        .skip(worker)
                        .step_by(worker_count)
                        .map(|(file_index, file)| {
                            (
                                file_index,
                                file_unused_definition_report(
                                    files,
                                    views,
                                    package_form_spans,
                                    atom_needles,
                                    file_index,
                                    file,
                                ),
                            )
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        for handle in handles {
            for (file_index, report) in handle
                .join()
                .expect("unused-definition reference worker thread panicked")
            {
                ordered[file_index] = Some(report);
            }
        }
    });
    ordered.into_iter().flatten().collect()
}

fn file_unused_definition_report(
    files: &[ParsedDefinitionFile],
    views: &[Option<ExpressionView>],
    package_form_spans: &[Vec<ByteSpan>],
    atom_needles: &[HashSet<String>],
    file_index: usize,
    file: &ParsedDefinitionFile,
) -> UnusedDefinitionFile {
    UnusedDefinitionFile {
        path: file.path.clone(),
        dialect: file.dialect,
        package: file.package.clone(),
        definitions: file
            .definitions
            .iter()
            .filter_map(|definition| {
                let name = definition.name.as_ref()?;
                let symbol = SymbolName::new(name.clone()).ok()?;
                let needle = crate::domain::common_lisp::common_lisp_symbol_reference_needle(
                    symbol.as_str(),
                );
                let references = files
                    .iter()
                    .enumerate()
                    .flat_map(|(other_index, other)| {
                        let mut spans = Vec::new();
                        if let Some(view) = views[other_index]
                            .as_ref()
                            .filter(|_| atom_needles[other_index].contains(&needle))
                        {
                            collect_symbol_references(
                                other.dialect,
                                view,
                                &symbol,
                                &other.text,
                                &mut spans,
                            );
                        }

                        let other_package_spans = &package_form_spans[other_index];
                        spans.retain(|span| {
                            !other_package_spans
                                .iter()
                                .any(|package| package.contains_span(*span))
                        });
                        spans
                            .into_iter()
                            .filter(move |span| {
                                !(other_index == file_index && definition.span.contains_span(*span))
                            })
                            .map(move |span| DefinitionReference {
                                file_index: other_index,
                                path: String::new(),
                                span,
                            })
                    })
                    .collect();

                Some(UnusedDefinitionItem {
                    definition: definition.clone(),
                    references,
                })
            })
            .collect(),
    }
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
