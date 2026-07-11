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

## Notes for Reviewers

- risk areas:
- follow-up work:
