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
