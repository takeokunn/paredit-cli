# paredit-cli

`paredit` is a structure-aware CLI for inspecting and safely refactoring Lisp
S-expressions. Its canonical command layout is designed for both people and AI
coding agents:

- `paredit inspect ...` reads source and produces reports.
- `paredit edit ...` transforms one selected form and prints source to stdout.
- `paredit refactor ...` plans, previews, verifies, and applies semantic changes.

## Commands

```sh
paredit inspect ...
paredit edit ...
paredit refactor ...
```

## Quick Start

```sh
paredit inspect check --file src/example.lisp
paredit inspect outline --file src/example.lisp
paredit refactor plan --symbol old-name src/example.lisp
```

Start with the [documentation source](docs/src/README.md) for command selection,
safe workflows, and integration examples. The published site is available at
<https://takeokunn.github.io/paredit-cli/>.

## Project Policy

Security-sensitive reports and the currently supported release line are defined
in [SECURITY.md](SECURITY.md). `main` is the active development line. See
[CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) for collaboration and community
expectations. Contributions and releases follow
[CONTRIBUTING.md](CONTRIBUTING.md), [RELEASE.md](RELEASE.md), and
[COMPATIBILITY.md](COMPATIBILITY.md).

## Install

```sh
nix develop -c cargo install --path . --locked
cargo install --git https://github.com/takeokunn/paredit-cli --locked
```

The current minimum supported Rust version is `1.85`.

## Development

```sh
nix develop
cargo +1.85 test --locked
nix flake check
```

Pull requests run `nix flake check`.

A typed Rust library API behind the CLI is available on
[docs.rs](https://docs.rs/paredit-cli).
