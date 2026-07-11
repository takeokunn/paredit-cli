# paredit-cli

[![CI](https://github.com/takeokunn/paredit-cli/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/takeokunn/paredit-cli/actions/workflows/ci.yml)
![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)

`paredit-cli` is a Rust command line tool for safe S-expression refactoring.
It gives AI coding agents deterministic tree paths, byte spans, dialect hints,
and balanced structural edits so Lisp refactors do not devolve into manual
parenthesis surgery.

The core rule is: do not rewrite delimiters by hand. Validate the file, locate
the exact form or symbol, apply a structural edit, then validate again.

## Installation

Build from a checkout:

```sh
nix develop -c cargo install --path . --locked
```

Install directly from GitHub:

```sh
cargo install --git https://github.com/takeokunn/paredit-cli --locked
```

Pin a released line for stable automation:

```sh
cargo install --git https://github.com/takeokunn/paredit-cli --tag v0.1.1 --locked
```

Run without installing (Nix):

```sh
nix run github:takeokunn/paredit-cli -- check --file source.lisp
```

The current minimum supported Rust version is `1.85`.

## Quickstart

Start with a read-only inspection pass:

```sh
paredit check --file source.lisp
paredit workspace report --output json .
paredit form-report --file source.lisp --path 0 --include-source --output json
```

For rename-style refactors, keep the workflow plan-first and write-last:

```sh
paredit refactor plan --symbol old-name --operation rename --fail-on-blocking-gate --require-definitions 1 --require-references 1 --output json src/*.lisp lisp/*.el
paredit refactor preview --from old-name --to new-name --mode function --fail-on-no-change --fail-on-parse-error --fail-on-target-conflict --require-definitions 1 --require-edits 1 --output json src/*.lisp lisp/*.el
paredit refactor verify --symbol old-name --new-symbol new-name --operation rename --phase post --output json src/*.lisp lisp/*.el
```

Only add `--write` after reviewing the JSON output and confirming that policy
gates, parse checks, and target-file scope are all correct.

Use the grouped entrypoints `paredit refactor ...` and
`paredit workspace ...` as the canonical automation surface.

## Lint and Format Integration

`paredit-cli` ships as a reusable structural linter and canonical formatter
for any repository that contains Lisp-family sources. Both gates are
read-only unless you explicitly opt into rewriting.

- `paredit-lint` fails when any discovered Lisp source has structural parse
  errors, and emits GitHub error annotations in CI.
- `paredit-format --check` fails when any parsed source differs from the
  canonical `paredit format` rendering; without `--check` it rewrites files
  in place.

### GitHub Actions

Use the bundled composite action from any workflow. Prebuilt binaries are
pulled from the public `takeokunn-paredit-cli` Cachix cache:

```yaml
jobs:
  paredit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v7
      - uses: takeokunn/paredit-cli@v0.1.1
        with:
          mode: lint
          paths: src
      - uses: takeokunn/paredit-cli@v0.1.1
        with:
          mode: format
          paths: src
```

Action inputs: `mode` (`lint`, `format`, or `fix`), `paths` (default `.`),
`version` (defaults to the action ref), and `cachix-name`.

### Nix Run

```sh
nix run github:takeokunn/paredit-cli#lint -- .
nix run github:takeokunn/paredit-cli#format -- --check .
nix run github:takeokunn/paredit-cli#format -- .
```

### Flake Integration

Add the flake input and reuse the packaged tools, the overlay, or the
ready-made flake checks:

```nix
{
  inputs.paredit-cli.url = "github:takeokunn/paredit-cli";

  outputs = { self, nixpkgs, paredit-cli, ... }: {
    # Overlay: provides pkgs.paredit-cli, pkgs.paredit-lint,
    # pkgs.paredit-format, and pkgs.paredit-format-files.
    # nixpkgs.overlays = [ paredit-cli.overlays.default ];

    checks.x86_64-linux = {
      paredit-lint = paredit-cli.lib.x86_64-linux.mkLintCheck { src = ./.; };
      paredit-format = paredit-cli.lib.x86_64-linux.mkFormatCheck { src = ./.; };
    };
  };
}
```

### treefmt-nix

Register paredit as a formatter for Lisp sources in a
[treefmt-nix](https://github.com/numtide/treefmt-nix) configuration; this
repository formats itself the same way:

```nix
treefmt-nix.lib.evalModule pkgs {
  projectRootFile = "flake.nix";
  settings.formatter.paredit = paredit-cli.lib.${system}.treefmtFormatter;
}
```

## Stability and Support

- `main` is the active development line. Behavior on `main` may change
  between releases as parser, refactor, and policy surfaces are tightened;
  pin the latest release tag (starting with `v0.1.0`) for stable automation.
- Machine-facing CLI and JSON compatibility are defined in
  [COMPATIBILITY.md](COMPATIBILITY.md), including how released contracts differ
  from unreleased `main`.
- User-visible behavior changes are tracked in [CHANGELOG.md](CHANGELOG.md).
- Security-sensitive reports and the currently supported release line are
  defined in [SECURITY.md](SECURITY.md).
- Usage questions, bug reports, and reproduction expectations belong in
  [SUPPORT.md](SUPPORT.md).

## Verification Model

- Pull requests run `nix flake check`, including workflow linting,
  formatting, clippy, nextest, and package build/tests. The `cargo publish
  --dry-run` step needs network access the Nix sandbox denies on CI, so it
  stays a local pre-release step (see [RELEASE.md](RELEASE.md)).
- The declared MSRV is part of the public contract. Until CI grows a dedicated
  MSRV lane, verify it locally with `cargo +1.85 test --locked` before release
  or when changing parser, refactor, packaging, or public API surfaces.
- Use the workflow page for the current CI run history; do not treat the badge
  alone as release evidence.
- Treat that automation as a baseline signal, not as complete release proof.
- Release readiness still requires the broader local verification loop in
  [CONTRIBUTING.md](CONTRIBUTING.md) and the maintainer checklist in
  [RELEASE.md](RELEASE.md), including tests, docs, packaging, and smoke
  checks.

## Project Documents

Pick the document that matches the decision you need to make:

- Users and integrators: [COMPATIBILITY.md](COMPATIBILITY.md),
  [CHANGELOG.md](CHANGELOG.md), [SECURITY.md](SECURITY.md),
  [SUPPORT.md](SUPPORT.md), [LICENSE](LICENSE),
  [API docs](https://docs.rs/paredit-cli)
- AI coding agents: [SKILLS.md](SKILLS.md) for the refactoring playbook and
  [skills/paredit-cli/SKILL.md](skills/paredit-cli/SKILL.md) for an
  installable agent skill definition
- Contributors: [CONTRIBUTING.md](CONTRIBUTING.md), [ROADMAP.md](ROADMAP.md),
  [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md)
- Maintainers and release reviewers: [GOVERNANCE.md](GOVERNANCE.md),
  [MAINTAINERS.md](MAINTAINERS.md), [RELEASE.md](RELEASE.md)

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

### Common Lisp Scope-Aware Rename Semantics

`paredit-cli` is not limited to text-level symbol replacement. Its Common Lisp
rename commands model lexical scope and callable namespaces so agents can see
which references are supposed to move and which must stay untouched.

- `rename-binding` follows lexical bindings through `let`-style forms, lambda
  lists, and destructuring while stopping at shadowing boundaries.
- `rename-local-function` distinguishes `flet` from `labels`: `labels`
  references inside local function bodies rename, while `flet` definition
  bodies keep outer visibility rules.
- `rename-function` follows Common Lisp callable designators such as
  `function`, `macro-function`, `compiler-macro-function`, `symbol-function`,
  `fdefinition`, reader-prefix forms such as `#'`, and `setf` callable names
  like `(setf accessor)`, while still skipping quoted data and arbitrary
  values.
- `rename-function` also keeps `defmacro` and `define-compiler-macro`
  definitions traversable inside reader-quoted lambda bodies, but treats
  `macrolet` and `compiler-macrolet` as scope boundaries so expander-local
  bodies keep their own shadowing rules.
- `rename-macrolet` renames local `macrolet` and `compiler-macrolet`
  bindings, including qualified forms such as `cl:macrolet` and
  `cl-user:compiler-macrolet`, while keeping expander bodies out of scope so
  only in-form call sites move. Quasiquote, unquote, and unquote-splicing are
  tracked explicitly so macro-introduced references do not get rewritten as if
  they were ordinary code.
- `rename-symbol-macro` and outer-binding renames across `symbol-macrolet`
  keep expansion references and body references separate, so shadowed body
  atoms do not move just because an expansion mentions the same symbol.
- Emacs Lisp support includes semantic handling for `cl-defun`, `cl-defmacro`,
  `cl-defgeneric`, `cl-defmethod`, and `defsubst` in addition to the usual
  `defun`-style entry points.

### Common Lisp Support Matrix

`paredit-cli` treats Common Lisp refactors as semantic operations, not raw text
edits. The current Common Lisp coverage is organized as follows:

- `rename-function`: top-level callable definitions and callable designators
  for `defun`, `defmacro`, `defgeneric`, `defmethod`,
  `define-method-combination`, `define-compiler-macro`,
  `define-setf-expander`, `function`, `macro-function`,
  `compiler-macro-function`, `symbol-function`, `fdefinition`, `#'`, and
  `(setf accessor)` forms.
- `rename-local-function`: lexical callable bindings introduced by `flet` and
  `labels`.
- `rename-macrolet`: local macro and compiler-macro bindings introduced by
  `macrolet` and `compiler-macrolet`.
- `rename-symbol-macro`: symbol macro bindings introduced by
  `define-symbol-macro` and `symbol-macrolet`.
- Scope boundaries: quoted data, arbitrary values, and expander-local bodies
  stay out of the generic traversal path so the tool can keep shadowing and
  expansion semantics intact.

```lisp
;; `labels` local calls rename inside definition bodies and the outer body.
(labels ((visit (node)
           (when node
             (visit (cdr node)))))
  (visit tree))
```

```lisp
;; `symbol-macrolet` expansion references and body references are distinct.
(let ((value 1))
  (symbol-macrolet ((current (compute value)))
    current))
```

## Commands

```sh
paredit check --file file.lisp
paredit dialect --file init.el
paredit stats --file system.asd --output json
paredit agent-report --file source.lisp --output json
paredit workspace --help
paredit workspace report --output json .
paredit refactor workspace-plan --symbol render-pane --operation rename --fail-on-blocking-gate --require-definitions 1 --require-references 1 --output json .
paredit outline --file source.lisp --output json
paredit form-report --file source.lisp --path 0 --include-source --output json
paredit find-symbol --file source.lisp --symbol old-name --output json
paredit symbol-report --symbol old-name --output json src/*.lisp lisp/*.el
paredit call-report --symbol render-pane --output json src/*.lisp lisp/*.el
paredit signature-report --symbol render-pane --fail-on-mismatch --require-definitions 1 --require-calls 1 --output json src/*.lisp lisp/*.el
paredit call-graph --symbol render-pane --fail-on-inbound-callers --require-edges 1 --require-internal-edges 1 --output json src/*.lisp lisp/*.el
paredit impact-report --symbol render-pane --fail-on-risk-level warning --require-definitions 1 --require-references 1 --require-calls 1 --output json src/*.lisp lisp/*.el
paredit refactor --help
paredit refactor plan --symbol render-pane --operation rename --fail-on-blocking-gate --require-definitions 1 --require-references 1 --output json src/*.lisp lisp/*.el
paredit refactor workspace-preview --from render-pane --to paint-pane --mode function --fail-on-no-change --fail-on-parse-error --fail-on-target-conflict --require-changed-files 1 --require-definitions 1 --require-edits 1 --output json .
paredit refactor preview --from render-pane --to paint-pane --mode function --fail-on-no-change --fail-on-parse-error --fail-on-target-conflict --require-definitions 1 --require-edits 1 --output json src/*.lisp lisp/*.el
paredit refactor check --manifest rename.preview.json --root . --output json
paredit refactor status --manifest rename.preview.json --root . --output json
HASH=<manifest.hash from refactor status JSON>
paredit refactor diff --manifest rename.preview.json --expect-manifest-hash "$HASH" --root . --output json
paredit refactor apply --manifest rename.preview.json --expect-manifest-hash "$HASH" --root . --output json
paredit refactor apply --manifest rename.preview.json --expect-manifest-hash "$HASH" --root . --write --output json
paredit refactor workspace-execute --from render-pane --to paint-pane --mode function --fail-on-no-change --fail-on-parse-error --fail-on-target-conflict --require-changed-files 1 --require-definitions 1 --require-edits 1 --output json .
paredit refactor workspace-execute --from render-pane --to paint-pane --mode function --write --fail-on-no-change --fail-on-parse-error --fail-on-target-conflict --require-changed-files 1 --require-definitions 1 --require-edits 1 --output json .
paredit refactor preview --from render-pane --to paint-pane --mode function --fail-on-no-change --fail-on-parse-error --fail-on-target-conflict --require-definitions 1 --require-edits 1 --write --output json src/*.lisp lisp/*.el
paredit refactor verify --symbol render-pane --new-symbol paint-pane --operation rename --phase post --output json src/*.lisp lisp/*.el
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
paredit sort-package-exports --file src/package.lisp --package demo
paredit sort-package-options --file src/package.lisp --package demo
paredit merge-package-options --file src/package.lisp --package demo
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
paredit rename-local-function --from old-name --to new-name --output json src/*.lisp lisp/*.el
paredit rename-local-function --from old-name --to new-name --write src/*.lisp lisp/*.el
paredit rename-macrolet --from old-name --to new-name --output json src/*.lisp lisp/*.el
paredit rename-macrolet --from old-name --to new-name --write src/*.lisp lisp/*.el
paredit rename-symbol-macro --from old-name --to new-name --output json src/*.lisp lisp/*.el
paredit rename-symbol-macro --from old-name --to new-name --write src/*.lisp lisp/*.el
paredit replace-function-calls --from fetch-user --to load-user --all-calls --output json src/service.lisp
paredit wrap-function-calls --function fetch-user --wrapper with-cache --all-calls --output json src/*.lisp lisp/*.el
paredit unwrap-function-calls --function fetch-user --wrapper with-cache --all-calls --output json src/service.lisp
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
paredit swap-function-parameters --file source.lisp --definition-path 0 --left-name width --right-name height --call-path 1.3 --output json
paredit swap-function-parameters --file source.lisp --definition-path 0 --left-name width --right-name height --all-calls --write
paredit reorder-function-parameters --file source.lisp --definition-path 0 --parameter height --parameter width --parameter scale --call-path 1.3 --output json
paredit remove-function-parameter --file source.lisp --definition-path 0 --name context --call-path 1.3 --output json
paredit remove-function-parameter --file source.lisp --definition-path 0 --name context --all-calls --write
paredit sort-definitions --file source.lisp --output json
paredit sort-definitions --file source.lisp --write --output json
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

The top-level command list above mirrors the canonical public surface exposed by
`paredit --help`. Use `paredit workspace ...` for repository inventory and
workspace discovery, and use `paredit refactor ...` for gated refactor plans,
previews, verification, diffs, and apply flows.

Most single-file structural commands accept stdin when `--file` is omitted.
Commands that operate on explicit file lists, and package-definition commands
such as `sort-package-exports`, still require concrete file arguments.

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

1. Run `paredit workspace report --output json .` from the repository root to
   discover Lisp files, dialects, parse errors, definition counts, and call
   counts before choosing a refactor boundary. Review `skipped` counts when
   generated, hidden, or unknown-extension files may be relevant.
1. Run `paredit refactor workspace-plan --symbol old --operation rename
   --fail-on-blocking-gate --require-definitions 1 --require-references 1
   --output json .` when an agent should discover Lisp files from repository
   roots before producing the gated refactor plan.
1. Run `paredit refactor workspace-preview --from old --to new --mode
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
1. Treat the command snippets below as intent-focused guides. When a snippet
   omits required file, symbol, package, or path arguments, use the complete
   examples in [Commands](#commands) or run the same subcommand with `--help`
   before copying it into automation.
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
1. Use `paredit refactor plan --symbol old --operation rename
   --fail-on-blocking-gate --require-definitions 1 --require-references 1
   --output json` for an agent-ready preflight that combines impact gates,
   dependency-report reminders, safe-to-automate status, ordered commands, and
   CI-friendly policy failures for rename, remove, move, or signature
   refactors.
1. Use `paredit refactor preview --from old --to new --mode function
   --fail-on-no-change --fail-on-parse-error --fail-on-target-conflict
   --require-definitions 1 --require-edits 1 --output json`
   before write-mode refactors to inspect exact per-file rewrites, byte-span
   edit scripts, stable content hashes, output parse status, replacement-symbol
   conflict counts, and CI-friendly policy failures without modifying files.
1. Save preview JSON and run `paredit refactor check --manifest
   rename.preview.json --root . --output json` when CI or an AI agent needs a
   cheap manifest health gate without rendering a diff. JSON output includes
   `manifest.path`, `manifest.hash`, `manifest_policy_passed`,
   `manifest_outputs_parse`, `summary.can_apply`, per-file hash/parse checks,
   and the `root` audit object.
1. Save preview JSON and run `paredit refactor status --manifest
   rename.preview.json --root . --output json` when an AI coding agent needs a
   non-failing decision response before choosing the next tool call. JSON
   output includes `status`, `next_action`, `blocked_reasons`, `write_plan`,
   `manifest.hash`, `summary.can_apply`, per-file hash/parse checks, and the
   `root` audit object. Use `refactor check` for CI gating and
   `refactor status` for agent branching.
1. Save preview JSON and run `paredit refactor diff --manifest
   rename.preview.json --expect-manifest-hash "$HASH" --root . --output json`
   to render a machine-readable unified diff from the same byte-span edits
   while rechecking the pinned manifest hash, input hashes, output hashes, parse
   status, manifest consistency, and workspace-root containment without writing
   files. JSON output includes a `root` audit object showing whether containment
   was enforced and which canonical root was used.
1. Save preview JSON and run `paredit refactor apply --manifest
   rename.preview.json --expect-manifest-hash "$HASH" --root . --output json`
   for a second dry-run validation pass. Add `--write` only after the manifest
   hash pin, manifest policy, input hashes, output hashes, rewritten parse
   status, manifest consistency, and root containment all pass. JSON output
   includes a `root` audit object for CI and agent logs. This is the safer
   AI-agent path because stale source files, modified manifests, or out-of-root
   manifest paths cannot be rewritten from an old manifest.
1. Use `paredit refactor workspace-execute --from old --to new --mode
   function --output json .` when an agent wants the preview policy checks,
   optional write step, and post-write verification in one command. Dry-run
   output includes `preflight_decision`, `execute_decision`, `write_plan`,
   scheduled verification steps, and the next action to take. Add `--write`
   only after reviewing the execute decision and changed-file summary.
1. Use `paredit refactor verify --symbol old --operation rename --phase pre
   --output json` before edits and `paredit refactor verify --symbol old
   --new-symbol new --operation rename --phase post --output json` after edits
   to produce fixed pass/fail checks for AI coding agents and CI gates.
1. Use `paredit add-export --output json` to plan a public API export after
   package review. The command updates an existing `:export`, creates one when
   missing, no-ops when the symbol is already exported, and reparses before
   `--write`.
1. Use `paredit sort-package-exports --output json` after package review to
   canonicalize one Common Lisp `:export` option without changing the rest of
   the `defpackage` form. Use `paredit sort-package-options --output json` to
   normalize option block order, and `paredit merge-package-options --output
   json` to collapse duplicate `:export`, `:import-from`, and similar option
   heads before `--write`.
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
   before changing positional parameters or adding reviewed new
   parameters. It joins callable definitions with
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
   (`defun`, `defmacro`, `defgeneric`, `defmethod`, `define-method-combination`,
   `define-compiler-macro`, `define-setf-expander`, and dialect equivalents).
   It rewrites definition names, Common Lisp callable designators such as `function`,
   `macro-function`, `compiler-macro-function`, `symbol-function`,
   `fdefinition`, and `setf` callable names like `(setf accessor)`, and
   list-head call sites, but does not rewrite arbitrary value references.
1. Use `paredit rename-local-function --output json` for local callable
   bindings such as `flet` and `labels`, including qualified forms like
   `cl:flet` and `cl-user:labels`. It rewrites the binding name and local call
   sites, but keeps expansion bodies and non-call references untouched.
1. Use `paredit rename-macrolet --output json` for `macrolet` and
   `compiler-macrolet` bindings, including qualified forms such as
   `cl:macrolet` and `cl-user:compiler-macrolet`. It rewrites the binding name
   and call sites, but not symbols inside the expander body.
1. Use `paredit rename-symbol-macro --output json` for
   `define-symbol-macro` bindings, including qualified forms such as
   `cl:define-symbol-macro` and `cl-user:define-symbol-macro`. It rewrites the
   binding name and value references, but respects lexical shadowing.
1. Refactor plans classify Lisp definition kinds separately so macro-like
   targets can skip function-signature compatibility gates when that would be a
   false positive. The JSON `target_kind` field includes `macro`,
   `compiler_macro`, `setf_expander`, and `symbol_macro` where appropriate.
1. Use `paredit wrap-function-calls --output json` when a refactor needs to
   introduce a wrapper macro or helper around reviewed call sites. Pass either
   `--all-calls` or repeated `--call-path`; review `calls`,
   `skippedAlreadyWrapped`, `skippedNested`, and policy fields before applying
   `--write`.
1. Use `paredit replace-function-calls --output json` when only callable
   list-head names should change while definitions, strings, comments, and
   value references stay untouched. Pass either `--all-calls` or repeated
   `--call-path`, then review `callCount`, each targeted `path`, and the
   rewritten call heads before `--write`.
1. Use `paredit unwrap-function-calls --output json` when a reviewed wrapper
   such as `with-cache` should be removed only around matching callable sites.
   Review `callCount`, `skippedNonUnaryWrapperCount`,
   `skippedNestedCount`, and the rewritten output before applying `--write`.
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
1. Add function parameters with
   `paredit add-function-parameter --output json` first. Review the selected
   definition, every explicit or discovered `call_paths` entry, and the
   inserted argument before applying `--write`. Use `--parameter-section` when
   the target belongs in a reviewed `&optional` or `&key` section. Run
   `signature-report` across the broader explicit file set first when callers
   can exist outside the single file being rewritten.
1. Reorder positional function parameters with
   `paredit move-function-parameter --output json` first. Review `from_index`,
   `to_index`, every explicit or discovered `call_paths` entry, and
   `moved_arguments` before applying `--write`.
1. Swap two positional function parameters with
   `paredit swap-function-parameters --output json` first. Review
   `left_index`, `right_index`, every explicit or discovered `call_paths`
   entry, and `swapped_arguments` before applying `--write`.
1. Reorder all positional function parameters with
   `paredit reorder-function-parameters --output json` first. Pass the full
   reviewed target order with repeated `--parameter`, then review
   `old_parameter_order`, `new_parameter_order`, and `reordered_arguments`
   before applying `--write`.
1. Remove obsolete positional function parameters with
   `paredit remove-function-parameter --output json` first. Review
   `parameter_index`, each selected or discovered call, and
   `removed_arguments` before applying `--write`.
1. Use `paredit sort-definitions --output json` to canonicalize contiguous
   top-level definition blocks inside one file after decomposition or API
   cleanup. Review `definition_count`, `strategy`, and `changed` before
   applying `--write`; non-definition barriers are preserved.
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
reviewing the JSON plan. It refuses a selection containing a comment, since
the pipeline is rebuilt from parsed parts and a comment would be silently
discarded rather than placed somewhere in the rewritten form.

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
macro uses a different name. Like `thread-expression`, it refuses a selection
containing a comment rather than discarding it while re-nesting into calls.

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
single-expression function or macro definition, exact arity, and a reviewed
call shape. For Common Lisp it supports flat lambda lists including positional
parameters, `&optional`, `&rest` or `&body`, `&key`, `&allow-other-keys`,
`&aux`, macro `&whole`, unused macro `&environment` bindings, and `defmacro`
destructuring parameters built from nested list patterns in required and
`&optional` or `&key` positions, including inner destructuring `&optional`,
`&whole`, `&rest`, `&body`, `&key`, and `&allow-other-keys` bindings.
It still refuses cases that would change semantics, including duplicate
argument evaluation, unused arguments by default, macro destructuring patterns
that use unsupported inner lambda-list markers outside this conservative
subset, and macros that actually reference an `&environment` parameter because
source-level inlining cannot reconstruct a macro expansion environment. Use
`--allow-duplicate-evaluation` or `--allow-drop-arguments` only after
reviewing the JSON plan. Pass repeated `--call-path` values for reviewed
specific calls, or `--all-calls` to discover every same-file call whose list
head matches the selected definition. The JSON plan reports both the legacy
single-call fields and a `calls` array so agents can review each replacement.
`--remove-definition` is refused when the definition body contains a comment,
since only the parsed body is copied into call sites and the comment would be
discarded along with the removed definition.

Add a parameter to a reviewed definition and selected call sites:

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
  --parameter-section keyword \
  --name context \
  --argument ':context *context*' \
  --all-calls \
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
`--insert start` for prefix arguments. By default `--parameter-section auto`
adds a positional parameter unless the selected Common Lisp lambda list already
contains `&optional` or `&key`, in which case the new parameter is appended to
that existing section. Use `--parameter-section positional`, `optional`, or
`keyword` to force a specific section; explicit `optional` or `keyword`
requests also create a missing `&optional` or `&key` section when the selected
Common Lisp lambda list supports that insertion. The JSON/text report returns
the resolved section as `required`, `optional`, or `keyword`.

Move a positional parameter within a reviewed definition and selected call sites:

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

`move-function-parameter` reorders a parameter within the selected definition
and moves the corresponding argument at each reviewed `--call-path` entry, or
each same-file call discovered by `--all-calls`. In Common Lisp, simple
positional parameters and lambda-list section members such as `&optional` and
`&key` are supported, but the command refuses moves across section boundaries.
It reports `from_index`, `to_index`, `call_paths`, and `moved_arguments`,
verifies each call head against the selected definition, and re-parses the
rewritten file. It also refuses to reorder a parameter list that contains a
comment, since the list is rebuilt from each parameter's own text and the
comment would be discarded.

Swap two positional parameters within a reviewed definition and selected call
sites:

```sh
paredit swap-function-parameters \
  --file src/renderer.lisp \
  --definition-path 0 \
  --left-name width \
  --right-name height \
  --call-path 3.2 \
  --output json
paredit swap-function-parameters \
  --file src/renderer.lisp \
  --definition-path 0 \
  --left-name width \
  --right-name height \
  --all-calls \
  --write
```

`swap-function-parameters` swaps parameters within the selected definition and
swaps the corresponding arguments in each reviewed `--call-path` entry, or
each same-file call discovered by `--all-calls`. In Common Lisp, simple
positional parameters and lambda-list section members such as `&optional` and
`&key` are supported, but the command refuses swaps across section boundaries.
It reports `left_index`, `right_index`, `call_paths`, and `swapped_arguments`,
verifies each call head against the selected definition, refuses calls missing
either argument, and re-parses the rewritten file. Like
`move-function-parameter`, it refuses to reorder a parameter list that
contains a comment.

Remove an obsolete positional parameter from a reviewed definition and selected
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

`remove-function-parameter` removes a parameter from the selected definition
and the corresponding argument from each reviewed `--call-path` entry, or each
same-file call discovered by `--all-calls`. In Common Lisp, simple positional
parameters and lambda-list section members such as `&optional` and `&key` are
supported when the call shape still remains valid; unsupported tails such as
entries after `&allow-other-keys` are rejected. It
verifies each call head against the selected definition, reports
`parameter_index`, `call_paths`, and `removed_arguments`, refuses missing call
arguments by default, and re-parses the rewritten file. Removing the first
parameter keeps any comment describing the next parameter in place; removing
any other parameter that has its own leading comment removes that comment
along with it, since it describes the parameter being removed.

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
- Warning-clean `cargo clippy --all-targets --all-features -- -D warnings`.
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
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo nextest run --locked
cargo publish --dry-run --allow-dirty --locked
cargo doc --no-deps
cargo package --allow-dirty --no-verify
cargo package --allow-dirty --list
nix flake check
nix build .#
```

## Project Policies

- See [CONTRIBUTING.md](CONTRIBUTING.md) for development and release
  expectations.
- See [GOVERNANCE.md](GOVERNANCE.md) for decision-making, scope control, and
  maintainer expansion rules.
- See [RELEASE.md](RELEASE.md) for maintainer release criteria and execution
  steps.
- See [COMPATIBILITY.md](COMPATIBILITY.md) for CLI, JSON, and `--write`
  stability guarantees.
- See [MAINTAINERS.md](MAINTAINERS.md) for maintainer responsibilities and
  response targets.
- See [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) for collaboration and community
  expectations.
- See [SECURITY.md](SECURITY.md) for vulnerability reporting and response
  expectations.
- See [SUPPORT.md](SUPPORT.md) for bug-reporting and usage-support guidance.
- See [ROADMAP.md](ROADMAP.md) for current priorities, contribution focus, and
  explicit non-goals.
- See [CHANGELOG.md](CHANGELOG.md) for user-visible changes.

## Scope

`paredit-cli` is a structural S-expression tool, not a Lisp evaluator or full
reader implementation. It preserves balanced list, vector, and map delimiters;
tracks comments and strings safely for symbol operations; and provides
dialect-aware definition hints. It does not macroexpand code or update ASDF,
package, autoload, or module manifests automatically.
