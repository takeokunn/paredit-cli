# Command model

Every command belongs to one of three namespaces. This gives automation a
stable first decision: inspect, edit, or refactor.

- `paredit inspect` reads and reports without writing.
- `paredit edit` transforms one selected form and writes source to stdout.
- `paredit refactor` plans, previews, verifies, and applies semantic changes.

There are no top-level command aliases outside these namespaces. Run
`paredit <namespace> --help` for the authoritative list on your installed
version, and `paredit <namespace> <command> --help` for each command's
contract, arguments, and output formats.

## Inspect

`paredit inspect` never writes source files. Prefer these commands for
discovery, impact analysis, and preflight checks.

| Command | Purpose |
| --- | --- |
| `check` | Validate that input is a balanced S-expression document. |
| `dialect` | Detect Lisp dialect from `--file` extension or explicit `--dialect`. |
| `stats` | Print parse, dialect, and structural metrics for agent planning. |
| `agent-report` | Print a complete JSON report for AI coding agent refactor planning. |
| `outline` | Print top-level forms with paths, spans, and definition hints. |
| `form` | Report one selected form with local structure for refactor planning. |
| `find-symbol` | Find exact atom occurrences without touching strings or comments. |
| `symbols` | Report exact atom occurrences across explicit files for rename planning. |
| `calls` | Report list-head call sites across explicit files for arity refactor planning. |
| `signature` | Compare callable definitions and call-site arity across explicit files. |
| `call-graph` | Report internal and optional external call graph edges. |
| `impact` | Report refactoring impact risks for one symbol across explicit files. |
| `workspace` | Discover Lisp sources under roots and report parse/refactor inventory. |
| `dependencies` | Report package, system, load, and qualified-symbol dependencies. |
| `packages` | Report Common Lisp package declarations across explicit files. |
| `definitions` | Report definition-like top-level forms across explicit files. |
| `unused-definitions` | Report definitions with no external exact atom references. |
| `duplicates` | Report repeated structural S-expression shapes across explicit files. |
| `similarity` | Report structurally similar S-expression forms across explicit files. |
| `lets` | Report local let bindings and inline safety for refactor planning. |

Most reports accept `--output json` for machine-readable results.

## Edit

`paredit edit` makes one structural transformation and writes the resulting
source to standard output. Files are never modified in place; review the
output before redirecting it.

| Command | Purpose |
| --- | --- |
| `format` | Print a canonical, indentation-based rendering. |
| `select` | Print the S-expression selected by `--path` or `--at`. |
| `replace` | Replace the selected S-expression with replacement text. |
| `kill` | Remove the selected S-expression. |
| `wrap` | Wrap the selected S-expression in a new list. |
| `splice` | Remove one list pair while keeping its children. |
| `raise` | Replace the selected expression's parent list with the selection. |
| `transpose-forward` | Exchange the selected expression with its next sibling while keeping trivia in place. |
| `transpose-backward` | Exchange the selected expression with its previous sibling while keeping trivia in place. |
| `slurp-forward` | Pull the next sibling into the selected list. |
| `slurp-backward` | Pull the previous sibling into the selected list. |
| `barf-forward` | Push the last child out of the selected list. |
| `barf-backward` | Push the first child out of the selected list. |

For example:

```sh
paredit edit wrap --file source.lisp --path 0.1
```

## Refactor

`paredit refactor` contains the reviewable workflow commands and the semantic
refactorings they gate. See [Refactor workflow](workflows.md) for the
plan/preview/verify/apply lifecycle.

### Workflow commands

| Command | Purpose |
| --- | --- |
| `plan` | Produce an ordered, gated refactoring plan for AI coding agents. |
| `verify` | Verify pre/post refactoring invariants for agents and CI gates. |
| `preview` | Preview exact refactoring rewrites without modifying files. |
| `check` | Validate a refactor preview manifest without writing files. |
| `status` | Summarize a preview manifest into agent-safe next actions. |
| `apply` | Apply a previously generated preview manifest with hash guards. |
| `diff` | Render a verified diff from a preview manifest without writing files. |
| `workspace-plan` | Discover Lisp sources under roots and build a gated refactor plan. |
| `workspace-preview` | Discover sources and preview exact refactoring rewrites. |
| `workspace-execute` | Execute a workspace refactor with preview gates and post-write verification. |

### Definition and file layout

| Command | Purpose |
| --- | --- |
| `remove-definition` | Plan or remove a top-level definition from one file. |
| `remove-unused-definitions` | Plan or remove unused top-level definitions across files. |
| `move-definition` | Plan or move a top-level definition between files. |
| `split-file` | Plan or split multiple top-level definitions into another file. |
| `sort-definitions` | Plan or sort contiguous top-level definition blocks in one file. |
| `move-form` | Plan or move any top-level form between files. |
| `replacement-plan` | Convert duplicate groups into reviewed replace-forms batches. |
| `replace-forms` | Plan or replace multiple reviewed forms in one file. |

### Packages

| Command | Purpose |
| --- | --- |
| `add-export` | Plan or add a symbol to a Common Lisp `defpackage` `:export` option. |
| `sort-package-exports` | Plan or sort `defpackage` `:export` symbol designators. |
| `sort-package-options` | Plan or sort `defpackage` option forms. |
| `merge-package-options` | Plan or merge duplicate `defpackage` option forms. |
| `rename-package` | Plan or rename package designators and qualified prefixes. |

### Renames

| Command | Purpose |
| --- | --- |
| `rename-symbol` | Rename exact atom occurrences without touching strings or comments. |
| `rename-in-form` | Rename exact atom occurrences inside one selected form. |
| `rename-binding` | Rename one local binding and only the references in its lexical scope. |
| `rename-symbols` | Plan or apply an exact atom rename across explicit files. |
| `rename-function` | Plan or apply a Common Lisp callable definition and designator rename. |
| `rename-macrolet` | Plan or apply a `macrolet`/`compiler-macrolet` binding and call-site rename. |
| `rename-symbol-macro` | Plan or apply a `define-symbol-macro` binding and value-reference rename. |
| `rename-local-function` | Plan or apply a `flet`/`labels` local function binding and call-site rename. |

### Calls and functions

| Command | Purpose |
| --- | --- |
| `replace-function-calls` | Plan or replace callable call-site heads across explicit files. |
| `wrap-function-calls` | Plan or wrap callable call sites in another function or macro call. |
| `unwrap-function-calls` | Plan or remove a unary wrapper around callable call sites. |
| `unwrap-call` | Replace one selected wrapper call with one selected argument. |
| `thread-expression` | Convert a nested call chain into a thread-first or thread-last pipeline. |
| `unthread-expression` | Convert a threading pipeline back into nested calls. |
| `extract-function` | Extract the selected expression into a top-level function with inferred parameters. |
| `extract-local-function` | Extract the selected expression into a Common Lisp `flet` or `labels` binding. |
| `extract-constant` | Extract the selected expression into a top-level constant. |
| `inline-function` | Inline one selected function call using a selected function definition. |
| `inline-lambda` | Replace a safe, immediately invoked Common Lisp lambda with a parallel `let`. |
| `inline-local-function` | Inline the sole direct call in a safe, single-binding Common Lisp `flet` form. |

### Parameters and bindings

| Command | Purpose |
| --- | --- |
| `add-function-parameter` | Add a parameter to a selected function and explicit call sites. |
| `move-function-parameter` | Move one positional parameter in a function and its call sites. |
| `swap-function-parameters` | Swap two positional parameters in a function and its call sites. |
| `reorder-function-parameters` | Reorder all positional parameters in a function and its call sites. |
| `remove-function-parameter` | Remove one positional parameter from a function and its call sites. |
| `introduce-let` | Replace the selected expression with a local binding in the enclosing list. |
| `inline-let` | Inline a single local let binding into its body. |
| `convert-let-star-to-let` | Convert a Common Lisp `let*` to `let` when later initializers do not reference earlier bindings. |
| `convert-if-to-cond` | Convert a Common Lisp or Emacs Lisp `(if test then [else])` form to `cond`. |
| `remove-unused-binding` | Plan or remove one unused local let binding. |
