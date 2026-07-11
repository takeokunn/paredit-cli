use super::*;

#[test]
fn cli_writes_function_rename_through_symbol_macrolet_without_touching_shadowing_binding_name() {
    assert_write_case(WriteCase {
        fixture_name: "rename-function-symbol-macrolet-shadowing",
        dialect: None,
        from: "helper",
        to: "renamed",
        input_files: &[FixtureFile {
            path: "core.lisp",
            contents: "(defun helper (x) x)\n\
(defun caller ()\n\
  (symbol-macrolet ((helper other))\n\
    helper\n\
    #'helper\n\
    (function helper)\n\
    (helper 1))\n\
  (helper 2)\n\
  #'helper\n\
  (function helper)\n\
  (helper 3))\n",
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
  (renamed 2)\n\
  #'renamed\n\
  (function renamed)\n\
  (renamed 3))\n",
        }],
        expected_definition_count: 1,
        expected_call_count: 7,
    });
}

#[test]
fn cli_writes_function_rename_through_qualified_symbol_macrolet_forms() {
    assert_write_case(WriteCase {
        fixture_name: "rename-function-qualified-symbol-macrolet-shadowing",
        dialect: None,
        from: "helper",
        to: "renamed",
        input_files: &[FixtureFile {
            path: "core.lisp",
            contents: "(cl:defun helper (x) x)\n\
(defun caller ()\n\
  (cl:symbol-macrolet ((helper other))\n\
    helper\n\
    #'helper\n\
    (function helper)\n\
    (helper 1))\n\
  (cl-user:symbol-macrolet ((helper other))\n\
    helper\n\
    #'helper\n\
    (function helper)\n\
    (helper 2))\n\
  (helper 3)\n\
  #'helper\n\
  (function helper)\n\
  (helper 4))\n",
        }],
        expected_files: &[FixtureFile {
            path: "core.lisp",
            contents: "(cl:defun renamed (x) x)\n\
(defun caller ()\n\
  (cl:symbol-macrolet ((helper other))\n\
    helper\n\
    #'renamed\n\
    (function renamed)\n\
    (renamed 1))\n\
  (cl-user:symbol-macrolet ((helper other))\n\
    helper\n\
    #'renamed\n\
    (function renamed)\n\
    (renamed 2))\n\
  (renamed 3)\n\
  #'renamed\n\
  (function renamed)\n\
  (renamed 4))\n",
        }],
        expected_definition_count: 1,
        expected_call_count: 10,
    });
}

#[test]
fn cli_writes_function_calls_inside_symbol_macrolet_expansion_forms() {
    assert_write_case(WriteCase {
        fixture_name: "rename-function-symbol-macrolet-expansion",
        dialect: None,
        from: "helper",
        to: "renamed",
        input_files: &[FixtureFile {
            path: "core.lisp",
            contents: "(defun helper (x) x)\n\
(defun caller ()\n\
  (symbol-macrolet ((helper (helper 0)))\n\
    helper\n\
    #'helper\n\
    (function helper)\n\
    (helper 1))\n\
  (helper 2))\n",
        }],
        expected_files: &[FixtureFile {
            path: "core.lisp",
            contents: "(defun renamed (x) x)\n\
(defun caller ()\n\
  (symbol-macrolet ((helper (renamed 0)))\n\
    helper\n\
    #'renamed\n\
    (function renamed)\n\
    (renamed 1))\n\
  (renamed 2))\n",
        }],
        expected_definition_count: 1,
        expected_call_count: 5,
    });
}

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
