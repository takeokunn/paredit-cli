#[test]
fn readme_links_every_public_top_level_support_document() {
    let readme = std::fs::read_to_string("README.md").expect("read README");

    for required in [
        "CHANGELOG.md",
        "CODE_OF_CONDUCT.md",
        "COMPATIBILITY.md",
        "CONTRIBUTING.md",
        "GOVERNANCE.md",
        "LICENSE",
        "MAINTAINERS.md",
        "RELEASE.md",
        "ROADMAP.md",
        "SECURITY.md",
        "SKILLS.md",
        "SUPPORT.md",
    ] {
        assert!(
            readme.contains(&format!("]({required})")),
            "README must link public support document for discoverability: {required}"
        );
    }
}

#[test]
fn readme_documents_common_lisp_callable_designators_for_function_rename() {
    let readme = std::fs::read_to_string("README.md").expect("read README");
    let normalized = normalize_whitespace(&readme);

    assert!(
        normalized.contains(
            "Common Lisp callable designators such as `function`, `macro-function`, `compiler-macro-function`, `symbol-function`, `fdefinition`, reader-prefix forms such as `#'`, and `setf` callable names like `(setf accessor)`"
        ),
        "README must describe the callable designators handled by rename-function"
    );
    assert!(
        normalized.contains("while still skipping quoted data and arbitrary values"),
        "README must state that rename-function leaves quoted data and arbitrary values untouched"
    );
}

#[test]
fn readme_documents_common_lisp_scope_aware_rename_semantics() {
    let readme = std::fs::read_to_string("README.md").expect("read README");
    let normalized = normalize_whitespace(&readme);

    for required in [
        "### Common Lisp Scope-Aware Rename Semantics",
        "`rename-local-function` distinguishes `flet` from `labels`",
        "`rename-function` follows Common Lisp callable designators such as",
        "`rename-function` also keeps `defmacro` and `define-compiler-macro` definitions traversable inside reader-quoted lambda bodies",
        "`rename-macrolet` renames local `macrolet` and `compiler-macrolet` bindings",
        "`rename-symbol-macro` and outer-binding renames across `symbol-macrolet`",
    ] {
        assert!(
            normalized.contains(required),
            "README must preserve the Common Lisp scope-aware rename semantics section: {required}"
        );
    }
}

#[test]
fn readme_documents_common_lisp_support_matrix() {
    let readme = std::fs::read_to_string("README.md").expect("read README");
    let normalized = normalize_whitespace(&readme);

    for required in [
        "### Common Lisp Support Matrix",
        "The current Common Lisp coverage is organized as follows:",
        "`rename-function`: top-level callable definitions and callable designators",
        "`rename-local-function`: lexical callable bindings introduced by `flet` and `labels`",
        "`rename-macrolet`: local macro and compiler-macro bindings introduced by `macrolet` and `compiler-macrolet`",
        "`rename-symbol-macro`: symbol macro bindings introduced by `define-symbol-macro` and `symbol-macrolet`",
        "Scope boundaries: quoted data, arbitrary values, and expander-local bodies stay out of the generic traversal path",
    ] {
        assert!(
            normalized.contains(required),
            "README must keep the Common Lisp support matrix explicit: {required}"
        );
    }
}

#[test]
fn readme_documents_macrolet_boundary_for_lisp_rename() {
    let readme = std::fs::read_to_string("README.md").expect("read README");
    let normalized = normalize_whitespace(&readme);

    assert!(
        normalized.contains(
            "`rename-macrolet` renames local `macrolet` and `compiler-macrolet` bindings"
        ),
        "README must describe the macrolet bindings handled by rename-macrolet"
    );
    assert!(
        normalized.contains("treats `macrolet` and `compiler-macrolet` as scope boundaries so expander-local bodies keep their own shadowing rules"),
        "README must state that rename-function treats macrolet forms as scope boundaries"
    );
    assert!(
        normalized.contains("keeping expander bodies out of scope so only in-form call sites move"),
        "README must state that rename-macrolet does not rewrite symbols inside expander bodies"
    );
}

#[test]
fn readme_routes_lisp_renames_to_their_scope_specific_commands() {
    let readme = std::fs::read_to_string("README.md").expect("read README");
    let normalized = normalize_whitespace(&readme);

    for required in [
        "Use `paredit rename-function --output json` for callable definitions",
        "Use `paredit rename-local-function --output json` for local callable bindings",
        "Use `paredit rename-macrolet --output json` for `macrolet` and `compiler-macrolet` bindings",
        "Use `paredit rename-symbol-macro --output json` for `define-symbol-macro` bindings",
    ] {
        assert!(
            normalized.contains(required),
            "README must route each Lisp rename shape to the matching command: {required}"
        );
    }

    assert!(
        normalized.contains("define-method-combination"),
        "README must call out define-method-combination in the callable-definition rename surface"
    );
}

fn normalize_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
