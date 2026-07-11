use super::*;

#[test]
fn rejects_unparseable_input_files_instead_of_panicking() {
    let request = RemoveUnusedDefinitionsRequest {
        files: vec![RemoveUnusedDefinitionInputFile {
            path: PathBuf::from("broken.lisp"),
            dialect: Dialect::CommonLisp,
            package: Some("app".to_owned()),
            definitions: Vec::new(),
            atoms: Vec::new(),
            text: "(defun broken ()".to_owned(),
        }],
        package_definitions: Vec::new(),
        include_protected: false,
        include_exported: false,
    };

    let error = plan_remove_unused_definitions(request).expect_err("invalid input must fail");

    assert!(
        error.to_string().contains("failed to parse broken.lisp"),
        "unexpected error: {error:#}"
    );
}

#[test]
fn rejects_invalid_definition_symbols_instead_of_panicking() {
    let text = "(in-package #:app)\n(defun still-valid () 1)\n";
    let request = RemoveUnusedDefinitionsRequest {
        files: vec![RemoveUnusedDefinitionInputFile {
            path: PathBuf::from("core.lisp"),
            dialect: Dialect::CommonLisp,
            package: Some("app".to_owned()),
            definitions: vec![UnusedDefinitionDefinition {
                path: "0".to_owned(),
                span: ByteSpan::new(ByteOffset::new(19), ByteOffset::new(41)),
                head: "defun".to_owned(),
                name: Some("not a symbol".to_owned()),
                category: DefinitionCategory::Function,
                parameter_count: Some(0),
                body_form_count: Some(1),
                package: Some("app".to_owned()),
            }],
            atoms: SyntaxTree::parse(text)
                .expect("fixture must parse")
                .atom_occurrences(),
            text: text.to_owned(),
        }],
        package_definitions: Vec::new(),
        include_protected: false,
        include_exported: false,
    };

    let error = plan_remove_unused_definitions(request).expect_err("invalid metadata must fail");

    assert!(
        error
            .to_string()
            .contains("remove-unused-definition found invalid symbol 'not a symbol' in core.lisp"),
        "unexpected error: {error:#}"
    );
}
