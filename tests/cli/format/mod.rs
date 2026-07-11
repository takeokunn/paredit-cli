use super::*;
use std::path::Path;

mod binding_forms;
mod body_forms;
mod declaration_forms;
mod definition_forms;

fn assert_format_output(fixture_name: &str, file_name: &str, input: &str, expected: &str) {
    let dir = fresh_temp_dir(fixture_name);
    let file = dir.join(Path::new(file_name));
    fs::write(&file, input).expect("write source fixture");

    let mut cmd = paredit();
    cmd.arg("edit")
        .arg("format")
        .arg("--file")
        .arg(&file)
        .assert()
        .success()
        .stdout(predicate::str::contains(expected));
}
