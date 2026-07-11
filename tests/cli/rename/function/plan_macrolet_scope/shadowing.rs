use super::*;

#[test]
fn cli_plans_function_rename_through_macrolet_expander_without_touching_shadowed_macro_body() {
    assert_plan_case(PlanCase {
        fixture_name: "rename-function-plan-macrolet-shadow",
        from: "old-name",
        to: "new-name",
        input_files: &[FixtureFile {
            path: "core.lisp",
            contents: "(defun old-name (x) x)\n\n(defun caller ()\n\n  (macrolet ((old-name (value) (list #'old-name value)))\n\n    (list (old-name 1) #'old-name)))\n",
        }],
        stdout_needles: &[
            "\"definitionCount\": 1",
            "\"callCount\": 1",
            "\"path\": \"0.1\"",
            "\"path\": \"1.3.1.0.2.1\"",
            "\"rewritten\": \"(defun new-name (x) x)\\n\\n(defun caller ()\\n\\n  (macrolet ((old-name (value) (list #'new-name value)))\\n\\n    (list (old-name 1) #'old-name)))\\n\"",
        ],
        unchanged_files: &[FixtureFile {
            path: "core.lisp",
            contents: "(defun old-name (x) x)\n\n(defun caller ()\n\n  (macrolet ((old-name (value) (list #'old-name value)))\n\n    (list (old-name 1) #'old-name)))\n",
        }],
    });
}

#[test]
fn cli_plans_function_rename_through_cl_user_compiler_macrolet_expander_only() {
    assert_plan_case(PlanCase {
        fixture_name: "rename-function-plan-cl-user-compiler-macrolet-shadow",
        from: "old-name",
        to: "new-name",
        input_files: &[FixtureFile {
            path: "core.lisp",
            contents: "(defun old-name (x) x)\n\n(defun caller (form)\n\n  (cl-user:compiler-macrolet ((old-name (value) (list #'old-name value)))\n\n    (list (old-name form) #'old-name)))\n",
        }],
        stdout_needles: &[
            "\"definitionCount\": 1",
            "\"callCount\": 1",
            "\"path\": \"0.1\"",
            "\"path\": \"1.3.1.0.2.1\"",
            "\"rewritten\": \"(defun new-name (x) x)\\n\\n(defun caller (form)\\n\\n  (cl-user:compiler-macrolet ((old-name (value) (list #'new-name value)))\\n\\n    (list (old-name form) #'old-name)))\\n\"",
        ],
        unchanged_files: &[FixtureFile {
            path: "core.lisp",
            contents: "(defun old-name (x) x)\n\n(defun caller (form)\n\n  (cl-user:compiler-macrolet ((old-name (value) (list #'old-name value)))\n\n    (list (old-name form) #'old-name)))\n",
        }],
    });
}

#[test]
fn cli_plans_function_rename_through_cl_user_macrolet_expander_without_touching_shadowed_macro_body()
 {
    assert_plan_case(PlanCase {
        fixture_name: "rename-function-plan-cl-user-macrolet-write",
        from: "old-name",
        to: "new-name",
        input_files: &[FixtureFile {
            path: "core.lisp",
            contents: "(defun old-name (x) x)\n\n(defun caller ()\n\n  (cl-user:macrolet ((old-name (value) (list #'old-name value)))\n\n    (list (old-name 1) #'old-name)))\n",
        }],
        stdout_needles: &[
            "\"definitionCount\": 1",
            "\"callCount\": 1",
            "\"path\": \"0.1\"",
            "\"path\": \"1.3.1.0.2.1\"",
            "\"rewritten\": \"(defun new-name (x) x)\\n\\n(defun caller ()\\n\\n  (cl-user:macrolet ((old-name (value) (list #'new-name value)))\\n\\n    (list (old-name 1) #'old-name)))\\n\"",
        ],
        unchanged_files: &[FixtureFile {
            path: "core.lisp",
            contents: "(defun old-name (x) x)\n\n(defun caller ()\n\n  (cl-user:macrolet ((old-name (value) (list #'old-name value)))\n\n    (list (old-name 1) #'old-name)))\n",
        }],
    });
}

#[test]
fn cli_plans_function_rename_through_cl_compiler_macrolet_expander_only() {
    assert_plan_case(PlanCase {
        fixture_name: "rename-function-plan-cl-compiler-macrolet-write",
        from: "old-name",
        to: "new-name",
        input_files: &[FixtureFile {
            path: "core.lisp",
            contents: "(defun old-name (x) x)\n\n(defun caller (form)\n\n  (cl:compiler-macrolet ((old-name (value) (list #'old-name value)))\n\n    (list (old-name form) #'old-name)))\n",
        }],
        stdout_needles: &[
            "\"definitionCount\": 1",
            "\"callCount\": 1",
            "\"path\": \"0.1\"",
            "\"path\": \"1.3.1.0.2.1\"",
            "\"rewritten\": \"(defun new-name (x) x)\\n\\n(defun caller (form)\\n\\n  (cl:compiler-macrolet ((old-name (value) (list #'new-name value)))\\n\\n    (list (old-name form) #'old-name)))\\n\"",
        ],
        unchanged_files: &[FixtureFile {
            path: "core.lisp",
            contents: "(defun old-name (x) x)\n\n(defun caller (form)\n\n  (cl:compiler-macrolet ((old-name (value) (list #'old-name value)))\n\n    (list (old-name form) #'old-name)))\n",
        }],
    });
}
