# Changelog

All notable user-visible changes to this project will be documented in this
file.

The format is based on Keep a Changelog, and the project follows Semantic
Versioning for released tags.

Entries should summarize user-visible behavior, policy, packaging, or support
changes that matter to users and coding agents, not internal-only refactors
with no external effect.

## [Unreleased]

### Fixed

- `let-report` and `remove-unused-binding --all-bindings` no longer treat
  rebinding an earmuffed (`*name*`) Common Lisp special variable with zero
  lexical references as dead code. `(let ((*read-eval* nil)) (read
  stream))` is meaningful purely through its dynamic-scope side effect for
  the body's dynamic extent — no reference to `*read-eval*` is needed or
  expected anywhere in the body. `let-report` now reports this shape with a
  distinct `possible-dynamic-variable-rebind` risk instead of
  `unused-binding`, and `--all-bindings` skips it from bulk removal
  (`--name` still removes it if explicitly targeted). Previously, trusting
  either report could delete a binding that changes program behavior —
  verified against a real target codebase, where this exact shape guards
  against `#.` read-time code execution while deserializing persisted
  data.
- `let-report` no longer analyzes a `let`-shaped form found inside a
  quasiquote code-generation template (e.g. `` `(let ((,x ,val)) ...) ``
  in a with-gensyms-style macro helper) as a real binding. Such a form's
  "binding name" is frequently an unquoted gensym variable determined at
  macro-expansion time, not a symbol whose unused-ness can be judged; it
  was previously reported with `unused-binding` risk, and acting on that
  report (or a future `remove-unused-binding` built on the same
  `--include-protected`-style trust) would corrupt the macro's generated
  code rather than remove dead code. A real `let` nested inside an
  unquoted (`,`/`,@`) sub-expression is still analyzed normally.
- Scope-aware reference collection (used by `unused-definition-report`,
  `remove-unused-definitions`, and every rename/refactor command built on
  it) no longer treats a Common Lisp `FUNCTION` *type specifier*
  (`(function (arg-types...) return-type)`, as in `(declaim (ftype
  (function (my-word) my-word) f))`) the same as the unrelated
  function-namespace accessor form `(function name)` (the explicit
  spelling of `#'name`). Both share the head symbol `function`, but only
  the accessor form names exactly one symbol; the type specifier's
  contents are ordinary type-position atoms — most commonly a
  `deftype`-defined alias used nowhere else — and were previously
  invisible to reference scanning entirely, so a type alias used
  correctly (only in `ftype`/`the`/`check-type` position) was reported as
  unused and would have been deleted by `--write`.
- `find-symbol`, `symbol-report`, `rename-symbol`, `refactor plan/preview`,
  `unused-definition-report`, and `remove-unused-definitions` now recognize
  a Common Lisp symbol referenced through a package qualifier
  (`nshell.application:execute-command-line`) or the `#:` uninterned-symbol
  syntax used in `defpackage` `:export` clauses (`#:execute-command-line`)
  as the same symbol as the bare name. Occurrence matching previously only
  stripped the four standard CL home-package aliases (`cl:`, `cl-user:`,
  `common-lisp:`, `common-lisp-user:`), so a definition referenced
  exclusively through a project-specific package qualifier — the normal way
  one package's test suite or consumer calls another package's exported
  function — had zero recognized references and was reported as unused,
  and would have been deleted by `--write`.
- `remove-unused-definitions`: Common Lisp `defstruct` definitions are now
  protected by default (require `--include-protected`), like package/
  system/test/customization/mode definitions. A structure's type-name
  symbol is only one of several symbols `defstruct` derives: it also
  implicitly generates a constructor (`make-<name>`, or an explicit
  `(:constructor other-name)`), a predicate (`<name>-p`), a copier, and
  per-slot accessors, none of which spell out the type name. A structure
  used exclusively through those derived names had zero direct references
  to its type-name symbol and was previously flagged as unused and
  removed by `--write`, silently breaking every remaining call site.
- `remove-unused-definitions`: Emacs Lisp `require`/`provide` forms are now
  categorized the same way Common Lisp `require`/`provide`/`defpackage`
  already were (`DefinitionCategory::Package`, protected by default)
  instead of falling into the generic, bulk-removable `Other` bucket. A
  `provide`d feature is definitionally only ever referenced as a quoted
  symbol argument to `require` in another file, which the reference
  scanner cannot see, so every `require`/`provide` in an Emacs Lisp
  workspace was previously flagged as "unused" and would have been
  deleted by `--write`, breaking that file's module loading.
- `remove-unused-definitions`: a definition referenced only as a bare atom
  inside a quoted list literal (dispatch tables, keymap alists like
  `'((key . command))`, `featurep`/`fboundp` argument lists) is no longer
  treated as unused. Reference collection previously skipped the entire
  contents of any plain-quoted form as opaque data, so a function or
  variable used exclusively through this extremely common Lisp idiom was
  reported as unreferenced and removable.
- `remove-unused-definitions`: an unrecognized `define-*`-prefixed macro
  invocation (a project's own strategy, schema, or handler DSL, for
  example `(define-trading-strategy foo ...)`) is now reported under a
  new `unknown-macro` category and protected by default like other
  categories this tool cannot fully verify, instead of falling into the
  generic, bulk-removable `other` bucket. Such a macro commonly derives
  *other* exported symbol names from its argument via string
  concatenation (the example above might generate and export
  `make-foo-strategy`), so the argument symbol having zero direct
  references does not mean the code it defines is unused. `other` itself
  is now reserved for a dialect's own recognized definition forms (Emacs
  Lisp `defun`, Clojure `defn`, ...) that are not broken out into a more
  specific category but are still known, non-generative shapes, and
  remains bulk-removable as before.
- `remove-unused-definitions`/`unused-definition-report`: a definition
  named only as the literal head of a quasiquoted code-generation template
  (`` `(validator-fn ,arg) ``, the pattern a macro uses to assemble a form
  it later splices into its own expansion) is no longer treated as
  unused. The quoted-dispatch-data supplemental scan only recognized a
  plain `Quote` reader prefix; quasiquote (`` ` ``) is now recognized the
  same way, since it is at least as common an idiom for naming a
  still-live callee indirectly.
- `unused-definition-report`: reference counting now uses the same
  scope-aware scan as `remove-unused-definitions` (previously a flat,
  scope-blind atom-text scan), so a global definition shadowed only by a
  same-named local `let`/`flet`/etc. binding elsewhere is no longer
  miscounted as referenced. The two commands could previously disagree on
  whether the same definition was unused.
- `sort-package-exports`: a `;; section` comment (or any own-line comment)
  that precedes an export symbol now travels with that symbol when the
  sort reorders the list, instead of staying at a fixed line and
  mislabeling whichever symbol landed there. Trailing same-line comments
  stay glued to the symbol they follow, and the closing delimiters are
  pushed to a fresh line when a commented entry would otherwise absorb
  them. Comment-free export lists reorder exactly as before.
- Parser: a Common Lisp/Scheme multiple-escape symbol (`|Foo Bar|`) no
  longer splits into two atoms at an embedded space or delimiter. Every
  character inside the `|...|` region, including whitespace, is now a
  literal symbol constituent per CLHS 2.1.4.2, and an unterminated region
  is a parse error instead of a silent misparse.
- Parser: `#+feature`/`#-feature` conditional dispatch now scans as its
  own token instead of gluing onto a bare feature symbol. `#+sbcl (form)`
  and `#+(and sbcl x86-64) (form)` now produce the same tree shape, so
  `find-symbol`/`rename-symbol` see the feature symbol in both spellings
  instead of only the compound one.
- Parser: Clojure/Fennel-style `#{...}` sets, `#(...)` anonymous
  functions (and Common Lisp/Scheme `#(...)` vector literals), `^...`
  metadata, and `#?(...)`/`#?@(...)` reader conditionals now parse as one
  node instead of a disconnected `#`/`^` atom followed by an unrelated
  sibling list, so structural edits (`kill`, `wrap`, `slurp`, `barf`,
  `select --path`) target the whole literal instead of leaving a dangling
  prefix behind.
- Parser: Clojure's `#_` discard reader macro is now treated the same
  way as Scheme/CL `#;` datum comments — it reads and discards exactly
  one following form instead of leaving both the `#_` marker and the
  discarded form as live tree nodes, so `find-symbol`/`rename-symbol` no
  longer see occurrences inside discarded code.
- `kill`/`slurp-forward`/`slurp-backward`/`barf-forward`/`barf-backward`:
  whitespace removal around the edited span no longer crosses a line
  comment's trailing newline. Previously, killing or slurping a form
  directly after a `;; comment` line could delete that newline and
  splice the next form onto the comment's line, silently commenting it
  out.
- `rename-function`/`rename-macrolet`: a bare `(lambda (params...) ...)`
  form (not just its `#'(lambda ...)` reader-quoted spelling) now skips
  its own parameter list during call-site traversal, matching the CLHS
  3.1.2.1.2.4 equivalence between the two spellings. Previously, a
  parameter name shadowing a renamed callable inside a bare `lambda`
  could be misidentified as a call-site reference.
- `unthread-expression`: an operator that is not `->`/`->>` and was not
  confirmed via `--operator` is now rejected immediately with a message
  naming the operator, instead of accepting a bare `--style` and
  rewriting an ordinary call that merely has an arrow-like head symbol
  into unrelated nested-call output.
- `move-definition`/`split-file`: a moved Common Lisp definition now carries
  an `(in-package ...)` declaration into the destination file whenever the
  destination's trailing package context does not already match the
  source's, instead of landing bare and interning into whichever package
  happens to be current when the file is later loaded.
- `merge-package-options`: a defpackage option group (`:export`,
  `:import-from`, ...) that has a comment anywhere in its source is now
  left unmerged instead of merging, since the merge rebuilds the kept
  option's text from parsed atoms and blanks the others — a step that
  previously discarded any interleaved comment and could leave a stray
  empty line where a removed option used to be.
- `sort-definitions`: a leading own-line comment (or blank run) above a
  top-level definition now travels with that definition when the sort
  reorders the block, instead of staying at its original line and ending
  up above whichever definition landed there. A definition that had no
  leading trivia of its own picks up a plain separator instead of gluing
  onto the previous definition's closing delimiter when it is reordered
  away from the front of the block.
- `sort-package-options`: the same fix as above, applied to `defpackage`
  option forms (`:use`, `:export`, `:documentation`, ...) reordered by
  `sort-package-options`.
- `thread-expression`/`unthread-expression`: a selection containing a
  comment is now rejected instead of silently discarding that comment.
  Both commands rebuild their target as new text from parsed parts, and
  a comment inside the selection lives outside the tree, so it had no
  slot in the rebuilt pipeline or nested calls and was dropped entirely.
- `inline-function`: `--remove-definition` is now rejected when the
  definition body contains a comment, instead of deleting the definition
  (and the comment along with it) after copying only the parsed body
  into call sites. Inlining without `--remove-definition` is unaffected,
  since the definition and its comment stay in place.
- `swap-function-parameters`/`move-function-parameter`/
  `reorder-function-parameters`: a parameter list containing a comment is
  now rejected instead of silently discarding that comment. All three
  commands share a definition rewrite that rebuilds the parameter list
  from each parameter's own bare span joined by a single space, so a
  comment anywhere in the list — and the list's original line layout —
  had no slot in the rebuilt text and was dropped.
- `remove-function-parameter`: removing a function's first parameter no
  longer deletes a comment that describes the *next* parameter. The
  removal span previously extended from the first parameter to wherever
  the second parameter's own text started, absorbing any comment in
  between; it now stops at the first newline after the removed
  parameter, leaving the next parameter's leading comment in place.
- `add-export`: a `--symbol` argument without a `#:` or `:` prefix (for
  example `--symbol foo` instead of `--symbol #:foo`) is now normalized to
  `#:foo` before insertion instead of being spliced into the `:export` list
  verbatim. A bare symbol name read back at load time interned an unrelated
  symbol in whatever package happened to be current at read time, instead of
  contributing only a name to the package being defined.
- `split-file`/`move-definition`: a leading own-line comment describing a
  moved definition now travels with it into the destination file instead of
  being orphaned above whichever definition happens to remain in its place
  in the source file. Both commands previously also over-absorbed trailing
  whitespace after a moved definition, which glued the two definitions that
  used to surround it directly together when a middle definition was moved;
  the removed region now stops exactly at the boundary the moved
  definition's own leading trivia claimed, leaving the original separator
  between the remaining neighbors intact.

## [0.1.2] - 2026-07-11

### Fixed

- Parser: a backslash in an atom now consumes the following character
  literally (the Lisp single-escape rule), so character literals whose
  value is a delimiter or whitespace (`#\[`, `#\)`, `#\]`, `#\(`, `#\Space`)
  and escaped symbol constituents like `\(` no longer split into a stray
  delimiter and cause a mismatched/unclosed-list error.
- Formatter: canonical rendering now preserves comments instead of
  silently dropping them. Leading own-line comments stay above their
  form, trailing same-line comments stay inline, and forms with interior
  comments render verbatim; comment-free output is unchanged and format
  stays idempotent.
- `package-report`/dependency-report: a `defpackage`/`in-package` form
  whose package designator is computed or quasiquoted (not a static atom)
  is now skipped instead of hard-erroring the whole report.

### Changed

- `nix flake check` no longer runs the network-bound `cargo publish
  --dry-run` check, which requires crates.io registry access unavailable
  in the sandboxed Linux CI build. The publish dry-run remains a
  documented local pre-release step in RELEASE.md.

## [0.1.1] - 2026-07-11

### Added

- A reusable GitHub composite action (`takeokunn/paredit-cli@<tag>`) that
  runs structural lint (`mode: lint`), canonical-format verification
  (`mode: format`), or in-place formatting (`mode: fix`) against any
  repository, pulling prebuilt binaries from the public
  `takeokunn-paredit-cli` Cachix cache.
- `paredit-lint` and `paredit-format` wrapper tools exposed as flake
  packages and apps (`nix run github:takeokunn/paredit-cli#lint`,
  `...#format -- --check`), with GitHub error annotations in CI.
- Flake integration surfaces for other projects: `overlays.default`
  (providing `paredit-cli`, `paredit-lint`, `paredit-format`, and
  `paredit-format-files`), `lib.<system>.mkLintCheck` /
  `lib.<system>.mkFormatCheck` flake-check helpers, and
  `lib.<system>.treefmtFormatter` for treefmt-nix configurations.
- treefmt-nix support in this repository itself: `nix fmt` now runs treefmt
  with rustfmt, nixfmt, and paredit as the Lisp formatter, and
  `nix flake check` enforces it (test fixtures stay byte-exact).
- A `lint-format-integration` flake check that exercises the lint failure
  path, the format `--check` failure path, and format idempotency.

### Changed

- CI now resolves the Cachix cache name from the `CACHIX_CACHE` repository
  variable instead of a hardcoded workflow value.
- All direct dependencies updated to the latest versions compatible with the
  declared 1.85 MSRV (clap 4.6, assert_cmd 2.2, proptest 1.11).
- Security, compatibility, and README release-stage wording now reference the
  shipped `v0.1.x` release line instead of a hypothetical first release.

## [0.1.0] - 2026-07-11

### Added

- Initial public release of the `paredit` command line tool for safe
  S-expression refactoring across Common Lisp, Emacs Lisp, Scheme, and
  Clojure sources.
- Structural editing commands (`wrap`, `splice`, `raise`, `slurp-forward`,
  `barf-forward`, `replace`, `replace-forms`) driven by deterministic tree
  paths and byte spans.
- Read-only analysis commands (`check`, `outline`, `form-report`,
  `agent-report`, `symbol-report`, `call-report`, `call-graph`,
  `signature-report`, `impact-report`, `definition-report`,
  `dependency-report`, `package-report`, `duplicate-report`, `let-report`,
  `unused-definition-report`, `workspace-report`) with JSON output and
  CI-friendly policy gates for AI coding agents.
- Scope-aware rename commands for symbols, bindings, callables, packages,
  `macrolet`, and `symbol-macrolet` definitions, including lexical shadowing
  semantics for Common Lisp special forms, lambda lists, `loop` clauses, and
  destructured bindings.
- Refactoring workflow commands (`refactor-plan`, `refactor-preview`,
  `refactor-check`, `refactor-status`, `refactor-diff`, `refactor-apply`,
  `verify-refactor`) with manifest-hash pinning, stale-file guards, parse
  gates, and all-or-nothing write semantics, plus workspace-level variants.
- Function and binding refactors: `extract-function`, `inline-function`,
  `introduce-let`, `inline-let`, `add-function-parameter`,
  `move-function-parameter`, `remove-function-parameter`,
  `remove-unused-binding`, `remove-unused-definitions`, `remove-definition`,
  `move-definition`, `move-form`, `sort-definitions`, `split-file`,
  `thread-expression`, and `unthread-expression`.
- An AI-agent skill guide ([SKILLS.md](SKILLS.md)) documenting the safety
  policy, plan-first/write-last refactoring loop, and per-command review
  checkpoints.
- GitHub issue forms and a pull request template that route support,
  security, roadmap, compatibility, and verification work through the
  repository's public policy documents.
- Maintainer-facing triage and review rules plus support/security wording that
  align the new GitHub issue forms with the repository's public operating
  policy.
- README now exposes a top-level document map so users, contributors, and
  maintainers can find compatibility, roadmap, governance, and release policy
  documents without scanning the full command reference.
- Public release-stage wording that distinguishes unstable `main`, the first
  tagged release line, and unsupported historical releases across README,
  compatibility, and security policy docs.
- Public project policy documents for contribution, security reporting,
  support, and release-facing change tracking.
- Release archives now ship compatibility, governance, maintainer, roadmap,
  and release-process documents together with the existing support and security
  policy files.
- Release and contribution docs now distinguish CI baseline checks from the
  broader local verification expected before cutting a release.
- README and roadmap wording now make the CI baseline boundary explicit so the
  public badge does not imply full release verification coverage.
- A compatibility policy that defines which CLI, JSON, and `--write` surfaces
  are treated as stable across releases.
- A maintainer policy that documents ownership, triage responsibility, and
  response targets.
- README installation and quickstart guidance for first-time users and coding
  agents.
- README now explains the Common Lisp scope-aware rename model for lexical
  bindings, local functions, and `symbol-macrolet` shadowing.
- Compatibility policy now states the Common Lisp scope boundary for
  callable, macro, and symbol-macro refactors so released behavior stays
  explicit.
- README and compatibility policy now spell out that `defmacro` and
  `define-compiler-macro` definitions remain traversable inside
  reader-quoted lambda bodies while `macrolet` and `compiler-macrolet`
  bodies stay scoped.
- Governance and release-process documents that define project decision-making,
  scope control, and release execution criteria.
- A public roadmap that defines current priorities, contribution focus, release
  direction, and explicit non-goals.
