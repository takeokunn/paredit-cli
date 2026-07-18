use super::*;

#[test]
fn replaces_expression() {
    let input = "(alpha beta gamma)";
    let tree = SyntaxTree::parse(input).expect("valid");
    let selection = tree.select_path(&parse_path("0.1")).expect("selection");
    assert_eq!(
        Edit::replace(input, selection, "delta").unwrap(),
        "(alpha delta gamma)"
    );
}

#[test]
fn edit_rejects_input_that_does_not_match_selection_source() {
    let source = "(alpha beta)";
    let tree = SyntaxTree::parse(source).expect("valid");
    let selection = tree.select_path(&parse_path("0.1")).expect("selection");

    assert!(Edit::replace("(alpha zeta)", selection, "delta").is_err());
    assert!(Edit::wrap("(alpha zeta)", &tree, selection).is_err());
}

#[test]
fn edit_rejects_input_with_incompatible_utf8_boundaries() {
    let source = "(a x)";
    let tree = SyntaxTree::parse(source).expect("valid");
    let selection = tree.select_path(&parse_path("0.1")).expect("selection");

    assert!(Edit::replace("(é x)", selection, "delta").is_err());
}

#[test]
fn edit_rejects_corrupted_selection_span_without_panicking() {
    let source = "(alpha beta)";
    let mut tree = SyntaxTree::parse(source).expect("valid");
    let selected_id = tree
        .select_path(&parse_path("0.1"))
        .expect("selection")
        .node_id;
    tree.nodes[selected_id.get()].span = ByteSpan::new(ByteOffset::new(9), ByteOffset::new(7));
    let selection = Selection {
        tree: &tree,
        node_id: selected_id,
    };

    assert!(Edit::replace(source, selection, "delta").is_err());
    assert!(Edit::kill(source, &tree, selection).is_err());
}

#[test]
fn edit_rejects_selection_from_a_different_tree() {
    let source = "(alpha beta)";
    let first = SyntaxTree::parse(source).expect("valid");
    let second = SyntaxTree::parse(source).expect("valid");
    let selection = first.select_path(&parse_path("0.1")).expect("selection");

    assert!(Edit::kill(source, &second, selection).is_err());
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
fn transposes_expression_forward_while_keeping_trivia_in_place() {
    let input = "(alpha  ;; slot comment\n beta gamma)";
    let tree = SyntaxTree::parse(input).expect("valid");
    let selection = tree.select_path(&parse_path("0.0")).expect("selection");
    let result = Edit::transpose_forward(input, &tree, selection).unwrap();
    assert_eq!(result, "(beta  ;; slot comment\n alpha gamma)");
    SyntaxTree::parse(&result).expect("result stays balanced");
}

#[test]
fn transposes_expression_backward_while_keeping_trivia_in_place() {
    let input = "(alpha  ;; slot comment\n beta gamma)";
    let tree = SyntaxTree::parse(input).expect("valid");
    let selection = tree.select_path(&parse_path("0.1")).expect("selection");
    let result = Edit::transpose_backward(input, &tree, selection).unwrap();
    assert_eq!(result, "(beta  ;; slot comment\n alpha gamma)");
    SyntaxTree::parse(&result).expect("result stays balanced");
}

#[test]
fn transpose_rejects_sibling_boundaries() {
    let input = "(alpha beta)";
    let tree = SyntaxTree::parse(input).expect("valid");
    let first = tree.select_path(&parse_path("0.0")).expect("selection");
    let last = tree.select_path(&parse_path("0.1")).expect("selection");
    assert!(Edit::transpose_backward(input, &tree, first).is_err());
    assert!(Edit::transpose_forward(input, &tree, last).is_err());
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
fn slurps_forward_preserves_trailing_newline() {
    let input = "(foo) bar\n";
    let tree = SyntaxTree::parse(input).expect("valid");
    let selection = tree.select_path(&parse_path("0")).expect("selection");
    assert_eq!(
        Edit::slurp_forward(input, &tree, selection).unwrap(),
        "(foo bar)\n"
    );
}

#[test]
fn slurps_forward_keeps_following_sibling_separator() {
    let input = "(foo) bar baz\n";
    let tree = SyntaxTree::parse(input).expect("valid");
    let selection = tree.select_path(&parse_path("0")).expect("selection");
    assert_eq!(
        Edit::slurp_forward(input, &tree, selection).unwrap(),
        "(foo bar) baz\n"
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
fn kills_last_child() {
    let input = "(defun f (x)\n  (* x x))";
    let tree = SyntaxTree::parse(input).expect("valid");
    let selection = tree.select_path(&parse_path("0.3")).expect("selection");
    assert_eq!(
        Edit::kill(input, &tree, selection).unwrap(),
        "(defun f (x))"
    );
}

#[test]
fn kills_last_child_without_swallowing_preceding_comment_newline() {
    let input = "(defun f (x)\n  ;; important comment\n  (* x x))";
    let tree = SyntaxTree::parse(input).expect("valid");
    let selection = tree.select_path(&parse_path("0.3")).expect("selection");
    let result = Edit::kill(input, &tree, selection).unwrap();
    assert_eq!(result, "(defun f (x)\n  ;; important comment\n)");
    SyntaxTree::parse(&result).expect("result stays balanced");
}

#[test]
fn slurps_forward_without_swallowing_preceding_comment_newline() {
    let input = "(let ((a 1))\n  (foo a)\n  ;; note\n  (bar a))";
    let tree = SyntaxTree::parse(input).expect("valid");
    let selection = tree.select_path(&parse_path("0.2")).expect("selection");
    let result = Edit::slurp_forward(input, &tree, selection).unwrap();
    SyntaxTree::parse(&result).expect("result stays balanced");
}

#[test]
fn barfs_forward_without_swallowing_preceding_comment_newline() {
    let input = "(list a\n  ;; last item\n  b)";
    let tree = SyntaxTree::parse(input).expect("valid");
    let selection = tree.select_path(&parse_path("0")).expect("selection");
    let result = Edit::barf_forward(input, &tree, selection).unwrap();
    assert_eq!(result, "(list a\n  ;; last item\n) b");
    SyntaxTree::parse(&result).expect("result stays balanced");
}

#[test]
fn normalizes_trailing_trivia_only_on_changed_lines() {
    let input = "(alpha beta) \n(unchanged) \n";
    let rewritten = "(alpha gamma) \n(unchanged) \n".to_owned();

    assert_eq!(
        Edit::normalize_changed_line_trivia(input, rewritten).unwrap(),
        "(alpha gamma)\n(unchanged) \n"
    );
}

#[test]
fn preserves_trailing_spaces_inside_multiline_atoms_and_block_comments() {
    let input = "(print \"old  \nvalue\")\n#| old  \ncomment |#\n";
    let rewritten = "(print \"new  \nvalue\")\n#| new  \ncomment |#\n".to_owned();

    assert_eq!(
        Edit::normalize_changed_line_trivia(input, rewritten.clone()).unwrap(),
        rewritten
    );
}
