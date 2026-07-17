# Contributing to paredit-cli

Thanks for improving `paredit-cli`. The project accepts bug reports,
documentation corrections, tests, and focused implementation changes.

## Before opening an issue

- Search existing issues for the same behavior.
- Reduce bugs to a minimal, balanced S-expression and include the command,
  expected result, actual result, and `paredit --version` output.
- Do not report security-sensitive behavior in a public issue. Follow
  [SECURITY.md](SECURITY.md) instead.

## Development environment

The committed Nix flake provides the complete development toolchain:

```sh
nix develop
cargo test --locked
nix flake check
```

`nix flake check` is the required verification gate. It checks formatting,
GitHub Actions syntax, Clippy, the test suite, package construction, rendered
documentation, the exact MSRV build/test, and the lint/format integration
paths. See the full
[development guide](docs/src/development.md) for the local development loop
and MSRV verification.

## Pull requests

- Keep each pull request focused on one user-visible problem.
- Add or update tests for behavior changes, including failure and safety paths.
- Update the command reference and workflow documentation when CLI behavior,
  JSON output, or Nix integration changes.
- Prefer preview and verification flows in examples; destructive file updates
  must require an explicit `--write` or apply action.
- Explain the user impact and the verification commands you ran in the pull
  request description.

## Design expectations

The CLI is structure-aware rather than text-replacement based. Changes must
preserve balanced S-expression syntax, avoid rewriting strings and comments
unless the command explicitly targets them, and retain the read/preview/write
separation documented in the project guides.
