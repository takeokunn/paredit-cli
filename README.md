# paredit-cli

[![CI](https://github.com/takeokunn/paredit-cli/actions/workflows/main.yml/badge.svg)](https://github.com/takeokunn/paredit-cli/actions/workflows/main.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://github.com/takeokunn/paredit-cli/blob/main/LICENSE)

`paredit` is a structure-aware CLI for inspecting and safely refactoring Lisp
S-expressions. Its canonical command layout is designed for both people and AI
coding agents:

- `paredit inspect ...` reads source and produces reports.
- `paredit edit ...` transforms one selected form; stdout by default, `--write`
  to update the file in place, `--diff` to preview a unified diff.
- `paredit refactor ...` plans, previews, verifies, and applies semantic changes.

## Commands

```sh
paredit inspect <report> [args]    # read-only inventory, validation, analysis
paredit edit <transform> [args]    # one structural edit (stdout, --diff, or --write)
paredit refactor <workflow> [args] # plan, preview, verify, and apply changes
paredit completions <shell>        # shell completion scripts (bash/zsh/fish/...)
```

Run `paredit --help`, then `paredit <namespace> --help` for the complete
command list. All source-facing commands live in these three namespaces;
`completions` is the only meta command. For machine-readable discovery, run:

```sh
paredit inspect capabilities --output json  # full catalog of commands and flags
```

## Quick Start

```sh
paredit inspect check --file src/example.lisp
paredit inspect outline --file src/example.lisp
paredit edit wrap --file src/example.lisp --path 0.2 --diff
paredit refactor plan --symbol old-name src/example.lisp
```

Start with the [documentation source](docs/src/README.md) for command selection,
safe workflows, and integration examples. The published site is available at
<https://takeokunn.github.io/paredit-cli/>.

## Install

```sh
nix run github:takeokunn/paredit-cli -- --help    # run without installing
nix profile install github:takeokunn/paredit-cli # install via Nix
cargo install --git https://github.com/takeokunn/paredit-cli --locked
nix develop -c cargo install --path . --locked   # from a local checkout
```

Prebuilt binaries are served from the public `takeokunn-paredit-cli` Cachix
cache. The current minimum supported Rust version is `1.85`. See the
[installation guide](docs/src/installation.md) for the flake overlay and
flake-input usage.

## Development

```sh
nix develop
cargo test
nix flake check
```

Verify the declared MSRV locally before touching parser, refactor, packaging,
or public API surfaces:

```sh
cargo +1.85 test --locked
```

Pull requests run `nix flake check`.

A typed Rust library API behind the CLI is available on
[docs.rs](https://docs.rs/paredit-cli).
