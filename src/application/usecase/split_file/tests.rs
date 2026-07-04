use std::path::PathBuf;

use proptest::{prelude::*, test_runner::TestCaseError};

use super::rewrite::{append_top_level_definitions, expand_definition_removal, replace_byte_span};
use super::*;
use crate::domain::definition::DefinitionCategory;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{ByteOffset, ByteSpan, Path, SyntaxTree};

fn split_request<'a>(from_input: &'a str, to_input: &'a str) -> SplitFileRequest<'a> {
    SplitFileRequest {
        from_file: PathBuf::from("src/core.lisp"),
        to_file: PathBuf::from("src/ui/render.lisp"),
        from_input,
        to_input,
        from_dialect: Dialect::CommonLisp,
        to_dialect: Dialect::CommonLisp,
        paths: vec![Path::from_indexes(vec![1]), Path::from_indexes(vec![2])],
        names: Vec::new(),
        categories: Vec::new(),
        to_file_existed: false,
        to_parent_existed: false,
        write: false,
    }
}

fn generated_split_source(definition_count: usize) -> String {
    let mut input = "(in-package #:demo)\n\n(defun keep () :keep)\n".to_owned();
    for index in 0..definition_count {
        input.push_str(&format!("\n(defun moved-{index} (arg) (+ arg {index}))\n"));
    }
    input
}

#[test]
fn plan_split_file_moves_selected_definitions_in_source_order() {
    let plan = plan_split_file(split_request(
        "(in-package #:demo)\n\n(defun keep () :keep)\n\n(defun render () :render)\n\n(defmacro with-render () nil)\n",
        "",
    ))
    .expect("split-file plan should be valid");

    assert_eq!(plan.items.len(), 2);
    assert_eq!(plan.items[0].definition.name.as_deref(), Some("keep"));
    assert_eq!(plan.items[1].definition.name.as_deref(), Some("render"));
    assert!(plan.from_rewritten.contains("in-package"));
    assert!(!plan.from_rewritten.contains("defun keep"));
    assert!(!plan.from_rewritten.contains("defun render"));
    assert!(plan.to_rewritten.contains("(defun keep () :keep)"));
    assert!(plan.to_rewritten.contains("(defun render () :render)"));
    assert!(plan.to_rewritten.find("defun keep") < plan.to_rewritten.find("defun render"));
    assert!(plan.changed);
    assert!(!plan.written);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(32))]

    #[test]
    fn pbt_plan_split_file_preserves_parseability_and_order(definition_count in 1usize..8) {
        let source = generated_split_source(definition_count);
        let mut request = split_request(&source, "(in-package #:demo.moved)\n");
        request.paths = Vec::new();
        request.names = (0..definition_count)
            .map(|index| format!("moved-{index}"))
            .collect();

        let plan = plan_split_file(request)
            .map_err(|error| TestCaseError::fail(error.to_string()))?;

        prop_assert_eq!(plan.items.len(), definition_count);
        prop_assert!(plan.changed);
        prop_assert!(!plan.written);
        prop_assert!(SyntaxTree::parse(&plan.from_rewritten).is_ok());
        prop_assert!(SyntaxTree::parse(&plan.to_rewritten).is_ok());
        prop_assert!(plan.from_rewritten.contains("defun keep"));

        let mut previous_position = 0;
        for index in 0..definition_count {
            let needle = format!("defun moved-{index}");
            prop_assert!(!plan.from_rewritten.contains(&needle));
            let position = plan.to_rewritten.find(&needle)
                .ok_or_else(|| TestCaseError::fail(format!("{needle} missing from destination")))?;
            prop_assert!(position >= previous_position);
            previous_position = position;
        }
    }
}

#[test]
fn plan_split_file_rejects_duplicate_paths() {
    let mut request = split_request("(defun a () 1)\n", "");
    request.paths = vec![Path::from_indexes(vec![0]), Path::from_indexes(vec![0])];

    let error = plan_split_file(request).expect_err("duplicate paths should be rejected");

    assert!(error.to_string().contains("duplicate split-file path"));
}

#[test]
fn plan_split_file_selects_definitions_by_name_and_kind() {
    let mut request = split_request(
        "(in-package #:demo)\n\n(defun keep () :keep)\n\n(defun render () :render)\n\n(defmacro with-render () nil)\n\n(define-symbol-macro current-user (slot-value *session* 'user))\n",
        "",
    );
    request.paths = Vec::new();
    request.names = vec!["render".to_owned()];
    request.categories = vec![DefinitionCategory::Macro, DefinitionCategory::Variable];

    let plan = plan_split_file(request).expect("name and kind selectors should be valid");

    assert_eq!(plan.items.len(), 3);
    assert_eq!(plan.items[0].definition.name.as_deref(), Some("render"));
    assert_eq!(
        plan.items[1].definition.name.as_deref(),
        Some("with-render")
    );
    assert_eq!(
        plan.items[2].definition.name.as_deref(),
        Some("current-user")
    );
    assert_eq!(
        plan.items[2].definition.category,
        DefinitionCategory::Variable
    );
    assert!(plan.from_rewritten.contains("defun keep"));
    assert!(!plan.from_rewritten.contains("defun render"));
    assert!(!plan.from_rewritten.contains("defmacro with-render"));
    assert!(
        !plan
            .from_rewritten
            .contains("define-symbol-macro current-user")
    );
}

#[test]
fn plan_split_file_rejects_missing_name_selector() {
    let mut request = split_request("(defun a () 1)\n", "");
    request.paths = Vec::new();
    request.names = vec!["missing".to_owned()];

    let error = plan_split_file(request).expect_err("missing selector should fail");

    assert!(error.to_string().contains("--name did not match"));
}

#[test]
fn plan_split_file_requires_at_least_one_selector() {
    let mut request = split_request("(defun a () 1)\n", "");
    request.paths = Vec::new();

    let error = plan_split_file(request).expect_err("empty selectors should fail");

    assert!(error.to_string().contains("requires at least one"));
}

#[test]
fn append_top_level_definitions_keeps_spacing() {
    let definitions = vec![
        "(defun moved-a () 1)".to_owned(),
        "(defmacro moved-b () 2)".to_owned(),
    ];

    assert_eq!(
        append_top_level_definitions("(in-package #:demo)\n\n", &definitions),
        "(in-package #:demo)\n\n(defun moved-a () 1)\n\n(defmacro moved-b () 2)\n"
    );
}

#[test]
fn append_top_level_definitions_creates_destination_from_empty_input() {
    let definitions = vec!["(defun moved () :ok)".to_owned()];

    assert_eq!(
        append_top_level_definitions("", &definitions),
        "(defun moved () :ok)\n"
    );
}

#[test]
fn expand_definition_removal_consumes_following_separator_when_available() {
    let input = "(defun a () 1)\n\n(defun b () 2)\n";
    let span = ByteSpan::new(ByteOffset::new(0), ByteOffset::new("(defun a () 1)".len()));

    let expanded = expand_definition_removal(input, span);

    assert_eq!(expanded.start().get(), 0);
    assert_eq!(expanded.end().get(), "(defun a () 1)\n\n".len());
    assert_eq!(replace_byte_span(input, expanded, ""), "(defun b () 2)\n");
}

#[test]
fn expand_definition_removal_consumes_previous_separator_at_eof() {
    let input = "(defun a () 1)\n\n(defun b () 2)";
    let start = "(defun a () 1)\n\n".len();
    let span = ByteSpan::new(ByteOffset::new(start), ByteOffset::new(input.len()));

    let expanded = expand_definition_removal(input, span);

    assert_eq!(expanded.start().get(), "(defun a () 1)".len());
    assert_eq!(expanded.end().get(), input.len());
    assert_eq!(replace_byte_span(input, expanded, ""), "(defun a () 1)");
}
