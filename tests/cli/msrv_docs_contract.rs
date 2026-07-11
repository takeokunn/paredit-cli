use super::*;

#[test]
fn msrv_verification_docs_match_manifest_contract() {
    let manifest = fs::read_to_string("Cargo.toml").expect("read Cargo.toml");
    let readme = fs::read_to_string("README.md").expect("read README");
    let contributing = fs::read_to_string("CONTRIBUTING.md").expect("read CONTRIBUTING");
    let release = fs::read_to_string("RELEASE.md").expect("read RELEASE");
    let compatibility = fs::read_to_string("COMPATIBILITY.md").expect("read COMPATIBILITY");

    let rust_version = manifest
        .lines()
        .find_map(|line| line.trim().strip_prefix("rust-version = "))
        .map(|value| value.trim_matches('"').to_owned())
        .expect("Cargo.toml rust-version");
    let msrv_command = format!("cargo +{rust_version} test --locked");

    assert!(
        readme.contains(&msrv_command),
        "README must document the local MSRV verification command"
    );
    assert!(
        contributing.contains(&msrv_command),
        "CONTRIBUTING must require the local MSRV verification command"
    );
    assert!(
        release.contains(&msrv_command),
        "RELEASE must require the local MSRV verification command"
    );
    assert!(
        normalize_whitespace(&compatibility).contains(&normalize_whitespace(
            "Changes that raise the MSRV must update `Cargo.toml`, `README.md`, and the \
             release notes together."
        )),
        "COMPATIBILITY must define the documentation discipline for MSRV bumps"
    );
}

fn normalize_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
