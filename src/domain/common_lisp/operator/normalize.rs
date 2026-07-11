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

pub(crate) fn is_common_lisp_declaration_form(head: &str) -> bool {
    common_lisp_operator_head_eq(head, "declare")
        || common_lisp_operator_head_eq(head, "declaim")
        || common_lisp_operator_head_eq(head, "proclaim")
}
