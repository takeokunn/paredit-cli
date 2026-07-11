use super::*;

#[test]
fn cli_writes_function_references_inside_reader_quoted_lambda_bodies_only() {
    assert_write_case(WriteCase {
        fixture_name: "rename-function-reader-quoted-lambda-symbol-macrolet",
        dialect: None,
        from: "helper",
        to: "renamed",
        input_files: &[FixtureFile {
            path: "core.lisp",
            contents: "(defun helper (x) x)\n\
(defun caller ()\n\
  #'(lambda ()\n\
      (symbol-macrolet ((helper other)) helper)\n\
      helper\n\
      #'helper\n\
      (function helper)\n\
      (helper 1)))\n",
        }],
        expected_files: &[FixtureFile {
            path: "core.lisp",
            contents: "(defun renamed (x) x)\n\
(defun caller ()\n\
  #'(lambda ()\n\
      (symbol-macrolet ((helper other)) helper)\n\
      helper\n\
      #'renamed\n\
      (function renamed)\n\
      (renamed 1)))\n",
        }],
        expected_definition_count: 1,
        expected_call_count: 3,
    });
}

#[test]
fn cli_writes_package_qualified_function_references_inside_symbol_macrolet_bodies() {
    assert_write_case(WriteCase {
        fixture_name: "rename-function-qualified-callable-references-symbol-macrolet",
        dialect: None,
        from: "helper",
        to: "renamed",
        input_files: &[FixtureFile {
            path: "core.lisp",
            contents: "(defun helper (x) x)\n\
(defun caller ()\n\
  (symbol-macrolet ((helper other))\n\
    helper\n\
    #'cl-user:helper\n\
    (function cl-user:helper)\n\
    (cl-user:helper 1))\n\
  #'cl-user:helper\n\
  (function cl-user:helper)\n\
  (cl-user:helper 2))\n",
        }],
        expected_files: &[FixtureFile {
            path: "core.lisp",
            contents: "(defun renamed (x) x)\n\
(defun caller ()\n\
  (symbol-macrolet ((helper other))\n\
    helper\n\
    #'renamed\n\
    (function renamed)\n\
    (renamed 1))\n\
  #'renamed\n\
  (function renamed)\n\
  (renamed 2))\n",
        }],
        expected_definition_count: 1,
        expected_call_count: 6,
    });
}
