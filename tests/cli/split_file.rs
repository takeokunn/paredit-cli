use super::*;

#[test]
fn cli_plans_file_split_without_writing() {
    let dir = fresh_temp_dir("split-file-plan");
    let from_file = dir.join("core.lisp");
    let to_file = dir.join("ui").join("render.lisp");
    let original_from = "(in-package #:demo)\n\
                         (defun keep () :ok)\n\
                         (defun render-pane () :render)\n\
                         (defmacro with-render (() &body body) `(progn ,@body))\n";
    fs::write(&from_file, original_from).expect("write source fixture");

    let mut cmd = paredit();
    cmd.arg("split-file")
        .arg("--from-file")
        .arg(&from_file)
        .arg("--to-file")
        .arg(&to_file)
        .arg("--path")
        .arg("2")
        .arg("--path")
        .arg("3")
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"command\": \"split-file\""))
        .stdout(predicate::str::contains("\"definition_count\": 2"))
        .stdout(predicate::str::contains("\"to_file_existed\": false"))
        .stdout(predicate::str::contains("\"to_parent_existed\": false"))
        .stdout(predicate::str::contains("\"written\": false"))
        .stdout(predicate::str::contains("\"name\": \"render-pane\""))
        .stdout(predicate::str::contains("\"name\": \"with-render\""));

    assert_eq!(
        fs::read_to_string(&from_file).expect("read unchanged source"),
        original_from
    );
    assert!(
        !to_file.exists(),
        "planning should not create the destination"
    );
}

#[test]
fn cli_writes_file_split_into_nested_directory() {
    let dir = fresh_temp_dir("split-file-write");
    let from_file = dir.join("core.lisp");
    let to_file = dir.join("features").join("render").join("render.lisp");
    fs::write(
        &from_file,
        "(in-package #:demo)\n\
         (defun keep () :ok)\n\
         (defun render-pane () :render)\n\
         (defmacro with-render (() &body body) `(progn ,@body))\n",
    )
    .expect("write source fixture");

    let mut cmd = paredit();
    cmd.arg("split-file")
        .arg("--from-file")
        .arg(&from_file)
        .arg("--to-file")
        .arg(&to_file)
        .arg("--path")
        .arg("2")
        .arg("--path")
        .arg("3")
        .arg("--write")
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"definition_count\": 2"))
        .stdout(predicate::str::contains("\"to_parent_existed\": false"))
        .stdout(predicate::str::contains("\"written\": true"));

    let source = fs::read_to_string(&from_file).expect("read rewritten source");
    let destination = fs::read_to_string(&to_file).expect("read rewritten destination");
    assert!(source.contains("(in-package #:demo)"));
    assert!(source.contains("(defun keep () :ok)"));
    assert!(!source.contains("render-pane"));
    assert!(!source.contains("with-render"));
    let render_index = destination
        .find("(defun render-pane () :render)")
        .expect("render function should exist");
    let macro_index = destination
        .find("(defmacro with-render")
        .expect("render macro should exist");
    assert!(render_index < macro_index);
}

#[test]
fn cli_plans_file_split_by_name_and_kind() {
    let dir = fresh_temp_dir("split-file-selectors");
    let from_file = dir.join("core.lisp");
    let to_file = dir.join("ui").join("render.lisp");
    fs::write(
        &from_file,
        "(in-package #:demo)\n\
         (defun keep () :ok)\n\
         (defun render-pane () :render)\n\
         (defmacro with-render (() &body body) `(progn ,@body))\n\
         (defclass renderer () ())\n",
    )
    .expect("write source fixture");

    let mut cmd = paredit();
    cmd.arg("split-file")
        .arg("--from-file")
        .arg(&from_file)
        .arg("--to-file")
        .arg(&to_file)
        .arg("--name")
        .arg("render-pane")
        .arg("--kind")
        .arg("macro")
        .arg("--output")
        .arg("json")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"definition_count\": 2"))
        .stdout(predicate::str::contains("\"name\": \"render-pane\""))
        .stdout(predicate::str::contains("\"category\": \"function\""))
        .stdout(predicate::str::contains("\"name\": \"with-render\""))
        .stdout(predicate::str::contains("\"category\": \"macro\""))
        .stdout(predicate::str::contains("\"written\": false"));

    assert!(
        !to_file.exists(),
        "selector planning should not create the destination"
    );
}

fn assert_split_file_property(definition_count: usize) -> Result<(), TestCaseError> {
    let dir = fresh_temp_dir(&format!("split-file-pbt-{definition_count}"));
    let from_file = dir.join("core.lisp");
    let to_file = dir.join("generated").join("moved.lisp");
    let mut source = String::from("(in-package #:demo)\n");
    for index in 0..definition_count {
        source.push_str(&format!("(defun moved-{index} () {index})\n"));
    }
    source.push_str("(defun keep () :ok)\n");
    fs::write(&from_file, source).expect("write generated source");

    let mut cmd = paredit();
    cmd.arg("split-file")
        .arg("--from-file")
        .arg(&from_file)
        .arg("--to-file")
        .arg(&to_file)
        .arg("--write")
        .arg("--output")
        .arg("json");
    for index in 0..definition_count {
        cmd.arg("--name").arg(format!("moved-{index}"));
    }
    let output = cmd
        .output()
        .map_err(|err| TestCaseError::fail(format!("run split-file: {err}")))?;
    prop_assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let report = serde_json::from_slice::<serde_json::Value>(&output.stdout)
        .map_err(|err| TestCaseError::fail(format!("parse json: {err}")))?;
    prop_assert_eq!(
        report["definition_count"].as_u64(),
        Some(definition_count as u64)
    );
    prop_assert_eq!(report["written"].as_bool(), Some(true));

    let rewritten_source = fs::read_to_string(&from_file)
        .map_err(|err| TestCaseError::fail(format!("read source: {err}")))?;
    let destination = fs::read_to_string(&to_file)
        .map_err(|err| TestCaseError::fail(format!("read destination: {err}")))?;
    prop_assert!(rewritten_source.contains("(defun keep () :ok)"));
    let mut previous_position = 0;
    for index in 0..definition_count {
        let name = format!("moved-{index}");
        prop_assert!(!rewritten_source.contains(&name));
        let position = destination
            .find(&name)
            .ok_or_else(|| TestCaseError::fail(format!("missing {name}")))?;
        prop_assert!(position >= previous_position);
        previous_position = position;
    }

    for rewritten in [rewritten_source, destination] {
        let check_output = paredit()
            .arg("check")
            .write_stdin(rewritten)
            .output()
            .map_err(|err| TestCaseError::fail(format!("run check: {err}")))?;
        prop_assert!(
            check_output.status.success(),
            "check stderr={}",
            String::from_utf8_lossy(&check_output.stderr)
        );
    }

    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(12))]

    #[test]
    fn cli_split_file_preserves_order_and_parseability(definition_count in 1usize..6) {
        assert_split_file_property(definition_count)?;
    }
}
