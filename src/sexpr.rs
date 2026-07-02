use std::fmt;
use std::ops::Range;
use std::str::FromStr;

use anyhow::{anyhow, Result};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Path(Vec<usize>);

impl FromStr for Path {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        if s.trim().is_empty() {
            return Ok(Self(Vec::new()));
        }
        let mut indexes = Vec::new();
        for part in s.split('.') {
            indexes.push(
                part.parse::<usize>()
                    .map_err(|_| anyhow!("invalid path segment: {part}"))?,
            );
        }
        Ok(Self(indexes))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxTree {
    nodes: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Node {
    kind: NodeKind,
    parent: Option<usize>,
    children: Vec<usize>,
    span: Range<usize>,
    open: Option<usize>,
    close: Option<usize>,
    text: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NodeKind {
    Root,
    List,
    Atom,
}

#[derive(Debug, Clone, Copy)]
pub struct Selection<'a> {
    tree: &'a SyntaxTree,
    node_id: usize,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ParseError {
    #[error("unexpected ')' at byte {0}")]
    UnexpectedClose(usize),
    #[error("unclosed list starting at byte {0}")]
    UnclosedList(usize),
    #[error("unterminated string starting at byte {0}")]
    UnterminatedString(usize),
}

impl SyntaxTree {
    pub fn parse(input: &str) -> std::result::Result<Self, ParseError> {
        let mut parser = Parser::new(input);
        parser.parse()
    }

    pub fn root_children(&self) -> &[usize] {
        &self.nodes[0].children
    }

    pub fn select_path(&self, path: &Path) -> Result<Selection<'_>> {
        let mut node_id = 0;
        for index in &path.0 {
            let node = &self.nodes[node_id];
            node_id = *node
                .children
                .get(*index)
                .ok_or_else(|| anyhow!("path segment {index} is out of range"))?;
        }
        if node_id == 0 {
            anyhow::bail!("root document cannot be edited directly");
        }
        Ok(Selection {
            tree: self,
            node_id,
        })
    }

    pub fn select_at(&self, offset: usize) -> Result<Selection<'_>> {
        let mut best = None;
        for (id, node) in self.nodes.iter().enumerate().skip(1) {
            if node.span.start <= offset && offset < node.span.end {
                match best {
                    None => best = Some(id),
                    Some(best_id) if node.span.len() < self.nodes[best_id].span.len() => {
                        best = Some(id)
                    }
                    _ => {}
                }
            }
        }
        best.map(|node_id| Selection {
            tree: self,
            node_id,
        })
        .ok_or_else(|| anyhow!("no expression contains byte offset {offset}"))
    }
}

impl<'a> Selection<'a> {
    pub fn text(self, input: &str) -> &str {
        let span = self.span();
        &input[span]
    }

    fn node(self) -> &'a Node {
        &self.tree.nodes[self.node_id]
    }

    fn span(self) -> Range<usize> {
        self.node().span.clone()
    }
}

struct Parser<'a> {
    input: &'a str,
    bytes: &'a [u8],
    pos: usize,
    nodes: Vec<Node>,
    stack: Vec<usize>,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        let root = Node {
            kind: NodeKind::Root,
            parent: None,
            children: Vec::new(),
            span: 0..input.len(),
            open: None,
            close: None,
            text: None,
        };
        Self {
            input,
            bytes: input.as_bytes(),
            pos: 0,
            nodes: vec![root],
            stack: vec![0],
        }
    }

    fn parse(&mut self) -> std::result::Result<SyntaxTree, ParseError> {
        while self.pos < self.bytes.len() {
            self.skip_trivia();
            if self.pos >= self.bytes.len() {
                break;
            }
            match self.bytes[self.pos] {
                b'(' => self.open_list(),
                b')' => self.close_list()?,
                b'"' => self.atom_string()?,
                _ => self.atom(),
            }
        }
        if self.stack.len() > 1 {
            let open = self.nodes[*self.stack.last().expect("root is always present")]
                .open
                .expect("list has an open byte");
            return Err(ParseError::UnclosedList(open));
        }
        Ok(SyntaxTree {
            nodes: std::mem::take(&mut self.nodes),
        })
    }

    fn skip_trivia(&mut self) {
        loop {
            while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_whitespace() {
                self.pos += 1;
            }
            if self.pos < self.bytes.len() && self.bytes[self.pos] == b';' {
                while self.pos < self.bytes.len() && self.bytes[self.pos] != b'\n' {
                    self.pos += 1;
                }
                continue;
            }
            break;
        }
    }

    fn open_list(&mut self) {
        let parent = *self.stack.last().expect("root is always present");
        let id = self.nodes.len();
        self.nodes.push(Node {
            kind: NodeKind::List,
            parent: Some(parent),
            children: Vec::new(),
            span: self.pos..self.pos + 1,
            open: Some(self.pos),
            close: None,
            text: None,
        });
        self.nodes[parent].children.push(id);
        self.stack.push(id);
        self.pos += 1;
    }

    fn close_list(&mut self) -> std::result::Result<(), ParseError> {
        if self.stack.len() == 1 {
            return Err(ParseError::UnexpectedClose(self.pos));
        }
        let id = self.stack.pop().expect("checked stack length");
        self.nodes[id].span.end = self.pos + 1;
        self.nodes[id].close = Some(self.pos);
        self.pos += 1;
        Ok(())
    }

    fn atom_string(&mut self) -> std::result::Result<(), ParseError> {
        let start = self.pos;
        self.pos += 1;
        let mut escaped = false;
        while self.pos < self.bytes.len() {
            let byte = self.bytes[self.pos];
            self.pos += 1;
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                self.push_atom(start, self.pos);
                return Ok(());
            }
        }
        Err(ParseError::UnterminatedString(start))
    }

    fn atom(&mut self) {
        let start = self.pos;
        while self.pos < self.bytes.len() {
            let byte = self.bytes[self.pos];
            if byte.is_ascii_whitespace() || matches!(byte, b'(' | b')' | b';') {
                break;
            }
            self.pos += 1;
        }
        self.push_atom(start, self.pos);
    }

    fn push_atom(&mut self, start: usize, end: usize) {
        let parent = *self.stack.last().expect("root is always present");
        let id = self.nodes.len();
        self.nodes.push(Node {
            kind: NodeKind::Atom,
            parent: Some(parent),
            children: Vec::new(),
            span: start..end,
            open: None,
            close: None,
            text: Some(self.input[start..end].to_string()),
        });
        self.nodes[parent].children.push(id);
    }
}

pub struct Formatter {
    indent: usize,
}

impl Formatter {
    pub fn new(indent: usize) -> Self {
        Self { indent }
    }

    pub fn format(&self, tree: &SyntaxTree) -> String {
        let mut output = String::new();
        for (position, child) in tree.root_children().iter().enumerate() {
            if position > 0 {
                output.push('\n');
            }
            self.format_node(tree, *child, 0, &mut output);
            output.push('\n');
        }
        output
    }

    fn format_node(&self, tree: &SyntaxTree, node_id: usize, depth: usize, output: &mut String) {
        let node = &tree.nodes[node_id];
        match node.kind {
            NodeKind::Root => unreachable!("root is not formatted directly"),
            NodeKind::Atom => {
                output.push_str(node.text.as_deref().expect("atom has source text"));
            }
            NodeKind::List if node.children.is_empty() => output.push_str("()"),
            NodeKind::List if self.inline_list(tree, node_id).is_some() => {
                output.push_str(
                    &self
                        .inline_list(tree, node_id)
                        .expect("checked inline list"),
                );
            }
            NodeKind::List => {
                output.push('(');
                for (position, child) in node.children.iter().enumerate() {
                    if position == 0 {
                        self.format_node(tree, *child, depth + 1, output);
                    } else {
                        output.push('\n');
                        output.push_str(&" ".repeat((depth + 1) * self.indent));
                        self.format_node(tree, *child, depth + 1, output);
                    }
                }
                output.push(')');
            }
        }
    }

    fn inline_list(&self, tree: &SyntaxTree, node_id: usize) -> Option<String> {
        let node = &tree.nodes[node_id];
        let mut output = String::from("(");
        for (position, child) in node.children.iter().enumerate() {
            let child = &tree.nodes[*child];
            if child.kind != NodeKind::Atom {
                return None;
            }
            if position > 0 {
                output.push(' ');
            }
            output.push_str(child.text.as_deref().expect("atom has source text"));
        }
        output.push(')');
        (output.len() <= 80).then_some(output)
    }
}

pub struct Edit;

impl Edit {
    pub fn replace(input: &str, selection: Selection<'_>, replacement: &str) -> String {
        let span = selection.span();
        replace_range(input, span, replacement)
    }

    pub fn kill(input: &str, _tree: &SyntaxTree, selection: Selection<'_>) -> Result<String> {
        let span = expand_removal(input, selection.span());
        Ok(replace_range(input, span, ""))
    }

    pub fn wrap(input: &str, _tree: &SyntaxTree, selection: Selection<'_>) -> Result<String> {
        Ok(Self::replace(
            input,
            selection,
            &format!("({})", selection.text(input)),
        ))
    }

    pub fn splice(input: &str, _tree: &SyntaxTree, selection: Selection<'_>) -> Result<String> {
        let node = selection.node();
        ensure_list(node)?;
        let open = node.open.expect("list has open byte");
        let close = node.close.expect("list has close byte");
        let mut output = String::with_capacity(input.len().saturating_sub(2));
        output.push_str(&input[..open]);
        output.push_str(&input[open + 1..close]);
        output.push_str(&input[close + 1..]);
        Ok(output)
    }

    pub fn raise(input: &str, _tree: &SyntaxTree, selection: Selection<'_>) -> Result<String> {
        let node = selection.node();
        let parent_id = node
            .parent
            .ok_or_else(|| anyhow!("selected node has no parent"))?;
        let parent = &selection.tree.nodes[parent_id];
        if parent.kind == NodeKind::Root {
            anyhow::bail!("cannot raise a top-level expression");
        }
        Ok(replace_range(
            input,
            parent.span.clone(),
            selection.text(input),
        ))
    }

    pub fn slurp_forward(
        input: &str,
        tree: &SyntaxTree,
        selection: Selection<'_>,
    ) -> Result<String> {
        let node = selection.node();
        ensure_list(node)?;
        let sibling = next_sibling(tree, selection.node_id)
            .ok_or_else(|| anyhow!("selected list has no next sibling to slurp"))?;
        let close = node.close.expect("list has close byte");
        let insertion = format!(" {}", &input[tree.nodes[sibling].span.clone()]);
        let removal = expand_removal(input, tree.nodes[sibling].span.clone());
        Ok(remove_then_insert(input, removal, close, &insertion))
    }

    pub fn slurp_backward(
        input: &str,
        tree: &SyntaxTree,
        selection: Selection<'_>,
    ) -> Result<String> {
        let node = selection.node();
        ensure_list(node)?;
        let sibling = previous_sibling(tree, selection.node_id)
            .ok_or_else(|| anyhow!("selected list has no previous sibling to slurp"))?;
        let open = node.open.expect("list has open byte") + 1;
        let insertion = format!("{} ", &input[tree.nodes[sibling].span.clone()]);
        let removal = expand_removal(input, tree.nodes[sibling].span.clone());
        Ok(remove_then_insert(input, removal, open, &insertion))
    }

    pub fn barf_forward(
        input: &str,
        tree: &SyntaxTree,
        selection: Selection<'_>,
    ) -> Result<String> {
        let node = selection.node();
        ensure_list(node)?;
        let child = *node
            .children
            .last()
            .ok_or_else(|| anyhow!("cannot barf from an empty list"))?;
        let close = node.close.expect("list has close byte");
        let child_span = tree.nodes[child].span.clone();
        let insertion = format!(" {}", &input[child_span.clone()]);
        let removal = expand_removal(input, child_span);
        Ok(remove_then_insert(input, removal, close + 1, &insertion))
    }

    pub fn barf_backward(
        input: &str,
        tree: &SyntaxTree,
        selection: Selection<'_>,
    ) -> Result<String> {
        let node = selection.node();
        ensure_list(node)?;
        let child = *node
            .children
            .first()
            .ok_or_else(|| anyhow!("cannot barf from an empty list"))?;
        let open = node.open.expect("list has open byte");
        let child_span = tree.nodes[child].span.clone();
        let insertion = format!("{} ", &input[child_span.clone()]);
        let removal = expand_removal(input, child_span);
        Ok(remove_then_insert(input, removal, open, &insertion))
    }
}

fn ensure_list(node: &Node) -> Result<()> {
    if node.kind != NodeKind::List {
        anyhow::bail!("operation requires a list expression");
    }
    Ok(())
}

fn next_sibling(tree: &SyntaxTree, node_id: usize) -> Option<usize> {
    let parent = tree.nodes[node_id].parent?;
    let siblings = &tree.nodes[parent].children;
    let position = siblings.iter().position(|id| *id == node_id)?;
    siblings.get(position + 1).copied()
}

fn previous_sibling(tree: &SyntaxTree, node_id: usize) -> Option<usize> {
    let parent = tree.nodes[node_id].parent?;
    let siblings = &tree.nodes[parent].children;
    let position = siblings.iter().position(|id| *id == node_id)?;
    position
        .checked_sub(1)
        .and_then(|previous| siblings.get(previous).copied())
}

fn replace_range(input: &str, range: Range<usize>, replacement: &str) -> String {
    let mut output = String::with_capacity(input.len() + replacement.len());
    output.push_str(&input[..range.start]);
    output.push_str(replacement);
    output.push_str(&input[range.end..]);
    output
}

fn expand_removal(input: &str, span: Range<usize>) -> Range<usize> {
    let bytes = input.as_bytes();
    let mut start = span.start;
    let mut end = span.end;
    if end < bytes.len() && bytes[end].is_ascii_whitespace() {
        while end < bytes.len() && bytes[end].is_ascii_whitespace() {
            end += 1;
        }
    } else {
        while start > 0 && bytes[start - 1].is_ascii_whitespace() {
            start -= 1;
        }
    }
    start..end
}

fn remove_then_insert(
    input: &str,
    removal: Range<usize>,
    insertion_at: usize,
    insertion: &str,
) -> String {
    let adjusted_insertion_at = if insertion_at > removal.end {
        insertion_at - (removal.end - removal.start)
    } else {
        insertion_at
    };
    let removed = replace_range(input, removal, "");
    replace_range(
        &removed,
        adjusted_insertion_at..adjusted_insertion_at,
        insertion,
    )
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (position, index) in self.0.iter().enumerate() {
            if position > 0 {
                write!(f, ".")?;
            }
            write!(f, "{index}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_path(path: &str) -> Path {
        path.parse().expect("valid path")
    }

    #[test]
    fn parses_balanced_document() {
        let tree = SyntaxTree::parse("(defun add (x y) (+ x y))").expect("valid");
        assert_eq!(tree.root_children().len(), 1);
    }

    #[test]
    fn rejects_unbalanced_document() {
        assert_eq!(
            SyntaxTree::parse("(defun x").unwrap_err(),
            ParseError::UnclosedList(0)
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
}
