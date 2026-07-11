use super::*;

#[test]
fn cli_plans_function_rename_without_renaming_value_references() {
    assert_plan_case(PlanCase {
        fixture_name: "rename-function-plan",
        from: "old-name",
        to: "new-name",
        input_files: &[FixtureFile {
            path: "core.lisp",
            contents: "(defun old-name (x) x)\n(defun caller () (old-name 1) old-name)\n",
        }],
        stdout_needles: &[
            "\"definitionCount\": 1",
            "\"callCount\": 1",
            "\"path\": \"0.1\"",
            "\"path\": \"1.3.0\"",
        ],
        unchanged_files: &[FixtureFile {
            path: "core.lisp",
            contents: "(defun old-name (x) x)\n(defun caller () (old-name 1) old-name)\n",
        }],
    });
}
