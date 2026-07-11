#[test]
fn compatibility_policy_keeps_common_lisp_scope_boundaries_explicit() {
    let compatibility = std::fs::read_to_string("COMPATIBILITY.md").expect("read COMPATIBILITY");
    let normalized_compatibility = normalize_whitespace(&compatibility);

    for needle in [
        "Common Lisp scope-aware refactors must preserve callable and macro binding boundaries",
        "local `macrolet`, `compiler-macrolet`, and `symbol-macrolet` forms",
        "`defmacro` and `define-compiler-macro` definitions remain traversable inside reader-quoted lambda bodies",
        "expander bodies are treated as separate reviewable scopes rather than generic traversal targets.",
        "The Common Lisp support matrix documented in [README.md](README.md) is part of the released contract:",
        "`rename-function`, `rename-local-function`, `rename-macrolet`, and `rename-symbol-macro` must preserve the same scope boundaries and callable namespaces described there.",
    ] {
        assert!(
            normalized_compatibility.contains(needle),
            "COMPATIBILITY must keep the Lisp scope boundary explicit: {needle}"
        );
    }
}

fn normalize_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
