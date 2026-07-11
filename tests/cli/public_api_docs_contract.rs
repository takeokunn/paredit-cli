#[test]
fn public_library_api_keeps_docs_rs_surface_docs() {
    for (path, required) in [
        (
            "src/domain/dialect/mod.rs",
            "/// Selects Lisp-family parsing and refactoring rules for a source file.",
        ),
        ("src/domain/dialect/mod.rs", "/// # Examples"),
        (
            "src/domain/dialect/mod.rs",
            "/// use paredit_cli::dialect::Dialect;",
        ),
        (
            "src/domain/dialect/mod.rs",
            "/// Resolves the effective dialect from an explicit override or file extension.",
        ),
        (
            "src/domain/sexpr/types.rs",
            "/// A byte offset into the original source text.",
        ),
        (
            "src/domain/sexpr/types.rs",
            "/// A zero-based path from the virtual root to a nested expression.",
        ),
        ("src/domain/sexpr/types.rs", "/// # Examples"),
        (
            "src/domain/sexpr/types.rs",
            "/// use paredit_cli::sexpr::ExpressionPath;",
        ),
        (
            "src/domain/sexpr/types.rs",
            "/// A validated Lisp-family symbol name without reader delimiters or whitespace.",
        ),
        (
            "src/domain/sexpr/tree.rs",
            "/// A parsed S-expression document with tree navigation and query helpers.",
        ),
        ("src/domain/sexpr/tree.rs", "/// # Examples"),
        (
            "src/domain/sexpr/tree.rs",
            "/// use paredit_cli::sexpr::{ExpressionPath, SyntaxTree};",
        ),
        (
            "src/domain/sexpr/tree.rs",
            "/// use paredit_cli::sexpr::{SymbolName, SyntaxTree};",
        ),
        (
            "src/domain/sexpr/tree.rs",
            "/// Builds an outline of root-level lists and marks definition-like forms.",
        ),
        (
            "src/domain/sexpr/tree.rs",
            "/// A validated selection of one non-root expression inside a syntax tree.",
        ),
    ] {
        let contents = std::fs::read_to_string(path).expect("read public api file");
        assert!(
            contents.contains(required),
            "public API docs drifted for {path}: missing `{required}`"
        );
    }
}
