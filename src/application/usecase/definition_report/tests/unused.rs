use super::*;

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
fn unused_candidates_count_unqualified_references_to_package_qualified_common_lisp_definition() {
    let input = "(defun cl-user:used () :ok)\n\
             (defun caller () (used))\n";
    let tree = SyntaxTree::parse(input).expect("parse input");
    let parsed =
        build_parsed_definition_file(PathBuf::from("core.lisp"), Dialect::CommonLisp, &tree)
            .expect("build parsed file");

    let reports = collect_unused_definition_candidates(&[parsed]);
    let used = reports[0]
        .definitions
        .iter()
        .find(|item| item.definition.name.as_deref() == Some("cl-user:used"))
        .expect("qualified used definition");

    assert_eq!(used.references.len(), 1);
    assert!(!used.references.is_empty());
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
