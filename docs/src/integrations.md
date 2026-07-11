# Integrations

## GitHub Actions

Use the Nix package in CI and invoke canonical command paths:

```yaml
- name: Check Lisp source
  run: nix run github:takeokunn/paredit-cli -- inspect check --file source.lisp
```

The repository action can also run the lint mode:

```yaml
- uses: takeokunn/paredit-cli@main
  with:
    mode: lint
```

For a refactoring workflow, produce a plan and preview as CI artifacts before allowing an apply step in a controlled environment.

## Nix development shell

```sh
nix develop
cargo test
paredit inspect check --file source.lisp
```

## GitHub Pages

The repository publishes this mdBook site from `docs/src`. The deployment workflow builds the book and uploads `docs/book` as the GitHub Pages artifact.
