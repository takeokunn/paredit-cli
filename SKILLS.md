# SKILLS: S-expression Refactoring With paredit-cli

Use this skill when refactoring Common Lisp, Scheme, Clojure, Emacs Lisp, or any Lisp-like S-expression file.

## Safety Policy

- Never hand-edit balanced delimiters during large refactors.
- Always validate with `paredit check` before and after edits.
- Prefer `--path` for deterministic scripted edits.
- Prefer `--at` when an error report or grep result gives a byte offset.
- Keep data and logic changes separate: structure-edit first, semantic rewrite second.
- Use explicit command timeouts when running project tests.

## Refactoring Loop

```sh
timeout 10s paredit check --file target.lisp
timeout 10s paredit select --file target.lisp --path 0.2
timeout 10s paredit replace --file target.lisp --path 0.2 --with '(new-form ...)' > /tmp/target.lisp
timeout 10s paredit check --file /tmp/target.lisp
mv /tmp/target.lisp target.lisp
```

## Common Operations

- Rename a function symbol: `paredit replace --path 0.1 --with new-name`
- Wrap an argument list: `paredit wrap --path 0.2`
- Inline a nested list: `paredit splice --path 0.3`
- Promote a child expression: `paredit raise --path 0.3.1`
- Move a following sibling into a list: `paredit slurp-forward --path 0`
- Move the last list child out: `paredit barf-forward --path 0`

## 2026 Common Lisp Refactoring Checklist

- Push reusable syntax into `defmacro` only when it reduces duplicated structure.
- Keep Prolog-style declarative facts separate from imperative execution logic.
- Prefer CPS only where it makes control flow explicit and testable.
- Delete dead code instead of maintaining backward compatibility shims.
- Split large files by coherent data, macro, logic, and test boundaries.
- Raise test abstraction while preserving concrete behavior coverage.
- Treat human readability as a verification target, not a post-processing step.
