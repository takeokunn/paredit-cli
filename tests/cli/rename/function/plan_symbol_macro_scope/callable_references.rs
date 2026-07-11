use super::*;

#[test]
fn cli_plans_function_references_inside_reader_quoted_lambda_bodies_only() {
    assert_plan_case(PlanCase {
        fixture_name: "rename-function-plan-reader-quoted-lambda-symbol-macrolet",
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
        stdout_needles: &[
            "\"definitionCount\": 1",
            "\"callCount\": 3",
            "\"path\": \"0.1\"",
            "\"path\": \"1.3.4\"",
            "\"rewritten\": \"(defun renamed (x) x)",
            "(symbol-macrolet ((helper other)) helper)",
            "#'renamed",
            "(function renamed)",
            "(renamed 1)))\\n\"",
        ],
        unchanged_files: &[FixtureFile {
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
    });
}

#[test]
fn cli_plans_package_qualified_function_references_inside_symbol_macrolet_bodies() {
    assert_plan_case(PlanCase {
        fixture_name: "rename-function-plan-qualified-callable-references-symbol-macrolet",
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
        stdout_needles: &[
            "\"definitionCount\": 1",
            "\"callCount\": 6",
            "\"path\": \"0.1\"",
            "\"rewritten\": \"(defun renamed (x) x)",
            "(symbol-macrolet ((helper other))",
            "#'renamed",
            "(function renamed)",
            "(renamed 2))\\n\"",
        ],
        unchanged_files: &[FixtureFile {
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
    });
}
