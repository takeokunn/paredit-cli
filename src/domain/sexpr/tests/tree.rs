use super::*;

#[test]
fn selects_by_path() {
    let input = "(defun add (x y) (+ x y))";
    let tree = SyntaxTree::parse(input).expect("valid");
    let selection = tree.select_path(&parse_path("0.2")).expect("selection");
    assert_eq!(selection.text(input), "(x y)");
}

#[test]
fn selects_by_offset() {
    let input = "(alpha (beta gamma))";
    let tree = SyntaxTree::parse(input).expect("valid");
    let selection = tree.select_at(9).expect("selection");
    assert_eq!(selection.text(input), "beta");
}

#[test]
fn outlines_top_level_forms() {
    let input = "(defun add (x y) (+ x y))\n(defvar *x* 1)";
    let tree = SyntaxTree::parse(input).expect("valid");
    let outline = tree.outline(|head| head.starts_with("def"));
    assert_eq!(outline.len(), 2);
    assert_eq!(outline[0].path.to_string(), "0");
    assert_eq!(outline[0].head.as_deref(), Some("defun"));
    assert!(outline[0].definition_like);
}

#[test]
fn finds_atoms_without_comments_or_string_contents() {
    let input = "(message \"foo\") ; foo\n(foo foo)";
    let tree = SyntaxTree::parse(input).expect("valid");
    let paths = tree
        .atom_occurrences()
        .into_iter()
        .filter(|occurrence| occurrence.text == "foo")
        .map(|occurrence| occurrence.path.to_string())
        .collect::<Vec<_>>();
    assert_eq!(paths, vec!["1.0", "1.1"]);
}

#[test]
fn renames_symbols_without_touching_strings_or_comments() {
    let input = "(message \"foo\") ; foo\n(foo foo)";
    let tree = SyntaxTree::parse(input).expect("valid");
    let output = tree.rename_symbol(
        input,
        &SymbolName::new("foo").unwrap(),
        &SymbolName::new("bar").unwrap(),
    );
    assert_eq!(output, "(message \"foo\") ; foo\n(bar bar)");
}

#[test]
fn renames_unqualified_occurrences_of_package_qualified_symbol() {
    let input = "(defun cl-user:foo () foo)\n(foo cl-user:foo)";
    let tree = SyntaxTree::parse(input).expect("valid");
    let output = tree.rename_symbol(
        input,
        &SymbolName::new("cl-user:foo").unwrap(),
        &SymbolName::new("bar").unwrap(),
    );
    assert_eq!(output, "(defun bar () bar)\n(bar bar)");
}

#[test]
fn treats_reader_prefix_as_part_of_selection_span() {
    let input = "'(alpha beta)";
    let tree = SyntaxTree::parse(input).expect("valid");
    let selection = tree.select_at(0).expect("selection");
    assert_eq!(selection.text(input), "'(alpha beta)");
}

#[test]
fn does_not_rename_quoted_atom_occurrences() {
    let input = "'foo foo #'foo";
    let tree = SyntaxTree::parse(input).expect("valid");
    let output = tree.rename_symbol(
        input,
        &SymbolName::new("foo").unwrap(),
        &SymbolName::new("bar").unwrap(),
    );
    assert_eq!(output, "'foo bar #'foo");
}

#[test]
fn does_not_rename_atoms_inside_reader_eval_forms() {
    let input = "#.(foo (bar foo)) foo";
    let tree = SyntaxTree::parse(input).expect("valid");
    let output = tree.rename_symbol(
        input,
        &SymbolName::new("foo").unwrap(),
        &SymbolName::new("bar").unwrap(),
    );
    assert_eq!(output, "#.(foo (bar foo)) bar");
}
