use super::*;

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
fn cli_plans_emacs_lisp_function_rename_inside_cl_symbol_macrolet_expansion_forms() {
    assert_plan_case_with_dialect(
        PlanCase {
            fixture_name: "rename-function-plan-emacs-lisp-cl-symbol-macrolet-expansion",
            from: "helper",
            to: "renamed",
            input_files: &[FixtureFile {
                path: "core.el",
                contents: "(defun helper (x) x)\n\n(defun caller ()\n\n  (cl-symbol-macrolet ((helper (helper 0)))\n\n    helper\n\n    #'helper\n\n    (function helper)\n\n    (helper 1))\n\n  (helper 2))\n",
            }],
            stdout_needles: &[
                "\"definitionCount\": 1",
                "\"callCount\": 5",
                "\"path\": \"1.3.1.0.1.0\"",
                "\"rewritten\": \"(defun renamed (x) x)",
                "(cl-symbol-macrolet ((helper (renamed 0)))",
                "#'renamed",
                "(function renamed)",
                "(renamed 2))\\n\"",
            ],
            unchanged_files: &[FixtureFile {
                path: "core.el",
                contents: "(defun helper (x) x)\n\n(defun caller ()\n\n  (cl-symbol-macrolet ((helper (helper 0)))\n\n    helper\n\n    #'helper\n\n    (function helper)\n\n    (helper 1))\n\n  (helper 2))\n",
            }],
        },
        Some("emacs-lisp"),
    );
}

#[test]
fn cli_plans_function_rename_inside_package_qualified_symbol_macrolet_expansion_forms() {
    assert_plan_case(PlanCase {
        fixture_name: "rename-function-plan-qualified-symbol-macrolet-expansion",
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
        stdout_needles: &[
            "\"definitionCount\": 1",
            "\"callCount\": 8",
            "\"path\": \"0.1\"",
            "\"path\": \"1.3.1.0.1.0\"",
            "\"path\": \"1.4.0\"",
            "\"path\": \"1.7.0\"",
            "\"rewritten\": \"(defun renamed (x) x)",
            "(symbol-macrolet ((cl-user:helper (renamed 0)))",
            "#'renamed",
            "(function renamed)",
            "(renamed 3))\\n\"",
        ],
        unchanged_files: &[FixtureFile {
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
    });
}
