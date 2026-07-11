# Roadmap

`paredit-cli` aims to be a dependable structural editing and refactoring tool
for Lisp-family code that can be driven safely by humans and coding agents.
The project optimizes for parse-aware correctness, explicit verification, and
stable machine-readable output.

## Current Priorities

1. Keep structural editing commands correct across supported dialects and edge
   cases involving comments, strings, vectors, maps, reader forms, and
   Common Lisp macro binding scopes.
2. Expand refactor workflows that can be planned, reviewed, validated, and
   applied with stable JSON contracts instead of opaque side effects.
3. Preserve release quality through deterministic CLI behavior, warning-clean
   Rust builds, and reproducible Nix-based verification.
4. Improve contributor and maintainer ergonomics with policy documents, clear
   ownership boundaries, and explicit release procedures.

## Contribution Focus

The highest-value contributions are:

- parser or edit correctness fixes backed by focused regression tests;
- CLI features that expose existing structural or refactor capabilities through
  stable machine-readable output;
- dialect-aware analysis that improves safe planning before any write action,
  especially for Common Lisp callable and macro bindings such as
  `macrolet`, `compiler-macrolet`, and `symbol-macrolet`, including
  reader-quoted lambda bodies where top-level macro definitions remain
  traversable but expander-local bodies must stay scoped;
- documentation that reduces ambiguity around compatibility, releases, or
  operational expectations.

Changes are less likely to be accepted when they expand surface area without
clear structural-editing value or when they add write behavior without
plan-first verification gates.

## Non-Goals

`paredit-cli` is not trying to become:

- a full Lisp evaluator, compiler, or language server;
- a tool that depends on evaluating Lisp code, compiling projects, or acting
  as a language server;
- a text-substitution rewrite engine that edits comments or strings blindly;
- a project-wide manifest updater for package systems, autoload files, or build
  definitions that depends on implicit traversal instead of explicit reviewable
  scope;
- an automation layer that applies large refactors without explicit reviewable
  plans.

## Release Direction

Near-term project maturity is measured by:

- stable CLI and JSON behavior for documented commands;
- broad test coverage for structural edits and refactor verification paths;
- reproducible release checks with an explicit boundary between CI baseline
  automation and broader release-time local verification;
- public project documentation that makes support, governance, compatibility,
  and release expectations explicit.

## How To Use This Roadmap

Use this document together with [GOVERNANCE.md](GOVERNANCE.md) and
[COMPATIBILITY.md](COMPATIBILITY.md) when deciding whether a proposed change is
in scope, and use [RELEASE.md](RELEASE.md) when deciding whether the project is
ready to ship that change.
