# Contributing

`paredit-cli` is a structural editing tool. Changes should preserve the core
contract: parse first, edit only balanced S-expression structure or exact atom
tokens, then validate the result.

## Development Loop

```sh
nix develop
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test
nix flake check
nix build .#
```

## Change Guidelines

- Keep parser and edit primitives typed with newtypes for offsets, spans, paths,
  ids, and symbol names.
- Do not add text-based Lisp rewrites that can touch strings or comments.
- Add CLI tests for every user-visible command or flag.
- Prefer explicit file lists over implicit project traversal for write
  operations.
- Keep JSON output stable and machine-readable for AI coding agents.

## Release Checklist

- Verify `Cargo.toml` metadata and README examples.
- Run the full development loop.
- Confirm generated archives include `LICENSE`, `README.md`, and `SKILLS.md`.
