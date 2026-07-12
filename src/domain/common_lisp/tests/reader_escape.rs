use super::*;
use crate::domain::common_lisp::common_lisp_reader_escape_diagnostics;

#[test]
fn detects_escaped_closing_bar_that_absorbs_following_forms() {
    let input = "|\\|\n(define-arithmetic-table *binary-arithmetic-functions*)\n|";

    let diagnostics = common_lisp_reader_escape_diagnostics(input, Dialect::CommonLisp);

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code(), "suspicious-reader-escape");
}

#[test]
fn accepts_closed_symbol_with_an_escaped_pipe() {
    let diagnostics = common_lisp_reader_escape_diagnostics("|a\\|b|", Dialect::CommonLisp);

    assert!(diagnostics.is_empty());
}

#[test]
fn ignores_reader_escape_text_in_strings_and_comments() {
    let input = "\"|\\|\n(list)|\"\n; |\\| (list)|\n#| |\\|\n(list)| |#";

    let diagnostics = common_lisp_reader_escape_diagnostics(input, Dialect::CommonLisp);

    assert!(diagnostics.is_empty());
}
