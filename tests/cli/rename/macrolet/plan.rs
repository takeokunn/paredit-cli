use super::*;

fn assert_plan_case_with_counts(
    fixture_name: &str,
    file_name: &str,
    input: &str,
    expected: &str,
    definition_count: u64,
    call_count: u64,
) {
    let dir = fresh_temp_dir(fixture_name);
    let file = dir.join(file_name);
    write_fixture(&file, input, "plan fixture");

    let output = run_rename_macrolet(&file, "old-name", "new-name", false);
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let report = parse_definition_call_report(&output.stdout).expect("parse plan report");
    assert_eq!(report.definition_count, definition_count);
    assert_eq!(report.call_count, call_count);
    assert_eq!(report.files.first().map(|entry| entry.written), Some(false));
    assert!(String::from_utf8_lossy(&output.stdout).contains(expected));
    assert_eq!(read_fixture(&file, "unchanged plan fixture"), input);
}

macrolet_plan_case!(
    cli_plans_macrolet_rename_without_touching_noncall_values,
    "rename-macrolet-plan",
    "core.lisp",
    "(macrolet ((old-name (x) (list old-name x))) (old-name 1) old-name)\n",
    "(macrolet ((new-name (x) (list old-name x))) (new-name 1) old-name)"
);

macrolet_plan_case!(
    cli_plans_compiler_macrolet_rename_without_touching_noncall_values,
    "rename-compiler-macrolet-plan",
    "core.lisp",
    "(compiler-macrolet ((old-name (x) (list old-name x))) (old-name 1) old-name)\n",
    "(compiler-macrolet ((new-name (x) (list old-name x))) (new-name 1) old-name)"
);

macrolet_plan_case!(
    cli_plans_cl_user_qualified_compiler_macrolet_rename_without_touching_noncall_values,
    "rename-cl-user-compiler-macrolet-plan",
    "core.lisp",
    "(cl-user:compiler-macrolet ((old-name (x) (list old-name x))) (old-name 1) old-name)\n",
    "(cl-user:compiler-macrolet ((new-name (x) (list old-name x))) (new-name 1) old-name)"
);

macrolet_plan_case!(
    cli_plans_cl_qualified_compiler_macrolet_rename_without_touching_noncall_values,
    "rename-cl-compiler-macrolet-plan",
    "core.lisp",
    "(cl:compiler-macrolet ((old-name (x) (list old-name x))) (old-name 1) old-name)\n",
    "(cl:compiler-macrolet ((new-name (x) (list old-name x))) (new-name 1) old-name)"
);

macrolet_plan_case!(
    cli_plans_emacs_lisp_cl_macrolet_rename_without_touching_noncall_values,
    "rename-cl-macrolet-plan",
    "core.el",
    "(cl-macrolet ((old-name (x) (list old-name x))) (old-name 1) old-name)\n",
    "(cl-macrolet ((new-name (x) (list old-name x))) (new-name 1) old-name)"
);

macrolet_plan_case!(
    cli_plans_cl_user_macrolet_rename_without_touching_noncall_values,
    "rename-cl-user-macrolet-plan",
    "core.lisp",
    "(cl-user:macrolet ((old-name (x) (list old-name x))) (old-name 1) old-name)\n",
    "(cl-user:macrolet ((new-name (x) (list old-name x))) (new-name 1) old-name)"
);

#[test]
fn cli_plans_macrolet_rename_inside_reader_quoted_lambda_bodies_without_touching_function_designators()
 {
    assert_plan_case_with_counts(
        "rename-macrolet-reader-quoted-lambda-function-designator-plan",
        "core.lisp",
        "(macrolet ((old-name (x) #'(lambda () (list #'old-name (function old-name) (old-name x) old-name)))) (old-name 1) old-name)\n",
        "(macrolet ((new-name (x) #'(lambda () (list #'old-name (function old-name) (new-name x) old-name)))) (new-name 1) old-name)",
        1,
        2,
    );
}

#[test]
fn cli_plans_compiler_macrolet_rename_inside_reader_quoted_lambda_bodies_without_touching_function_designators()
 {
    assert_plan_case_with_counts(
        "rename-compiler-macrolet-reader-quoted-lambda-function-designator-plan",
        "core.lisp",
        "(compiler-macrolet ((old-name (x) #'(lambda () (list #'old-name (function old-name) (old-name x) old-name)))) (old-name 1) old-name)\n",
        "(compiler-macrolet ((new-name (x) #'(lambda () (list #'old-name (function old-name) (new-name x) old-name)))) (new-name 1) old-name)",
        1,
        2,
    );
}

#[test]
fn cli_plans_cl_qualified_compiler_macrolet_rename_inside_reader_quoted_lambda_bodies_without_touching_function_designators()
 {
    assert_plan_case_with_counts(
        "rename-cl-cmacrolet-reader-fn-plan",
        "core.lisp",
        "(cl:compiler-macrolet ((old-name (x) #'(lambda () (list #'old-name (function old-name) (old-name x) old-name)))) (old-name 1) old-name)\n",
        "(cl:compiler-macrolet ((new-name (x) #'(lambda () (list #'old-name (function old-name) (new-name x) old-name)))) (new-name 1) old-name)",
        1,
        2,
    );
}

#[test]
fn cli_plans_cl_user_qualified_compiler_macrolet_rename_inside_reader_quoted_lambda_bodies_without_touching_function_designators()
 {
    assert_plan_case_with_counts(
        "rename-cl-user-cmacrolet-reader-fn-plan",
        "core.lisp",
        "(cl-user:compiler-macrolet ((old-name (x) #'(lambda () (list #'old-name (function old-name) (old-name x) old-name)))) (old-name 1) old-name)\n",
        "(cl-user:compiler-macrolet ((new-name (x) #'(lambda () (list #'old-name (function old-name) (new-name x) old-name)))) (new-name 1) old-name)",
        1,
        2,
    );
}

#[test]
fn cli_plans_macrolet_rename_inside_quasiquote_with_unquote_prefixes() {
    assert_plan_case_with_counts(
        "rename-macrolet-quasiquote-plan",
        "core.lisp",
        "(macrolet ((old-name (x) x)) `(list ,(old-name 1) ,@(old-name 2) (old-name 3)))\n",
        "(macrolet ((new-name (x) x)) `(list ,(new-name 1) ,@(new-name 2) (old-name 3)))",
        1,
        2,
    );
}

#[test]
fn cli_skips_macrolet_definitions_and_calls_inside_quoted_data_during_plan() {
    assert_plan_case_with_counts(
        "rename-macrolet-quoted-data-plan",
        "core.lisp",
        "(macrolet ((old-name (x) x)) '(macrolet ((old-name (y) y)) (old-name 1)) `(progn (macrolet ((old-name (z) z)) (old-name 2)) ,(old-name 3)))\n",
        "(macrolet ((new-name (x) x)) '(macrolet ((old-name (y) y)) (old-name 1)) `(progn (macrolet ((old-name (z) z)) (old-name 2)) ,(new-name 3)))",
        1,
        1,
    );
}

#[test]
fn cli_plans_macrolet_rename_without_touching_global_macro_cell_accessors() {
    assert_plan_case_with_counts(
        "rename-macrolet-global-accessors-plan",
        "core.lisp",
        "(macrolet ((old-name (x) x)) (macro-function 'old-name) (compiler-macro-function 'old-name) (old-name 1) old-name)\n",
        "(macrolet ((new-name (x) x)) (macro-function 'old-name) (compiler-macro-function 'old-name) (new-name 1) old-name)",
        1,
        1,
    );
}

#[test]
fn cli_plans_compiler_macrolet_rename_without_touching_global_macro_cell_accessors() {
    assert_plan_case_with_counts(
        "rename-compiler-macrolet-global-accessors-plan",
        "core.lisp",
        "(compiler-macrolet ((old-name (x) x)) (macro-function 'old-name) (compiler-macro-function 'old-name) (old-name 1) old-name)\n",
        "(compiler-macrolet ((new-name (x) x)) (macro-function 'old-name) (compiler-macro-function 'old-name) (new-name 1) old-name)",
        1,
        1,
    );
}

#[test]
fn cli_plans_macrolet_rename_without_touching_setf_function_call_heads() {
    assert_plan_case_with_counts(
        "rename-macrolet-setf-function-call-heads-plan",
        "core.lisp",
        "(macrolet ((old-name (x) x)) ((setf old-name) 1 thing) (old-name 1) old-name)\n",
        "(macrolet ((new-name (x) x)) ((setf old-name) 1 thing) (new-name 1) old-name)",
        1,
        1,
    );
}
