# Safety reference

`paredit-cli` keeps inspection, source edits, and semantic refactorings separate so automated clients can choose an appropriate review path.

## Inspect is read-only

All `paredit inspect` commands report information without modifying source files. Prefer these commands for discovery, impact analysis, and preflight checks.

## Edit writes to standard output

`paredit edit` commands return transformed source on standard output. Redirect the output only after reviewing it:

```sh
paredit edit format --file source.lisp > source.lisp.new
```

## Refactor is explicit

Use `paredit refactor plan`, `paredit refactor preview`, and `paredit refactor verify` before `paredit refactor apply` when the workflow is available. These commands make planned changes and verification results visible before a write is requested.

## Workspace scope

For workspace operations, start with `paredit inspect workspace` to identify the affected files. Use the workspace planning and preview commands before `paredit refactor workspace-execute`.

## Automation guidance

1. Discover with `paredit inspect`.
2. Review an `edit` result before redirecting it to a file.
3. Plan, preview, and verify a `refactor` before applying it.
4. Treat non-zero exits and validation failures as blockers.
