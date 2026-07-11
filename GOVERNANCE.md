# Governance

`paredit-cli` is maintained as an evidence-first engineering project for
structural S-expression editing and refactoring.

## Project Principles

- Preserve structural correctness before convenience. Refactors must keep edits
  balanced, syntax-aware, and reviewable.
- Prefer explicit plans, previews, and verification over hidden mutation.
- Treat machine-facing CLI and JSON output as public contracts once released.
- Keep bug reports, feature requests, and design discussion grounded in minimal
  reproductions, fixtures, or measurable behavior.

## Decision Making

- Day-to-day technical decisions are made in public through issues, pull
  requests, and review discussion.
- The maintainer of record makes final decisions when a change affects release
  policy, compatibility guarantees, security handling, or long-term project
  scope.
- Decisions that change stable CLI, JSON, or `--write` behavior must reference
  [COMPATIBILITY.md](COMPATIBILITY.md) and document the user-visible effect in
  [CHANGELOG.md](CHANGELOG.md).
- Security-sensitive decisions follow [SECURITY.md](SECURITY.md) even when the
  final fix lands through a normal pull request later.

## Contribution Expectations

- New commands, flags, or behavior changes need tests that prove the intended
  contract.
- Refactors should preserve the architecture boundary described in
  [README.md](README.md): syntax and typed domain rules stay out of CLI and
  terminal formatting layers.
- Process or policy changes should update the relevant public document instead
  of living only in review comments.

## Scope Control

- The project accepts changes that improve safe structural editing, typed Lisp
  analysis, and agent-friendly planning or verification workflows, including
  scope-aware handling of Common Lisp callable and macro bindings in Common
  Lisp macro binding scopes.
- Feature or refactor proposals that claim current-project priority should
  point to [ROADMAP.md](ROADMAP.md) instead of relying on review-thread
  interpretation alone.
- The project does not accept features that require evaluating Lisp code,
  performing text-based rewrites across comments or strings, or hiding file
  mutations behind implicit traversal. Macro expander bodies are treated as
  their own reviewable scopes rather than as a license for unrestricted
  rewriting.
- Experimental behavior should be marked clearly until it is ready to enter the
  compatibility surface.

## Maintainer Changes

- The maintainer of record may add maintainers based on sustained review
  quality, responsiveness, and demonstrated understanding of compatibility and
  security policy.
- When maintainership changes, update [MAINTAINERS.md](MAINTAINERS.md) with the
  new scope of authority.

## Escalation Path

- Usage questions and reproducible bugs belong in the support path documented in
  [SUPPORT.md](SUPPORT.md).
- Conduct issues follow [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
- Security-sensitive reports follow [SECURITY.md](SECURITY.md).
