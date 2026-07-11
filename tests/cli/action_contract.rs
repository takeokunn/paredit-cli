use std::fs;

#[test]
fn composite_action_exposes_lint_format_and_fix_modes() {
    let action = fs::read_to_string("action.yml").expect("read action.yml");

    assert!(
        action.contains("using: composite"),
        "action.yml must stay a composite action so consumers need no container"
    );
    for needle in [
        "cachix/install-nix-action",
        "cachix/cachix-action",
        "default: takeokunn-paredit-cli",
        "default: lint",
    ] {
        assert!(
            action.contains(needle),
            "action.yml must keep the documented toolchain and defaults: {needle}"
        );
    }
    for mode in ["lint)", "format)", "fix)"] {
        assert!(
            action.contains(mode),
            "action.yml dispatch must handle the documented mode {mode}"
        );
    }
    assert!(
        action.contains("github.action_ref"),
        "action.yml must default the binary ref to the pinned action ref"
    );
}

#[test]
fn readme_documents_lint_and_format_integration_surfaces() {
    let readme = fs::read_to_string("README.md").expect("read README");

    assert!(
        readme.contains("## Lint and Format Integration"),
        "README must document the lint/format integration section"
    );
    for needle in [
        "uses: takeokunn/paredit-cli@",
        "mode: lint",
        "mode: format",
        "nix run github:takeokunn/paredit-cli#lint -- .",
        "nix run github:takeokunn/paredit-cli#format -- --check .",
        "overlays.default",
        "mkLintCheck",
        "mkFormatCheck",
        "treefmtFormatter",
    ] {
        assert!(
            readme.contains(needle),
            "README lint/format integration must document: {needle}"
        );
    }
}

#[test]
fn flake_exposes_the_documented_integration_surfaces() {
    let flake = fs::read_to_string("flake.nix").expect("read flake.nix");

    for needle in [
        "paredit-lint",
        "paredit-format",
        "paredit-format-files",
        "overlays.default",
        "mkLintCheck",
        "mkFormatCheck",
        "treefmtFormatter",
        "treefmt-nix",
        "lint-format-integration",
    ] {
        assert!(
            flake.contains(needle),
            "flake.nix must keep the documented integration surface: {needle}"
        );
    }
    assert!(
        flake.contains("excludes = [ \"tests/fixtures/*\" ]"),
        "flake.nix must keep test fixtures out of the paredit treefmt formatter"
    );
}
