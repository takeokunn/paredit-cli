use super::*;

#[test]
fn cli_plans_package_qualified_function_references_inside_macrolet_body_and_expander() {
    assert_plan_case(PlanCase {
        fixture_name: "rename-function-plan-qualified-callable-references-macrolet",
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
        stdout_needles: &[
            "\"definitionCount\": 1",
            "\"callCount\": 6",
            "\"path\": \"0.1\"",
            "\"rewritten\": \"(defun renamed (x) x)",
            "(macrolet ((helper () #'renamed (function renamed) (renamed 1)))",
            "#'renamed",
            "(function renamed)",
            "(renamed 2)))\\n\"",
        ],
        unchanged_files: &[FixtureFile {
            path: "core.lisp",
            contents: "(defun helper (x) x)\n\
(defun caller ()\n\
  (macrolet ((helper () #'cl-user:helper (function cl-user:helper) (cl-user:helper 1)))\n\
    (helper)\n\
    #'cl-user:helper\n\
    (function cl-user:helper)\n\
    (cl-user:helper 2)))\n",
        }],
    });
}

#[test]
fn cli_plans_package_qualified_function_references_inside_compiler_macrolet_body_and_expander() {
    assert_plan_case(PlanCase {
        fixture_name: "rename-function-plan-qualified-callable-references-compiler-macrolet",
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
        stdout_needles: &[
            "\"definitionCount\": 1",
            "\"callCount\": 6",
            "\"path\": \"0.1\"",
            "\"rewritten\": \"(defun renamed (x) x)",
            "(compiler-macrolet ((helper () #'renamed (function renamed) (renamed 1)))",
            "#'renamed",
            "(function renamed)",
            "(renamed 2)))\\n\"",
        ],
        unchanged_files: &[FixtureFile {
            path: "core.lisp",
            contents: "(defun helper (x) x)\n\
(defun caller ()\n\
  (compiler-macrolet ((helper () #'cl-user:helper (function cl-user:helper) (cl-user:helper 1)))\n\
    (helper)\n\
    #'cl-user:helper\n\
    (function cl-user:helper)\n\
    (cl-user:helper 2)))\n",
        }],
    });
}
