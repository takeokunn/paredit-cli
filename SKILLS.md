# SKILLS: S-expression Refactoring With paredit-cli

Use this skill when refactoring Common Lisp, Scheme, Clojure, Emacs Lisp, or any Lisp-like S-expression file.

## Safety Policy

- Never hand-edit balanced delimiters during large refactors.
- Always validate with `paredit inspect check` before and after edits.
- Prefer `--path` for deterministic scripted edits.
- Prefer `--at` when an error report or grep result gives a byte offset.
- Keep data and logic changes separate: structure-edit first, semantic rewrite second.
- Use explicit command timeouts when running project tests.

## Implementation Boundaries

- Keep parser, S-expression invariants, byte-span/path newtypes, and Lisp
  dialect rules under `src/domain`.
- Keep CLI argument parsing, JSON/text output, and write/no-write command
  behavior under `src/presentation`.
- Promote command orchestration into `src/application` when multiple delivery
  adapters need the same use case.
- Put filesystem/process adapters under `src/infrastructure`; do not leak
  adapter-specific error types into `domain`.
- Preserve public compatibility through `paredit_cli::sexpr` and
  `paredit_cli::dialect` re-exports when moving domain modules.

## Refactoring Loop

```sh
timeout 10s paredit inspect check --file target.lisp
timeout 10s paredit inspect agent-report --file target.lisp --output json
timeout 10s paredit inspect workspace --output json .
timeout 10s paredit refactor workspace-plan --symbol old-name --operation rename --fail-on-blocking-gate --require-definitions 1 --require-references 1 --output json .
timeout 10s paredit inspect outline --file target.lisp --output json
timeout 10s paredit inspect form --file target.lisp --path 0 --include-source --output json
timeout 10s paredit inspect find-symbol --file target.lisp --symbol old-name --output json
timeout 10s paredit inspect symbols --symbol old-name --output json src/*.lisp elisp/*.el
timeout 10s paredit inspect calls --symbol old-name --output json src/*.lisp elisp/*.el
timeout 10s paredit inspect signature --symbol old-name --fail-on-mismatch --require-definitions 1 --require-calls 1 --output json src/*.lisp elisp/*.el
timeout 10s paredit inspect call-graph --symbol old-name --fail-on-inbound-callers --require-edges 1 --require-internal-edges 1 --output json src/*.lisp elisp/*.el
timeout 10s paredit inspect impact --symbol old-name --fail-on-risk-level warning --require-definitions 1 --require-references 1 --require-calls 1 --output json src/*.lisp elisp/*.el
timeout 10s paredit refactor plan --symbol old-name --operation rename --fail-on-blocking-gate --require-definitions 1 --require-references 1 --output json src/*.lisp elisp/*.el
timeout 10s paredit refactor workspace-preview --from old-name --to new-name --mode function --fail-on-no-change --fail-on-parse-error --fail-on-target-conflict --require-changed-files 1 --require-definitions 1 --require-edits 1 --output json .
timeout 10s paredit refactor preview --from old-name --to new-name --mode function --fail-on-no-change --fail-on-parse-error --fail-on-target-conflict --require-edits 1 --output json src/*.lisp elisp/*.el
timeout 10s paredit refactor check --manifest rename.preview.json --root . --output json
timeout 10s paredit refactor status --manifest rename.preview.json --root . --output json
HASH=<manifest.hash from 'paredit refactor status' JSON>
timeout 10s paredit refactor diff --manifest rename.preview.json --expect-manifest-hash "$HASH" --root . --output json
timeout 10s paredit refactor apply --manifest rename.preview.json --expect-manifest-hash "$HASH" --root . --output json
timeout 10s paredit refactor apply --manifest rename.preview.json --expect-manifest-hash "$HASH" --root . --write --output json
timeout 10s paredit refactor preview --from old-name --to new-name --mode function --fail-on-no-change --fail-on-parse-error --fail-on-target-conflict --require-edits 1 --write --output json src/*.lisp elisp/*.el
timeout 10s paredit refactor verify --symbol old-name --operation rename --phase pre --output json src/*.lisp elisp/*.el
timeout 10s paredit refactor verify --symbol old-name --new-symbol new-name --operation rename --phase post --output json src/*.lisp elisp/*.el
timeout 10s paredit inspect dependencies --output json system.asd src/*.lisp elisp/*.el
timeout 10s paredit inspect packages --output json system.asd src/*.lisp
timeout 10s paredit inspect definitions --output json system.asd src/*.lisp elisp/*.el
timeout 10s paredit inspect unused-definitions --output json system.asd src/*.lisp elisp/*.el
timeout 10s paredit inspect unused-definitions --fail-on-unused --output json system.asd src/*.lisp elisp/*.el
timeout 10s paredit refactor remove-unused-definitions --output json system.asd src/*.lisp elisp/*.el
timeout 10s paredit refactor remove-unused-definitions --write system.asd src/*.lisp elisp/*.el
timeout 10s paredit refactor remove-unused-definitions --include-exported --write system.asd src/*.lisp
timeout 10s paredit refactor remove-definition --file src/core.lisp --path 2 --output json
timeout 10s paredit refactor remove-definition --file src/core.lisp --path 2 --write
timeout 10s paredit refactor move-definition --from-file src/core.lisp --to-file src/render.lisp --path 2 --output json
timeout 10s paredit refactor move-definition --from-file src/core.lisp --to-file src/render.lisp --path 2 --write
timeout 10s paredit refactor move-form --from-file src/core.lisp --to-file src/system.lisp --path 2 --output json
timeout 10s paredit refactor move-form --from-file src/core.lisp --to-file src/system.lisp --path 2 --insert before --anchor-path 1 --write
timeout 10s paredit inspect duplicates --output json src/*.lisp test/*.lisp elisp/*.el
timeout 10s paredit inspect similarity --output json src/*.lisp test/*.lisp elisp/*.el
timeout 10s paredit refactor replacement-plan --replacement '(run-case)' --output json src/*.lisp test/*.lisp elisp/*.el
timeout 10s paredit edit replace-forms --file test/suite.lisp --path 0 --path 1 --with '(run-case)' --require-same-shape --output json
timeout 10s paredit edit replace-forms --file test/suite.lisp --path 0 --path 1 --with '(run-case)' --require-same-shape --write
timeout 10s paredit refactor add-export --file src/package.lisp --package demo --symbol #:new-api --output json
timeout 10s paredit refactor add-export --file src/package.lisp --package demo --symbol #:new-api --write
timeout 10s paredit refactor rename-package --from old.pkg --to new.pkg --output json system.asd src/*.lisp
timeout 10s paredit refactor rename-package --from old.pkg --to new.pkg --write system.asd src/*.lisp
timeout 10s paredit refactor rename-binding --file target.lisp --path 0.3 --from old-name --to new-name --output json
timeout 10s paredit refactor rename-binding --file target.lisp --path 0.3 --from old-name --to new-name --write
timeout 10s paredit refactor rename-in-form --file target.lisp --path 0.3 --from old-name --to new-name --output json
timeout 10s paredit refactor rename-in-form --file target.lisp --path 0.3 --from old-name --to new-name --write
timeout 10s paredit refactor rename-symbol --file target.lisp --from old-name --to new-name --plan --output json
timeout 10s paredit refactor rename-function --from old-name --to new-name --output json src/*.lisp elisp/*.el
timeout 10s paredit refactor rename-function --from old-name --to new-name --write src/*.lisp elisp/*.el
timeout 10s paredit refactor rename-symbols --from old-name --to new-name --output json src/*.lisp elisp/*.el
timeout 10s paredit refactor rename-symbols --from old-name --to new-name --write src/*.lisp elisp/*.el
timeout 10s paredit refactor thread-expression --file target.clj --path 0 --style last --output json
timeout 10s paredit refactor thread-expression --file target.clj --path 0 --style last --write
timeout 10s paredit refactor unthread-expression --file target.clj --path 0 --output json
timeout 10s paredit refactor unthread-expression --file target.clj --path 0 --write
timeout 10s paredit refactor extract-function --file target.lisp --path 0.3 --name helper --param value --output json
timeout 10s paredit refactor extract-function --file target.lisp --path 0.3 --name helper --param value --insert before --anchor-path 2 --write
timeout 10s paredit refactor extract-constant --file target.lisp --path 0.3.1 --name +max-retries+ --output json
timeout 10s paredit refactor extract-constant --file target.lisp --path 0.3.1 --name +max-retries+ --insert before --anchor-path 2 --write
timeout 10s paredit refactor inline-function --file target.lisp --definition-path 0 --call-path 1.3 --output json
timeout 10s paredit refactor inline-function --file target.lisp --definition-path 0 --all-calls --output json
timeout 10s paredit refactor inline-function --file target.lisp --definition-path 0 --call-path 1.3 --remove-definition --write
timeout 10s paredit refactor add-function-parameter --file target.lisp --definition-path 0 --name context --argument '*context*' --call-path 1.3 --output json
timeout 10s paredit refactor add-function-parameter --file target.lisp --definition-path 0 --name context --argument '*context*' --all-calls --output json
timeout 10s paredit refactor move-function-parameter --file target.lisp --definition-path 0 --name context --to-index 0 --call-path 1.3 --output json
timeout 10s paredit refactor move-function-parameter --file target.lisp --definition-path 0 --name context --to-index 0 --all-calls --output json
timeout 10s paredit refactor remove-function-parameter --file target.lisp --definition-path 0 --name context --call-path 1.3 --output json
timeout 10s paredit refactor remove-function-parameter --file target.lisp --definition-path 0 --name context --all-calls --output json
timeout 10s paredit refactor introduce-let --file target.lisp --path 0.3.1 --name value --output json
timeout 10s paredit refactor introduce-let --file target.lisp --path 0.3.1 --name value --write
timeout 10s paredit inspect lets --file target.lisp --fail-on-duplicate-evaluation --fail-on-unused-binding --require-inlineable-bindings 1 --output json
timeout 10s paredit refactor inline-let --file target.lisp --path 0.3 --output json
timeout 10s paredit refactor inline-let --file target.lisp --path 0.3 --write
timeout 10s paredit refactor remove-unused-binding --file target.lisp --path 0.3 --name unused --output json
timeout 10s paredit refactor remove-unused-binding --file target.lisp --path 0.3 --name unused --allow-drop-value --write
timeout 10s paredit refactor remove-unused-binding --file target.lisp --path 0.3 --all-bindings --output json
timeout 10s paredit refactor remove-unused-binding --file target.lisp --path 0.3 --all-bindings --allow-drop-value --write
timeout 10s paredit edit select --file target.lisp --path 0.2
timeout 10s paredit edit replace --file target.lisp --path 0.2 --with '(new-form ...)' > /tmp/target.lisp
timeout 10s paredit inspect check --file /tmp/target.lisp
mv /tmp/target.lisp target.lisp
```

## Common Operations

- Detect a Lisp dialect: `paredit inspect dialect --file target.el --output json`
- Discover Lisp files and parse/refactor inventory from repository roots: `paredit inspect workspace --output json .`
- Discover Lisp files from repository roots and build an ordered, gated refactor plan with skipped-file accounting: `paredit refactor workspace-plan --symbol old --operation rename --fail-on-blocking-gate --require-definitions 1 --require-references 1 --output json .`
- Report project symbol occurrences with outline context: `paredit inspect symbols --symbol old --output json src/*.lisp elisp/*.el`
- Report callable list-head sites before signature or inline refactors: `paredit inspect calls --symbol old --output json src/*.lisp elisp/*.el`
- Compare callable definition arity with call-site argument counts before required-parameter changes, and fail CI when the scan is empty or incompatible: `paredit inspect signature --symbol old --fail-on-mismatch --require-definitions 1 --require-calls 1 --output json src/*.lisp elisp/*.el`
- Report internal call graph edges before moving, inlining, deleting, or splitting definitions, and fail CI when an expected graph is empty or the focused symbol has inbound internal callers: `paredit inspect call-graph --symbol old --fail-on-inbound-callers --require-edges 1 --require-internal-edges 1 --output json src/*.lisp elisp/*.el`
- Run a preflight refactor blast-radius gate before rename, move, remove, inline, extraction, or required-parameter edits, and fail CI when the scan is empty or the risk exceeds policy: `paredit inspect impact --symbol old --fail-on-risk-level warning --require-definitions 1 --require-references 1 --require-calls 1 --output json src/*.lisp elisp/*.el`
- Build an ordered, gated plan with machine-checkable policy failures for AI coding agents before rename, remove, move, or signature edits: `paredit refactor plan --symbol old --operation rename --fail-on-blocking-gate --require-definitions 1 --require-references 1 --output json src/*.lisp elisp/*.el`
- Discover Lisp files from repository roots and preview exact rename rewrites with skipped-file accounting, byte-span edits, content hashes, target-symbol conflict checks, parse gates, and CI-friendly policy failures: `paredit refactor workspace-preview --from old --to new --mode function --fail-on-no-change --fail-on-parse-error --fail-on-target-conflict --require-changed-files 1 --require-definitions 1 --require-edits 1 --output json .`
- Preview refactor rewrites with byte-span edit scripts and CI-friendly policy failures before applying changes, then add `--write` to apply only after target-symbol conflict checks, policy gates, and rewritten-output parsing pass: `paredit refactor preview --from old --to new --mode function --fail-on-no-change --fail-on-parse-error --fail-on-target-conflict --require-definitions 1 --require-edits 1 --output json src/*.lisp elisp/*.el`
- Validate a saved preview manifest without writing files or rendering a diff, and fail when preview policy, output parsing, hashes, manifest flags, or root containment drift. Inspect JSON `manifest.path`, `manifest.hash`, `summary.can_apply`, `manifest_policy_passed`, `manifest_outputs_parse`, and `root` in agent logs: `paredit refactor check --manifest rename.preview.json --root . --output json`
- Summarize a saved preview manifest into AI-agent next actions without writing files or failing on stale state. Inspect JSON `status`, `next_action`, `blocked_reasons`, `write_plan`, `manifest.hash`, `summary.can_apply`, and `root` before deciding whether to regenerate, diff, or apply: `paredit refactor status --manifest rename.preview.json --root . --output json`
- Render a saved preview manifest as a verified unified diff without writing files, and fail when the pinned manifest hash, file hashes, parse status, manifest consistency, or root containment drift. Inspect JSON `root.enforced` and `root.path` in agent logs: `paredit refactor diff --manifest rename.preview.json --expect-manifest-hash "$HASH" --root . --output json`
- Apply a saved preview manifest with manifest-hash pinning, stale-file hash guards, output-hash verification, parse gates, optional root containment, all-or-nothing write semantics, and JSON `root` audit evidence: `paredit refactor apply --manifest rename.preview.json --expect-manifest-hash "$HASH" --root . --write --output json`
- Verify pre/post refactor invariants with fixed pass/fail checks for AI coding agents and CI gates: `paredit refactor verify --symbol old --new-symbol new --operation rename --phase post --output json src/*.lisp elisp/*.el`
- Report ASDF, package, load, provide/require, and qualified-symbol dependencies before file or package boundary changes: `paredit inspect dependencies --output json system.asd src/*.lisp elisp/*.el`
- Report Common Lisp package declarations before package/export/import refactors: `paredit inspect packages --output json system.asd src/*.lisp`
- Report definition inventory before file decomposition, API cleanup, macro consolidation, or test restructuring: `paredit inspect definitions --output json system.asd src/*.lisp elisp/*.el`
- Report externally unreferenced definitions before dead-code removal, optionally as a CI gate: `paredit inspect unused-definitions --fail-on-unused --output json system.asd src/*.lisp elisp/*.el`
- Plan bulk removal of unreferenced non-protected, non-exported definitions: `paredit refactor remove-unused-definitions --output json system.asd src/*.lisp elisp/*.el`
- Apply reviewed bulk dead-code removal: `paredit refactor remove-unused-definitions --write system.asd src/*.lisp elisp/*.el`
- Remove a reviewed dead top-level definition: `paredit refactor remove-definition --file src/core.lisp --path 2 --write`
- Move a reviewed top-level definition between files: `paredit refactor move-definition --from-file src/core.lisp --to-file src/render.lisp --path 2 --write`
- Move a reviewed non-definition top-level form between files: `paredit refactor move-form --from-file src/core.lisp --to-file src/system.lisp --path 2 --insert before --anchor-path 1 --write`
- Report repeated S-expression shapes before helper extraction or table-driven refactors: `paredit inspect duplicates --output json src/*.lisp test/*.lisp elisp/*.el`
- Report structurally similar forms that exact shape matching misses, ranked by normalized similarity: `paredit inspect similarity --threshold 0.87 --output json src test`
- Convert duplicate-shape findings into reviewed per-file replacement batches: `paredit refactor replacement-plan --replacement '(run-case)' --output json src/*.lisp test/*.lisp elisp/*.el`
- Replace reviewed duplicate forms with a helper, macro, or table-driven call: `paredit edit replace-forms --file test/suite.lisp --path 0 --path 1 --with '(run-case)' --require-same-shape --output json`
- Add a reviewed Common Lisp public export: `paredit refactor add-export --file src/package.lisp --package demo --symbol #:new-api --write`
- Plan a Common Lisp package rename: `paredit refactor rename-package --from old.pkg --to new.pkg --output json system.asd src/*.lisp`
- Apply a reviewed package rename: `paredit refactor rename-package --from old.pkg --to new.pkg --write system.asd src/*.lisp`
- Plan a symbol rename: `paredit refactor rename-symbol --file target.lisp --from old --to new --plan --output json`
- Rename exact atom occurrences: `paredit refactor rename-symbol --file target.lisp --from old --to new`
- Plan a lexical local binding rename: `paredit refactor rename-binding --file target.lisp --path 0.3 --from old --to new --output json`
- Apply a reviewed local binding rename: `paredit refactor rename-binding --file target.lisp --path 0.3 --from old --to new --write`
- Plan a scoped rename inside one selected form: `paredit refactor rename-in-form --file target.lisp --path 0.3 --from old --to new --output json`
- Apply a reviewed scoped rename: `paredit refactor rename-in-form --file target.lisp --path 0.3 --from old --to new --write`
- Plan a callable definition rename before using raw atom rename: `paredit refactor rename-function --from old --to new --output json src/*.lisp elisp/*.el`
- Apply a reviewed callable rename across explicit files: `paredit refactor rename-function --from old --to new --write src/*.lisp elisp/*.el`
- Plan wrapper insertion around reviewed call sites: `paredit edit wrap-function-calls --function fetch-user --wrapper with-cache --all-calls --fail-on-no-change --require-calls 1 --output json src/*.lisp elisp/*.el`
- Apply wrapper insertion only after reviewing `calls`, `skippedAlreadyWrapped`, and `skippedNested`: `paredit edit wrap-function-calls --function fetch-user --wrapper with-cache --call-path 0.4 --write src/service.lisp`
- Plan removing one reviewed wrapper call while keeping a selected argument: `paredit refactor unwrap-call --file target.lisp --path 0.3 --function with-cache --argument-index 0 --output json`
- Apply wrapper removal only after checking `function`, `argumentSpan`, `replacement`, and `rewritten`: `paredit refactor unwrap-call --file target.lisp --path 0.3 --function with-cache --argument-index 0 --write`
- Convert nested calls into a reviewed thread-first or thread-last pipeline: `paredit refactor thread-expression --file target.clj --path 0 --style last --output json`
- Apply the reviewed thread pipeline only after checking `base`, `steps`, `replacement`, and dialect: `paredit refactor thread-expression --file target.clj --path 0 --style last --write`
- Convert a reviewed thread pipeline back into nested calls: `paredit refactor unthread-expression --file target.clj --path 0 --output json`
- Apply the reviewed unthread rewrite only after checking `operator`, `style`, `steps`, `replacement`, and dialect: `paredit refactor unthread-expression --file target.clj --path 0 --write`
- Plan a multi-file rename: `paredit refactor rename-symbols --from old --to new --output json src/*.lisp elisp/*.el`
- Apply a multi-file rename after review: `paredit refactor rename-symbols --from old --to new --write src/*.lisp elisp/*.el`
- Extract a selected expression into a helper with explicit parameters: `paredit refactor extract-function --file target.lisp --path 0.3 --name helper --param value --output json`
- Apply the reviewed helper extraction at an anchored top-level position: `paredit refactor extract-function --file target.lisp --path 0.3 --name helper --param value --insert before --anchor-path 2 --write`
- Extract a reviewed magic value or repeated expression into a top-level constant: `paredit refactor extract-constant --file target.lisp --path 0.3.1 --name +max-retries+ --output json`
- Apply the reviewed constant extraction at an anchored top-level position: `paredit refactor extract-constant --file target.lisp --path 0.3.1 --name +max-retries+ --insert before --anchor-path 2 --write`
- Plan a reviewed helper inline at one call site: `paredit refactor inline-function --file target.lisp --definition-path 0 --call-path 1.3 --output json`
- Plan a reviewed helper inline across all same-file calls: `paredit refactor inline-function --file target.lisp --definition-path 0 --all-calls --output json`
- Apply the reviewed helper inline and remove the selected definition only after every reported `calls` entry has been checked: `paredit refactor inline-function --file target.lisp --definition-path 0 --all-calls --remove-definition --write`
- Audit cross-file signature compatibility before rewriting required parameters, and fail empty or incompatible scans: `paredit inspect signature --symbol old --fail-on-mismatch --require-definitions 1 --require-calls 1 --output json src/*.lisp elisp/*.el`
- Plan a required function parameter addition across reviewed call sites: `paredit refactor add-function-parameter --file target.lisp --definition-path 0 --name context --argument '*context*' --call-path 1.3 --output json`
- Plan a required function parameter addition across all same-file calls: `paredit refactor add-function-parameter --file target.lisp --definition-path 0 --name context --argument '*context*' --all-calls --output json`
- Apply the reviewed parameter addition only after every reported `call_paths` entry has been checked: `paredit refactor add-function-parameter --file target.lisp --definition-path 0 --name context --argument '*context*' --all-calls --write`
- Plan a required function parameter reorder across reviewed call sites: `paredit refactor move-function-parameter --file target.lisp --definition-path 0 --name context --to-index 0 --call-path 1.3 --output json`
- Plan a required function parameter reorder across all same-file calls: `paredit refactor move-function-parameter --file target.lisp --definition-path 0 --name context --to-index 0 --all-calls --output json`
- Apply the reviewed parameter reorder only after checking `from_index`, `to_index`, `call_paths`, and `moved_arguments`: `paredit refactor move-function-parameter --file target.lisp --definition-path 0 --name context --to-index 0 --all-calls --write`
- Plan a required function parameter removal across reviewed call sites: `paredit refactor remove-function-parameter --file target.lisp --definition-path 0 --name context --call-path 1.3 --output json`
- Plan a required function parameter removal across all same-file calls: `paredit refactor remove-function-parameter --file target.lisp --definition-path 0 --name context --all-calls --output json`
- Apply the reviewed parameter removal only after checking `parameter_index`, `call_paths`, and `removed_arguments`: `paredit refactor remove-function-parameter --file target.lisp --definition-path 0 --name context --all-calls --write`
- Introduce a local binding for a selected expression: `paredit refactor introduce-let --file target.lisp --path 0.3.1 --name value --output json`
- Apply the reviewed local binding introduction: `paredit refactor introduce-let --file target.lisp --path 0.3.1 --name value --write`
- Audit local binding inline safety, optionally as a CI gate: `paredit inspect lets --file target.lisp --fail-on-duplicate-evaluation --fail-on-unused-binding --require-inlineable-bindings 1 --output json`
- Inline a reviewed single-binding local name: `paredit refactor inline-let --file target.lisp --path 0.3 --output json`
- Apply the reviewed local binding inline: `paredit refactor inline-let --file target.lisp --path 0.3 --write`
- Plan unused local binding removal: `paredit refactor remove-unused-binding --file target.lisp --path 0.3 --name unused --output json`
- Apply unused local binding removal only after value-drop review: `paredit refactor remove-unused-binding --file target.lisp --path 0.3 --name unused --allow-drop-value --write`
- Plan all unused local binding removals in a selected let form: `paredit refactor remove-unused-binding --file target.lisp --path 0.3 --all-bindings --output json`
- Apply reviewed all-binding cleanup only after checking every reported `bindings` entry: `paredit refactor remove-unused-binding --file target.lisp --path 0.3 --all-bindings --allow-drop-value --write`
- Inspect top-level forms: `paredit inspect outline --file target.lisp --output json`
- Build an agent planning payload: `paredit inspect agent-report --file target.lisp --output json`
- Inspect one selected form before scoped rewrites and review `head`, `symbols`, `span`, and `definitionLike`: `paredit inspect form --file target.lisp --path 0 --include-source --output json`
- Wrap an argument list: `paredit edit wrap --path 0.2`
- Inline a nested list: `paredit edit splice --path 0.3`
- Promote a child expression: `paredit edit raise --path 0.3.1`
- Move a following sibling into a list: `paredit edit slurp-forward --path 0`
- Move the last list child out: `paredit edit barf-forward --path 0`

## 2026 Common Lisp Refactoring Checklist

- Inspect `defpackage`, `in-package`, nicknames, `:use`, `:import-from`, and
  `:export` with `package-report` before changing package-qualified symbols.
- Inspect functions, macros, methods, classes, variables, modes, systems, and
  tests with `definition-report` before splitting files or consolidating APIs.
- Inspect callable list-head sites with `call-report` before callable rename,
  helper inline/extraction, or function arity changes; review `argumentCount`,
  `path`, span, dialect, and `enclosingDefinition` before choosing
  `--call-path` values or applying a multi-file plan.
- Inspect callable signatures with `signature-report` before adding, moving,
  or removing required parameters; use `--fail-on-mismatch`,
  `--require-definitions`, and `--require-calls` to make empty or incompatible
  scans fail in CI, and resolve `missing-arguments`, `extra-arguments`,
  `unknown-definition`, and `ambiguous-definition` results before applying
  same-file rewrite commands.
- Inspect dependency edges with `call-graph` before file decomposition,
  definition moves, public API cleanup, or dead-code removal; use
  `--fail-on-inbound-callers`, `--require-edges`, and
  `--require-internal-edges` to make unsafe or unexpectedly empty scans fail
  in CI, and use `--include-external` when external package, macro, or
  runtime dependencies affect the refactoring boundary.
- Inspect externally unreferenced definitions with `unused-definition-report`
  before dead-code removal; it excludes references inside the defining
  top-level form so recursive private definitions still need explicit review.
- Use `remove-unused-definitions` for reviewed bulk deletion of unreferenced
  non-protected definitions; Common Lisp `defpackage` exports are skipped
  unless `--include-exported` is explicitly passed, and package, system, test,
  customization, and mode definitions are skipped unless `--include-protected`
  is explicitly passed.
- Remove reviewed dead top-level definitions with `remove-definition`; run the
  JSON plan first, then apply `--write` only after confirming the path, name,
  category, span, and rewritten file.
- Move top-level definitions with `move-definition` after reviewing
  `definition-report`; it preserves balanced forms, supports new destination
  files, and reparses both source and destination before writing.
- Move non-definition top-level forms with `move-form` after reviewing
  `outline` or `agent-report`; use it for `defpackage`, `in-package`,
  `eval-when`, ASDF fragments, feature conditionals, or migration scaffolding.
  Prefer `--insert before/after --anchor-path PATH` when destination order
  affects package, compile-time, or load-time behavior.
- Use `add-export` after package review for public API additions; it updates an
  existing `:export`, creates one when absent, and no-ops for existing symbols.
- Use `rename-package` after package review for package name changes; inspect
  `defpackage-name`, `in-package-name`, `package-option`, and
  `qualified-prefix` occurrences before writing. It is package-aware and does
  not rewrite comments, strings, or ordinary same-named atoms.
- Use `duplicate-report` before macro extraction, helper extraction, and
  table-driven test refactors; compare repeated shapes with behavior coverage
  before abstracting.
- Use `paredit inspect similarity` when exact duplicate shapes miss
  near-duplicates; tune `--threshold`, `--min-node-count`, and
  `--comparison-scope` before proposing consolidation, and use
  `--fail-on-duplicates` as a CI gate.
- Use `extract-constant` for reviewed magic-value cleanup; run the JSON plan
  first, then apply `--write` with `--insert before/after --anchor-path` when
  constant placement affects compile or load order.
- Use `replacement-plan` after duplicate review to generate per-file
  `replace-forms` command batches; inspect `paths`, `replace_forms_args`, and
  original texts before applying a real replacement.
- Use `replace-forms` only after reviewing every target path; keep
  `--require-same-shape` on for batches copied from `duplicate-report`.
- Use `inline-function` as the inverse of `extract-function` when removing
  accidental abstraction. Review `parameters[].reference_count`,
  `replacement`, and the optional definition removal before writing.
- Use `add-function-parameter` for intentional signature changes only after
  enumerating explicit call paths or reviewing `--all-calls` output; do not
  use broad text replacement for function arity changes.
- Use `move-function-parameter` for intentional required parameter reordering
  only after reviewing `from_index`, `to_index`, `call_paths`, and every
  `moved_arguments` entry.
- Use `remove-function-parameter` for intentional signature cleanup only after
  reviewing `parameter_index`, `call_paths`, and every `removed_arguments`
  entry; keep `--allow-missing-argument` off unless legacy partial call sites
  were audited.
- Use `let-report` and `remove-unused-binding` for local dead-binding cleanup;
  enable `let-report` policy gates when agents or CI need fixed pass/fail
  checks for duplicate evaluation, unused bindings, or minimum inlineable
  candidates. Pass `--all-bindings` only after reviewing the selected `let`
  form, and write only with `--allow-drop-value` after checking that each
  removed value expression is not needed for side effects.
- Push reusable syntax into `defmacro` only when it reduces duplicated structure.
- Keep Prolog-style declarative facts separate from imperative execution logic.
- Prefer CPS only where it makes control flow explicit and testable.
- Delete dead code instead of maintaining backward compatibility shims.
- Split large files by coherent data, macro, logic, and test boundaries.
- Raise test abstraction while preserving concrete behavior coverage.
- Treat human readability as a verification target, not a post-processing step.
