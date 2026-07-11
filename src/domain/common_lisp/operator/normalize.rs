fn strip_common_lisp_package_prefix<'a>(head: &'a str, prefix: &str) -> Option<&'a str> {
    head.get(..prefix.len())
        .filter(|candidate| candidate.eq_ignore_ascii_case(prefix))?;
    Some(&head[prefix.len()..])
}

pub(crate) fn normalize_common_lisp_operator_head(head: &str) -> &str {
    strip_common_lisp_package_prefix(head, "cl:")
        .or_else(|| strip_common_lisp_package_prefix(head, "cl-user:"))
        .or_else(|| strip_common_lisp_package_prefix(head, "common-lisp:"))
        .or_else(|| strip_common_lisp_package_prefix(head, "common-lisp-user:"))
        .unwrap_or(head)
}

pub(crate) fn common_lisp_operator_head_eq(head: &str, expected: &str) -> bool {
    common_lisp_symbol_name_eq(head, expected)
}

pub(crate) fn common_lisp_symbol_name_eq(head: &str, expected: &str) -> bool {
    normalize_common_lisp_operator_head(head)
        .eq_ignore_ascii_case(normalize_common_lisp_operator_head(expected))
}

/// Strips a Common Lisp package qualifier (`pkg:sym` or `pkg::sym`) and the
/// `#:` uninterned-symbol reader prefix used in `defpackage` `:export`
/// clauses, returning the bare symbol name.
///
/// A leading colon with nothing before it (`:keyword`) is left untouched:
/// that names a distinct symbol in the `KEYWORD` package, not a qualified
/// reference to a same-named symbol elsewhere.
fn strip_common_lisp_symbol_qualifiers(head: &str) -> &str {
    let head = head.strip_prefix("#:").unwrap_or(head);
    match head.rfind(':') {
        Some(index) if index > 0 => &head[index + 1..],
        _ => head,
    }
}

/// General-purpose Common Lisp symbol-name equality for occurrence matching,
/// rename, and unused-definition detection.
///
/// Unlike [`common_lisp_symbol_name_eq`], which only recognizes the four
/// standard CL home-package aliases so an unrelated dialect or package is
/// never misclassified as a builtin special form, this strips *any* package
/// qualifier or `#:` prefix: `nshell.application:execute-command-line` and
/// `#:execute-command-line` both denote the same symbol as bare
/// `execute-command-line` for the purpose of asking "is this symbol
/// referenced anywhere?" Comparison is case-insensitive per the CLHS reader.
pub(crate) fn common_lisp_symbol_reference_eq(candidate: &str, expected: &str) -> bool {
    strip_common_lisp_symbol_qualifiers(candidate)
        .eq_ignore_ascii_case(strip_common_lisp_symbol_qualifiers(expected))
}

pub(crate) fn is_common_lisp_declaration_form(head: &str) -> bool {
    common_lisp_operator_head_eq(head, "declare")
        || common_lisp_operator_head_eq(head, "declaim")
        || common_lisp_operator_head_eq(head, "proclaim")
}
