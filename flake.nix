{
  description = "paredit-cli: structure-editing CLI for S-expression refactoring";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ rust-overlay.overlays.default ];
        pkgs = import nixpkgs { inherit system overlays; };
        rustToolchain = pkgs.rust-bin.stable.latest.default;
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "paredit-cli";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
        };

        apps.default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/paredit";
          meta.description = "Run paredit-cli";
        };

        devShells.default = pkgs.mkShell {
          packages = [
            rustToolchain
            pkgs.rust-analyzer
            pkgs.cargo-nextest
            pkgs.rustfmt
            pkgs.clippy
          ];
          shellHook = ''
            cat <<'USAGE_EOF'

            === paredit-cli Development Shell ===

            Development loop:
              cargo fmt --all
              cargo clippy --all-targets -- -D warnings
              cargo test

            Quick verification:
              nix flake check  # fmt check + clippy -D warnings + package build/tests

            Build and run:
              nix build .#              # result/bin/paredit
              nix run .# -- check --file source.lisp

            Format everything (Rust + Nix):
              nix fmt

            USAGE_EOF
          '';
        };

        formatter = pkgs.writeShellApplication {
          name = "fmt";
          runtimeInputs = [
            rustToolchain
            pkgs.nixfmt
          ];
          text = ''
            nixfmt "$@"
            cargo fmt
          '';
        };

        checks = {
          default = pkgs.runCommand "paredit-cli-check"
            {
              nativeBuildInputs = [ rustToolchain ];
              src = self;
            }
            ''
              cp -r $src/. .
              chmod -R u+w .
              cargo fmt --check
              touch $out
            '';
          clippy = (self.packages.${system}.default).overrideAttrs (old: {
            pname = "paredit-cli-clippy";
            nativeBuildInputs = (old.nativeBuildInputs or [ ]) ++ [ pkgs.clippy ];
            doCheck = false;
            buildPhase = ''
              cargo clippy --all-targets -- -D warnings
            '';
            installPhase = ''
              touch $out
            '';
          });
          package = self.packages.${system}.default;
        };
      });
}
