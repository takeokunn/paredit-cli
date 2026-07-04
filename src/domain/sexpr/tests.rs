use super::*;

fn parse_path(path: &str) -> ExpressionPath {
    path.parse().expect("valid path")
}

#[test]
fn parses_balanced_document() {
    let tree = SyntaxTree::parse("(defun add (x y) (+ x y))").expect("valid");
    assert_eq!(tree.root_children().len(), 1);
}

#[test]
fn parses_reader_delimiters() {
    let tree = SyntaxTree::parse("(mapv inc [1 2 {:x 3}])").expect("valid");
    assert_eq!(
        Formatter::new(2).format(&tree),
        "(mapv\n  inc\n  [1\n    2\n    {:x 3}])\n"
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
fn replaces_expression() {
    let input = "(alpha beta gamma)";
    let tree = SyntaxTree::parse(input).expect("valid");
    let selection = tree.select_path(&parse_path("0.1")).expect("selection");
    assert_eq!(
        Edit::replace(input, selection, "delta"),
        "(alpha delta gamma)"
    );
}

#[test]
fn wraps_expression() {
    let input = "(alpha beta)";
    let tree = SyntaxTree::parse(input).expect("valid");
    let selection = tree.select_path(&parse_path("0.1")).expect("selection");
    assert_eq!(
        Edit::wrap(input, &tree, selection).unwrap(),
        "(alpha (beta))"
    );
}

#[test]
fn splices_list() {
    let input = "(alpha (beta gamma) delta)";
    let tree = SyntaxTree::parse(input).expect("valid");
    let selection = tree.select_path(&parse_path("0.1")).expect("selection");
    assert_eq!(
        Edit::splice(input, &tree, selection).unwrap(),
        "(alpha beta gamma delta)"
    );
}

#[test]
fn raises_expression() {
    let input = "(alpha (beta gamma) delta)";
    let tree = SyntaxTree::parse(input).expect("valid");
    let selection = tree.select_path(&parse_path("0.1.1")).expect("selection");
    assert_eq!(
        Edit::raise(input, &tree, selection).unwrap(),
        "(alpha gamma delta)"
    );
}

#[test]
fn slurps_forward() {
    let input = "(alpha beta) gamma";
    let tree = SyntaxTree::parse(input).expect("valid");
    let selection = tree.select_path(&parse_path("0")).expect("selection");
    assert_eq!(
        Edit::slurp_forward(input, &tree, selection).unwrap(),
        "(alpha beta gamma)"
    );
}

#[test]
fn barfs_forward() {
    let input = "(alpha beta gamma)";
    let tree = SyntaxTree::parse(input).expect("valid");
    let selection = tree.select_path(&parse_path("0")).expect("selection");
    assert_eq!(
        Edit::barf_forward(input, &tree, selection).unwrap(),
        "(alpha beta) gamma"
    );
}

#[test]
fn slurps_backward() {
    let input = "alpha (beta gamma)";
    let tree = SyntaxTree::parse(input).expect("valid");
    let selection = tree.select_path(&parse_path("1")).expect("selection");
    assert_eq!(
        Edit::slurp_backward(input, &tree, selection).unwrap(),
        "(alpha beta gamma)"
    );
}

#[test]
fn barfs_backward() {
    let input = "(alpha beta gamma)";
    let tree = SyntaxTree::parse(input).expect("valid");
    let selection = tree.select_path(&parse_path("0")).expect("selection");
    assert_eq!(
        Edit::barf_backward(input, &tree, selection).unwrap(),
        "alpha (beta gamma)"
    );
}

#[test]
fn formats_short_atom_lists_inline() {
    let input = "(defun add (x y) (+ x y))";
    let tree = SyntaxTree::parse(input).expect("valid");
    assert_eq!(
        Formatter::new(2).format(&tree),
        "(defun\n  add\n  (x y)\n  (+ x y))\n"
    );
}

#[test]
fn property_generated_formatter_output_is_parseable_and_stable() {
    let delimiters = [('(', ')'), ('[', ']'), ('{', '}')];

    for depth in 1..5 {
        for width in 1..6 {
            for (open, close) in delimiters {
                let mut input = String::new();
                for form_index in 0..8 {
                    input.push(open);
                    input.push_str(&format!("root-{depth}-{width}-{form_index}"));
                    for item_index in 0..width {
                        input.push(' ');
                        input.push_str(&format!("atom-{depth}-{item_index}"));
                    }
                    for nested_depth in 0..depth {
                        input.push(' ');
                        input.push(open);
                        input.push_str(&format!("nested-{nested_depth} leaf-{form_index}"));
                        input.push(close);
                    }
                    input.push(close);
                    input.push('\n');
                }

                let formatter = Formatter::new(2);
                let tree = SyntaxTree::parse(&input).expect("generated input parses");
                let formatted = formatter.format(&tree);
                let reparsed =
                    SyntaxTree::parse(&formatted).expect("formatted output parses again");
                let reformatted = formatter.format(&reparsed);

                assert_eq!(
                    formatted, reformatted,
                    "formatter output must be stable after reparsing"
                );
            }
        }
    }
}

#[test]
fn property_generated_rename_preserves_parse_and_atom_spans() {
    for index in 0..64 {
        let from = SymbolName::new(format!("old-symbol-{index}")).expect("valid symbol");
        let to = SymbolName::new(format!("new-symbol-{index}")).expect("valid symbol");
        let input = format!(
            "(defun {from} (x{index} y{index})\n  (let ((local-{index} ({from} x{index})))\n    (list local-{index} y{index} \"{from}\")))\n; {from} in comment\n({from} 1 2)\n",
            from = from.as_str()
        );

        let tree = SyntaxTree::parse(&input).expect("generated input parses");
        for occurrence in tree.atom_occurrences() {
            assert_eq!(
                &input[occurrence.span.as_range()],
                occurrence.text,
                "atom span must slice back to the exact atom text"
            );
        }

        let output = tree.rename_symbol(&input, &from, &to);
        let output_tree = SyntaxTree::parse(&output).expect("renamed output parses");
        let output_atoms = output_tree
            .atom_occurrences()
            .into_iter()
            .map(|occurrence| occurrence.text)
            .collect::<Vec<_>>();

        assert!(!output_atoms.iter().any(|atom| atom == from.as_str()));
        assert!(output_atoms.iter().any(|atom| atom == to.as_str()));
        assert!(output.contains(&format!("\"{}\"", from.as_str())));
        assert!(output.contains(&format!("; {} in comment", from.as_str())));
    }
}
