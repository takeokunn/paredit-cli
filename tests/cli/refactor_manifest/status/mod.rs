use super::*;
use std::path::Path;

const FUNCTION_FIXTURE: &str = "(defun old-name (x) x)\n(defun caller () (old-name 1) old-name)\n";

fn write_manifest_fixture(path: &Path, contents: &str) {
    fs::write(path, contents).expect("write refactor manifest fixture");
}

fn preview_function_manifest(path: &Path, strict: bool) -> Vec<u8> {
    let mut preview = paredit();
    preview
        .args(["refactor", "preview"])
        .arg("--from")
        .arg("old-name")
        .arg("--to")
        .arg("new-name")
        .arg("--mode")
        .arg("function");
    if strict {
        preview
            .arg("--fail-on-no-change")
            .arg("--fail-on-parse-error")
            .arg("--require-definitions")
            .arg("1")
            .arg("--require-edits")
            .arg("2");
    }
    preview
        .arg("--output")
        .arg("json")
        .arg(path)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone()
}

fn assert_status_preserves_file(path: &Path, expected: &str, description: &str) {
    assert_eq!(
        fs::read_to_string(path).unwrap_or_else(|err| panic!("read {description}: {err}")),
        expected
    );
}

mod blocked;
mod invalid;
mod ready;
