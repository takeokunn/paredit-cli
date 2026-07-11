# Release Process

This document defines the minimum release discipline for `paredit-cli`.

## Release Criteria

- The branch is warning-clean under the documented development loop.
- User-visible behavior changes are recorded in [CHANGELOG.md](CHANGELOG.md).
- Compatibility-impacting changes were reviewed against
  [COMPATIBILITY.md](COMPATIBILITY.md).
- New or changed commands have CLI coverage for success and failure modes that
  matter to users and coding agents.
- README examples and policy links still match the shipped behavior.

## Automation Boundary

- The repository CI currently enforces `nix flake check`.
- Treat that automation as a baseline gate, not as complete release proof.
- Release readiness still requires explicit local verification of targeted test,
  documentation, packaging, and smoke-test steps from the checklist below.

## Pre-Release Checklist

1. Run the documented development loop:

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

2. Review the unreleased section in [CHANGELOG.md](CHANGELOG.md) and remove
   vague entries.
3. Verify the declared MSRV locally:

   ```sh
   cargo +1.85 test --locked
   ```

4. Confirm stable-surface changes are intentional and described in
   [COMPATIBILITY.md](COMPATIBILITY.md) when required.
5. Confirm package metadata and README install examples are current.
6. Confirm the release archive includes the expected policy and support
   documents that users rely on: `CHANGELOG.md`, `CODE_OF_CONDUCT.md`,
   `COMPATIBILITY.md`, `CONTRIBUTING.md`, `GOVERNANCE.md`, `LICENSE`,
   `MAINTAINERS.md`, `README.md`, `RELEASE.md`, `ROADMAP.md`,
   `SECURITY.md`, `SKILLS.md`, and `SUPPORT.md`.

## Cut and Verify

1. Create the release tag only after the checklist is complete.
2. Build the release artifact from the tagged revision.
3. Smoke-test representative commands against fixture inputs, including:
   - a read-only report command;
   - a plan/preview/verification refactor workflow; and
   - one write-capable command with explicit `--write`.
4. Verify the release notes summarize user-visible behavior instead of internal
   refactors.

## Post-Release

- Move shipped entries out of the unreleased section in
  [CHANGELOG.md](CHANGELOG.md).
- Watch incoming issue reports for regression signals in stable CLI, JSON, or
  write-path behavior.
- If a release introduces a compatibility regression, prioritize a corrective
  follow-up release over batching unrelated work.
