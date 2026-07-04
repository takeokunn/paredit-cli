use super::*;

#[test]
fn formats_short_atom_lists_inline() {
    let input = "(defun add (x y) (+ x y))";
    let tree = SyntaxTree::parse(input).expect("valid");
    assert_eq!(
        Formatter::new(2).format(&tree),
        "(defun\n  add\n  (x y)\n  (+ x y))\n"
    );
}
