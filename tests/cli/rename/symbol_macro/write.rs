use super::*;

#[test]
fn cli_writes_define_symbol_macro_rename_without_touching_expansion_body() {
    assert_write_case(
        "rename-symbol-macro-write",
        "(define-symbol-macro old-name (list old-name :tag)) (list old-name (setf old-name 1))\n",
        "(define-symbol-macro new-name (list old-name :tag)) (list new-name (setf new-name 1))\n",
        1,
        2,
    );
}

#[test]
fn cli_writes_define_symbol_macro_rename_inside_reader_quoted_lambda_body() {
    assert_write_case(
        "rename-symbol-macro-reader-quoted-lambda",
        "(define-symbol-macro old-name current-user) #'(lambda () (define-symbol-macro old-name (list old-name :tag)) old-name) old-name\n",
        "(define-symbol-macro new-name current-user) #'(lambda () (define-symbol-macro new-name (list old-name :tag)) new-name) new-name\n",
        2,
        2,
    );
}

#[test]
fn cli_writes_define_symbol_macro_rename_inside_reader_quoted_lambda_body_without_touching_function_designators()
 {
    assert_write_case(
        "rename-symbol-macro-reader-quoted-lambda-function-designators",
        "(define-symbol-macro old-name current-user) #'(lambda () (define-symbol-macro old-name (list #'old-name (function old-name) old-name)) old-name) old-name\n",
        "(define-symbol-macro new-name current-user) #'(lambda () (define-symbol-macro new-name (list #'old-name (function old-name) old-name)) new-name) new-name\n",
        2,
        2,
    );
}

#[test]
fn cli_writes_define_symbol_macro_rename_inside_reader_quoted_lambda_with_cl_symbol_macrolet_shadowing()
 {
    assert_write_case(
        "rename-symbol-macro-reader-quoted-cl-shadowing",
        "(define-symbol-macro old-name current-user) #'(lambda () (cl:symbol-macrolet ((old-name other-user)) old-name) old-name) old-name\n",
        "(define-symbol-macro new-name current-user) #'(lambda () (cl:symbol-macrolet ((old-name other-user)) old-name) new-name) new-name\n",
        1,
        2,
    );
}

#[test]
fn cli_writes_define_symbol_macro_rename_inside_reader_quoted_lambda_with_cl_user_symbol_macrolet_shadowing()
 {
    assert_write_case(
        "rename-symbol-macro-reader-quoted-cl-user-shadowing",
        "(define-symbol-macro old-name current-user) #'(lambda () (cl-user:symbol-macrolet ((old-name other-user)) old-name) old-name) old-name\n",
        "(define-symbol-macro new-name current-user) #'(lambda () (cl-user:symbol-macrolet ((old-name other-user)) old-name) new-name) new-name\n",
        1,
        2,
    );
}

#[test]
fn cli_writes_common_lisp_qualified_define_symbol_macro_rename() {
    assert_write_case(
        "rename-symbol-macro-qualified",
        "(cl-user:define-symbol-macro old-name current-user) (list old-name (old-name 1) (setf old-name 2))\n",
        "(cl-user:define-symbol-macro new-name current-user) (list new-name (old-name 1) (setf new-name 2))\n",
        1,
        2,
    );
}

#[test]
fn cli_writes_cl_qualified_define_symbol_macro_rename() {
    assert_write_case(
        "rename-symbol-macro-cl-qualified",
        "(cl:define-symbol-macro old-name current-user) (list old-name (old-name 1) (setf old-name 2))\n",
        "(cl:define-symbol-macro new-name current-user) (list new-name (old-name 1) (setf new-name 2))\n",
        1,
        2,
    );
}

#[test]
fn cli_writes_rename_symbol_macro_across_definition_and_reference_files() {
    let dir = fresh_temp_dir("rename-symbol-macro-multi-file");
    let definitions_file = dir.join("definitions.lisp");
    let references_file = dir.join("references.lisp");
    let unchanged_file = dir.join("unchanged.lisp");
    write_fixture(
        &definitions_file,
        "(define-symbol-macro old-name current-user)\n",
        "definitions fixture",
    );
    write_fixture(
        &references_file,
        "(list old-name (setf old-name 1) (old-name 2))\n",
        "references fixture",
    );
    write_fixture(
        &unchanged_file,
        "(list current-user)\n",
        "unchanged fixture",
    );

    let output = run_rename_symbol_macro(
        &[
            definitions_file.as_path(),
            references_file.as_path(),
            unchanged_file.as_path(),
        ],
        "old-name",
        "new-name",
        true,
    );
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report = parse_definition_reference_report(&output.stdout)
        .expect("parse definition-reference report");
    assert_eq!(report.definition_count, 1);
    assert_eq!(report.reference_count, 2);
    assert_eq!(
        report
            .files
            .iter()
            .map(|file| file.written)
            .collect::<Vec<_>>(),
        vec![true, true, false]
    );

    let stdout = String::from_utf8(output.stdout).expect("decode stdout");
    assert!(stdout.contains(&definitions_file.display().to_string()));
    assert!(stdout.contains(&references_file.display().to_string()));
    assert!(stdout.contains(&unchanged_file.display().to_string()));
    assert!(stdout.contains("\"rewritten\": \"(define-symbol-macro new-name current-user)\\n\""));
    assert!(
        stdout.contains("\"rewritten\": \"(list new-name (setf new-name 1) (old-name 2))\\n\"")
    );
    assert!(stdout.contains("\"rewritten\": \"(list current-user)\\n\""));
    assert!(stdout.contains("\"changed\": true"));
    assert!(stdout.contains("\"changed\": false"));
    assert!(stdout.contains("\"written\": true"));
    assert!(stdout.contains("\"written\": false"));

    assert_eq!(
        read_fixture(&definitions_file, "rewritten definitions fixture"),
        "(define-symbol-macro new-name current-user)\n"
    );
    assert_eq!(
        read_fixture(&references_file, "rewritten references fixture"),
        "(list new-name (setf new-name 1) (old-name 2))\n"
    );
    assert_eq!(
        read_fixture(&unchanged_file, "unchanged fixture"),
        "(list current-user)\n"
    );
}

#[test]
fn cli_writes_define_symbol_macro_rename_in_locally_body_without_counting_declarations() {
    assert_write_case(
        "rename-symbol-macro-locally-body",
        "(define-symbol-macro old-name current-user) (locally (declare (special old-name)) old-name) (locally (declaim (special old-name)) (proclaim (special old-name)) old-name) old-name\n",
        "(define-symbol-macro new-name current-user) (locally (declare (special old-name)) new-name) (locally (declaim (special old-name)) (proclaim (special old-name)) new-name) new-name\n",
        1,
        3,
    );
}

#[test]
fn cli_writes_define_symbol_macro_rename_skipping_definition_scope_boundaries() {
    assert_write_case(
        "rename-symbol-macro-definition-boundaries",
        "(define-symbol-macro old-name current-user) (list old-name (define-setf-expander slot (place) (list old-name place)) (define-compiler-macro render (place) (list old-name place)) old-name)\n",
        "(define-symbol-macro new-name current-user) (list new-name (define-setf-expander slot (place) (list old-name place)) (define-compiler-macro render (place) (list old-name place)) new-name)\n",
        1,
        2,
    );
}
