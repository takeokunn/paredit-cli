# Refactoring History Audit

This audit compares refactorings recorded in Serena memories for Common Lisp
and Emacs Lisp projects with the transformations exposed by `paredit-cli`.
The comparison treats a transformation as a CLI candidate only when its safety
can be established from the parsed source and explicit command arguments.

## Coverage

The reviewed histories included `cl-boundary-kit`, `cl-cc`, `cl-regex`,
`cl-tmux`, `cl-tty-kit`, `cl-weave`, `nshell`, `private-trade-fx`, `kuro`,
`nskk.el`, `dasel-el`, `ob-gleam`, and `doclive`. Projects without relevant
memories were also checked. Some unavailable third-party checkouts and one
project with a cyclic symlink could not be activated by Serena.

## CLI Mapping

| Historical refactoring | Representative evidence | CLI coverage | Boundary |
| --- | --- | --- | --- |
| Rename a symbol, binding, function, macro parameter, or package | `cl-regex` group and function renames; `cl-cc` and `private-trade-fx` alias cleanup | `rename-symbol`, `rename-binding`, `rename-function`, `rename-local-function`, `rename-macrolet`, `rename-symbol-macro`, `rename-package` | Scope-aware commands preserve binding and callable semantics. |
| Remove an unused definition or form | Dead stubs in `cl-cc`; dead helpers in `nshell` and `kuro` | definition reports plus `remove-definition` and structural deletion | Cross-file reachability still requires project-level evidence. |
| Extract repeated logic into a function | Lowering helpers in `cl-cc`; parser helpers in `cl-tmux`; traversal helpers in `private-trade-fx` | `extract-function` | Macro extraction is excluded because evaluation count and expansion phase are not generally inferable. |
| Extract repeated literals into constants | Format strings in `nskk.el`; command tables in `cl-tmux` | `extract-constant` | Emits `defconstant` for Common Lisp and `defconst` for Emacs Lisp, with quote and parse guards. |
| Inline a one-use helper | Predicate cleanup in `private-trade-fx`; thin wrappers in `nshell` | `inline-function` | Package import/export cleanup remains explicit. |
| Replace or wrap calls | Canonical function migrations and Lisp idiom changes across the reviewed projects | `replace-call`, `wrap-call`, `unwrap-call` | The caller supplies the semantic decision; the CLI validates structure. |
| Normalize local bindings | Alias removal and binding simplification in `nskk.el` | let transformations and let reports | Commands account for implicit `nil` bindings. |
| Change function parameters | Signature cleanup and parameter migrations | function-parameter commands and signature reports | Call-site compatibility must satisfy the command's safety gates. |
| Reorder or relocate definitions and forms | Source organization changes across `cl-cc`, `cl-boundary-kit`, and `kuro` | move, split, and sort definition/form commands | Operations are source-local and do not edit ASDF or module graphs. |
| Update package declarations | Export and canonical-name cleanup in Common Lisp projects | package export and option commands | Package traversal is shared so all commands interpret `defpackage` consistently. |
| Inspect impact before editing | Dead code, arity, and duplication investigations throughout the histories | definition, signature, impact, let, and related reports | Reports produce actionable locations and avoid quoted/data false positives. |

## Changes Resulting From The Audit

- Added `extract-constant`, the principal safe source-level transformation that
  was present in the histories but absent from the CLI.
- Corrected rename handling for callable positions and quoted function
  designators.
- Corrected let reports for implicit `nil` bindings.
- Extended formatting support for definition-oriented forms.
- Made definition reports safer and directly actionable.
- Centralized `defpackage` traversal to keep package commands consistent.

## Deliberately Excluded Transformations

The following historical changes are not generic AST refactorings and are not
safe to expose as automatic commands:

- Splitting or deleting files while changing ASDF or Emacs module load order.
- Extracting or generating macros where hygiene, evaluation count, or
  compile-time availability must be proven.
- Consolidating implementations based on presumed semantic equivalence.
- Expanding static dispatch tables at macro-expansion time.
- Migrating dependency APIs or command-line options.
- Changing algorithms, return-value contracts, security behavior, or domain
  semantics.
- Removing wrappers together with project-wide package/import graph cleanup.

These changes can use structural and call-replacement primitives as building
blocks, but require project-specific review. Treating them as universally safe
would exceed the CLI's evidence boundary.
