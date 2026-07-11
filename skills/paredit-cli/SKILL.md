---
name: paredit-cli
description: This skill should be used when refactoring Common Lisp, Emacs Lisp, Scheme, Clojure, Janet, or Fennel source files, or any other Lisp-like S-expression code. Use when renaming symbols, functions, macrolet/flet/labels bindings, or packages; moving or splitting definitions between files; extracting, inlining, or reparametrizing functions; threading/unthreading call chains; or removing unused definitions. Use whenever an edit to balanced-parenthesis code is needed and the `paredit` binary is available, instead of hand-editing delimiters.
version: 1.1.0
---

<purpose>
  Provide usage patterns for `paredit`, a Rust CLI that performs structure-aware S-expression
  refactoring. It gives deterministic zero-based tree paths, byte spans, and dialect-aware
  balanced edits so Lisp refactors do not require manually counting or rewriting parentheses.
</purpose>

<overview>
  paredit detects the Lisp dialect (Common Lisp, Emacs Lisp, Scheme, Clojure, Janet, Fennel)
  from file extension or an explicit --dialect flag, and exposes every read and write
  operation as a separate subcommand under `paredit <command> ...`.

  The core rule: never hand-edit balanced delimiters during a refactor. Validate the file,
  locate the exact form or symbol with a report command, apply one structural edit, then
  validate again.

  Grouped entrypoints:
  - `paredit refactor ...` — plan, preview, verify, apply/diff flows for a rename or move.
  - `paredit inspect workspace ...` — directory-root discovery and inventory across many files.
</overview>

<command_groups>
  <group name="inspect">
    <description>Read-only structural inspection; always safe, never writes.</description>
    <command>paredit inspect check --file f.lisp</command>
    <command>paredit inspect dialect --file f.lisp</command>
    <command>paredit inspect stats --file f.lisp</command>
    <command>paredit inspect agent-report --file f.lisp --output json</command>
    <command>paredit inspect outline --file f.lisp --output json</command>
    <command>paredit inspect form --file f.lisp --path 0 --include-source --output json</command>
    <command>paredit edit select --file f.lisp --path 0.3</command>
    <command>paredit inspect workspace --output json .</command>
  </group>

  <group name="search_across_files">
    <description>Exact atom/call/definition inventories across explicit file lists; ignores comments and strings.</description>
    <command>paredit inspect find-symbol --file f.lisp --symbol old-name --output json</command>
    <command>paredit inspect symbols --symbol old-name --output json src/*.lisp elisp/*.el</command>
    <command>paredit inspect calls --symbol old-name --output json src/*.lisp</command>
    <command>paredit inspect signature --symbol old-name --require-definitions 1 --require-calls 1 --output json src/*.lisp</command>
    <command>paredit inspect call-graph --symbol old-name --require-edges 1 --output json src/*.lisp</command>
    <command>paredit inspect impact --symbol old-name --fail-on-risk-level warning --output json src/*.lisp</command>
    <command>paredit inspect definitions --output json src/*.lisp</command>
    <command>paredit inspect unused-definitions --fail-on-unused --output json src/*.lisp</command>
    <command>paredit inspect dependencies --output json system.asd src/*.lisp</command>
    <command>paredit inspect packages --output json system.asd src/*.lisp</command>
    <command>paredit inspect duplicates --output json src/*.lisp</command>
    <command>paredit inspect similarity --threshold 0.87 --output json src</command>
  </group>

  <group name="rename">
    <description>Rename operations, narrowest scope first. Prefer the scope-aware command over the generic atom rename when the binding kind is known.</description>
    <command>paredit refactor rename-symbol --file f.lisp --from old --to new --plan --output json</command>
    <command>paredit refactor rename-in-form --file f.lisp --path 0.3 --from old --to new --output json</command>
    <command>paredit refactor rename-binding --file f.lisp --path 0.3 --from old --to new --output json</command>
    <command>paredit refactor rename-function --from old --to new --output json src/*.lisp elisp/*.el</command>
    <command>paredit refactor rename-local-function --from old --to new --output json src/*.lisp</command>
    <command>paredit refactor rename-macrolet --from old --to new --output json src/*.lisp</command>
    <command>paredit refactor rename-symbol-macro --from old --to new --output json src/*.lisp</command>
    <command>paredit refactor rename-package --from old.pkg --to new.pkg --output json system.asd src/*.lisp</command>
    <command>paredit refactor plan --symbol old --operation rename --fail-on-blocking-gate --require-definitions 1 --require-references 1 --output json src/*.lisp</command>
    <command>paredit refactor workspace-plan --symbol old --output json .</command>
  </group>

  <group name="move_and_organize">
    <description>Relocate or reshape top-level structure across files.</description>
    <command>paredit refactor move-definition --from-file a.lisp --to-file b.lisp --path 2 --output json</command>
    <command>paredit refactor move-form --from-file a.lisp --to-file b.lisp --path 2 --insert before --anchor-path 1 --output json</command>
    <command>paredit refactor split-file --from-file a.lisp --to-file b.lisp --path 2 --path 5 --output json</command>
    <command>paredit refactor sort-definitions --file a.lisp --output json</command>
    <command>paredit refactor remove-definition --file a.lisp --path 2 --output json</command>
    <command>paredit refactor remove-unused-definitions --output json system.asd src/*.lisp</command>
    <command>paredit refactor replacement-plan --replacement '(run-case)' --output json src/*.lisp</command>
    <command>paredit edit replace-forms --file a.lisp --path 0 --path 1 --with '(run-case)' --require-same-shape --output json</command>
  </group>

  <group name="function_shape">
    <description>Function-level refactors: signature changes propagate to explicit call sites.</description>
    <command>paredit refactor extract-function --file f.lisp --path 0.3 --name helper --param value --output json</command>
    <command>paredit refactor extract-constant --file f.lisp --path 0.3.1 --name +limit+ --output json</command>
    <command>paredit refactor inline-function --file f.lisp --definition-path 0 --call-path 1.3 --output json</command>
    <command>paredit refactor add-function-parameter --file f.lisp --definition-path 0 --name ctx --argument '*ctx*' --all-calls --output json</command>
    <command>paredit refactor move-function-parameter --file f.lisp --definition-path 0 --name ctx --to-index 0 --all-calls --output json</command>
    <command>paredit refactor swap-function-parameters --file f.lisp --definition-path 0 --left-name a --right-name b --all-calls --output json</command>
    <command>paredit refactor reorder-function-parameters --file f.lisp --definition-path 0 --parameter b --parameter a --all-calls --output json</command>
    <command>paredit refactor remove-function-parameter --file f.lisp --definition-path 0 --name ctx --all-calls --output json</command>
    <command>paredit refactor thread-expression --file f.clj --path 0 --style last --output json</command>
    <command>paredit refactor unthread-expression --file f.clj --path 0 --output json</command>
    <command>paredit refactor introduce-let --file f.lisp --path 0.3.1 --name value --output json</command>
    <command>paredit refactor inline-let --file f.lisp --path 0.3.1 --output json</command>
    <command>paredit inspect lets --output json f.lisp</command>
    <command>paredit inspect lets --output json src/*.lisp</command>
  </group>

  <group name="structural_primitives">
    <description>
      Low-level paredit-style structural edits on one selected form (--path or --at).
      None of these accept --write: each prints the whole rewritten document to stdout.
      Review the output, then redirect into the file, for example
      `paredit edit wrap --file f.lisp --path 0.3 > /tmp/out && mv /tmp/out f.lisp`.
    </description>
    <command>paredit edit format --file f.lisp</command>
    <command>paredit edit replace --file f.lisp --path 0.3 --with '(new-form)'</command>
    <command>paredit edit kill --file f.lisp --path 0.3</command>
    <command>paredit edit wrap --file f.lisp --path 0.3</command>
    <command>paredit edit splice --file f.lisp --path 0.3</command>
    <command>paredit edit raise --file f.lisp --path 0.3</command>
    <command>paredit edit slurp-forward --file f.lisp --path 0.3</command>
    <command>paredit edit slurp-backward --file f.lisp --path 0.3</command>
    <command>paredit edit barf-forward --file f.lisp --path 0.3</command>
    <command>paredit edit barf-backward --file f.lisp --path 0.3</command>
  </group>
</command_groups>

<patterns>
  <pattern name="rename_symbol_across_workspace">
    <description>Canonical plan-preview-verify-write loop for a cross-file rename.</description>
    <example>
paredit refactor plan --symbol old-name --operation rename --fail-on-blocking-gate --require-definitions 1 --require-references 1 --output json src/*.lisp elisp/*.el
paredit refactor preview --from old-name --to new-name --mode function --fail-on-no-change --fail-on-parse-error --fail-on-target-conflict --require-edits 1 --output json src/*.lisp elisp/*.el
paredit refactor verify --symbol old-name --operation rename --phase pre --output json src/*.lisp elisp/*.el
paredit refactor preview --from old-name --to new-name --mode function --fail-on-no-change --fail-on-parse-error --fail-on-target-conflict --require-edits 1 --write --output json src/*.lisp elisp/*.el
paredit refactor verify --symbol old-name --new-symbol new-name --operation rename --phase post --output json src/*.lisp elisp/*.el
    </example>
  </pattern>

  <pattern name="manifest_based_apply">
    <description>Hash-guarded apply flow when the preview is saved as a manifest file, for larger or riskier edits.</description>
    <example>
paredit refactor preview --from old-name --to new-name --mode function --fail-on-no-change --output json src/*.lisp > rename.preview.json
paredit refactor status --manifest rename.preview.json --root . --output json
HASH=$(paredit refactor status --manifest rename.preview.json --root . --output json | jq -r '.manifest.hash')
paredit refactor diff --manifest rename.preview.json --expect-manifest-hash "$HASH" --root . --output json
paredit refactor apply --manifest rename.preview.json --expect-manifest-hash "$HASH" --root . --write --output json
    </example>
  </pattern>

  <pattern name="whole_directory_rename">
    <description>Discover sources under one or more roots instead of enumerating files explicitly.</description>
    <example>
paredit inspect workspace --output json .
paredit refactor workspace-plan --symbol old-name --fail-on-blocking-gate --output json .
paredit refactor workspace-preview --from old-name --to new-name --fail-on-no-change --output json .
paredit refactor workspace-execute --from old-name --to new-name --write --output json .
    </example>
  </pattern>

  <pattern name="safe_removal">
    <description>Only remove definitions after confirming they have no external references.</description>
    <example>
paredit inspect unused-definitions --fail-on-unused --output json system.asd src/*.lisp
paredit refactor remove-unused-definitions --output json system.asd src/*.lisp
paredit refactor remove-unused-definitions --write system.asd src/*.lisp
    </example>
  </pattern>
</patterns>

<decision_tree name="which_command_family">
  <question>What is the refactor goal?</question>
  <branch condition="Just need to know what's in a file or directory">Use inspect group (check, outline, form-report, workspace report)</branch>
  <branch condition="Renaming something and the binding kind is known (function, local var, macrolet, package)">Use the specific rename-* command; fall back to rename-symbol only for a plain atom with no scope semantics</branch>
  <branch condition="Renaming or moving across many files at once">Use `paredit refactor plan/preview/verify` (explicit files) or `workspace-plan/workspace-preview/workspace-execute` (directory roots)</branch>
  <branch condition="Changing a function's parameter list">Use add/move/swap/reorder/remove-function-parameter, always with --all-calls or explicit --call-path</branch>
  <branch condition="Relocating top-level forms">Use move-definition, move-form, or split-file</branch>
  <branch condition="One-off structural edit at a specific path">Use a structural primitive (replace, wrap, splice, raise, slurp/barf)</branch>
  <branch condition="Consolidating duplicated or near-duplicated code">Use `inspect duplicates` for exact shapes and `inspect similarity` for near-duplicates, then replacement-plan/replace-forms or extract-function/extract-constant</branch>
  <branch condition="Deleting dead code">Use unused-definition-report first, only then remove-unused-definitions --write</branch>
</decision_tree>

<best_practices>
  <practice priority="critical">Run a plan/preview command without --write first, inspect the JSON, then re-run with --write</practice>
  <practice priority="critical">Prefer --output json for anything other than a single human-inspected file; it is the stable, parseable contract</practice>
  <practice priority="high">Use --path for deterministic scripted edits; use --at (byte offset) when a prior report or grep result already gives an offset</practice>
  <practice priority="high">Use --fail-on-* and --require-* gates (e.g. --fail-on-blocking-gate, --require-definitions 1) so a plan command exits non-zero instead of silently under-matching</practice>
  <practice priority="medium">Run refactor verify with --phase pre before editing and --phase post after, to catch regressions the preview step could not predict</practice>
  <practice priority="medium">Wrap every invocation in a command timeout in automated pipelines; a hang should not block the surrounding agent loop</practice>
</best_practices>

<anti_patterns>
  <avoid name="hand_edited_delimiters">
    <description>Manually adding/removing parentheses, brackets, or quotes to fix a refactor by hand</description>
    <instead>Use the matching structural primitive (wrap, splice, raise, slurp/barf) or a rename/move command</instead>
  </avoid>
  <avoid name="write_without_preview">
    <description>Passing --write on the first invocation of a plan/preview command</description>
    <instead>Run without --write, review the JSON output (edits, gates, risk level), then re-run with --write</instead>
  </avoid>
  <avoid name="generic_rename_for_scoped_binding">
    <description>Using rename-symbol on a flet/labels/macrolet/symbol-macrolet binding or a package-qualified name</description>
    <instead>Use rename-local-function, rename-macrolet, rename-symbol-macro, or rename-package so shadowing and lexical scope are respected</instead>
  </avoid>
  <avoid name="removal_without_reference_check">
    <description>Deleting a definition because it looks unused without checking call sites</description>
    <instead>Run unused-definition-report (or symbol-report/call-report) across every relevant file first</instead>
  </avoid>
</anti_patterns>

<rules priority="critical">
  <rule>Never hand-edit balanced delimiters; every structural change goes through a paredit subcommand</rule>
  <rule>Always validate with `paredit inspect check` before and after a batch of edits</rule>
  <rule>Never pass --write until a no-write preview/plan has been reviewed</rule>
</rules>

<rules priority="standard">
  <rule>Use --path for scripted, deterministic targeting; reserve --at for offsets sourced from another tool's output</rule>
  <rule>Keep structural edits (paredit) and semantic/logic rewrites (hand-written code changes) in separate steps</rule>
  <rule>Use workspace-* commands for directory roots and refactor plan/preview/verify for explicit file lists; do not mix the two styles in one step</rule>
</rules>

<workflow>
  <phase name="inspect">
    <objective>Establish ground truth about the current file or workspace before changing anything</objective>
    <step order="1">Run `paredit inspect check` on every target file</step>
    <step order="2">Run outline/form-report or workspace report to get paths and spans</step>
    <step order="3">Run the relevant *-report command (symbol, call, signature, impact, unused-definition) to see the full blast radius</step>
  </phase>
  <phase name="plan_and_preview">
    <objective>Produce a reviewable, no-write description of the exact edit</objective>
    <step order="1">Choose the narrowest command that matches the binding kind (rename-function vs rename-symbol, etc.)</step>
    <step order="2">Run the command's plan/preview form with --fail-on-* and --require-* gates and --output json</step>
    <step order="3">Read the JSON: confirm edit count, risk level, and gate results before proceeding</step>
  </phase>
  <phase name="apply_and_verify">
    <objective>Write the change and confirm nothing else broke</objective>
    <step order="1">Re-run the same command with --write (or refactor apply with the previewed manifest hash)</step>
    <step order="2">Run `paredit inspect check` again on every touched file</step>
    <step order="3">Run `paredit refactor verify --phase post` (or the equivalent report command) to confirm the rename/move is complete and consistent</step>
  </phase>
</workflow>

<error_escalation>
  <examples>
    <example severity="low">A report command returns zero matches for a symbol expected to exist — check the dialect/extension and file list first</example>
    <example severity="medium">Preview shows more or fewer edits than expected — narrow the command (specific rename-* variant) or the file list before writing</example>
    <example severity="high">--fail-on-blocking-gate or --fail-on-target-conflict trips — stop and inspect the JSON gate reason instead of forcing --write</example>
    <example severity="critical">`paredit inspect check` fails after a --write — the file is unbalanced; do not run further paredit commands against it until a human confirms recovery</example>
  </examples>
</error_escalation>

<constraints>
  <must>Run `paredit inspect check --file &lt;f&gt;` before and after any batch of structural edits to that file</must>
  <must>Preview (no --write) before ever passing --write</must>
  <must>Use the scope-aware rename-* command when the binding kind (function, local function, macrolet, symbol-macro, package) is known</must>
  <avoid>Hand-editing parentheses, brackets, or quoting to "fix" a refactor</avoid>
  <avoid>Passing --write on a first, unreviewed invocation</avoid>
  <avoid>Deleting definitions without an unused-definition-report or reference check</avoid>
</constraints>
