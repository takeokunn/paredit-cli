use super::*;

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
fn ignores_common_lisp_local_callable_calls_when_checking_global_signatures() {
    let reports = build_signature_reports(
        vec![source(
            "(defun helper (x y) y)\n(defun main () (flet ((helper (x) x)) (helper 1)))",
        )],
        None,
    )
    .unwrap();

    assert!(
        !reports[0]
            .calls
            .iter()
            .any(|item| item.call.head == "helper")
    );
}

#[test]
fn classifies_unqualified_call_against_package_qualified_common_lisp_definition() {
    let reports = build_signature_reports(
        vec![source(
            "(defun cl-user:target (x y) target)\n(defun caller () (target 1 2))",
        )],
        Some(&crate::domain::sexpr::SymbolName::new("target").unwrap()),
    )
    .unwrap();

    assert_eq!(reports[0].definitions.len(), 1);
    assert_eq!(
        reports[0].definitions[0].name.as_deref(),
        Some("cl-user:target")
    );
    assert_eq!(reports[0].definitions[0].parameter_count, Some(2));
    assert_eq!(reports[0].calls.len(), 1);
    assert_eq!(reports[0].calls[0].call.head, "target");
    assert_eq!(reports[0].calls[0].expected_parameter_count, Some(2));
    assert_eq!(reports[0].calls[0].status, SignatureCallStatus::Exact);
}

#[test]
fn classifies_common_lisp_symbol_macrolet_expansion_and_body_calls_without_binding_name_calls() {
    let reports = build_signature_reports(
        vec![source(
            "(defun helper (x) (+ x 10))\n(defun target (x) x)\n(defun render () (symbol-macrolet ((helper (target 1))) (list helper (target 2))))",
        )],
        Some(&crate::domain::sexpr::SymbolName::new("target").unwrap()),
    )
    .unwrap();

    assert_eq!(reports[0].definitions.len(), 1);
    assert_eq!(reports[0].definitions[0].name.as_deref(), Some("target"));
    assert_eq!(reports[0].definitions[0].parameter_count, Some(1));
    assert_eq!(reports[0].calls.len(), 2);
    assert!(
        reports[0]
            .calls
            .iter()
            .all(|item| item.call.head == "target")
    );
    assert!(
        reports[0]
            .calls
            .iter()
            .all(|item| item.expected_parameter_count == Some(1))
    );
    assert!(
        reports[0]
            .calls
            .iter()
            .all(|item| item.status == SignatureCallStatus::Exact)
    );
}
