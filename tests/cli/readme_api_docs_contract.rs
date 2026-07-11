use super::*;

#[test]
fn readme_links_manifest_documentation_url_when_library_api_is_public() {
    let readme = fs::read_to_string("README.md").expect("read README");
    let manifest = fs::read_to_string("Cargo.toml").expect("read Cargo.toml");

    let documentation_url = manifest
        .lines()
        .find_map(|line| line.trim().strip_prefix("documentation = "))
        .map(|value| value.trim_matches('"').to_owned())
        .expect("Cargo.toml documentation url");

    assert!(
        readme.contains("A typed Rust library API behind the CLI"),
        "contract only applies while README advertises a public Rust library API"
    );
    assert!(
        readme.contains(&format!("]({documentation_url})")),
        "README must link Cargo.toml documentation URL for API discoverability"
    );
}
