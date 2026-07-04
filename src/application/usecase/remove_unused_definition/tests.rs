use std::path::PathBuf;

use proptest::prelude::*;

use super::*;
use crate::application::usecase::package_report::PackageDefinitionReport;
use crate::domain::definition::DefinitionCategory;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{ByteOffset, ByteSpan, SyntaxTree};

fn file_with_definitions(
    text: &str,
    definitions: Vec<UnusedDefinitionDefinition>,
) -> RemoveUnusedDefinitionInputFile {
    let tree = SyntaxTree::parse(text).expect("fixture must parse");
    RemoveUnusedDefinitionInputFile {
        path: PathBuf::from("core.lisp"),
        dialect: Dialect::CommonLisp,
        package: Some("app".to_owned()),
        definitions,
        atoms: tree.atom_occurrences(),
        text: text.to_owned(),
    }
}

fn definition(
    text: &str,
    form: &str,
    name: &str,
    category: DefinitionCategory,
) -> UnusedDefinitionDefinition {
    let start = text.find(form).expect("form must exist");
    UnusedDefinitionDefinition {
        path: "0".to_owned(),
        span: ByteSpan::new(ByteOffset::new(start), ByteOffset::new(start + form.len())),
        head: match category {
            DefinitionCategory::Test => "deftest",
            _ => "defun",
        }
        .to_owned(),
        name: Some(name.to_owned()),
        category,
        parameter_count: Some(0),
        body_form_count: Some(1),
        package: Some("app".to_owned()),
    }
}

fn request_for(
    text: &str,
    definitions: Vec<UnusedDefinitionDefinition>,
) -> RemoveUnusedDefinitionsRequest {
    RemoveUnusedDefinitionsRequest {
        files: vec![file_with_definitions(text, definitions)],
        package_definitions: Vec::new(),
        include_protected: false,
        include_exported: false,
    }
}

#[test]
fn plans_private_unused_definition_removal() {
    let text = "(in-package #:app)\n(defun stale-helper () 1)\n(defun live () 2)\n(live)\n";
    let stale_form = "(defun stale-helper () 1)";
    let live_form = "(defun live () 2)";
    let plan = plan_remove_unused_definitions(request_for(
        text,
        vec![
            definition(
                text,
                stale_form,
                "stale-helper",
                DefinitionCategory::Function,
            ),
            definition(text, live_form, "live", DefinitionCategory::Function),
        ],
    ))
    .expect("plan should build");

    assert_eq!(plan.candidate_count, 1);
    assert_eq!(plan.removal_count, 1);
    assert_eq!(plan.skipped_count, 0);
    assert!(!plan.files[0].rewritten.contains(stale_form));
    SyntaxTree::parse(&plan.files[0].rewritten).expect("rewrite must stay parseable");
}

#[test]
fn skips_protected_definition_categories_by_default() {
    let text = "(in-package #:app)\n(deftest stale-test () (is t))\n";
    let form = "(deftest stale-test () (is t))";
    let plan = plan_remove_unused_definitions(request_for(
        text,
        vec![definition(
            text,
            form,
            "stale-test",
            DefinitionCategory::Test,
        )],
    ))
    .expect("plan should build");

    assert_eq!(plan.candidate_count, 1);
    assert_eq!(plan.removal_count, 0);
    assert_eq!(plan.skipped_count, 1);
    assert_eq!(
        plan.files[0].skipped[0].reason,
        SkippedDefinitionRemovalReason::ProtectedDefinitionCategory
    );
}

#[test]
fn skips_exported_definition_by_default() {
    let text = "(in-package #:app)\n(defun public-entry () 1)\n";
    let form = "(defun public-entry () 1)";
    let mut request = request_for(
        text,
        vec![definition(
            text,
            form,
            "public-entry",
            DefinitionCategory::Function,
        )],
    );
    request.package_definitions.push(PackageDefinitionReport {
        path: "0".to_owned(),
        span: ByteSpan::new(ByteOffset::new(0), ByteOffset::new(0)),
        name: "#:app".to_owned(),
        nicknames: Vec::new(),
        uses: Vec::new(),
        exports: vec!["#:public-entry".to_owned()],
        imports: Vec::new(),
        option_count: 1,
    });

    let plan = plan_remove_unused_definitions(request).expect("plan should build");

    assert_eq!(plan.candidate_count, 1);
    assert_eq!(plan.removal_count, 0);
    assert_eq!(plan.skipped_count, 1);
    assert_eq!(
        plan.files[0].skipped[0].reason,
        SkippedDefinitionRemovalReason::ExportedDefinition
    );
}

fn lisp_symbol_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9-]{0,12}".prop_filter("avoid obvious generated edge names", |name| {
        !matches!(
            name.as_str(),
            "defun" | "in-package" | "lambda" | "nil" | "t"
        )
    })
}

proptest! {
    #[test]
    fn pbt_unused_private_function_rewrite_is_parseable(name in lisp_symbol_strategy()) {
        let form = format!("(defun {name} () 1)");
        let text = format!("(in-package #:app)\n{form}\n(defun live () 2)\n");
        let definitions = vec![
            definition(&text, &form, &name, DefinitionCategory::Function),
            definition(&text, "(defun live () 2)", "live", DefinitionCategory::Function),
        ];

        let plan = plan_remove_unused_definitions(request_for(&text, definitions)).expect("plan should build");

        prop_assert!(plan.changed);
        prop_assert_eq!(plan.removal_count, 2);
        prop_assert!(!plan.files[0].rewritten.contains(&form));
        prop_assert!(SyntaxTree::parse(&plan.files[0].rewritten).is_ok());
    }
}
