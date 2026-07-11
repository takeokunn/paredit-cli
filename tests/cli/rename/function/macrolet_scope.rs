use super::*;

#[test]
fn cli_writes_function_rename_inside_macrolet_expander_only() {
    assert_write_case(WriteCase {
        fixture_name: "rename-function-macrolet-expander-write",
        dialect: None,
        from: "old-name",
        to: "new-name",
        input_files: &[FixtureFile {
            path: "core.lisp",
            contents: "(defun old-name (x) x)\n(defun caller () (macrolet ((old-name () #'old-name (function old-name) (old-name 1))) (old-name) #'old-name (function old-name) (old-name 2)))\n",
        }],
        expected_files: &[FixtureFile {
            path: "core.lisp",
            contents: "(defun new-name (x) x)\n(defun caller () (macrolet ((old-name () #'new-name (function new-name) (new-name 1))) (old-name) #'old-name (function old-name) (old-name 2)))\n",
        }],
        expected_definition_count: 1,
        expected_call_count: 3,
    });
}

#[test]
fn cli_writes_function_rename_inside_compiler_macrolet_expander_only() {
    assert_write_case(WriteCase {
        fixture_name: "rename-function-compiler-macrolet-expander-write",
        dialect: None,
        from: "old-name",
        to: "new-name",
        input_files: &[FixtureFile {
            path: "core.lisp",
            contents: "(defun old-name (x) x)\n(defun caller () (compiler-macrolet ((old-name () #'old-name (function old-name) (old-name 1))) (old-name) #'old-name (function old-name) (old-name 2)))\n",
        }],
        expected_files: &[FixtureFile {
            path: "core.lisp",
            contents: "(defun new-name (x) x)\n(defun caller () (compiler-macrolet ((old-name () #'new-name (function new-name) (new-name 1))) (old-name) #'old-name (function old-name) (old-name 2)))\n",
        }],
        expected_definition_count: 1,
        expected_call_count: 3,
    });
}

#[test]
fn cli_writes_function_rename_through_macrolet_expander_without_touching_shadowed_macro_body() {
    assert_write_case(WriteCase {
        fixture_name: "rename-function-macrolet-shadow",
        dialect: None,
        from: "old-name",
        to: "new-name",
        input_files: &[FixtureFile {
            path: "core.lisp",
            contents: "(defun old-name (x) x)\n\
(defun caller ()\n\
  (macrolet ((old-name (value) (list #'old-name value)))\n\
    (list (old-name 1) #'old-name)))\n",
        }],
        expected_files: &[FixtureFile {
            path: "core.lisp",
            contents: "(defun new-name (x) x)\n\
(defun caller ()\n\
  (macrolet ((old-name (value) (list #'new-name value)))\n\
    (list (old-name 1) #'old-name)))\n",
        }],
        expected_definition_count: 1,
        expected_call_count: 1,
    });
}

#[test]
fn cli_writes_function_rename_through_cl_user_compiler_macrolet_expander_only() {
    assert_write_case(WriteCase {
        fixture_name: "rename-function-compiler-macrolet-shadow",
        dialect: None,
        from: "old-name",
        to: "new-name",
        input_files: &[FixtureFile {
            path: "core.lisp",
            contents: "(defun old-name (x) x)\n\
(defun caller (form)\n\
  (cl-user:compiler-macrolet ((old-name (value) (list #'old-name value)))\n\
    (list (old-name form) #'old-name)))\n",
        }],
        expected_files: &[FixtureFile {
            path: "core.lisp",
            contents: "(defun new-name (x) x)\n\
(defun caller (form)\n\
  (cl-user:compiler-macrolet ((old-name (value) (list #'new-name value)))\n\
    (list (old-name form) #'old-name)))\n",
        }],
        expected_definition_count: 1,
        expected_call_count: 1,
    });
}

#[test]
fn cli_writes_function_rename_through_cl_user_macrolet_expander_without_touching_shadowed_macro_body()
 {
    assert_write_case(WriteCase {
        fixture_name: "rename-function-cl-user-macrolet-write",
        dialect: None,
        from: "old-name",
        to: "new-name",
        input_files: &[FixtureFile {
            path: "core.lisp",
            contents: "(defun old-name (x) x)\n\
(defun caller ()\n\
  (cl-user:macrolet ((old-name (value) (list #'old-name value)))\n\
    (list (old-name 1) #'old-name)))\n",
        }],
        expected_files: &[FixtureFile {
            path: "core.lisp",
            contents: "(defun new-name (x) x)\n\
(defun caller ()\n\
  (cl-user:macrolet ((old-name (value) (list #'new-name value)))\n\
    (list (old-name 1) #'old-name)))\n",
        }],
        expected_definition_count: 1,
        expected_call_count: 1,
    });
}

#[test]
fn cli_writes_function_rename_through_cl_compiler_macrolet_expander_only() {
    assert_write_case(WriteCase {
        fixture_name: "rename-function-cl-compiler-macrolet-write",
        dialect: None,
        from: "old-name",
        to: "new-name",
        input_files: &[FixtureFile {
            path: "core.lisp",
            contents: "(defun old-name (x) x)\n\
(defun caller (form)\n\
  (cl:compiler-macrolet ((old-name (value) (list #'old-name value)))\n\
    (list (old-name form) #'old-name)))\n",
        }],
        expected_files: &[FixtureFile {
            path: "core.lisp",
            contents: "(defun new-name (x) x)\n\
(defun caller (form)\n\
  (cl:compiler-macrolet ((old-name (value) (list #'new-name value)))\n\
    (list (old-name form) #'old-name)))\n",
        }],
        expected_definition_count: 1,
        expected_call_count: 1,
    });
}

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
