use std::path::PathBuf;

use proptest::prelude::*;

use super::*;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::SyntaxTree;

fn source(input: &str) -> SignatureReportSource {
    SignatureReportSource {
        path: PathBuf::from("input.lisp"),
        dialect: Dialect::CommonLisp,
        tree: SyntaxTree::parse(input).expect("test input should parse"),
    }
}

#[test]
fn classifies_exact_missing_extra_and_unknown_calls() {
    let reports = build_signature_reports(
        vec![source("(defun f (x y) (g x) (f x) (f x y) (f x y z))")],
        None,
    )
    .unwrap();

    let statuses = reports[0]
        .calls
        .iter()
        .map(|item| (item.call.head.as_str(), item.status))
        .collect::<Vec<_>>();

    assert_eq!(
        statuses,
        vec![
            ("g", SignatureCallStatus::UnknownDefinition),
            ("f", SignatureCallStatus::MissingArguments),
            ("f", SignatureCallStatus::Exact),
            ("f", SignatureCallStatus::ExtraArguments),
        ]
    );
}

#[test]
fn reports_ambiguous_definition_when_multiple_signatures_exist() {
    let reports = build_signature_reports(
        vec![source("(defun f (x) (f x))\n(defun f (x y) (f x y))")],
        None,
    )
    .unwrap();

    assert!(
        reports[0]
            .calls
            .iter()
            .all(|item| item.status == SignatureCallStatus::AmbiguousDefinition)
    );
}

#[test]
fn evaluates_policy_thresholds() {
    let reports = build_signature_reports(vec![source("(defun f (x) (f) (f x))")], None).unwrap();
    let policy = evaluate_signature_report_policy(&reports, true, Some(1), Some(2));

    assert_eq!(policy.definition_count, 1);
    assert_eq!(policy.call_count, 2);
    assert_eq!(policy.mismatch_count, 1);
    assert!(!policy.passed);
    assert_eq!(policy.violations.len(), 1);
}

fn symbol_strategy() -> impl Strategy<Value = String> {
    "[a-z][a-z0-9-]{0,8}".prop_filter("exclude definition heads", |symbol| {
        !matches!(
            symbol.as_str(),
            "defun" | "fn" | "lambda" | "let" | "nil" | "t" | "true" | "false"
        )
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn pbt_classifies_generated_arity(
        function in symbol_strategy(),
        parameter_count in 0usize..6,
        call_argument_count in 0usize..6,
    ) {
        let parameters = (0..parameter_count)
            .map(|index| format!("p{index}"))
            .collect::<Vec<_>>()
            .join(" ");
        let arguments = (0..call_argument_count)
            .map(|index| format!("a{index}"))
            .collect::<Vec<_>>()
            .join(" ");
        let input = format!(
            "(defun {function} ({parameters}) ({function}{prefix}{arguments}))",
            prefix = if arguments.is_empty() { "" } else { " " },
        );
        let reports = build_signature_reports(vec![source(&input)], None).unwrap();
        let call = reports[0]
            .calls
            .iter()
            .find(|item| item.call.head == function)
            .expect("generated self-call should be reported");
        let expected_status = match call_argument_count.cmp(&parameter_count) {
            std::cmp::Ordering::Equal => SignatureCallStatus::Exact,
            std::cmp::Ordering::Less => SignatureCallStatus::MissingArguments,
            std::cmp::Ordering::Greater => SignatureCallStatus::ExtraArguments,
        };

        prop_assert_eq!(call.expected_parameter_count, Some(parameter_count));
        prop_assert_eq!(call.status, expected_status);
    }
}
