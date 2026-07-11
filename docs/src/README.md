# paredit-cli

`paredit-cli` is a command-line tool for inspecting, editing, and safely
refactoring Lisp source. It parses first, edits only balanced S-expression
structure or exact atom tokens, and validates the result — so symbol-oriented
rewrites never touch strings or comments.

It supports Common Lisp, Emacs Lisp, Scheme, Clojure, Janet, and Fennel
sources, and is designed for both people and AI coding agents.

## Quick start

```sh
paredit inspect check --file source.lisp
paredit edit format --file source.lisp
paredit refactor rename-symbol --file source.lisp --from old-name --to new-name
```

The CLI has exactly three namespaces:

- [`paredit inspect`](commands.md#inspect): read-only reports and analysis.
- [`paredit edit`](commands.md#edit): structural edits of a selected form,
  written to standard output.
- [`paredit refactor`](commands.md#refactor): planned semantic changes with
  preview and verification workflows — see the
  [refactor workflow](workflows.md).

There are no legacy top-level command aliases. Use the namespace paths shown
in this documentation.

## Install

```sh
nix run github:takeokunn/paredit-cli -- inspect check --file source.lisp
```

See [Installation](installation.md) for Nix profiles, the flake overlay,
Cachix binary caches, and `cargo install`. Contributors should start with
[Development](development.md).
