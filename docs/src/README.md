# paredit-cli

`paredit-cli` is a command-line tool for inspecting, editing, and safely refactoring Lisp source.

## Quick start

```sh
paredit inspect check --file source.lisp
paredit edit format --file source.lisp
paredit refactor rename-symbol --file source.lisp --from old-name --to new-name
```

The CLI has exactly three namespaces:

- `paredit inspect`: read-only reports and analysis.
- `paredit edit`: structural edits of a selected form, written to standard output.
- `paredit refactor`: planned semantic changes with preview and verification workflows.

There are no legacy top-level command aliases. Use the namespace paths shown in this documentation.

## Install

```sh
nix run github:takeokunn/paredit-cli -- inspect check --file source.lisp
```

For local development, use `nix develop`.
