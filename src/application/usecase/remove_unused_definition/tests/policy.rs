use super::*;

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

#[test]
fn skips_exported_definition_when_file_uses_package_nickname() {
    let text = "(in-package #:main)\n(defun public-entry () 1)\n";
    let form = "(defun public-entry () 1)";
    let mut public_entry = definition(text, form, "public-entry", DefinitionCategory::Function);
    public_entry.package = Some("main".to_owned());
    let mut request = request_for(text, vec![public_entry]);
    request.files[0].package = Some("main".to_owned());
    request.package_definitions.push(PackageDefinitionReport {
        path: "0".to_owned(),
        span: ByteSpan::new(ByteOffset::new(0), ByteOffset::new(0)),
        name: "#:app.main".to_owned(),
        nicknames: vec!["#:main".to_owned()],
        uses: Vec::new(),
        exports: vec!["#:public-entry".to_owned()],
        imports: Vec::new(),
        option_count: 2,
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
