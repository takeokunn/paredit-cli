use std::collections::BTreeSet;
use std::process::Command as ProcessCommand;

#[test]
fn cargo_package_includes_public_oss_docs() {
    let cargo = std::env::var("CARGO").expect("CARGO env var");
    let output = ProcessCommand::new(cargo)
        .args(["package", "--allow-dirty", "--list"])
        .output()
        .expect("run cargo package --list");

    assert!(
        output.status.success(),
        "cargo package --list failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let packaged_output = String::from_utf8(output.stdout).expect("package output is utf-8");
    let packaged_files: BTreeSet<&str> = packaged_output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();

    for required in ["LICENSE", "README.md"] {
        assert!(
            packaged_files.contains(required),
            "cargo package archive is missing required public document: {required}"
        );
    }
}
