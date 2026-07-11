# Compatibility Policy

`paredit-cli` is a command-line tool for coding agents and automation, so
compatibility is defined around reproducible machine use, not around terminal
copy or incidental text layout.

## Release Stages

- Unreleased `main` is the active development line. Until a tag is cut, CLI
  behavior, JSON fields, and refactor workflows on `main` may change as
  correctness gaps and architecture boundaries are tightened.
- Once a release is tagged, the documented stable surfaces below become public
  contracts for that released line unless a security or correctness fix
  requires a documented break.
- Older released lines are unsupported unless maintainers explicitly publish a
  wider support window in [SECURITY.md](SECURITY.md).

## Stability Contract

The following surfaces are intended to remain stable for a released line unless
a security fix or correctness bug requires a documented break:

- Command names, flag names, and exit-code meaning for released commands.
- JSON field names and value semantics for documented `--output json` modes.
- Write behavior that is explicitly guarded by `--write`.
- Zero-based path conventions, byte-span semantics, and dialect labels emitted
  by reports and refactor plans.
- The rule that symbol-oriented rewrites must not touch comments or strings.
- Common Lisp scope-aware refactors must preserve callable and macro binding
  boundaries, including local `macrolet`, `compiler-macrolet`, and
  `symbol-macrolet` forms; `defmacro` and `define-compiler-macro`
  definitions remain traversable inside reader-quoted lambda bodies, while
  expander bodies are treated as separate reviewable scopes rather than
  generic traversal targets.
- The Common Lisp support matrix documented in [README.md](README.md) is part
  of the released contract: `rename-function`, `rename-local-function`,
  `rename-macrolet`, and `rename-symbol-macro` must preserve the same scope
  boundaries and callable namespaces described there.

Breaking changes to these surfaces must be called out in
[CHANGELOG.md](CHANGELOG.md).

## What May Change Without Notice

The following are not stable APIs:

- Human-oriented prose, spacing, or column layout in text output.
- Ordering of advisory messages when the JSON schema already carries the same
  information.
- Experimental commands, flags, or JSON fields that are documented as
  preview, internal, or unstable.
- Performance characteristics that are not explicitly promised for a release.
- Rejection of previously accepted invalid inputs when the old behavior was a
  bug.

## Versioning and Support Window

- Released tags follow Semantic Versioning.
- Before `1.0`, breaking changes may still happen between release lines, but
  any intentional break to a documented surface must be called out in
  [CHANGELOG.md](CHANGELOG.md).
- Security and correctness fixes are applied on the active development line.
- The minimum supported Rust version is the value declared in `Cargo.toml`.
- Changes that raise the MSRV must update `Cargo.toml`, `README.md`, and the
  release notes together.
- Unreleased `main` behavior may change until it is cut into a release and
  recorded in [CHANGELOG.md](CHANGELOG.md).

## Change Discipline

When changing a stable surface:

- Update the relevant README examples and command descriptions.
- Add or update CLI tests that prove the intended behavior.
- Record the user-visible effect in [CHANGELOG.md](CHANGELOG.md).
- Prefer additive changes over silent behavior drift.

## Scope Boundary

`paredit-cli` does not promise compatibility for:

- Internal Rust module structure.
- Private helper types not exposed through the CLI or library API.
- Unreleased manifests, temporary fixtures, or ad hoc maintainer scripts.
