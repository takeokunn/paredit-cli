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

- `sort-package-exports`: a `;; section` comment (or any own-line comment)
  that precedes an export symbol now travels with that symbol when the
  sort reorders the list, instead of staying at a fixed line and
  mislabeling whichever symbol landed there. Trailing same-line comments
  stay glued to the symbol they follow, and the closing delimiters are
  pushed to a fresh line when a commented entry would otherwise absorb
  them. Comment-free export lists reorder exactly as before.

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
