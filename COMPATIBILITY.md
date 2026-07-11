# Command Evolution

`paredit-cli` does not provide backward compatibility for command paths, flags,
text output, or JSON schemas. Automation must use the command reference in the
current documentation release.

The supported command layout has exactly three top-level namespaces:

- `paredit inspect ...`
- `paredit edit ...`
- `paredit refactor ...`

When these interfaces change, update the command reference, agent skills, and
the release notes in the same change. Record the user-visible effect in [CHANGELOG.md](CHANGELOG.md).

Changes that raise the MSRV must update `Cargo.toml`, `README.md`, and the
release notes together.
