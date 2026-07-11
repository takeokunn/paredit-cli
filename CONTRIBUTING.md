# Contributing

`paredit-cli` is a structural editing tool. Changes should preserve the core
contract: parse first, edit only balanced S-expression structure or exact atom
tokens, then validate the result.

## Development Loop

```sh
nix develop
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo nextest run --locked
cargo publish --dry-run --allow-dirty --locked
cargo doc --no-deps
cargo package --allow-dirty --no-verify
cargo package --allow-dirty --list
nix flake check
nix build .#
```

`nix flake check` is the automated baseline for workflow linting, formatting,
clippy, nextest, package build/tests, and publish dry-run, but it does not
replace the full release checklist in [RELEASE.md](RELEASE.md).

The declared MSRV is also part of the public contract. Before release, and when
changing parser, refactor, packaging, or public API surfaces, verify it
explicitly with:

```sh
cargo +1.85 test --locked
```

## Change Guidelines

- Keep parser and edit primitives typed with newtypes for offsets, spans, paths,
  ids, and symbol names.
- Do not add text-based Lisp rewrites that can touch strings or comments.
- Add CLI tests for every user-visible command or flag.
- Prefer explicit file lists over implicit project traversal for write
  operations.
- Keep JSON output stable and machine-readable for AI coding agents.

## Project Policies

- Follow [GOVERNANCE.md](GOVERNANCE.md) when proposing scope, policy, or
  maintainer-process changes.
- Follow [RELEASE.md](RELEASE.md) when preparing or reviewing a release.
- Follow [COMPATIBILITY.md](COMPATIBILITY.md) when changing CLI behavior, JSON
  output, or `--write` semantics.
- Follow [MAINTAINERS.md](MAINTAINERS.md) for triage and response expectations
  when proposing process changes.
- Follow [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) in issues, reviews, and other
  project discussions.
- Follow [SECURITY.md](SECURITY.md) when handling vulnerability reports or
  security-sensitive bug fixes.
- Send users to [SUPPORT.md](SUPPORT.md) for usage questions, issue reporting,
  and reproduction expectations.
- Use [ROADMAP.md](ROADMAP.md) to check whether a proposed feature or refactor
  aligns with current project priorities.
- Record user-visible behavior changes in [CHANGELOG.md](CHANGELOG.md).

## Release Checklist

- Follow the release procedure in [RELEASE.md](RELEASE.md).
- Verify `Cargo.toml` metadata and README examples.
- Review [COMPATIBILITY.md](COMPATIBILITY.md) for any stable-surface changes.
- Run the full development loop.
- Confirm generated archives include `CHANGELOG.md`, `CODE_OF_CONDUCT.md`,
  `COMPATIBILITY.md`, `CONTRIBUTING.md`, `GOVERNANCE.md`, `LICENSE`,
  `MAINTAINERS.md`, `README.md`, `RELEASE.md`, `ROADMAP.md`, `SECURITY.md`,
  `SKILLS.md`, and `SUPPORT.md`.
