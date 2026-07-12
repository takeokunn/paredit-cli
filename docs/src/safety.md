# Safety reference

`paredit-cli` keeps inspection, source edits, and semantic refactorings separate so automated clients can choose an appropriate review path.

## Inspect is read-only

All `paredit inspect` commands report information without modifying source files. Prefer these commands for discovery, impact analysis, and preflight checks.

## Edit previews before it writes

`paredit edit` commands return transformed source on standard output by default and never touch the file. Preview the change as a diff, then apply it in place with `--write`:

```sh
paredit edit format --file source.lisp --diff
paredit edit format --file source.lisp --write
```

`--write` refuses to persist a result that no longer parses, and writes are staged with automatic rollback, so a failed write cannot leave a truncated or unbalanced file behind.

## Refactor is explicit

Use `paredit refactor plan`, `paredit refactor preview`, and `paredit refactor verify` before `paredit refactor apply` when the workflow is available. These commands make planned changes and verification results visible before a write is requested.

## Workspace scope

For workspace operations, start with `paredit inspect workspace` to identify the affected files. Use the workspace planning and preview commands before `paredit refactor workspace-execute`.

## Automation guidance

1. Discover with `paredit inspect`.
2. Review an `edit` result (`--diff` or stdout) before passing `--write`.
3. Plan, preview, and verify a `refactor` before applying it.
4. Treat non-zero exits and validation failures as blockers.

See the [agent interface](agents.md) for exit codes, the JSON output
contract, and a complete safe editing loop.
