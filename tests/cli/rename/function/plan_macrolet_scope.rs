use super::*;

#[test]
fn cli_plans_function_rename_inside_macrolet_expander_only() {
    assert_plan_case(PlanCase {
        fixture_name: "rename-function-plan-macrolet-expander",
        from: "old-name",
        to: "new-name",
        input_files: &[FixtureFile {
            path: "core.lisp",
            contents: "(defun old-name (x) x)\n(defun caller () (macrolet ((old-name () #'old-name (function old-name) (old-name 1))) (old-name) #'old-name (function old-name) (old-name 2)))\n",
        }],
        stdout_needles: &[
            "\"definitionCount\": 1",
            "\"callCount\": 3",
            "\"path\": \"0.1\"",
            "\"path\": \"1.3.1.0.2\"",
            "\"rewritten\": \"(defun new-name (x) x)",
            "(macrolet ((old-name () #'new-name (function new-name) (new-name 1)))",
            "(old-name) #'old-name (function old-name) (old-name 2)))\\n\"",
        ],
        unchanged_files: &[FixtureFile {
            path: "core.lisp",
            contents: "(defun old-name (x) x)\n(defun caller () (macrolet ((old-name () #'old-name (function old-name) (old-name 1))) (old-name) #'old-name (function old-name) (old-name 2)))\n",
        }],
    });
}
