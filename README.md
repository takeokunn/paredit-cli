# paredit-cli

`paredit-cli` is a Rust command line tool for structure editing S-expressions.
It is designed for AI coding agents that often break Lisp parentheses during refactors.

The core rule is: do not rewrite parentheses manually. Select an expression by tree path or byte offset, then ask `paredit` to perform a balanced edit.

## Commands

```sh
paredit check < file.lisp
paredit format --indent 2 < file.lisp
paredit select --path 0.2 < file.lisp
paredit replace --path 0.1 --with new-name < file.lisp
paredit wrap --path 0.2 < file.lisp
paredit splice --path 0.2 < file.lisp
paredit raise --path 0.2.1 < file.lisp
paredit slurp-forward --path 0 < file.lisp
paredit slurp-backward --path 1 < file.lisp
paredit barf-forward --path 0 < file.lisp
paredit barf-backward --path 0 < file.lisp
paredit kill --path 0.3 < file.lisp
```

Paths are zero-based child indexes from the virtual document root.
For example, in `(defun add (x y) (+ x y))`, path `0.2` selects `(x y)`.

## AI Agent Workflow

1. Run `paredit check` before editing.
2. Use `paredit select --path ...` or `paredit select --at ...` to confirm the exact target.
3. Pipe the source through one edit command.
4. Run `paredit check` again after writing the result.
5. Run the project test suite with explicit timeouts.

## Development

```sh
nix develop
cargo fmt
cargo test
nix flake check
```

## Scope

The first implementation focuses on deterministic S-expression structure edits.
It preserves byte ranges for targeted edits and provides canonical formatting for generated or heavily rewritten code.
