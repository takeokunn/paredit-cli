use super::*;

#[test]
fn cli_plans_function_rename_through_symbol_macrolet_without_touching_binding_name() {
    assert_plan_case(PlanCase {
        fixture_name: "rename-function-plan-symbol-macrolet-shadowing",
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
  (helper 2))\n",
        }],
        stdout_needles: &[
            "\"definitionCount\": 1",
            "\"callCount\": 4",
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
    #'helper\n\
    (function helper)\n\
    (helper 1))\n\
  (helper 2))\n",
        }],
    });
}

#[test]
fn cli_plans_function_rename_inside_symbol_macrolet_expansion_forms() {
    assert_plan_case(PlanCase {
        fixture_name: "rename-function-plan-symbol-macrolet-expansion",
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
        stdout_needles: &[
            "\"definitionCount\": 1",
            "\"callCount\": 5",
            "\"path\": \"1.3.1.0.1.0\"",
            "\"rewritten\": \"(defun renamed (x) x)",
            "(symbol-macrolet ((helper (renamed 0)))",
            "#'renamed",
            "(function renamed)",
            "(renamed 2))\\n\"",
        ],
        unchanged_files: &[FixtureFile {
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
    });
}

#[test]
fn cli_plans_function_rename_through_qualified_symbol_macrolet_forms() {
    assert_plan_case(PlanCase {
        fixture_name: "rename-function-plan-qualified-symbol-macrolet-shadowing",
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
        stdout_needles: &[
            "\"definitionCount\": 1",
            "\"callCount\": 10",
            "\"path\": \"0.1\"",
            "\"path\": \"1.3.3\"",
            "\"path\": \"1.4.3\"",
            "\"path\": \"1.8.0\"",
            "\"rewritten\": \"(cl:defun renamed (x) x)",
            "(cl:symbol-macrolet ((helper other))",
            "(cl-user:symbol-macrolet ((helper other))",
            "#'renamed",
            "(function renamed)",
            "(renamed 4))\\n\"",
        ],
        unchanged_files: &[FixtureFile {
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
    });
}

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
