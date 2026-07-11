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
fn preserves_stacked_quasiquote_and_unquote_prefix_order() {
    let tree = SyntaxTree::parse("``(list ,quoted ,,evaluated)").expect("valid");
    let quasiquoted = tree.select_path(&parse_path("0")).expect("quasiquoted").view();

    assert_eq!(
        quasiquoted.reader_prefixes,
        vec![ReaderPrefix::Quasiquote, ReaderPrefix::Quasiquote]
    );
    assert_eq!(
        quasiquoted.children[1].reader_prefixes,
        vec![ReaderPrefix::Unquote]
    );
    assert_eq!(
        quasiquoted.children[2].reader_prefixes,
        vec![ReaderPrefix::Unquote, ReaderPrefix::Unquote]
    );
}

#[test]
fn parses_clojure_hash_literals_as_one_node() {
    // `#{...}` (set) and `#(...)` (anonymous fn / CL-Scheme vector literal)
    // glue `#` directly onto the following collection with no space in every
    // supported dialect, so both must parse as one prefixed list rather than
    // a disconnected `#` atom followed by an unrelated sibling list.
    let tree = SyntaxTree::parse("#{1 2 3} #(+ % 1)").expect("valid");
    let root = tree.root_children();
    assert_eq!(root.len(), 2);

    let set = tree.select_path(&parse_path("0")).expect("set").view();
    assert_eq!(set.reader_prefixes, vec![ReaderPrefix::HashLiteral]);
    assert_eq!(set.delimiter, Some(Delimiter::Brace));

    let anon_fn = tree.select_path(&parse_path("1")).expect("anon_fn").view();
    assert_eq!(anon_fn.reader_prefixes, vec![ReaderPrefix::HashLiteral]);
    assert_eq!(anon_fn.delimiter, Some(Delimiter::Paren));
}

#[test]
fn parses_clojure_metadata_prefix_on_map_and_atom() {
    let tree = SyntaxTree::parse(r#"^{:doc "x"} target ^:private y"#).expect("valid");
    let root = tree.root_children();
    assert_eq!(root.len(), 4);

    let metadata_map = tree.select_path(&parse_path("0")).expect("map").view();
    assert_eq!(metadata_map.reader_prefixes, vec![ReaderPrefix::Metadata]);
    assert_eq!(metadata_map.delimiter, Some(Delimiter::Brace));

    let target = tree.select_path(&parse_path("1")).expect("target").view();
    assert_eq!(target.reader_prefixes, Vec::new());
    assert_eq!(target.text.as_deref(), Some("target"));

    let metadata_keyword = tree.select_path(&parse_path("2")).expect("kw").view();
    assert_eq!(
        metadata_keyword.reader_prefixes,
        vec![ReaderPrefix::Metadata]
    );
    assert_eq!(metadata_keyword.text.as_deref(), Some("^:private"));
}

#[test]
fn parses_clojure_reader_conditionals_as_one_node() {
    let tree =
        SyntaxTree::parse("#?(:clj (foo) :cljs (bar)) #?@(:clj [a] :cljs [b])").expect("valid");
    let root = tree.root_children();
    assert_eq!(root.len(), 2);

    let conditional = tree
        .select_path(&parse_path("0"))
        .expect("conditional")
        .view();
    assert_eq!(
        conditional.reader_prefixes,
        vec![ReaderPrefix::ReaderConditional]
    );
    assert_eq!(conditional.delimiter, Some(Delimiter::Paren));

    let splicing = tree.select_path(&parse_path("1")).expect("splicing").view();
    assert_eq!(
        splicing.reader_prefixes,
        vec![ReaderPrefix::ReaderConditionalSplicing]
    );
    assert_eq!(splicing.delimiter, Some(Delimiter::Paren));
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
fn skips_clojure_discard_forms() {
    // `#_` is Clojure's discard reader macro: it reads and discards exactly
    // one following form, the same shape as Scheme/CL `#;` datum comments,
    // so it must not surface as a live tree node or a rename target.
    let input = "(foo bar) #_(foo baz) (foo qux)";
    let tree = SyntaxTree::parse(input).expect("valid");
    assert_eq!(tree.root_children().len(), 2);
    let from = SymbolName::new("foo").expect("symbol");
    let to = SymbolName::new("bar").expect("symbol");
    assert_eq!(
        tree.rename_symbol(input, &from, &to),
        "(bar bar) #_(foo baz) (bar qux)"
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
fn parses_pipe_escaped_symbol_with_embedded_space_as_one_atom() {
    // `|Foo Bar|` is a single multiple-escaped symbol (CLHS 2.1.4.2); the
    // embedded space must not act as a token boundary.
    let input = "(defun |Foo Bar| (x) (+ x 1))";
    let tree = SyntaxTree::parse(input).expect("valid");
    let form = tree.select_path(&parse_path("0")).expect("form").view();
    assert_eq!(form.children[1].text.as_deref(), Some("|Foo Bar|"));
    assert_eq!(form.children[2].span.slice(input), "(x)");
}

#[test]
fn parses_pipe_escaped_symbol_with_nested_single_escape() {
    let tree = SyntaxTree::parse(r"|a\|b|").expect("valid");
    let atom = tree.select_path(&parse_path("0")).expect("atom").view();
    assert_eq!(atom.text.as_deref(), Some(r"|a\|b|"));
}

#[test]
fn rejects_unterminated_pipe_escaped_symbol() {
    assert_eq!(
        SyntaxTree::parse("(defun |Foo (x) (+ x 1))").unwrap_err(),
        ParseError::UnterminatedSymbol(7)
    );
}

#[test]
fn feature_dispatch_scans_separately_from_feature_expression() {
    // `#+`/`#-` must scan as their own token so `#+sbcl` and
    // `#+(and sbcl x86-64)` produce the same tree shape: dispatch, feature
    // expression, guarded datum as three siblings. Otherwise a bare feature
    // symbol glues onto `#+` into one opaque atom while a compound feature
    // expression stays a separate list, hiding the feature symbol from
    // find/rename in the bare spelling only.
    let simple_input = "(defun f () #+sbcl (declare (optimize speed)) 1)";
    let simple = SyntaxTree::parse(simple_input).expect("valid");
    let form = simple.select_path(&parse_path("0")).expect("form").view();
    assert_eq!(form.children[3].text.as_deref(), Some("#+"));
    assert_eq!(form.children[4].text.as_deref(), Some("sbcl"));
    assert_eq!(
        form.children[5].span.slice(simple_input),
        "(declare (optimize speed))"
    );

    let compound_input = "(defun f () #+(and sbcl x86-64) (declare (optimize speed)) 1)";
    let compound = SyntaxTree::parse(compound_input).expect("valid");
    let form = compound.select_path(&parse_path("0")).expect("form").view();
    assert_eq!(form.children[3].text.as_deref(), Some("#+"));
    assert_eq!(
        form.children[4].span.slice(compound_input),
        "(and sbcl x86-64)"
    );
    assert_eq!(
        form.children[5].span.slice(compound_input),
        "(declare (optimize speed))"
    );
}

#[test]
fn feature_dispatch_negative_variant_scans_separately_too() {
    let input = "(declare #-sbcl (optimize speed))";
    let tree = SyntaxTree::parse(input).expect("valid");
    let form = tree.select_path(&parse_path("0")).expect("form").view();
    assert_eq!(form.children[1].text.as_deref(), Some("#-"));
    assert_eq!(form.children[2].text.as_deref(), Some("sbcl"));
    assert_eq!(form.children[3].span.slice(input), "(optimize speed)");
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
