# Changelog

All notable user-visible changes to this project will be documented in this
file.

The format is based on Keep a Changelog, and the project follows Semantic
Versioning for released tags.

Entries should summarize user-visible behavior, policy, packaging, or support
changes that matter to users and coding agents, not internal-only refactors
with no external effect.

## [Unreleased]

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
