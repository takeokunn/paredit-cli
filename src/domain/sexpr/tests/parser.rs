use super::*;

#[test]
fn parses_balanced_document() {
    let tree = SyntaxTree::parse("(defun add (x y) (+ x y))").expect("valid");
    assert_eq!(tree.root_children().len(), 1);
}

#[test]
fn parses_reader_delimiters() {
    let tree = SyntaxTree::parse("(mapv inc [1 2 {:x 3}])").expect("valid");
    assert_eq!(Formatter::new(2).format(&tree), "(mapv inc [1 2 {:x 3}])\n");
}

#[test]
fn parses_common_lisp_reader_prefixes() {
    let tree = SyntaxTree::parse("'value #'call `(list ,item ,@rest)").expect("valid");
    let root = tree.root_children();
    assert_eq!(root.len(), 3);

    let quoted = tree.select_path(&parse_path("0")).expect("quoted").view();
    assert_eq!(quoted.reader_prefixes, vec![ReaderPrefix::Quote]);
    assert_eq!(quoted.text.as_deref(), Some("'value"));

    let function = tree.select_path(&parse_path("1")).expect("function").view();
    assert_eq!(function.reader_prefixes, vec![ReaderPrefix::Function]);
    assert_eq!(function.text.as_deref(), Some("#'call"));

    let quasiquoted = tree
        .select_path(&parse_path("2"))
        .expect("quasiquoted")
        .view();
    assert_eq!(quasiquoted.reader_prefixes, vec![ReaderPrefix::Quasiquote]);
    assert_eq!(
        quasiquoted.children[1].reader_prefixes,
        vec![ReaderPrefix::Unquote]
    );
    assert_eq!(
        quasiquoted.children[2].reader_prefixes,
        vec![ReaderPrefix::UnquoteSplicing]
    );
}

#[test]
fn parses_common_lisp_reader_eval_as_opaque_form() {
    let tree = SyntaxTree::parse("#.(foo (bar baz))").expect("valid");
    let root = tree.root_children();
    assert_eq!(root.len(), 1);

    let expression = tree
        .select_path(&parse_path("0"))
        .expect("expression")
        .view();
    assert_eq!(expression.reader_prefixes, vec![ReaderPrefix::ReadEval]);
    assert_eq!(expression.kind, ExpressionKind::List);
}

#[test]
fn skips_common_lisp_reader_comments() {
    let input = "(foo bar) #;(foo baz) (foo qux)";
    let tree = SyntaxTree::parse(input).expect("valid");
    assert_eq!(tree.root_children().len(), 2);
    let from = SymbolName::new("foo").expect("symbol");
    let to = SymbolName::new("bar").expect("symbol");
    assert_eq!(
        tree.rename_symbol(input, &from, &to),
        "(bar bar) #;(foo baz) (bar qux)"
    );
}

#[test]
fn keeps_reader_eval_body_opaque_during_rename() {
    let input = "#.(foo foo) foo";
    let tree = SyntaxTree::parse(input).expect("valid");
    let output = tree.rename_symbol(
        input,
        &SymbolName::new("foo").unwrap(),
        &SymbolName::new("bar").unwrap(),
    );
    assert_eq!(output, "#.(foo foo) bar");
}

#[test]
fn skips_nested_common_lisp_block_comments() {
    let input = "(foo #| outer foo #| nested |# still outer |# bar) (foo baz)";
    let tree = SyntaxTree::parse(input).expect("valid");
    assert_eq!(tree.root_children().len(), 2);
    let from = SymbolName::new("foo").expect("symbol");
    let to = SymbolName::new("bar").expect("symbol");
    assert_eq!(
        tree.rename_symbol(input, &from, &to),
        "(bar #| outer foo #| nested |# still outer |# bar) (bar baz)"
    );
}

#[test]
fn rejects_unterminated_common_lisp_block_comment() {
    assert_eq!(
        SyntaxTree::parse("#| outer #| nested |#").unwrap_err(),
        ParseError::UnterminatedBlockComment(0)
    );
}

#[test]
fn parses_character_literals_with_delimiter_values() {
    // `#\[`, `#\)`, and `#\]` are character literals, not structural delimiters.
    let input = "(write-char #\\[ stream) (list #\\) #\\] #\\()";
    let tree = SyntaxTree::parse(input).expect("valid");
    assert_eq!(tree.root_children().len(), 2);

    let first = tree.select_path(&parse_path("0")).expect("first").view();
    assert_eq!(first.children[1].text.as_deref(), Some("#\\["));

    let second = tree.select_path(&parse_path("1")).expect("second").view();
    assert_eq!(second.children[1].text.as_deref(), Some("#\\)"));
    assert_eq!(second.children[2].text.as_deref(), Some("#\\]"));
    assert_eq!(second.children[3].text.as_deref(), Some("#\\("));
}

#[test]
fn parses_named_and_whitespace_character_literals() {
    // Named characters keep their trailing constituents; `#\ ` escapes a space.
    let tree = SyntaxTree::parse("(char= c #\\Space #\\a)").expect("valid");
    let form = tree.select_path(&parse_path("0")).expect("form").view();
    assert_eq!(form.children[2].text.as_deref(), Some("#\\Space"));
    assert_eq!(form.children[3].text.as_deref(), Some("#\\a"));

    let space = SyntaxTree::parse("(x #\\ )").expect("valid");
    let form = space.select_path(&parse_path("0")).expect("form").view();
    assert_eq!(form.children[1].text.as_deref(), Some("#\\ "));
}

#[test]
fn character_literal_does_not_break_rename() {
    let input = "(defun f () (write-char #\\[ out) (foo))";
    let tree = SyntaxTree::parse(input).expect("valid");
    assert_eq!(
        tree.rename_symbol(
            input,
            &SymbolName::new("foo").unwrap(),
            &SymbolName::new("bar").unwrap(),
        ),
        "(defun f () (write-char #\\[ out) (bar))"
    );
}

#[test]
fn rejects_unbalanced_document() {
    assert_eq!(
        SyntaxTree::parse("(defun x").unwrap_err(),
        ParseError::UnclosedList(0)
    );
}

#[test]
fn rejects_mismatched_delimiter() {
    assert_eq!(
        SyntaxTree::parse("(alpha]").unwrap_err(),
        ParseError::MismatchedClose {
            found: ']',
            expected: ')',
            position: 6
        }
    );
}
