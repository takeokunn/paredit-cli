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
timeout 10s paredit agent-report --file target.lisp --output json
timeout 10s paredit outline --file target.lisp --output json
timeout 10s paredit find-symbol --file target.lisp --symbol old-name --output json
timeout 10s paredit rename-symbol --file target.lisp --from old-name --to new-name --plan --output json
timeout 10s paredit rename-symbols --from old-name --to new-name --output json src/*.lisp elisp/*.el
timeout 10s paredit rename-symbols --from old-name --to new-name --write src/*.lisp elisp/*.el
timeout 10s paredit extract-function --file target.lisp --path 0.3 --name helper --output json
timeout 10s paredit extract-function --file target.lisp --path 0.3 --name helper --write
timeout 10s paredit select --file target.lisp --path 0.2
timeout 10s paredit replace --file target.lisp --path 0.2 --with '(new-form ...)' > /tmp/target.lisp
timeout 10s paredit check --file /tmp/target.lisp
mv /tmp/target.lisp target.lisp
```

## Common Operations

- Detect a Lisp dialect: `paredit dialect --file target.el --output json`
- Plan a symbol rename: `paredit rename-symbol --file target.lisp --from old --to new --plan --output json`
- Rename exact atom occurrences: `paredit rename-symbol --file target.lisp --from old --to new`
- Plan a multi-file rename: `paredit rename-symbols --from old --to new --output json src/*.lisp elisp/*.el`
- Apply a multi-file rename after review: `paredit rename-symbols --from old --to new --write src/*.lisp elisp/*.el`
- Extract a selected expression into a helper: `paredit extract-function --file target.lisp --path 0.3 --name helper --output json`
- Apply the reviewed helper extraction: `paredit extract-function --file target.lisp --path 0.3 --name helper --write`
- Inspect top-level forms: `paredit outline --file target.lisp --output json`
- Build an agent planning payload: `paredit agent-report --file target.lisp --output json`
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
