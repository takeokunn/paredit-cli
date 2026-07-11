# Command model

Every command belongs to one of three namespaces. This gives automation a stable first decision: inspect, edit, or refactor.

## Inspect

`paredit inspect` never writes source files. Its subcommands are `check`, `dialect`, `stats`, `agent-report`, `outline`, `form`, `find-symbol`, `symbols`, `calls`, `signature`, `call-graph`, `impact`, `workspace`, `dependencies`, `packages`, `definitions`, `unused-definitions`, `duplicates`, `similarity`, and `lets`.

Use `paredit inspect <command> --help` to see the arguments and output formats for a report.

## Edit

`paredit edit` makes one structural transformation and writes the resulting source to standard output. Its subcommands are `format`, `select`, `replace`, `kill`, `wrap`, `splice`, `raise`, `slurp-forward`, `slurp-backward`, `barf-forward`, and `barf-backward`.

For example:

```sh
paredit edit wrap --file source.lisp --start 0 --end 5 --wrapper list
```

## Refactor

`paredit refactor` contains the reviewable workflow commands `plan`, `verify`, `preview`, `check`, `status`, `apply`, `diff`, `workspace-plan`, `workspace-preview`, and `workspace-execute`, plus semantic refactorings.

Semantic refactorings include definition movement and removal, package edits, rename operations, function-call rewrites, function and constant extraction, function inlining, function-parameter changes, and `let` binding transformations. Run `paredit refactor --help` for the complete command list, then `paredit refactor <command> --help` for its contract.

Before applying a change, use `plan`, `preview`, and `verify` when the refactoring supports them.
