# Installation

`paredit` ships as a single binary. Nix is the primary distribution channel;
Cargo works anywhere a Rust toolchain is available.

## Run without installing (Nix)

```sh
nix run github:takeokunn/paredit-cli -- inspect check --file source.lisp
```

The companion lint and format gates are exposed as flake apps:

```sh
nix run github:takeokunn/paredit-cli#lint -- .
nix run github:takeokunn/paredit-cli#format -- --check .
```

## Install into a Nix profile

```sh
nix profile install github:takeokunn/paredit-cli
```

Prebuilt binaries are published to the public
[`takeokunn-paredit-cli`](https://takeokunn-paredit-cli.cachix.org) Cachix
cache, so neither command has to compile the crate from source:

```sh
cachix use takeokunn-paredit-cli
```

## Use as a flake input

Add the flake and pick the packages or the overlay:

```nix
{
  inputs.paredit-cli.url = "github:takeokunn/paredit-cli";

  outputs = { nixpkgs, paredit-cli, ... }: {
    # Directly as a package:
    #   paredit-cli.packages.${system}.default
    # Or through the overlay, which provides pkgs.paredit-cli,
    # pkgs.paredit-lint, pkgs.paredit-format, and pkgs.paredit-format-files:
    #   nixpkgs.overlays = [ paredit-cli.overlays.default ];
  };
}
```

The flake also exports `lib.${system}.mkLintCheck`, `mkFormatCheck`, and
`treefmtFormatter` for wiring structural checks into another project's
`nix flake check` — see [Integrations](integrations.md).

## Install with Cargo

```sh
cargo install --git https://github.com/takeokunn/paredit-cli --locked
```

The minimum supported Rust version is `1.85` (edition 2024).

## Pin automation

The examples above follow the latest default branch. For CI, production
automation, or a reproducible developer environment, pin an immutable commit
instead. Replace `<commit>` with a full commit SHA that you have reviewed:

```sh
nix run github:takeokunn/paredit-cli/<commit> -- --help
nix profile install github:takeokunn/paredit-cli/<commit>
cargo install --git https://github.com/takeokunn/paredit-cli --rev <commit> --locked
```

When upgrading a pin, inspect the release notes and compare the machine-readable
command catalog before accepting the new revision:

```sh
paredit inspect capabilities --output json
```

## Verify

```sh
paredit --help
paredit inspect --help
```
