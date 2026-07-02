use std::fmt;
use std::ops::Range;
use std::str::FromStr;

use anyhow::{Result, anyhow};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ByteOffset(usize);

impl ByteOffset {
    pub const fn new(value: usize) -> Self {
        Self(value)
    }

    pub const fn get(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ByteSpan {
    start: ByteOffset,
    end: ByteOffset,
}

impl ByteSpan {
    pub const fn new(start: ByteOffset, end: ByteOffset) -> Self {
        Self { start, end }
    }

    pub const fn start(&self) -> ByteOffset {
        self.start
    }

    pub const fn end(&self) -> ByteOffset {
        self.end
    }

    pub fn len(&self) -> usize {
        self.end.get().saturating_sub(self.start.get())
    }

    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    pub fn contains(&self, offset: ByteOffset) -> bool {
        self.start.get() <= offset.get() && offset.get() < self.end.get()
    }

    pub fn as_range(&self) -> Range<usize> {
        self.start.get()..self.end.get()
    }

    pub fn slice<'a>(&self, input: &'a str) -> &'a str {
        &input[self.as_range()]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChildIndex(usize);

impl ChildIndex {
    pub const fn new(value: usize) -> Self {
        Self(value)
    }

    pub const fn get(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExpressionPath(Vec<ChildIndex>);

pub type Path = ExpressionPath;

impl ExpressionPath {
    pub fn from_indexes(indexes: Vec<usize>) -> Self {
        Self(indexes.into_iter().map(ChildIndex::new).collect())
    }

    pub fn indexes(&self) -> &[ChildIndex] {
        &self.0
    }
}

impl FromStr for ExpressionPath {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        if s.trim().is_empty() {
            return Ok(Self(Vec::new()));
        }
        let mut indexes = Vec::new();
        for part in s.split('.') {
            indexes.push(ChildIndex::new(
                part.parse::<usize>()
                    .map_err(|_| anyhow!("invalid path segment: {part}"))?,
            ));
        }
        Ok(Self(indexes))
    }
}

impl fmt::Display for ExpressionPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (position, index) in self.0.iter().enumerate() {
            if position > 0 {
                write!(f, ".")?;
            }
            write!(f, "{}", index.get())?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SymbolName(String);

impl SymbolName {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        if value.is_empty() {
            anyhow::bail!("symbol must not be empty");
        }
        if value.bytes().any(is_symbol_boundary) || value.contains('"') {
            anyhow::bail!("symbol contains reader delimiter or whitespace: {value}");
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for SymbolName {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        Self::new(s)
    }
}

impl fmt::Display for SymbolName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxTree {
    nodes: Vec<Node>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(usize);

impl NodeId {
    const ROOT: Self = Self(0);

    const fn new(value: usize) -> Self {
        Self(value)
    }

    const fn get(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Node {
    kind: NodeKind,
    delimiter: Option<Delimiter>,
    parent: Option<NodeId>,
    children: Vec<NodeId>,
    span: ByteSpan,
    open: Option<ByteOffset>,
    close: Option<ByteOffset>,
    text: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NodeKind {
    Root,
    List,
    Atom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delimiter {
    Paren,
    Bracket,
    Brace,
}

impl Delimiter {
    fn from_open(byte: u8) -> Option<Self> {
        match byte {
            b'(' => Some(Self::Paren),
            b'[' => Some(Self::Bracket),
            b'{' => Some(Self::Brace),
            _ => None,
        }
    }

    fn from_close(byte: u8) -> Option<Self> {
        match byte {
            b')' => Some(Self::Paren),
            b']' => Some(Self::Bracket),
            b'}' => Some(Self::Brace),
            _ => None,
        }
    }

    fn open(self) -> char {
        match self {
            Self::Paren => '(',
            Self::Bracket => '[',
            Self::Brace => '{',
        }
    }

    fn close(self) -> char {
        match self {
            Self::Paren => ')',
            Self::Bracket => ']',
            Self::Brace => '}',
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutlineEntry {
    pub path: ExpressionPath,
    pub span: ByteSpan,
    pub head: Option<String>,
    pub definition_like: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtomOccurrence {
    pub path: ExpressionPath,
    pub span: ByteSpan,
    pub text: String,
}

#[derive(Debug, Clone, Copy)]
pub struct Selection<'a> {
    tree: &'a SyntaxTree,
    node_id: NodeId,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ParseError {
    #[error("unexpected closing delimiter '{delimiter}' at byte {position}")]
    UnexpectedClose { delimiter: char, position: usize },
    #[error("mismatched closing delimiter '{found}' at byte {position}; expected '{expected}'")]
    MismatchedClose {
        found: char,
        expected: char,
        position: usize,
    },
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

    pub fn root_children(&self) -> &[NodeId] {
        &self.node(NodeId::ROOT).children
    }

    pub fn outline(&self, is_definition_head: impl Fn(&str) -> bool) -> Vec<OutlineEntry> {
        self.node(NodeId::ROOT)
            .children
            .iter()
            .enumerate()
            .filter_map(|(index, node_id)| {
                let node = self.node(*node_id);
                if node.kind != NodeKind::List {
                    return None;
                }
                let head = node
                    .children
                    .first()
                    .and_then(|child| self.atom_text(*child))
                    .map(ToOwned::to_owned);
                let definition_like = head.as_deref().is_some_and(&is_definition_head);
                Some(OutlineEntry {
                    path: ExpressionPath::from_indexes(vec![index]),
                    span: node.span,
                    head,
                    definition_like,
                })
            })
            .collect()
    }

    pub fn atom_occurrences(&self) -> Vec<AtomOccurrence> {
        let mut occurrences = Vec::new();
        let mut path = Vec::new();
        self.collect_atoms(NodeId::ROOT, &mut path, &mut occurrences);
        occurrences
    }

    pub fn rename_symbol(&self, input: &str, from: &SymbolName, to: &SymbolName) -> String {
        let mut output = input.to_owned();
        let mut occurrences = self
            .atom_occurrences()
            .into_iter()
            .filter(|occurrence| occurrence.text == from.as_str())
            .collect::<Vec<_>>();
        occurrences.sort_by_key(|occurrence| occurrence.span.start());
        for occurrence in occurrences.into_iter().rev() {
            output.replace_range(occurrence.span.as_range(), to.as_str());
        }
        output
    }

    pub fn select_path(&self, path: &ExpressionPath) -> Result<Selection<'_>> {
        let mut node_id = NodeId::ROOT;
        for index in path.indexes() {
            let node = self.node(node_id);
            node_id = *node
                .children
                .get(index.get())
                .ok_or_else(|| anyhow!("path segment {} is out of range", index.get()))?;
        }
        if node_id == NodeId::ROOT {
            anyhow::bail!("root document cannot be edited directly");
        }
        Ok(Selection {
            tree: self,
            node_id,
        })
    }

    pub fn select_at(&self, offset: usize) -> Result<Selection<'_>> {
        let offset = ByteOffset::new(offset);
        let mut best = None;
        for id in 1..self.nodes.len() {
            let node_id = NodeId::new(id);
            let node = self.node(node_id);
            if node.span.contains(offset) {
                match best {
                    None => best = Some(node_id),
                    Some(best_id) if node.span.len() < self.node(best_id).span.len() => {
                        best = Some(node_id)
                    }
                    _ => {}
                }
            }
        }
        best.map(|node_id| Selection {
            tree: self,
            node_id,
        })
        .ok_or_else(|| anyhow!("no expression contains byte offset {}", offset.get()))
    }

    fn collect_atoms(
        &self,
        node_id: NodeId,
        path: &mut Vec<usize>,
        output: &mut Vec<AtomOccurrence>,
    ) {
        let node = self.node(node_id);
        if node.kind == NodeKind::Atom {
            output.push(AtomOccurrence {
                path: ExpressionPath::from_indexes(path.clone()),
                span: node.span,
                text: node.text.clone().expect("atom has source text"),
            });
            return;
        }
        for (index, child) in node.children.iter().enumerate() {
            path.push(index);
            self.collect_atoms(*child, path, output);
            path.pop();
        }
    }

    fn atom_text(&self, node_id: NodeId) -> Option<&str> {
        let node = self.node(node_id);
        (node.kind == NodeKind::Atom)
            .then_some(node.text.as_deref())
            .flatten()
    }

    fn node(&self, node_id: NodeId) -> &Node {
        &self.nodes[node_id.get()]
    }
}

impl<'a> Selection<'a> {
    pub fn text(self, input: &str) -> &str {
        self.span().slice(input)
    }

    fn node(self) -> &'a Node {
        self.tree.node(self.node_id)
    }

    pub fn span(self) -> ByteSpan {
        self.node().span
    }
}

struct Parser<'a> {
    input: &'a str,
    bytes: &'a [u8],
    pos: ByteOffset,
    nodes: Vec<Node>,
    stack: Vec<NodeId>,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        let root = Node {
            kind: NodeKind::Root,
            delimiter: None,
            parent: None,
            children: Vec::new(),
            span: ByteSpan::new(ByteOffset::new(0), ByteOffset::new(input.len())),
            open: None,
            close: None,
            text: None,
        };
        Self {
            input,
            bytes: input.as_bytes(),
            pos: ByteOffset::new(0),
            nodes: vec![root],
            stack: vec![NodeId::ROOT],
        }
    }

    fn parse(&mut self) -> std::result::Result<SyntaxTree, ParseError> {
        while self.pos.get() < self.bytes.len() {
            self.skip_trivia();
            if self.pos.get() >= self.bytes.len() {
                break;
            }
            match self.current_byte() {
                byte if Delimiter::from_open(byte).is_some() => self.open_list(),
                byte if Delimiter::from_close(byte).is_some() => self.close_list()?,
                b'"' => self.atom_string()?,
                _ => self.atom(),
            }
        }
        if self.stack.len() > 1 {
            let open = self
                .nodes
                .get(self.stack.last().expect("root is always present").get())
                .and_then(|node| node.open)
                .expect("list has an open byte");
            return Err(ParseError::UnclosedList(open.get()));
        }
        Ok(SyntaxTree {
            nodes: std::mem::take(&mut self.nodes),
        })
    }

    fn skip_trivia(&mut self) {
        loop {
            while self.pos.get() < self.bytes.len() && self.current_byte().is_ascii_whitespace() {
                self.advance();
            }
            if self.pos.get() < self.bytes.len() && self.current_byte() == b';' {
                while self.pos.get() < self.bytes.len() && self.current_byte() != b'\n' {
                    self.advance();
                }
                continue;
            }
            break;
        }
    }

    fn open_list(&mut self) {
        let parent = *self.stack.last().expect("root is always present");
        let id = NodeId::new(self.nodes.len());
        let delimiter = Delimiter::from_open(self.current_byte()).expect("open delimiter");
        self.nodes.push(Node {
            kind: NodeKind::List,
            delimiter: Some(delimiter),
            parent: Some(parent),
            children: Vec::new(),
            span: ByteSpan::new(self.pos, ByteOffset::new(self.pos.get() + 1)),
            open: Some(self.pos),
            close: None,
            text: None,
        });
        self.nodes[parent.get()].children.push(id);
        self.stack.push(id);
        self.advance();
    }

    fn close_list(&mut self) -> std::result::Result<(), ParseError> {
        let delimiter = Delimiter::from_close(self.current_byte()).expect("close delimiter");
        if self.stack.len() == 1 {
            return Err(ParseError::UnexpectedClose {
                delimiter: delimiter.close(),
                position: self.pos.get(),
            });
        }
        let current = *self.stack.last().expect("checked stack length");
        let expected = self.nodes[current.get()]
            .delimiter
            .expect("list has delimiter")
            .close();
        if delimiter.close() != expected {
            return Err(ParseError::MismatchedClose {
                found: delimiter.close(),
                expected,
                position: self.pos.get(),
            });
        }
        let id = self.stack.pop().expect("checked stack length");
        self.nodes[id.get()].span = ByteSpan::new(
            self.nodes[id.get()].span.start(),
            ByteOffset::new(self.pos.get() + 1),
        );
        self.nodes[id.get()].close = Some(self.pos);
        self.advance();
        Ok(())
    }

    fn atom_string(&mut self) -> std::result::Result<(), ParseError> {
        let start = self.pos;
        self.advance();
        let mut escaped = false;
        while self.pos.get() < self.bytes.len() {
            let byte = self.current_byte();
            self.advance();
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                self.push_atom(start, self.pos);
                return Ok(());
            }
        }
        Err(ParseError::UnterminatedString(start.get()))
    }

    fn atom(&mut self) {
        let start = self.pos;
        while self.pos.get() < self.bytes.len() {
            let byte = self.current_byte();
            if is_symbol_boundary(byte) {
                break;
            }
            self.advance();
        }
        self.push_atom(start, self.pos);
    }

    fn push_atom(&mut self, start: ByteOffset, end: ByteOffset) {
        let parent = *self.stack.last().expect("root is always present");
        let id = NodeId::new(self.nodes.len());
        self.nodes.push(Node {
            kind: NodeKind::Atom,
            delimiter: None,
            parent: Some(parent),
            children: Vec::new(),
            span: ByteSpan::new(start, end),
            open: None,
            close: None,
            text: Some(self.input[start.get()..end.get()].to_string()),
        });
        self.nodes[parent.get()].children.push(id);
    }

    fn current_byte(&self) -> u8 {
        self.bytes[self.pos.get()]
    }

    fn advance(&mut self) {
        self.pos = ByteOffset::new(self.pos.get() + 1);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    fn format_node(&self, tree: &SyntaxTree, node_id: NodeId, depth: usize, output: &mut String) {
        let node = tree.node(node_id);
        match node.kind {
            NodeKind::Root => unreachable!("root is not formatted directly"),
            NodeKind::Atom => {
                output.push_str(node.text.as_deref().expect("atom has source text"));
            }
            NodeKind::List if node.children.is_empty() => {
                let delimiter = node.delimiter.expect("list has delimiter");
                output.push(delimiter.open());
                output.push(delimiter.close());
            }
            NodeKind::List if self.inline_list(tree, node_id).is_some() => {
                output.push_str(
                    &self
                        .inline_list(tree, node_id)
                        .expect("checked inline list"),
                );
            }
            NodeKind::List => {
                let delimiter = node.delimiter.expect("list has delimiter");
                output.push(delimiter.open());
                for (position, child) in node.children.iter().enumerate() {
                    if position == 0 {
                        self.format_node(tree, *child, depth + 1, output);
                    } else {
                        output.push('\n');
                        output.push_str(&" ".repeat((depth + 1) * self.indent));
                        self.format_node(tree, *child, depth + 1, output);
                    }
                }
                output.push(delimiter.close());
            }
        }
    }

    fn inline_list(&self, tree: &SyntaxTree, node_id: NodeId) -> Option<String> {
        let node = tree.node(node_id);
        let delimiter = node.delimiter.expect("list has delimiter");
        let mut output = String::from(delimiter.open());
        for (position, child) in node.children.iter().enumerate() {
            let child = tree.node(*child);
            if child.kind != NodeKind::Atom {
                return None;
            }
            if position > 0 {
                output.push(' ');
            }
            output.push_str(child.text.as_deref().expect("atom has source text"));
        }
        output.push(delimiter.close());
        (output.len() <= 80).then_some(output)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Edit;

impl Edit {
    pub fn replace(input: &str, selection: Selection<'_>, replacement: &str) -> String {
        replace_span(input, selection.span(), replacement)
    }

    pub fn kill(input: &str, _tree: &SyntaxTree, selection: Selection<'_>) -> Result<String> {
        let span = expand_removal(input, selection.span());
        Ok(replace_span(input, span, ""))
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
        let open = node.open.expect("list has open byte").get();
        let close = node.close.expect("list has close byte").get();
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
        let parent = selection.tree.node(parent_id);
        if parent.kind == NodeKind::Root {
            anyhow::bail!("cannot raise a top-level expression");
        }
        Ok(replace_span(input, parent.span, selection.text(input)))
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
        let close = node.close.expect("list has close byte").get();
        let insertion = format!(" {}", tree.node(sibling).span.slice(input));
        let removal = expand_removal(input, tree.node(sibling).span);
        Ok(remove_then_insert(
            input,
            removal,
            ByteOffset::new(close),
            &insertion,
        ))
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
        let open = node.open.expect("list has open byte").get() + 1;
        let insertion = format!("{} ", tree.node(sibling).span.slice(input));
        let removal = expand_removal(input, tree.node(sibling).span);
        Ok(remove_then_insert(
            input,
            removal,
            ByteOffset::new(open),
            &insertion,
        ))
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
        let close = node.close.expect("list has close byte").get();
        let child_span = tree.node(child).span;
        let insertion = format!(" {}", child_span.slice(input));
        let removal = expand_removal(input, child_span);
        Ok(remove_then_insert(
            input,
            removal,
            ByteOffset::new(close + 1),
            &insertion,
        ))
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
        let child_span = tree.node(child).span;
        let insertion = format!("{} ", child_span.slice(input));
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

fn next_sibling(tree: &SyntaxTree, node_id: NodeId) -> Option<NodeId> {
    let parent = tree.node(node_id).parent?;
    let siblings = &tree.node(parent).children;
    let position = siblings.iter().position(|id| *id == node_id)?;
    siblings.get(position + 1).copied()
}

fn previous_sibling(tree: &SyntaxTree, node_id: NodeId) -> Option<NodeId> {
    let parent = tree.node(node_id).parent?;
    let siblings = &tree.node(parent).children;
    let position = siblings.iter().position(|id| *id == node_id)?;
    position
        .checked_sub(1)
        .and_then(|previous| siblings.get(previous).copied())
}

fn replace_span(input: &str, span: ByteSpan, replacement: &str) -> String {
    let mut output = String::with_capacity(input.len() + replacement.len());
    output.push_str(&input[..span.start().get()]);
    output.push_str(replacement);
    output.push_str(&input[span.end().get()..]);
    output
}

fn expand_removal(input: &str, span: ByteSpan) -> ByteSpan {
    let bytes = input.as_bytes();
    let mut start = span.start().get();
    let mut end = span.end().get();
    if end < bytes.len() && bytes[end].is_ascii_whitespace() {
        while end < bytes.len() && bytes[end].is_ascii_whitespace() {
            end += 1;
        }
    } else {
        while start > 0 && bytes[start - 1].is_ascii_whitespace() {
            start -= 1;
        }
    }
    ByteSpan::new(ByteOffset::new(start), ByteOffset::new(end))
}

fn remove_then_insert(
    input: &str,
    removal: ByteSpan,
    insertion_at: ByteOffset,
    insertion: &str,
) -> String {
    let adjusted_insertion_at = if insertion_at.get() > removal.end().get() {
        insertion_at.get() - removal.len()
    } else {
        insertion_at.get()
    };
    let removed = replace_span(input, removal, "");
    replace_span(
        &removed,
        ByteSpan::new(
            ByteOffset::new(adjusted_insertion_at),
            ByteOffset::new(adjusted_insertion_at),
        ),
        insertion,
    )
}

fn is_symbol_boundary(byte: u8) -> bool {
    byte.is_ascii_whitespace() || matches!(byte, b'(' | b')' | b'[' | b']' | b'{' | b'}' | b';')
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
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
}
