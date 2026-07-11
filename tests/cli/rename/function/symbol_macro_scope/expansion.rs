use super::*;

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
fn cli_writes_function_calls_inside_package_qualified_symbol_macrolet_expansion_forms() {
    assert_write_case(WriteCase {
        fixture_name: "rename-function-qualified-symbol-macrolet-expansion",
        dialect: None,
        from: "helper",
        to: "renamed",
        input_files: &[FixtureFile {
            path: "core.lisp",
            contents: "(defun helper (x) x)\n\
(defun caller ()\n\
  (symbol-macrolet ((cl-user:helper (helper 0)))\n\
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
  (symbol-macrolet ((cl-user:helper (renamed 0)))\n\
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
        expected_call_count: 8,
    });
}
