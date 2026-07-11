# Maintainers

This repository is maintained as an evidence-first engineering project.

## Maintainer of Record

Until this file says otherwise, the repository owner is the maintainer of
record and is responsible for triage, release decisions, and policy
enforcement.

## Maintainer Responsibilities

- Keep user-visible behavior changes documented in [CHANGELOG.md](CHANGELOG.md).
- Enforce the stable-surface contract in [COMPATIBILITY.md](COMPATIBILITY.md).
- Follow [GOVERNANCE.md](GOVERNANCE.md) when making scope or maintainer
  decisions.
- Follow [RELEASE.md](RELEASE.md) when cutting or approving releases.
- Route conduct issues through [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).
- Route security-sensitive reports through [SECURITY.md](SECURITY.md).
- Keep reproducible bug reports and feature requests moving through public
  issue discussion when disclosure risk is low.

## Response Expectations

The project aims to:

- acknowledge security reports within 7 days;
- acknowledge reproducible bug reports within 14 days; and
- close or re-scope stale requests when reproduction details never arrive.

These are operating targets, not SLA guarantees.

## Triage Rules

- Apply `bug` to reproducible correctness, safety, or regression reports.
- Apply `enhancement` only when the request matches
  [ROADMAP.md](ROADMAP.md) and has a clear structural-editing payoff.
- Apply `documentation` when the fix is primarily about public guidance,
  release policy wording, or examples.
- Apply `question` sparingly; support requests should usually resolve into a
  reproducible `bug`, a scoped `enhancement`, or closure with a pointer to
  [SUPPORT.md](SUPPORT.md).
- Apply `good first issue` only when the change has bounded scope, no policy
  ambiguity, and an obvious verification path.
- Apply `help wanted` when outside contributors can take the issue without
  privileged release or security context.
- Close `invalid`, `duplicate`, or `wontfix` requests with a short rationale
  and a link to the controlling public document when scope or policy is the
  deciding factor.

## Pull Request Review Minimum

- Require an explicit verification list for every user-visible change.
- Require [COMPATIBILITY.md](COMPATIBILITY.md) review when CLI, JSON, or
  `--write` behavior changes.
- Require [CHANGELOG.md](CHANGELOG.md) updates for user-visible behavior.
- Route security-sensitive fixes through [SECURITY.md](SECURITY.md) before any
  public review thread contains exploit details.

## Expanding the Maintainer Set

If additional maintainers are added, list them here together with the scope
they can approve or release.
