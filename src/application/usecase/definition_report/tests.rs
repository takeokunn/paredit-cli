use std::path::PathBuf;

use proptest::prelude::*;

use crate::domain::definition::DefinitionCategory;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::SyntaxTree;

use super::*;

#[test]
fn builds_definition_inventory_with_package_and_counts() {
    let input = "(in-package #:demo)\n\
             (defun render-pane (session pane) (list session pane))\n\
             (defmacro with-pane ((pane) &body body) `(progn ,pane ,@body))\n\
             (define-symbol-macro current-user (slot-value *session* 'user))\n";
    let tree = SyntaxTree::parse(input).expect("parse input");

    let report = build_definition_report(PathBuf::from("core.lisp"), Dialect::CommonLisp, &tree)
        .expect("build report");

    assert_eq!(report.package.as_deref(), Some("#:demo"));
    assert_eq!(report.definitions.len(), 3);
    assert_eq!(report.definitions[0].name.as_deref(), Some("render-pane"));
    assert_eq!(report.definitions[0].parameter_count, Some(2));
    assert_eq!(report.definitions[0].body_form_count, Some(1));
    assert_eq!(report.definitions[0].package.as_deref(), Some("#:demo"));
    assert_eq!(report.definitions[1].name.as_deref(), Some("with-pane"));
    assert_eq!(report.definitions[1].parameter_count, Some(2));
    assert_eq!(report.definitions[2].name.as_deref(), Some("current-user"));
    assert_eq!(report.definitions[2].category, DefinitionCategory::Variable);
    assert_eq!(report.definitions[2].parameter_count, None);
    assert_eq!(report.definitions[2].body_form_count, Some(1));
    assert_eq!(report.definitions[2].package.as_deref(), Some("#:demo"));
}

#[test]
fn unused_candidates_ignore_self_references_and_count_external_references() {
    let input = "(defun used () :ok)\n\
             (defun caller () (used))\n\
             (defun recursive-unused () (recursive-unused))\n";
    let tree = SyntaxTree::parse(input).expect("parse input");
    let parsed =
        build_parsed_definition_file(PathBuf::from("core.lisp"), Dialect::CommonLisp, &tree)
            .expect("build parsed file");

    let reports = collect_unused_definition_candidates(&[parsed]);
    let used = reports[0]
        .definitions
        .iter()
        .find(|item| item.definition.name.as_deref() == Some("used"))
        .expect("used definition");
    let recursive_unused = reports[0]
        .definitions
        .iter()
        .find(|item| item.definition.name.as_deref() == Some("recursive-unused"))
        .expect("recursive definition");

    assert_eq!(used.references.len(), 1);
    assert_eq!(recursive_unused.references.len(), 0);
    assert_eq!(unused_definition_candidate_count(&reports), 2);
}

#[test]
fn policy_reports_fail_on_unused_and_required_minimum() {
    let input = "(defun stale () :stale)\n";
    let tree = SyntaxTree::parse(input).expect("parse input");
    let parsed =
        build_parsed_definition_file(PathBuf::from("core.lisp"), Dialect::CommonLisp, &tree)
            .expect("build parsed file");
    let reports = collect_unused_definition_candidates(&[parsed]);

    let policy = evaluate_unused_definition_policy(
        UnusedDefinitionPolicyOptions {
            fail_on_unused: true,
            require_unused_definitions: Some(2),
        },
        &reports,
    );

    assert_eq!(policy.definition_count, 1);
    assert_eq!(policy.candidate_count, 1);
    assert!(!policy.passed);
    assert_eq!(policy.violations.len(), 2);
}

proptest! {
    #[test]
    fn pbt_referenced_generated_function_is_not_unused(
        function_name in "[a-z][a-z0-9-]{0,12}",
        caller_name in "[a-z][a-z0-9-]{0,12}",
        arg_count in 0usize..8,
    ) {
        prop_assume!(function_name != caller_name);
        let params = (0..arg_count)
            .map(|index| format!("arg{index}"))
            .collect::<Vec<_>>();
        let args = (0..arg_count)
            .map(|_| ":value".to_owned())
            .collect::<Vec<_>>();
        let input = format!(
            "(defun {function_name} ({}) :ok)\n(defun {caller_name} () ({function_name} {}))\n",
            params.join(" "),
            args.join(" ")
        );
        let tree = SyntaxTree::parse(&input).expect("parse generated input");
        let parsed = build_parsed_definition_file(
            PathBuf::from("generated.lisp"),
            Dialect::CommonLisp,
            &tree,
        )
        .expect("build generated parsed file");

        let reports = collect_unused_definition_candidates(&[parsed]);
        let generated = reports[0]
            .definitions
            .iter()
            .find(|item| item.definition.name.as_deref() == Some(function_name.as_str()))
            .expect("generated function definition");

        prop_assert_eq!(generated.definition.parameter_count, Some(arg_count));
        prop_assert_eq!(generated.references.len(), 1);
    }
}
