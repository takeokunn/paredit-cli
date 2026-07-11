use super::*;

#[test]
fn cli_writes_package_qualified_function_references_inside_macrolet_body_and_expander() {
    assert_write_case(WriteCase {
        fixture_name: "rename-function-qualified-callable-references-macrolet",
        dialect: None,
        from: "helper",
        to: "renamed",
        input_files: &[FixtureFile {
            path: "core.lisp",
            contents: "(defun helper (x) x)\n\
(defun caller ()\n\
  (macrolet ((helper () #'cl-user:helper (function cl-user:helper) (cl-user:helper 1)))\n\
    (helper)\n\
    #'cl-user:helper\n\
    (function cl-user:helper)\n\
    (cl-user:helper 2)))\n",
        }],
        expected_files: &[FixtureFile {
            path: "core.lisp",
            contents: "(defun renamed (x) x)\n\
(defun caller ()\n\
  (macrolet ((helper () #'renamed (function renamed) (renamed 1)))\n\
    (helper)\n\
    #'renamed\n\
    (function renamed)\n\
    (renamed 2)))\n",
        }],
        expected_definition_count: 1,
        expected_call_count: 6,
    });
}

#[test]
fn cli_writes_package_qualified_function_references_inside_compiler_macrolet_body_and_expander() {
    assert_write_case(WriteCase {
        fixture_name: "rename-function-qualified-callable-references-compiler-macrolet",
        dialect: None,
        from: "helper",
        to: "renamed",
        input_files: &[FixtureFile {
            path: "core.lisp",
            contents: "(defun helper (x) x)\n\
(defun caller ()\n\
  (compiler-macrolet ((helper () #'cl-user:helper (function cl-user:helper) (cl-user:helper 1)))\n\
    (helper)\n\
    #'cl-user:helper\n\
    (function cl-user:helper)\n\
    (cl-user:helper 2)))\n",
        }],
        expected_files: &[FixtureFile {
            path: "core.lisp",
            contents: "(defun renamed (x) x)\n\
(defun caller ()\n\
  (compiler-macrolet ((helper () #'renamed (function renamed) (renamed 1)))\n\
    (helper)\n\
    #'renamed\n\
    (function renamed)\n\
    (renamed 2)))\n",
        }],
        expected_definition_count: 1,
        expected_call_count: 6,
    });
}
