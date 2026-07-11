use super::*;
use proptest::test_runner::TestCaseError;

fn assert_cli_rename_local_function_property(
    from: String,
    to: String,
) -> Result<(), TestCaseError> {
    prop_assume!(from != to);

    let dir = fresh_temp_dir("rename-local-function-cli-pbt");
    let lisp_file = dir.join("core.lisp");
    let input = format!("(labels (({from} (x) ({from} x))) ({from} 1) {from})\n");
    fs::write(&lisp_file, &input)
        .map_err(|err| TestCaseError::fail(format!("write lisp fixture: {err}")))?;

    let output = paredit()
        .args([
            "rename-local-function",
            "--from",
            &from,
            "--to",
            &to,
            "--write",
        ])
        .arg(&lisp_file)
        .output()
        .map_err(|err| TestCaseError::fail(format!("run paredit: {err}")))?;

    prop_assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report = parse_definition_call_report(&output.stdout)?;
    prop_assert_eq!(report.definition_count, 1);
    prop_assert_eq!(report.call_count, 2);
    prop_assert_eq!(report.files.first().map(|file| file.written), Some(true));

    let rewritten = fs::read_to_string(&lisp_file)
        .map_err(|err| TestCaseError::fail(format!("read rewritten fixture: {err}")))?;
    let expected = format!("(labels (({to} (x) ({to} x))) ({to} 1) {from})\n");
    prop_assert_eq!(rewritten, expected);

    assert_cli_check_succeeds(&lisp_file)?;

    Ok(())
}

proptest! {
    #![proptest_config(cli_proptest_config(24))]

    #[test]
    fn pbt_cli_rename_labels_output_remains_parseable_and_updates_recursive_calls(
        from in "[a-z][a-z0-9-]{0,8}",
        to in "[a-z][a-z0-9-]{0,8}",
    ) {
        assert_cli_rename_local_function_property(from, to)?;
    }
}

#[test]
fn cli_plans_flet_rename_without_touching_definition_body_or_noncall_values() {
    let dir = fresh_temp_dir("rename-local-function-flet-plan");
    let lisp_file = dir.join("core.lisp");
    fs::write(
        &lisp_file,
        "(flet ((old-name (x) (old-name x))) (old-name 1) old-name)\n",
    )
    .expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("rename-local-function")
        .arg("--from")
        .arg("old-name")
        .arg("--to")
        .arg("new-name")
        .arg(&lisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"definitionCount\": 1"))
        .stdout(predicate::str::contains("\"callCount\": 1"))
        .stdout(predicate::str::contains(
            "(flet ((new-name (x) (old-name x))) (new-name 1) old-name)",
        ));

    assert_eq!(
        fs::read_to_string(&lisp_file).expect("read unchanged fixture"),
        "(flet ((old-name (x) (old-name x))) (old-name 1) old-name)\n"
    );
}

#[test]
fn cli_writes_labels_rename_with_recursive_calls() {
    let dir = fresh_temp_dir("rename-local-function-labels-write");
    let lisp_file = dir.join("core.lisp");
    fs::write(
        &lisp_file,
        "(labels ((old-name (x) (old-name x))) (old-name 1) old-name)\n",
    )
    .expect("write labels fixture");

    let mut cmd = paredit();
    cmd.arg("rename-local-function")
        .arg("--from")
        .arg("old-name")
        .arg("--to")
        .arg("new-name")
        .arg("--write")
        .arg(&lisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"definitionCount\": 1"))
        .stdout(predicate::str::contains("\"callCount\": 2"))
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(&lisp_file).expect("read rewritten labels fixture"),
        "(labels ((new-name (x) (new-name x))) (new-name 1) old-name)\n"
    );
}

#[test]
fn cli_writes_setf_local_callable_rename_updates_definition_and_call_site() {
    let dir = fresh_temp_dir("rename-local-function-setf-write");
    let lisp_file = dir.join("core.lisp");
    fs::write(
        &lisp_file,
        "(flet (((setf old-name) (value object) value)) ((setf old-name) 1 thing) old-name)\n",
    )
    .expect("write setf local callable fixture");

    let mut cmd = paredit();
    cmd.arg("rename-local-function")
        .arg("--from")
        .arg("old-name")
        .arg("--to")
        .arg("new-name")
        .arg("--write")
        .arg(&lisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"definitionCount\": 1"))
        .stdout(predicate::str::contains("\"callCount\": 1"))
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(&lisp_file).expect("read rewritten setf local callable fixture"),
        "(flet (((setf new-name) (value object) value)) ((setf new-name) 1 thing) old-name)\n"
    );
}

#[test]
fn cli_writes_labels_setf_local_callable_rename_updates_definition_and_call_site() {
    let dir = fresh_temp_dir("rename-local-function-labels-setf-write");
    let lisp_file = dir.join("core.lisp");
    fs::write(
        &lisp_file,
        "(labels (((setf old-name) (value object) value)) ((setf old-name) 1 thing) old-name)\n",
    )
    .expect("write labels setf local callable fixture");

    let mut cmd = paredit();
    cmd.arg("rename-local-function")
        .arg("--from")
        .arg("old-name")
        .arg("--to")
        .arg("new-name")
        .arg("--write")
        .arg(&lisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"definitionCount\": 1"))
        .stdout(predicate::str::contains("\"callCount\": 1"))
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(&lisp_file).expect("read rewritten labels setf local callable fixture"),
        "(labels (((setf new-name) (value object) value)) ((setf new-name) 1 thing) old-name)\n"
    );
}

#[test]
fn cli_writes_package_qualified_setf_local_callable_rename_updates_definition_and_call_site() {
    let dir = fresh_temp_dir("rename-local-function-qualified-setf-write");
    let lisp_file = dir.join("core.lisp");
    fs::write(
        &lisp_file,
        "(cl-user:flet (((setf old-name) (value object) value)) ((setf old-name) 1 thing) old-name)\n",
    )
    .expect("write qualified setf local callable fixture");

    let mut cmd = paredit();
    cmd.arg("rename-local-function")
        .arg("--from")
        .arg("old-name")
        .arg("--to")
        .arg("new-name")
        .arg("--write")
        .arg(&lisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"definitionCount\": 1"))
        .stdout(predicate::str::contains("\"callCount\": 1"))
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(&lisp_file)
            .expect("read rewritten qualified setf local callable fixture"),
        "(cl-user:flet (((setf new-name) (value object) value)) ((setf new-name) 1 thing) old-name)\n"
    );
}

#[test]
fn cli_plans_package_qualified_flet_rename() {
    let dir = fresh_temp_dir("rename-local-function-qualified-flet-plan");
    let lisp_file = dir.join("core.lisp");
    fs::write(
        &lisp_file,
        "(cl:flet ((old-name (x) (old-name x))) (old-name 1) old-name)\n",
    )
    .expect("write qualified flet fixture");

    let mut cmd = paredit();
    cmd.arg("rename-local-function")
        .arg("--from")
        .arg("old-name")
        .arg("--to")
        .arg("new-name")
        .arg(&lisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"definitionCount\": 1"))
        .stdout(predicate::str::contains("\"callCount\": 1"))
        .stdout(predicate::str::contains(
            "(cl:flet ((new-name (x) (old-name x))) (new-name 1) old-name)",
        ));

    assert_eq!(
        fs::read_to_string(&lisp_file).expect("read unchanged qualified flet fixture"),
        "(cl:flet ((old-name (x) (old-name x))) (old-name 1) old-name)\n"
    );
}

#[test]
fn cli_writes_package_qualified_labels_rename_with_recursive_calls() {
    let dir = fresh_temp_dir("rename-local-function-qualified-labels-write");
    let lisp_file = dir.join("core.lisp");
    fs::write(
        &lisp_file,
        "(cl-user:labels ((old-name (x) (old-name x))) (old-name 1) old-name)\n",
    )
    .expect("write qualified labels fixture");

    let mut cmd = paredit();
    cmd.arg("rename-local-function")
        .arg("--from")
        .arg("old-name")
        .arg("--to")
        .arg("new-name")
        .arg("--write")
        .arg(&lisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"definitionCount\": 1"))
        .stdout(predicate::str::contains("\"callCount\": 2"))
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(&lisp_file).expect("read rewritten qualified labels fixture"),
        "(cl-user:labels ((new-name (x) (new-name x))) (new-name 1) old-name)\n"
    );
}

#[test]
fn cli_plans_emacs_lisp_cl_flet_rename_without_touching_definition_body_or_noncall_values() {
    let dir = fresh_temp_dir("rename-local-function-emacs-cl-flet-plan");
    let lisp_file = dir.join("core.el");
    fs::write(
        &lisp_file,
        "(cl-flet ((old-name (x) (old-name x))) (old-name 1) old-name)\n",
    )
    .expect("write emacs lisp cl-flet fixture");

    let mut cmd = paredit();
    cmd.arg("rename-local-function")
        .arg("--from")
        .arg("old-name")
        .arg("--to")
        .arg("new-name")
        .arg(&lisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"definitionCount\": 1"))
        .stdout(predicate::str::contains("\"callCount\": 1"))
        .stdout(predicate::str::contains(
            "(cl-flet ((new-name (x) (old-name x))) (new-name 1) old-name)",
        ));

    assert_eq!(
        fs::read_to_string(&lisp_file).expect("read unchanged emacs lisp cl-flet fixture"),
        "(cl-flet ((old-name (x) (old-name x))) (old-name 1) old-name)\n"
    );
}

#[test]
fn cli_writes_emacs_lisp_cl_labels_rename_with_recursive_calls() {
    let dir = fresh_temp_dir("rename-local-function-emacs-cl-labels-write");
    let lisp_file = dir.join("core.el");
    fs::write(
        &lisp_file,
        "(cl-labels ((old-name (x) (old-name x))) (old-name 1) old-name)\n",
    )
    .expect("write emacs lisp cl-labels fixture");

    let mut cmd = paredit();
    cmd.arg("rename-local-function")
        .arg("--from")
        .arg("old-name")
        .arg("--to")
        .arg("new-name")
        .arg("--write")
        .arg(&lisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"definitionCount\": 1"))
        .stdout(predicate::str::contains("\"callCount\": 2"))
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(&lisp_file).expect("read rewritten emacs lisp cl-labels fixture"),
        "(cl-labels ((new-name (x) (new-name x))) (new-name 1) old-name)\n"
    );
}

#[test]
fn cli_writes_local_function_rename_without_crossing_nested_shadow() {
    let dir = fresh_temp_dir("rename-local-function-shadow-write");
    let lisp_file = dir.join("core.lisp");
    fs::write(
        &lisp_file,
        "(flet ((old-name (x) x)) (labels ((old-name (y) (old-name y))) (old-name 1)) (old-name 2))\n",
    )
    .expect("write shadow fixture");

    let mut cmd = paredit();
    cmd.arg("rename-local-function")
        .arg("--from")
        .arg("old-name")
        .arg("--to")
        .arg("new-name")
        .arg("--write")
        .arg(&lisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"definitionCount\": 1"))
        .stdout(predicate::str::contains("\"callCount\": 1"))
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(&lisp_file).expect("read rewritten shadow fixture"),
        "(flet ((new-name (x) x)) (labels ((old-name (y) (old-name y))) (old-name 1)) (new-name 2))\n"
    );
}

#[test]
fn cli_writes_labels_rename_with_function_designators() {
    let dir = fresh_temp_dir("rename-local-function-designators-write");
    let lisp_file = dir.join("core.lisp");
    fs::write(
        &lisp_file,
        "(labels ((old-name (x) #'old-name (function old-name) (old-name x))) #'old-name (function old-name) old-name)\n",
    )
    .expect("write designator fixture");

    let mut cmd = paredit();
    cmd.arg("rename-local-function")
        .arg("--from")
        .arg("old-name")
        .arg("--to")
        .arg("new-name")
        .arg("--write")
        .arg(&lisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"definitionCount\": 1"))
        .stdout(predicate::str::contains("\"callCount\": 5"))
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(&lisp_file).expect("read rewritten designator fixture"),
        "(labels ((new-name (x) #'new-name (function new-name) (new-name x))) #'new-name (function new-name) old-name)\n"
    );
}

#[test]
fn cli_writes_flet_rename_inside_reader_quoted_lambda_bodies() {
    let dir = fresh_temp_dir("rename-local-function-reader-quoted-lambda-write");
    let lisp_file = dir.join("core.lisp");
    fs::write(
        &lisp_file,
        "(flet ((old-name (x) #'(lambda () (old-name x) old-name))) (old-name 1) old-name)\n",
    )
    .expect("write reader quoted lambda fixture");

    let mut cmd = paredit();
    cmd.arg("rename-local-function")
        .arg("--from")
        .arg("old-name")
        .arg("--to")
        .arg("new-name")
        .arg("--write")
        .arg(&lisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"definitionCount\": 1"))
        .stdout(predicate::str::contains("\"callCount\": 3"))
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(&lisp_file).expect("read rewritten reader quoted lambda fixture"),
        "(flet ((new-name (x) #'(lambda () (new-name x) new-name))) (new-name 1) old-name)\n"
    );
}

#[test]
fn cli_writes_outer_flet_rename_inside_macrolet_expander_only() {
    let dir = fresh_temp_dir("rename-local-function-macrolet-expander-write");
    let lisp_file = dir.join("core.lisp");
    fs::write(
        &lisp_file,
        "(flet ((old-name (x) x)) (macrolet ((old-name () #'old-name (function old-name) (old-name 1))) (old-name) #'old-name (function old-name) (old-name 2)))\n",
    )
    .expect("write macrolet expander fixture");

    let mut cmd = paredit();
    cmd.arg("rename-local-function")
        .arg("--from")
        .arg("old-name")
        .arg("--to")
        .arg("new-name")
        .arg("--write")
        .arg(&lisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"definitionCount\": 1"))
        .stdout(predicate::str::contains("\"callCount\": 3"))
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(&lisp_file).expect("read rewritten macrolet expander fixture"),
        "(flet ((new-name (x) x)) (macrolet ((old-name () #'new-name (function new-name) (new-name 1))) (old-name) #'old-name (function old-name) (old-name 2)))\n"
    );
}

#[test]
fn cli_writes_outer_flet_rename_inside_compiler_macrolet_expander_only() {
    let dir = fresh_temp_dir("rename-local-function-compiler-macrolet-expander-write");
    let lisp_file = dir.join("core.lisp");
    fs::write(
        &lisp_file,
        "(flet ((old-name (x) x)) (compiler-macrolet ((old-name () #'old-name (function old-name) (old-name 1))) (old-name) #'old-name (function old-name) (old-name 2)))\n",
    )
    .expect("write compiler-macrolet expander fixture");

    let mut cmd = paredit();
    cmd.arg("rename-local-function")
        .arg("--from")
        .arg("old-name")
        .arg("--to")
        .arg("new-name")
        .arg("--write")
        .arg(&lisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"definitionCount\": 1"))
        .stdout(predicate::str::contains("\"callCount\": 3"))
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(&lisp_file).expect("read rewritten compiler-macrolet expander fixture"),
        "(flet ((new-name (x) x)) (compiler-macrolet ((old-name () #'new-name (function new-name) (new-name 1))) (old-name) #'old-name (function old-name) (old-name 2)))\n"
    );
}

#[test]
fn cli_writes_package_qualified_outer_flet_rename_inside_macrolet_expander_only() {
    let dir = fresh_temp_dir("rename-local-function-qualified-macrolet-expander-write");
    let lisp_file = dir.join("core.lisp");
    fs::write(
        &lisp_file,
        "(cl-user:flet ((old-name (x) x)) (cl-user:macrolet ((old-name () #'old-name (function old-name) (old-name 1))) (old-name) #'old-name (function old-name) (old-name 2)))\n",
    )
    .expect("write qualified macrolet expander fixture");

    let mut cmd = paredit();
    cmd.arg("rename-local-function")
        .arg("--from")
        .arg("old-name")
        .arg("--to")
        .arg("new-name")
        .arg("--write")
        .arg(&lisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"definitionCount\": 1"))
        .stdout(predicate::str::contains("\"callCount\": 3"))
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(&lisp_file).expect("read rewritten qualified macrolet expander fixture"),
        "(cl-user:flet ((new-name (x) x)) (cl-user:macrolet ((old-name () #'new-name (function new-name) (new-name 1))) (old-name) #'old-name (function old-name) (old-name 2)))\n"
    );
}

#[test]
fn cli_writes_package_qualified_outer_flet_rename_inside_compiler_macrolet_expander_only() {
    let dir = fresh_temp_dir("rename-local-function-qualified-compiler-macrolet-expander-write");
    let lisp_file = dir.join("core.lisp");
    fs::write(
        &lisp_file,
        "(cl-user:flet ((old-name (x) x)) (cl-user:compiler-macrolet ((old-name () #'old-name (function old-name) (old-name 1))) (old-name) #'old-name (function old-name) (old-name 2)))\n",
    )
    .expect("write qualified compiler-macrolet expander fixture");

    let mut cmd = paredit();
    cmd.arg("rename-local-function")
        .arg("--from")
        .arg("old-name")
        .arg("--to")
        .arg("new-name")
        .arg("--write")
        .arg(&lisp_file)
        .assert()
        .success()
        .stdout(predicate::str::contains("\"definitionCount\": 1"))
        .stdout(predicate::str::contains("\"callCount\": 3"))
        .stdout(predicate::str::contains("\"written\": true"));

    assert_eq!(
        fs::read_to_string(&lisp_file)
            .expect("read rewritten qualified compiler-macrolet expander fixture"),
        "(cl-user:flet ((new-name (x) x)) (cl-user:compiler-macrolet ((old-name () #'new-name (function new-name) (new-name 1))) (old-name) #'old-name (function old-name) (old-name 2)))\n"
    );
}

#[test]
fn cli_rejects_local_function_rename_without_matching_definition() {
    let dir = fresh_temp_dir("rename-local-function-missing-definition");
    let lisp_file = dir.join("core.lisp");
    fs::write(&lisp_file, "(old-name 1)\n").expect("write lisp fixture");

    let mut cmd = paredit();
    cmd.arg("rename-local-function")
        .arg("--from")
        .arg("old-name")
        .arg("--to")
        .arg("new-name")
        .arg("--write")
        .arg(&lisp_file)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "rename-local-function requires at least one matching local function definition",
        ));

    assert_eq!(
        fs::read_to_string(&lisp_file).expect("read unchanged missing-definition fixture"),
        "(old-name 1)\n"
    );
}

#[test]
fn cli_help_describes_rename_local_function_contract() {
    let mut cmd = paredit();
    cmd.arg("rename-local-function")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Plan or apply a Common Lisp flet/labels local function binding and call-site rename across explicit files",
        ))
        .stdout(predicate::str::contains(
            "preserving the difference between non-recursive flet bodies and recursive labels bodies",
        ))
        .stdout(predicate::str::contains("Usage:"))
        .stdout(predicate::str::contains("--from <FROM>"))
        .stdout(predicate::str::contains("--to <TO>"))
        .stdout(predicate::str::contains("--write"))
        .stdout(predicate::str::contains("--output <OUTPUT>"))
        .stdout(predicate::str::contains("<FILES>..."));
}
