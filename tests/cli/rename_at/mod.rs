use super::*;

mod failure;
mod success;

fn byte_offset(input: &str, marker: &str) -> usize {
    input.find(marker).expect("target marker")
}

fn write_rename_at(
    fixture_name: &str,
    dialect: Option<&str>,
    input: &str,
    marker: &str,
    to: &str,
) -> String {
    let dir = fresh_temp_dir(fixture_name);
    let file = dir.join("input.lisp");
    fs::write(&file, input).expect("write rename-at fixture");

    let mut cmd = paredit();
    cmd.arg("refactor")
        .arg("rename-at")
        .arg("--file")
        .arg(&file)
        .arg("--at")
        .arg(byte_offset(input, marker).to_string())
        .arg("--to")
        .arg(to)
        .arg("--write");
    if let Some(dialect) = dialect {
        cmd.arg("--dialect").arg(dialect);
    }
    cmd.assert().success();

    fs::read_to_string(file).expect("read renamed fixture")
}
