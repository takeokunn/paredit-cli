## Summary

- describe the change
- describe the user-visible effect

## Verification

- [ ] `cargo fmt --all`
- [ ] `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] `cargo test`
- [ ] `cargo nextest run --locked`
- [ ] `cargo publish --dry-run --allow-dirty --locked`
- [ ] `cargo doc --no-deps`
- [ ] `cargo package --allow-dirty --no-verify`
- [ ] `cargo package --allow-dirty --list`
- [ ] `nix flake check`
- [ ] `nix build .#`

## Policy Review

- [ ] `COMPATIBILITY.md` reviewed for CLI, JSON, or `--write` contract changes
- [ ] `CHANGELOG.md` updated for user-visible behavior changes
- [ ] `ROADMAP.md` reviewed for new feature or scope expansion
- [ ] `SECURITY.md` reviewed if the change affects file access, execution paths, or trust boundaries
- [ ] `RELEASE.md` reviewed if this PR changes release procedure or packaging expectations

## Notes for Reviewers

- risk areas:
- follow-up work:
