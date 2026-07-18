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
fn documentation_covers_lint_and_format_integration_surfaces() {
    let docs =
        fs::read_to_string("docs/src/integrations.md").expect("read integrations documentation");

    assert!(
        docs.contains("## GitHub Actions"),
        "documentation must include integration guidance"
    );
    for needle in [
        "uses: takeokunn/paredit-cli@",
        "mode: lint",
        "nix run github:takeokunn/paredit-cli -- inspect check",
        "paredit inspect check --file source.lisp",
    ] {
        assert!(
            docs.contains(needle),
            "integration documentation must include: {needle}"
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
        "pkgs.cargo-audit",
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

#[test]
fn ci_checks_each_host_and_audits_dependencies_on_linux() {
    let workflow = fs::read_to_string(".github/workflows/ci.yml").expect("read CI workflow");

    for needle in [
        "ubuntu-latest",
        "macos-latest",
        "runs-on: ${{ matrix.os }}",
        "supply-chain:",
        "nix develop --command cargo audit --deny warnings",
    ] {
        assert!(
            workflow.contains(needle),
            "CI must keep its cross-host and supply-chain gate: {needle}"
        );
    }
}

#[test]
fn external_github_actions_are_immutably_pinned() {
    let action = fs::read_to_string("action.yml").expect("read action.yml");
    let ci = fs::read_to_string(".github/workflows/ci.yml").expect("read CI workflow");
    let docs =
        fs::read_to_string(".github/workflows/docs.yml").expect("read documentation workflow");

    for pin in [
        "cachix/install-nix-action@630ae543ea3a38a9a4166f03376c02c50f408342 # v31",
        "cachix/cachix-action@5f2d7c5294214f71b873db4b969586b980625e71 # v17",
    ] {
        assert!(
            action.contains(pin),
            "action.yml must keep the approved immutable action pin: {pin}"
        );
    }

    for pin in [
        "actions/checkout@9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0 # v7",
        "cachix/install-nix-action@630ae543ea3a38a9a4166f03376c02c50f408342 # v31",
        "cachix/cachix-action@5f2d7c5294214f71b873db4b969586b980625e71 # v17",
    ] {
        assert!(
            ci.contains(pin),
            "CI must keep the approved immutable action pin: {pin}"
        );
    }

    for pin in [
        "actions/checkout@9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0 # v7",
        "cachix/install-nix-action@630ae543ea3a38a9a4166f03376c02c50f408342 # v31",
        "cachix/cachix-action@5f2d7c5294214f71b873db4b969586b980625e71 # v17",
        "actions/configure-pages@983d7736d9b0ae728b81ab479565c72886d7745b # v5",
        "actions/upload-pages-artifact@7b1f4a764d45c48632c6b24a0339c27f5614fb0b # v4",
        "actions/deploy-pages@d6db90164ac5ed86f2b6aed7e0febac5b3c0c03e # v4",
    ] {
        assert!(
            docs.contains(pin),
            "documentation workflow must keep the approved immutable action pin: {pin}"
        );
    }

    for (path, contents) in [
        ("action.yml", action.as_str()),
        (".github/workflows/ci.yml", ci.as_str()),
        (".github/workflows/docs.yml", docs.as_str()),
    ] {
        for mutable_tag in [
            "actions/checkout@v",
            "cachix/install-nix-action@v",
            "cachix/cachix-action@v",
            "actions/configure-pages@v",
            "actions/upload-pages-artifact@v",
            "actions/deploy-pages@v",
        ] {
            assert!(
                !contents.contains(mutable_tag),
                "{path} must not use mutable action tag {mutable_tag}"
            );
        }
    }
}
