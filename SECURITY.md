# Security Policy

## Supported Versions

Security fixes are applied on the active development line only. There is no
supported historical release branch today.

| Version | Supported |
| --- | --- |
| Unreleased `main` | Yes |
| First tagged release line after publication | Yes, until superseded by a newer supported line |
| Released versions older than `main` | No |

## Reporting a Vulnerability

Do not open public GitHub issues for vulnerabilities that could expose users,
repositories, or CI environments.

Use one of these private channels:

1. A direct maintainer contact method listed in
   [MAINTAINERS.md](MAINTAINERS.md) or on the repository owner's GitHub profile.
1. GitHub Security Advisories for this repository, after private vulnerability
   reporting is enabled for the repository.

Include:

- A clear impact summary.
- Reproduction steps or a minimal fixture.
- The affected command, flags, and input shape.
- Whether the issue can modify files outside the requested target set, corrupt
  source structure, leak data, or execute unintended code paths.

The project aims to acknowledge valid reports within 7 days and to publish a
fix or mitigation plan after the issue is reproduced and scoped.

Do not use public GitHub issues, pull requests, or issue forms for
vulnerability reports before a maintainer confirms the issue is safe to
disclose.
