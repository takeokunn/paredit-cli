# paredit-cli

[![CI](https://github.com/takeokunn/paredit-cli/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/takeokunn/paredit-cli/actions/workflows/ci.yml)
![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)

`paredit-cli` is a Rust command line tool for safe S-expression refactoring.
It gives AI coding agents deterministic tree paths, byte spans, dialect hints,
and balanced structural edits so Lisp refactors do not devolve into manual
parenthesis surgery.

The core rule is: do not rewrite delimiters by hand. Validate the file, locate
the exact form or symbol, apply a structural edit, then validate again.

## What Agents Get

- Extension-based Lisp dialect detection for Common Lisp, Emacs Lisp, Scheme,
  Clojure, Janet, and Fennel.
- Directory-root workspace discovery that turns `.lisp`, `.asd`, `.el`,
  `.scm`, `.clj`, `.janet`, and `.fnl` files into parse/refactor inventories
  while skipping generated trees and symlinks by default.
- Stable zero-based expression paths such as `0.2.1` from the virtual document
  root.
- Byte spans for every top-level form and atom occurrence.
- Exact atom search and rename that ignore comments and string contents.
- Common Lisp package declaration reports for `defpackage` and `in-package`
  planning.
- Common Lisp package rename plans that update package designators and
  qualified prefixes without touching comments, strings, or ordinary
  same-named atoms.
- Multi-file definition inventories with names, categories, spans, package
  context, and arity hints for decomposition and consolidation planning.
- Multi-file call-site inventories with list-head names, argument counts,
  spans, dialects, and enclosing definition context for arity refactors.
- Multi-file signature compatibility reports that compare callable definition
  arity with call-site argument counts before required-parameter changes.
- Agent-oriented refactor plans that combine impact gates, dependency checks,
  safe-to-automate decisions, and ordered command recommendations.
- Pre/post refactor verification gates that emit fixed JSON checks for AI
  coding agents and CI pipelines.
- Saved-manifest validation that checks refactor policy, content hashes,
  rewritten parse status, manifest consistency, and workspace root containment
  without writing files or rendering diffs.
- Top-level definition movement between files with plan-first JSON, dialect
  detection, missing-destination support, and reparse-before-write safety.
- Duplicate-shape replacement plans that turn repeated forms into per-file
  `replace-forms` batches for agent review.
- Multi-file exact atom rename plans with explicit `--write` application.
- Scoped exact atom rename inside one selected form for function-local or
  `let`-local refactors.
- Dialect-aware function extraction for turning a selected expression into a
  top-level helper definition.
- Dialect-aware local binding introduction for naming subexpressions without
  manual parenthesis surgery.
- Dialect-aware `let` reports with binding paths, value spans, reference
  counts, and inline risk flags.
- Plan-first unused local binding removal with reference checks, bulk
  zero-reference cleanup, and explicit value-drop approval for writes.
- Round-trip thread pipeline transforms that convert reviewed nested calls
  into `->`/`->>` forms and back without manual delimiter edits.
- JSON reports designed for coding-agent planning and verification loops.
- Balanced edits: replace, kill, wrap, splice, raise, slurp, and barf.
- A typed Rust library API behind the CLI for downstream automation.
- DDD-oriented crate layout that separates typed Lisp-domain rules from CLI
  delivery concerns.

## Commands

```sh
paredit check --file file.lisp
paredit dialect --file init.el
paredit stats --file system.asd --output json
paredit agent-report --file source.lisp --output json
paredit workspace-report --output json .
paredit workspace-refactor-plan --symbol render-pane --operation rename --fail-on-blocking-gate --require-definitions 1 --require-references 1 --output json .
paredit outline --file source.lisp --output json
paredit form-report --file source.lisp --path 0 --include-source --output json
paredit find-symbol --file source.lisp --symbol old-name --output json
paredit symbol-report --symbol old-name --output json src/*.lisp lisp/*.el
paredit call-report --symbol render-pane --output json src/*.lisp lisp/*.el
paredit signature-report --symbol render-pane --fail-on-mismatch --require-definitions 1 --require-calls 1 --output json src/*.lisp lisp/*.el
paredit call-graph --symbol render-pane --fail-on-inbound-callers --require-edges 1 --require-internal-edges 1 --output json src/*.lisp lisp/*.el
paredit impact-report --symbol render-pane --fail-on-risk-level warning --require-definitions 1 --require-references 1 --require-calls 1 --output json src/*.lisp lisp/*.el
paredit refactor-plan --symbol render-pane --operation rename --fail-on-blocking-gate --require-definitions 1 --require-references 1 --output json src/*.lisp lisp/*.el
paredit workspace-refactor-preview --from render-pane --to paint-pane --mode function --fail-on-no-change --fail-on-parse-error --fail-on-target-conflict --require-changed-files 1 --require-definitions 1 --require-edits 1 --output json .
paredit refactor-preview --from render-pane --to paint-pane --mode function --fail-on-no-change --fail-on-parse-error --fail-on-target-conflict --require-definitions 1 --require-edits 1 --output json src/*.lisp lisp/*.el
paredit refactor-check --manifest rename.preview.json --root . --output json
paredit refactor-status --manifest rename.preview.json --root . --output json
HASH=<manifest.hash from refactor-status JSON>
paredit refactor-diff --manifest rename.preview.json --expect-manifest-hash "$HASH" --root . --output json
paredit refactor-apply --manifest rename.preview.json --expect-manifest-hash "$HASH" --root . --output json
paredit refactor-apply --manifest rename.preview.json --expect-manifest-hash "$HASH" --root . --write --output json
paredit refactor-preview --from render-pane --to paint-pane --mode function --fail-on-no-change --fail-on-parse-error --fail-on-target-conflict --require-definitions 1 --require-edits 1 --write --output json src/*.lisp lisp/*.el
paredit verify-refactor --symbol render-pane --new-symbol paint-pane --operation rename --phase post --output json src/*.lisp lisp/*.el
paredit dependency-report --output json system.asd src/*.lisp lisp/*.el
paredit package-report --output json system.asd src/*.lisp
paredit definition-report --output json system.asd src/*.lisp lisp/*.el
paredit unused-definition-report --output json system.asd src/*.lisp lisp/*.el
paredit unused-definition-report --fail-on-unused --output json system.asd src/*.lisp lisp/*.el
paredit remove-unused-definitions --output json system.asd src/*.lisp lisp/*.el
paredit remove-unused-definitions --write system.asd src/*.lisp lisp/*.el
paredit remove-unused-definitions --include-exported --write system.asd src/*.lisp
paredit remove-definition --file src/core.lisp --path 2 --output json
paredit remove-definition --file src/core.lisp --path 2 --write
paredit move-definition --from-file src/core.lisp --to-file src/render.lisp --path 2 --output json
paredit move-definition --from-file src/core.lisp --to-file src/render.lisp --path 2 --write
paredit split-file --from-file src/core.lisp --to-file src/ui/render.lisp --path 2 --path 3 --output json
paredit split-file --from-file src/core.lisp --to-file src/ui/render.lisp --path 2 --path 3 --write
paredit split-file --from-file src/core.lisp --to-file src/ui/render.lisp --name render-pane --kind macro --output json
paredit move-form --from-file src/core.lisp --to-file src/system.lisp --path 2 --output json
paredit move-form --from-file src/core.lisp --to-file src/system.lisp --path 2 --insert before --anchor-path 1 --write
paredit duplicate-report --output json src/*.lisp test/*.lisp lisp/*.el
paredit replacement-plan --replacement "(run-case)" --output json src/*.lisp test/*.lisp lisp/*.el
paredit replace-forms --file test/suite.lisp --path 0 --path 1 --with "(run-case)" --require-same-shape --output json
paredit replace-forms --file test/suite.lisp --path 0 --path 1 --with "(run-case)" --require-same-shape --write
paredit add-export --file src/package.lisp --package demo --symbol #:new-api --output json
paredit add-export --file src/package.lisp --package demo --symbol #:new-api --write
paredit rename-package --from old.pkg --to new.pkg --output json system.asd src/*.lisp
paredit rename-package --from old.pkg --to new.pkg --write system.asd src/*.lisp
paredit rename-symbol --file source.lisp --from old-name --to new-name --plan --output json
paredit rename-symbol --file source.lisp --from old-name --to new-name
paredit rename-in-form --file source.lisp --path 0.3 --from old-name --to new-name --output json
paredit rename-in-form --file source.lisp --path 0.3 --from old-name --to new-name --write
paredit rename-binding --file source.lisp --path 0.3 --from old-name --to new-name --output json
paredit rename-binding --file source.lisp --path 0.3 --from old-name --to new-name --write
paredit rename-function --from old-name --to new-name --output json src/*.lisp lisp/*.el
paredit rename-function --from old-name --to new-name --write src/*.lisp lisp/*.el
paredit wrap-function-calls --function fetch-user --wrapper with-cache --all-calls --output json src/*.lisp lisp/*.el
paredit wrap-function-calls --function fetch-user --wrapper with-cache --call-path 0.4 --write src/service.lisp
paredit unwrap-call --file source.lisp --path 0.3 --function with-cache --output json
paredit unwrap-call --file source.lisp --path 0.3 --function with-cache --write
paredit thread-expression --file source.clj --path 0 --style last --output json
paredit thread-expression --file source.clj --path 0 --style last --write
paredit unthread-expression --file source.clj --path 0 --output json
paredit unthread-expression --file source.clj --path 0 --write
paredit rename-symbols --from old-name --to new-name src/*.lisp lisp/*.el
paredit rename-symbols --from old-name --to new-name --write src/*.lisp lisp/*.el
paredit extract-function --file source.lisp --path 0.3 --name helper --output json
paredit extract-function --file source.lisp --path 0.3 --name helper --write
paredit inline-function --file source.lisp --definition-path 0 --call-path 1.3 --output json
paredit inline-function --file source.lisp --definition-path 0 --all-calls --output json
paredit inline-function --file source.lisp --definition-path 0 --call-path 1.3 --remove-definition --write
paredit add-function-parameter --file source.lisp --definition-path 0 --name context --argument '*context*' --call-path 1.3 --output json
paredit add-function-parameter --file source.lisp --definition-path 0 --name context --argument '*context*' --all-calls --output json
paredit move-function-parameter --file source.lisp --definition-path 0 --name context --to-index 0 --call-path 1.3 --output json
paredit move-function-parameter --file source.lisp --definition-path 0 --name context --to-index 0 --all-calls --write
paredit remove-function-parameter --file source.lisp --definition-path 0 --name context --call-path 1.3 --output json
paredit remove-function-parameter --file source.lisp --definition-path 0 --name context --all-calls --write
paredit introduce-let --file source.lisp --path 0.3.1 --name value --output json
paredit introduce-let --file source.lisp --path 0.3.1 --name value --all-occurrences --output json
paredit introduce-let --file source.lisp --path 0.3.1 --name value --write
paredit let-report --file source.lisp --fail-on-duplicate-evaluation --fail-on-unused-binding --require-inlineable-bindings 1 --output json
paredit inline-let --file source.lisp --path 0.3 --output json
paredit inline-let --file source.lisp --path 0.3 --write
paredit remove-unused-binding --file source.lisp --path 0.3 --name unused --output json
paredit remove-unused-binding --file source.lisp --path 0.3 --name unused --allow-drop-value --write
paredit remove-unused-binding --file source.lisp --path 0.3 --all-bindings --output json
paredit remove-unused-binding --file source.lisp --path 0.3 --all-bindings --allow-drop-value --write
paredit format --file source.lisp --indent 2
paredit select --file source.lisp --path 0.2
paredit select --file source.lisp --at 42
paredit replace --file source.lisp --path 0.1 --with new-name
paredit wrap --file source.lisp --path 0.2
paredit splice --file source.lisp --path 0.2
paredit raise --file source.lisp --path 0.2.1
paredit slurp-forward --file source.lisp --path 0
paredit slurp-backward --file source.lisp --path 1
paredit barf-forward --file source.lisp --path 0
paredit barf-backward --file source.lisp --path 0
paredit kill --file source.lisp --path 0.3
```

All commands that accept `--file` read stdin when it is omitted.

## Dialect Detection

| Dialect | Extensions |
| --- | --- |
| Common Lisp | `lisp`, `lsp`, `cl`, `asd` |
| Emacs Lisp | `el` |
| Scheme | `scm`, `ss`, `sld` |
| Clojure | `clj`, `cljs`, `cljc`, `edn` |
| Janet | `janet` |
| Fennel | `fnl` |

Use `--dialect` to override extension detection when stdin or generated files
do not carry a useful filename.

## Agent Refactoring Workflow

1. Run `paredit workspace-report --output json .` from the repository root to
   discover Lisp files, dialects, parse errors, definition counts, and call
   counts before choosing a refactor boundary. Review `skipped` counts when
   generated, hidden, or unknown-extension files may be relevant.
1. Run `paredit workspace-refactor-plan --symbol old --operation rename
   --fail-on-blocking-gate --require-definitions 1 --require-references 1
   --output json .` when an agent should discover Lisp files from repository
   roots before producing the gated refactor plan.
1. Run `paredit workspace-refactor-preview --from old --to new --mode
   function --fail-on-no-change --fail-on-parse-error
   --fail-on-target-conflict --require-changed-files 1
   --require-definitions 1 --require-edits 1 --output json .` when an agent
   should discover Common Lisp, Emacs Lisp, Scheme, Clojure, Janet, or Fennel
   files by extension and produce exact byte-span edit scripts, content hashes,
   replacement-symbol conflict checks, and rewritten-output parse gates without
   hand-maintaining file globs.
1. Run `paredit check --file target.lisp`.
1. Run `paredit agent-report --file target.lisp --output json` and cache the
   top-level form paths and spans.
1. Use `paredit outline --output json` to identify definition-like forms such as
   `defun`, `defmacro`, `defclass`, `defpackage`, `asdf:defsystem`, and
   Emacs Lisp `defcustom` or `define-minor-mode`.
1. Use `paredit form-report --path 0 --include-source --output json` on the
   selected form before local rewrites. Review `span`, `head`,
   `definitionLike`, child counts, depth, and `symbols` so an agent can decide
   whether a rename, extract, inline, or threading rewrite is scoped correctly.
1. Use `paredit package-report --output json` on Common Lisp `.asd`, `.lisp`,
   `.lsp`, and `.cl` files before package, nickname, export, or import
   refactors. Review `defpackage`, `in_packages`, `uses`, `imports`, and
   `exports` before changing package-qualified symbols.
1. Use `paredit dependency-report --output json` across explicit `.asd`,
   Common Lisp, and Emacs Lisp files before file moves, system splits, package
   cleanup, or dependency inversion. Review `asdf-depends-on`,
   `asdf-component`, `require`, `provide`, `load`, `defpackage-*`, and
   `qualified-symbol` entries to decide the safe edit order.
1. Use `paredit refactor-plan --symbol old --operation rename
   --fail-on-blocking-gate --require-definitions 1 --require-references 1
   --output json` for an agent-ready preflight that combines impact gates,
   dependency-report reminders, safe-to-automate status, ordered commands, and
   CI-friendly policy failures for rename, remove, move, or signature
   refactors.
1. Use `paredit refactor-preview --from old --to new --mode function
   --fail-on-no-change --fail-on-parse-error --fail-on-target-conflict
   --require-definitions 1 --require-edits 1 --output json`
   before write-mode refactors to inspect exact per-file rewrites, byte-span
   edit scripts, stable content hashes, output parse status, replacement-symbol
   conflict counts, and CI-friendly policy failures without modifying files.
1. Save preview JSON and run `paredit refactor-check --manifest
   rename.preview.json --root . --output json` when CI or an AI agent needs a
   cheap manifest health gate without rendering a diff. JSON output includes
   `manifest.path`, `manifest.hash`, `manifest_policy_passed`,
   `manifest_outputs_parse`, `summary.can_apply`, per-file hash/parse checks,
   and the `root` audit object.
1. Save preview JSON and run `paredit refactor-status --manifest
   rename.preview.json --root . --output json` when an AI coding agent needs a
   non-failing decision response before choosing the next tool call. JSON
   output includes `status`, `next_action`, `blocked_reasons`, `write_plan`,
   `manifest.hash`, `summary.can_apply`, per-file hash/parse checks, and the
   `root` audit object. Use `refactor-check` for CI gating and
   `refactor-status` for agent branching.
1. Save preview JSON and run `paredit refactor-diff --manifest
   rename.preview.json --expect-manifest-hash "$HASH" --root . --output json`
   to render a machine-readable unified diff from the same byte-span edits
   while rechecking the pinned manifest hash, input hashes, output hashes, parse
   status, manifest consistency, and workspace-root containment without writing
   files. JSON output includes a `root` audit object showing whether containment
   was enforced and which canonical root was used.
1. Save preview JSON and run `paredit refactor-apply --manifest
   rename.preview.json --expect-manifest-hash "$HASH" --root . --output json`
   for a second dry-run validation pass. Add `--write` only after the manifest
   hash pin, manifest policy, input hashes, output hashes, rewritten parse
   status, manifest consistency, and root containment all pass. JSON output
   includes a `root` audit object for CI and agent logs. This is the safer
   AI-agent path because stale source files, modified manifests, or out-of-root
   manifest paths cannot be rewritten from an old manifest.
1. Use `paredit verify-refactor --symbol old --operation rename --phase pre
   --output json` before edits and `paredit verify-refactor --symbol old
   --new-symbol new --operation rename --phase post --output json` after edits
   to produce fixed pass/fail checks for AI coding agents and CI gates.
1. Use `paredit add-export --output json` to plan a public API export after
   package review. The command updates an existing `:export`, creates one when
   missing, no-ops when the symbol is already exported, and reparses before
   `--write`.
1. Use `paredit rename-package --output json` after package review when
   renaming a Common Lisp package. Review `defpackage-name`,
   `in-package-name`, `package-option`, and `qualified-prefix` occurrences;
   the command preserves package designator prefixes, skips comments and
   strings, and reparses before `--write`.
1. Use `paredit definition-report --output json` across explicit `.asd`,
   `.lisp`, `.cl`, and `.el` file sets before file decomposition, API surface
   cleanup, macro consolidation, or test-suite restructuring. Review each
   definition's `category`, `name`, `path`, span, package context,
   `parameter_count`, and `body_form_count`.
1. Use `paredit call-report --output json` across the same explicit file set
   before callable rename, inline/extract, or function arity changes. Filter
   with `--symbol name` when planning one API, and review each call's `path`,
   span, `head`, `argumentCount`, dialect, and `enclosingDefinition` before
   selecting `--call-path` values or applying a multi-file plan.
   Use `paredit signature-report --symbol name --fail-on-mismatch --require-definitions 1 --require-calls 1 --output json`
   before changing required parameters. It joins callable definitions with
   call sites across explicit files and reports each call as `exact`,
   `missing-arguments`, `extra-arguments`, `unknown-definition`, or
   `ambiguous-definition`; the policy flags turn missing/extra argument
   discoveries and unexpectedly empty scans into CI failures.
1. Use `paredit call-graph --symbol name --fail-on-inbound-callers --require-edges 1 --require-internal-edges 1 --output json`
   before file decomposition, definition moves, public API cleanup, or
   dead-code removal. Review `inbound_edge_count`, internal/external edge
   totals, and `policy.violations`; add `--include-external` when external
   package, macro, or runtime dependencies affect the refactoring boundary.
1. Use `paredit unused-definition-report --output json` before dead-code
   removal or public API shrinking. The report scans exact atom references
   across the explicit file set, excludes references inside the defining
   top-level form, and emits `candidates` plus per-definition
   `reference_count` for review. Add `--fail-on-unused` to make CI fail when
   any externally unreferenced definition remains, or
   `--require-unused-definitions N` when an agent expects a dead-code cleanup
   opportunity before planning removals.
1. Plan bulk dead-code cleanup with
   `paredit remove-unused-definitions --output json` after reviewing
   `unused-definition-report`. By default it removes only unreferenced
   non-protected definition categories, preserves definitions exported from
   Common Lisp `defpackage` forms, reports skipped package, system, test,
   customization, and mode definitions, deletes from the end of each file to
   avoid offset drift, and reparses before writing. Use `--include-exported`
   only after explicitly shrinking the public API, and `--include-protected`
   only after reviewing those protected categories.
1. Remove a reviewed dead top-level definition with
   `paredit remove-definition --output json` first, then apply with
   `--write`. The command accepts the same top-level `path` reported by
   `definition-report` and `unused-definition-report`, rejects non-definition
   forms, removes structurally, and reparses the file before writing.
1. Use `paredit duplicate-report --output json` before table-driven test,
   helper extraction, macro consolidation, or repeated branch cleanup work.
   Review each shape's `head`, `form_path`, span, node count, and original
   text before deciding whether the repetition is accidental duplication or a
   meaningful idiom.
1. Use `paredit replacement-plan --output json` to convert reviewed duplicate
   shapes into per-file `replace-forms` command batches. Inspect each batch's
   `paths`, `replace_forms_args`, replacement placeholder, and original form
   text before deciding the real helper, macro, or table-driven call.
1. Replace a reviewed batch of duplicate or table-driven candidate forms with
   `paredit replace-forms --output json` before applying `--write`. Pass every
   reviewed `--path`, use `--require-same-shape` for `duplicate-report`-derived
   batches, and inspect `targets`, `replacement_shape`, and `rewritten`.
1. Move coherent top-level definitions between files with
   `paredit move-definition --output json` after reviewing
   `definition-report`. The command accepts a top-level `path`, supports a
   missing destination file as empty, removes the source form structurally,
   appends the balanced definition to the destination, and reparses both files
   before `--write`.
1. Split multiple reviewed top-level definitions into a new file or nested
   directory with `paredit split-file --output json` after reviewing
   `definition-report`. Pass repeated `--path` values for exact moves, or use
   `--name` and `--kind` selectors to split a large file without manually
   collecting every path. Selector matches are de-duplicated, but a requested
   name or kind that matches nothing fails the plan. Plan mode reports
   `definition_count`, `from_rewritten`, `to_rewritten`, `to_file_existed`,
   and `to_parent_existed` without creating files. With `--write`, the command
   creates the destination parent directory when needed, removes selected
   definitions from the source in reverse span order, appends them to the
   destination in source order, and reparses both rewritten files.
1. Move non-definition top-level forms with
   `paredit move-form --output json` after reviewing `outline` or
   `agent-report`. Use it for `defpackage`, `in-package`, `eval-when`, ASDF
   fragments, feature conditionals, or migration scaffolding that is not a
   recognized definition. Inspect `head`, `text`, `from_rewritten`, and
   `to_rewritten`; use `--insert before/after --anchor-path PATH` when the
   destination order is semantically important.
1. Use `paredit find-symbol --symbol name --output json` for a focused
   single-file scan, or `paredit symbol-report --symbol name --output json`
   for an explicit file set. Review per-file counts and the outline context
   for each occurrence before any rename.
   Use `paredit call-report --symbol name --output json` when the rename,
   inline, extraction, or arity change depends on callable list-head sites
   rather than arbitrary atom references.
   Use `paredit signature-report --symbol name --fail-on-mismatch
   --require-definitions 1 --require-calls 1 --output json` when a required
   parameter change must be checked across Common Lisp and Emacs Lisp files;
   the policy flags fail empty scans and incompatible arity before writes.
   Use `paredit call-graph --symbol name --fail-on-inbound-callers
   --require-edges 1 --require-internal-edges 1 --output json` before moving,
   inlining, deleting, or splitting definitions; add `--include-external` when
   external API and macro dependencies are part of the blast-radius review.
   Use `paredit impact-report --symbol name --fail-on-risk-level warning
   --require-definitions 1 --require-references 1 --require-calls 1
   --output json` as the preflight gate before rename, move, remove, inline,
   extraction, or required-parameter edits. Review `policy`, `riskLevel`,
   `risks`, `inbound_edge_count`, `non_call_reference_count`, and signature
   `by_status` before applying write-mode commands.
1. Use `paredit rename-binding --output json` when the target is a local
   `let` or `let*` binding. Review `binding_span`, `reference_count`, and
   `shadowed_scope_count`; the command skips nested scopes that rebind the
   same name.
1. Use `paredit rename-in-form --output json` when the rename must stay inside
   a selected function, macro, `let`, or other local form. Review the scope
   span and occurrence count before applying `--write`.
1. Use `paredit rename-function --output json` for callable definitions
   (`defun`, `defmacro`, `defgeneric`, `defmethod`, and dialect equivalents).
   It rewrites definition names and list-head call sites, but does not rewrite
   arbitrary value references.
1. Use `paredit wrap-function-calls --output json` when a refactor needs to
   introduce a wrapper macro or helper around reviewed call sites. Pass either
   `--all-calls` or repeated `--call-path`; review `calls`,
   `skippedAlreadyWrapped`, `skippedNested`, and policy fields before applying
   `--write`.
1. Use `paredit unwrap-call --output json` when a selected wrapper call should
   be replaced by one of its arguments. Pass `--function` as a guard whenever
   possible, and review `argumentIndex`, `argumentSpan`, `replacement`, and
   `rewritten` before applying `--write`.
1. Use `paredit thread-expression --output json` when nested calls should be
   converted into a `->` or `->>` pipeline. Review `base`, `steps`,
   `replacement`, `span`, and dialect before applying `--write`.
1. Use `paredit unthread-expression --output json` when a reviewed thread
   pipeline should be converted back into nested calls. Standard `->` and
   `->>` operators infer the style; custom operators require explicit
   `--style`.
1. Use `paredit rename-symbol --plan --output json` for one file or
   `paredit rename-symbols --output json` for an explicit file set after
   reviewing `symbol-report`.
1. Apply a project-wide exact atom rename only with
   `paredit rename-symbols --write`; the command re-parses every rewritten file
   before saving.
1. Extract duplicated or complex subexpressions with
   `paredit extract-function --output json` first, then re-run with `--write`
   after reviewing the generated call and top-level definition.
1. Inline trivial or over-abstracted helpers with
   `paredit inline-function --output json` first. Review `definition_path`,
   `call_path`, parameter reference counts, and the replacement before
   applying `--write`; pass `--remove-definition` only after confirming no
   remaining callers.
1. Add required function parameters with
   `paredit add-function-parameter --output json` first. Review the selected
   definition, every explicit or discovered `call_paths` entry, and the
   inserted argument before applying `--write`. Run `signature-report` across
   the broader explicit file set first when callers can exist outside the
   single file being rewritten.
1. Reorder required function parameters with
   `paredit move-function-parameter --output json` first. Review `from_index`,
   `to_index`, every explicit or discovered `call_paths` entry, and
   `moved_arguments` before applying `--write`.
1. Remove obsolete required function parameters with
   `paredit remove-function-parameter --output json` first. Review
   `parameter_index`, each selected or discovered call, and
   `removed_arguments` before applying `--write`.
1. Introduce names for complex intermediate expressions with
   `paredit introduce-let --output json` first, then re-run with `--write`
   after reviewing the binding value and enclosing replacement.
1. Audit local bindings with `paredit let-report --output json` before
   inlining. Review each form path, binding value span, reference count, and
   risk list. For agent or CI workflows, add `--fail-on-duplicate-evaluation`,
   `--fail-on-unused-binding`, and `--require-inlineable-bindings N` to turn
   the report into a fixed pass/fail gate while still printing JSON.
1. Remove unnecessary single-binding `let` forms with
   `paredit inline-let --output json` first. The command refuses unused
   bindings and duplicate evaluation by default; pass
   `--allow-duplicate-evaluation` only after semantic review.
1. Remove unused local bindings with `paredit remove-unused-binding --output
   json` after `let-report` shows `reference_count` is zero. Use `--name` for a
   reviewed binding or `--all-bindings` to remove every zero-reference binding
   in the selected `let` or `let*`. The command plans deletion without writing
   by default; pass `--allow-drop-value --write` only after reviewing that
   dropping each binding value expression does not remove a required side
   effect.
1. Use structural edits for form movement: `wrap`, `splice`, `raise`,
   `slurp-*`, and `barf-*`.
1. Run `paredit check` again, then run the project test suite.

This workflow is intended for large Common Lisp and Emacs Lisp refactors where
the safe primitive operations are: discover definitions, isolate forms, rename
symbols exactly, move balanced forms, and verify after every generated change.

## Examples

Detect an Emacs Lisp file:

```sh
paredit dialect --file init.el --output json
```

Find all exact uses of a Common Lisp symbol without matching strings or
comments:

```sh
paredit find-symbol --file src/core.lisp --symbol make-session --output json
```

Plan a rename before applying it:

```sh
paredit rename-symbol \
  --file src/core.lisp \
  --from old-session-name \
  --to session-name \
  --plan \
  --output json
```

Apply the rename into a temporary file and re-check it:

```sh
paredit rename-symbol \
  --file src/core.lisp \
  --from old-session-name \
  --to session-name > /tmp/core.lisp
paredit check --file /tmp/core.lisp
```

Plan and apply a rename only inside one selected form:

```sh
paredit rename-in-form \
  --file src/core.lisp \
  --path 0.3 \
  --from session \
  --to tmux-session \
  --output json
paredit rename-in-form \
  --file src/core.lisp \
  --path 0.3 \
  --from session \
  --to tmux-session \
  --write
```

Plan and apply a lexical local binding rename:

```sh
paredit rename-binding \
  --file src/core.lisp \
  --path 0.3 \
  --from session \
  --to tmux-session \
  --output json
paredit rename-binding \
  --file src/core.lisp \
  --path 0.3 \
  --from session \
  --to tmux-session \
  --write
```

Plan and then apply an exact atom rename across a Common Lisp or Emacs Lisp
file set:

```sh
paredit rename-symbols \
  --from old-session-name \
  --to session-name \
  --output json \
  src/*.lisp elisp/*.el
paredit rename-symbols \
  --from old-session-name \
  --to session-name \
  --write \
  src/*.lisp elisp/*.el
```

Extract a complex expression into a top-level helper:

```sh
paredit extract-function \
  --file src/renderer.lisp \
  --path 0.3 \
  --name render-fragment \
  --param width \
  --param height \
  --output json
paredit extract-function \
  --file src/renderer.lisp \
  --path 0.3 \
  --name render-fragment \
  --param width \
  --param height \
  --insert before \
  --anchor-path 2 \
  --write
```

`extract-function` keeps parameter discovery explicit: pass `--param` in
call order when the extracted expression depends on local names. This keeps
agent-generated refactors deterministic and reviewable instead of relying on
implicit lexical inference.

Convert nested calls into a reviewed thread-last pipeline:

```sh
paredit thread-expression \
  --file src/pipeline.clj \
  --path 0 \
  --style last \
  --output json
paredit thread-expression \
  --file src/pipeline.clj \
  --path 0 \
  --style last \
  --write
```

`thread-expression` defaults to `->` for `--style first` and `->>` for
`--style last`; pass `--operator` for dialect-specific threading macros after
reviewing the JSON plan.

Remove a reviewed wrapper call while keeping one argument:

```sh
paredit unwrap-call \
  --file src/service.lisp \
  --path 0.3 \
  --function with-cache \
  --argument-index 0 \
  --output json
paredit unwrap-call \
  --file src/service.lisp \
  --path 0.3 \
  --function with-cache \
  --argument-index 0 \
  --write
```

`unwrap-call` is intentionally local: select one parenthesized call with
`--path` or `--at`, optionally guard the head with `--function`, then replace
the whole call with the selected zero-based argument.

Convert a reviewed thread pipeline back into nested calls:

```sh
paredit unthread-expression \
  --file src/pipeline.clj \
  --path 0 \
  --output json
paredit unthread-expression \
  --file src/pipeline.clj \
  --path 0 \
  --write
```

`unthread-expression` infers `--style first` from `->` and `--style last` from
`->>`. Pass both `--operator` and `--style` when a project-specific threading
macro uses a different name.

Inline a reviewed helper call back into its caller:

```sh
paredit inline-function \
  --file src/renderer.lisp \
  --definition-path 0 \
  --call-path 3.2 \
  --output json
paredit inline-function \
  --file src/renderer.lisp \
  --definition-path 0 \
  --all-calls \
  --remove-definition \
  --write
```

`inline-function` is intentionally conservative. It requires a supported
single-expression function definition, exact arity, and simple positional
parameters. It refuses duplicate argument evaluation and unused arguments by
default; use `--allow-duplicate-evaluation` or `--allow-drop-arguments` only
after reviewing the JSON plan. Pass repeated `--call-path` values for reviewed
specific calls, or `--all-calls` to discover every same-file call whose list
head matches the selected definition. The JSON plan reports both the legacy
single-call fields and a `calls` array so agents can review each replacement.

Add a required parameter to a reviewed definition and selected call sites:

```sh
paredit add-function-parameter \
  --file src/renderer.lisp \
  --definition-path 0 \
  --name context \
  --argument '*context*' \
  --call-path 3.2 \
  --output json
paredit add-function-parameter \
  --file src/renderer.lisp \
  --definition-path 0 \
  --name context \
  --argument '*context*' \
  --all-calls \
  --write
```

`add-function-parameter` updates only the selected function definition and the
reviewed `--call-path` entries, or every same-file call discovered by
`--all-calls`. It detects supported Lisp function forms from the file extension
or `--dialect`, verifies each call head against the selected definition,
reports the final `call_paths`, re-parses the rewritten file, and supports
`--insert start` for prefix arguments.

Move a required parameter within a reviewed definition and selected call sites:

```sh
paredit move-function-parameter \
  --file src/renderer.lisp \
  --definition-path 0 \
  --name context \
  --to-index 0 \
  --call-path 3.2 \
  --output json
paredit move-function-parameter \
  --file src/renderer.lisp \
  --definition-path 0 \
  --name context \
  --to-index 0 \
  --all-calls \
  --write
```

`move-function-parameter` reorders only a simple positional parameter in the
selected definition and moves the same positional argument in each reviewed
`--call-path` entry, or each same-file call discovered by `--all-calls`. It
reports `from_index`, `to_index`, `call_paths`, and `moved_arguments`, verifies
each call head against the selected definition, and re-parses the rewritten
file.

Remove an obsolete required parameter from a reviewed definition and selected
call sites:

```sh
paredit remove-function-parameter \
  --file src/renderer.lisp \
  --definition-path 0 \
  --name context \
  --call-path 3.2 \
  --output json
paredit remove-function-parameter \
  --file src/renderer.lisp \
  --definition-path 0 \
  --name context \
  --all-calls \
  --write
```

`remove-function-parameter` removes only a simple positional parameter from the
selected definition and the same positional argument from each reviewed
`--call-path` entry, or each same-file call discovered by `--all-calls`. It
verifies each call head against the selected definition, reports
`parameter_index`, `call_paths`, and `removed_arguments`, refuses missing call
arguments by default, and re-parses the rewritten file.

Introduce a local name for a complex subexpression:

```sh
paredit introduce-let \
  --file src/renderer.lisp \
  --path 0.3.1 \
  --name fragment \
  --output json
paredit introduce-let \
  --file src/renderer.lisp \
  --path 0.3.1 \
  --name fragment \
  --all-occurrences \
  --output json
paredit introduce-let \
  --file src/renderer.lisp \
  --path 0.3.1 \
  --name fragment \
  --write
```

By default, `introduce-let` replaces only the selected expression. Add
`--all-occurrences` to replace every structurally identical expression inside
the enclosing list; JSON output includes `occurrence_count` and
`occurrence_spans` for review before `--write`. Equivalent expressions under a
nested binding form that already binds the introduced name are skipped to avoid
accidental capture; review `skipped_shadowed_occurrence_count` and
`skipped_shadowed_occurrence_spans` when using `--all-occurrences`.

Audit local bindings before inlining them:

```sh
paredit let-report \
  --file src/renderer.lisp \
  --fail-on-duplicate-evaluation \
  --fail-on-unused-binding \
  --require-inlineable-bindings 1 \
  --output json
```

Inline a reviewed single-binding local name:

```sh
paredit inline-let \
  --file src/renderer.lisp \
  --path 0.3 \
  --output json
paredit inline-let \
  --file src/renderer.lisp \
  --path 0.3 \
  --write
```

Remove a reviewed unused local binding:

```sh
paredit remove-unused-binding \
  --file src/renderer.lisp \
  --path 0.3 \
  --name scratch \
  --output json
paredit remove-unused-binding \
  --file src/renderer.lisp \
  --path 0.3 \
  --name scratch \
  --allow-drop-value \
  --write
paredit remove-unused-binding \
  --file src/renderer.lisp \
  --path 0.3 \
  --all-bindings \
  --output json
paredit remove-unused-binding \
  --file src/renderer.lisp \
  --path 0.3 \
  --all-bindings \
  --allow-drop-value \
  --write
```

## Rust Quality Bar

- Rust edition 2024 with a minimum supported Rust version in `Cargo.toml`.
- `unsafe_code = "forbid"`.
- Newtypes for byte offsets, byte spans, expression paths, node ids, child
  indexes, and symbol names.
- `thiserror` for parse errors and `anyhow` for CLI boundary errors.
- Warning-clean `cargo clippy --all-targets -- -D warnings`.
- Nix flake verification for reproducible development.

## Architecture

The crate follows a DDD-oriented module layout:

```text
src/
  domain/          Typed S-expression model, parser, Lisp dialect rules
  application/     Use-case orchestration boundary
  infrastructure/  Filesystem and process adapter boundary
  presentation/    CLI parser, command dispatch, output formatting
```

Keep Lisp semantics, structural edit invariants, and dialect detection in
`domain`. Keep `anyhow`, `clap`, terminal output, and write/no-write command
behavior at the `presentation` boundary unless a use case is promoted into
`application`.

## Development

```sh
nix develop
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test
nix flake check
nix build .#
```

## Scope

`paredit-cli` is a structural S-expression tool, not a Lisp evaluator or full
reader implementation. It preserves balanced list, vector, and map delimiters;
tracks comments and strings safely for symbol operations; and provides
dialect-aware definition hints. It does not macroexpand code or update ASDF,
package, autoload, or module manifests automatically.
