# Releases and compatibility

The command reference in this documentation describes the current release.
Automation should pin the `paredit-cli` version it validates and run
`paredit inspect capabilities --output json` during upgrades.

## Machine-readable output

Use `--output json` whenever a command offers it. JSON reports use a top-level
`schema_version`: fields may be added within a version, while field removals or
renames require a version bump. Consumers should reject unsupported schema
versions and tolerate unknown fields in supported versions.

Human-readable text output is intentionally not a machine contract and may
change between releases. `inspect outline` returns a bare JSON array and is the
documented exception to the top-level-object convention.

## Command changes

Command paths, flags, defaults, and output schemas can change between releases.
Release notes must identify changes that affect automation, and integrations
must be validated against the command reference for the target version.

Maintainers follow
[RELEASING.md](https://github.com/takeokunn/paredit-cli/blob/main/RELEASING.md)
to verify the package, documentation, and Nix checks before publication.
