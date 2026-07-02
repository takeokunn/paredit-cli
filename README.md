# paredit-cli

`paredit-cli` is a Rust command line tool for safe S-expression refactoring.
It gives AI coding agents deterministic tree paths, byte spans, dialect hints,
and balanced structural edits so Lisp refactors do not devolve into manual
parenthesis surgery.

The core rule is: do not rewrite delimiters by hand. Validate the file, locate
the exact form or symbol, apply a structural edit, then validate again.

## What Agents Get

- Extension-based Lisp dialect detection for Common Lisp, Emacs Lisp, Scheme,
  Clojure, Janet, and Fennel.
- Stable zero-based expression paths such as `0.2.1` from the virtual document
  root.
- Byte spans for every top-level form and atom occurrence.
- Exact atom search and rename that ignore comments and string contents.
- Multi-file exact atom rename plans with explicit `--write` application.
- JSON reports designed for coding-agent planning and verification loops.
- Balanced edits: replace, kill, wrap, splice, raise, slurp, and barf.
- A typed Rust library API behind the CLI for downstream automation.

## Commands

```sh
paredit check --file file.lisp
paredit dialect --file init.el
paredit stats --file system.asd --output json
paredit agent-report --file source.lisp --output json
paredit outline --file source.lisp --output json
paredit find-symbol --file source.lisp --symbol old-name --output json
paredit rename-symbol --file source.lisp --from old-name --to new-name --plan --output json
paredit rename-symbol --file source.lisp --from old-name --to new-name
paredit rename-symbols --from old-name --to new-name src/*.lisp lisp/*.el
paredit rename-symbols --from old-name --to new-name --write src/*.lisp lisp/*.el
paredit format --file source.lisp --indent 2
paredit select --file source.lisp --path 0.2
paredit select --file source.lisp --at 42
paredit replace --file source.lisp --path 0.1 --with new-name
paredit wrap --file source.lisp --path 0.2
paredit splice --file source.lisp --path 0.2
paredit raise --file source.lisp --path 0.2.1
paredit slurp-forward --file source.lisp --path 0
paredit slurp-backward --file source.lisp --path 1
paredit barf-forward --file source.lisp --path 0
paredit barf-backward --file source.lisp --path 0
paredit kill --file source.lisp --path 0.3
```

All commands that accept `--file` read stdin when it is omitted.

## Dialect Detection

| Dialect | Extensions |
| --- | --- |
| Common Lisp | `lisp`, `lsp`, `cl`, `asd` |
| Emacs Lisp | `el` |
| Scheme | `scm`, `ss`, `sld` |
| Clojure | `clj`, `cljs`, `cljc`, `edn` |
| Janet | `janet` |
| Fennel | `fnl` |

Use `--dialect` to override extension detection when stdin or generated files
do not carry a useful filename.

## Agent Refactoring Workflow

1. Run `paredit check --file target.lisp`.
2. Run `paredit agent-report --file target.lisp --output json` and cache the
   top-level form paths and spans.
3. Use `paredit outline --output json` to identify definition-like forms such as
   `defun`, `defmacro`, `defclass`, `defpackage`, `asdf:defsystem`, and
   Emacs Lisp `defcustom` or `define-minor-mode`.
4. Use `paredit find-symbol --symbol name --output json` before any rename.
5. Use `paredit rename-symbol --plan --output json` for one file or
   `paredit rename-symbols --output json` for an explicit file set to review
   exact atom occurrences.
6. Apply a project-wide exact atom rename only with
   `paredit rename-symbols --write`; the command re-parses every rewritten file
   before saving.
7. Use structural edits for form movement: `wrap`, `splice`, `raise`,
   `slurp-*`, and `barf-*`.
8. Run `paredit check` again, then run the project test suite.

This workflow is intended for large Common Lisp and Emacs Lisp refactors where
the safe primitive operations are: discover definitions, isolate forms, rename
symbols exactly, move balanced forms, and verify after every generated change.

## Examples

Detect an Emacs Lisp file:

```sh
paredit dialect --file init.el --output json
```

Find all exact uses of a Common Lisp symbol without matching strings or
comments:

```sh
paredit find-symbol --file src/core.lisp --symbol make-session --output json
```

Plan a rename before applying it:

```sh
paredit rename-symbol \
  --file src/core.lisp \
  --from old-session-name \
  --to session-name \
  --plan \
  --output json
```

Apply the rename into a temporary file and re-check it:

```sh
paredit rename-symbol \
  --file src/core.lisp \
  --from old-session-name \
  --to session-name > /tmp/core.lisp
paredit check --file /tmp/core.lisp
```

Plan and then apply an exact atom rename across a Common Lisp or Emacs Lisp
file set:

```sh
paredit rename-symbols \
  --from old-session-name \
  --to session-name \
  --output json \
  src/*.lisp elisp/*.el
paredit rename-symbols \
  --from old-session-name \
  --to session-name \
  --write \
  src/*.lisp elisp/*.el
```

## Rust Quality Bar

- Rust edition 2024 with a minimum supported Rust version in `Cargo.toml`.
- `unsafe_code = "forbid"`.
- Newtypes for byte offsets, byte spans, expression paths, node ids, child
  indexes, and symbol names.
- `thiserror` for parse errors and `anyhow` for CLI boundary errors.
- Warning-clean `cargo clippy --all-targets -- -D warnings`.
- Nix flake verification for reproducible development.

## Development

```sh
nix develop
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test
nix flake check
nix build .#
```

## Scope

`paredit-cli` is a structural S-expression tool, not a Lisp evaluator or full
reader implementation. It preserves balanced list, vector, and map delimiters;
tracks comments and strings safely for symbol operations; and provides
dialect-aware definition hints. It does not macroexpand code or update ASDF,
package, autoload, or module manifests automatically.
