use super::*;

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

        let output = tree.rename_symbol(&from, &to);
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
