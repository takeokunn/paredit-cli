# Support

## Before Opening an Issue

- Reproduce on the latest `main` branch or current release.
- Run the documented development loop when building locally.
- Reduce the problem to the smallest Lisp input that still fails.
- Include the exact command line, expected result, actual result, and dialect.
- For rename/refactor reports, include the selected form or path and whether
  the problem involves lexical bindings, local functions, or symbol macros.

## Where to Ask

- Use the GitHub issue forms for reproducible bugs and feature requests.
- Start from the bug report form when the problem is incorrect refactor output,
  parse failure, performance regression, packaging breakage, or documentation
  drift with a clear reproducer.
- Start from the feature request form when you can explain the structural
  editing value, the expected CLI or JSON contract, and how the request fits
  the current roadmap.
- Use [ROADMAP.md](ROADMAP.md) to check whether a request matches the current
  direction before proposing large new feature surfaces.
- Use [GOVERNANCE.md](GOVERNANCE.md) when the question is about project scope,
  decision-making, or maintainer authority rather than command behavior.
- Use [MAINTAINERS.md](MAINTAINERS.md) to understand expected maintainer
  response windows.
- If you do not yet have a reproducer, open the smallest issue that states the
  blocked workflow, current command, and missing evidence rather than posting a
  broad idea dump.

## What Maintainers Need

- Input files or minimized forms that trigger the problem.
- Whether the command was run with `--write`.
- JSON output when the command supports `--output json`.
- For Common Lisp rename issues, describe which references should rename, which
  should stay fixed, and whether shadowing is expected.
- The current commit or release version.
- Whether the report is about a stable contract covered by
  [COMPATIBILITY.md](COMPATIBILITY.md) or about preview/unstable behavior.

Security-sensitive reports must follow [SECURITY.md](SECURITY.md) instead of
public issues or issue forms.
