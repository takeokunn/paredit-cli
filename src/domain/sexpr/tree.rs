use anyhow::{Result, anyhow};

use crate::domain::common_lisp::common_lisp_symbol_name_eq;

use super::parser::{ParseError, Parser};
use super::types::{ByteOffset, ByteSpan, Delimiter, ExpressionPath, NodeId, SymbolName};

/// A parsed S-expression document with tree navigation and query helpers.
///
/// # Examples
///
/// ```
/// use paredit_cli::sexpr::{ExpressionPath, SyntaxTree};
///
/// let input = "(let ((value 1)) (+ value 2))";
/// let tree = SyntaxTree::parse(input).unwrap();
/// let selection = tree
///     .select_path(&ExpressionPath::from_indexes(vec![0, 2, 1]))
///     .unwrap();
///
/// assert_eq!(selection.text(input), "value");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxTree {
    pub(in crate::domain::sexpr) nodes: Vec<Node>,
    /// Comments discovered during parsing, in source order. They are kept
    /// outside the node tree so structural refactors that walk `children` never
    /// have to reason about them; only the canonical formatter re-emits them.
    pub(in crate::domain::sexpr) comments: Vec<Comment>,
    /// The exact source text the tree was parsed from, used by the formatter to
    /// slice comment-bearing forms verbatim and to measure line breaks.
    pub(in crate::domain::sexpr) source: String,
}

/// A comment captured verbatim during parsing together with the placement
/// metadata the formatter needs to re-emit it without losing information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::domain::sexpr) struct Comment {
    /// Byte range of the comment in the original source.
    pub(in crate::domain::sexpr) span: ByteSpan,
    /// Exact comment text (`; ...`, `#| ... |#`, or `#; <form>`), trailing
    /// whitespace preserved as parsed.
    pub(in crate::domain::sexpr) text: String,
    /// `true` when only whitespace precedes the comment on its source line, i.e.
    /// it stands on its own line rather than trailing code.
    pub(in crate::domain::sexpr) own_line: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::domain::sexpr) struct Node {
    pub(in crate::domain::sexpr) kind: NodeKind,
    pub(in crate::domain::sexpr) delimiter: Option<Delimiter>,
    pub(in crate::domain::sexpr) reader_prefixes: Vec<ReaderPrefix>,
    pub(in crate::domain::sexpr) parent: Option<NodeId>,
    pub(in crate::domain::sexpr) children: Vec<NodeId>,
    pub(in crate::domain::sexpr) span: ByteSpan,
    pub(in crate::domain::sexpr) open: Option<ByteOffset>,
    pub(in crate::domain::sexpr) close: Option<ByteOffset>,
    pub(in crate::domain::sexpr) text: Option<String>,
    pub(in crate::domain::sexpr) source_text: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::domain::sexpr) enum NodeKind {
    Root,
    List,
    Atom,
}

/// Reader sugar that prefixes an expression in source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReaderPrefix {
    Quote,
    Quasiquote,
    Unquote,
    UnquoteSplicing,
    Function,
    ReadEval,
}

impl ReaderPrefix {
    /// Returns the exact source spelling for this reader prefix.
    pub fn as_source(self) -> &'static str {
        match self {
            Self::Quote => "'",
            Self::Quasiquote => "`",
            Self::Unquote => ",",
            Self::UnquoteSplicing => ",@",
            Self::Function => "#'",
            Self::ReadEval => "#.",
        }
    }

    /// Returns true when this prefix makes the following form opaque to structural refactors.
    pub fn is_opaque_reader_form(self) -> bool {
        matches!(self, Self::ReadEval)
    }
}

/// Summary of one root-level list in outline-oriented reports.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutlineEntry {
    pub path: ExpressionPath,
    pub span: ByteSpan,
    pub head: Option<String>,
    pub definition_like: bool,
}

/// One atom plus its tree path and byte span.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtomOccurrence {
    pub path: ExpressionPath,
    pub span: ByteSpan,
    pub text: String,
}

/// The high-level shape of an expression node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpressionKind {
    Root,
    List,
    Atom,
}

/// Immutable tree view data for one expression and its descendants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpressionView {
    pub kind: ExpressionKind,
    pub delimiter: Option<Delimiter>,
    pub reader_prefixes: Vec<ReaderPrefix>,
    pub span: ByteSpan,
    pub text: Option<String>,
    pub children: Vec<ExpressionView>,
}

/// A validated selection of one non-root expression inside a syntax tree.
#[derive(Debug, Clone, Copy)]
pub struct Selection<'a> {
    pub(in crate::domain::sexpr) tree: &'a SyntaxTree,
    pub(in crate::domain::sexpr) node_id: NodeId,
}

impl SyntaxTree {
    /// Parses source text into a syntax tree that preserves byte spans.
    pub fn parse(input: &str) -> std::result::Result<Self, ParseError> {
        let mut parser = Parser::new(input);
        parser.parse()
    }

    /// Returns the direct children of the virtual root document node.
    pub fn root_children(&self) -> &[NodeId] {
        &self.node(NodeId::ROOT).children
    }

    /// Returns an immutable tree view rooted at the virtual document node.
    pub fn root_view(&self) -> ExpressionView {
        self.expression_view(NodeId::ROOT)
    }

    /// Builds an outline of root-level lists and marks definition-like forms.
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
                    path: ExpressionPath::root_child(index),
                    span: node.span,
                    head,
                    definition_like,
                })
            })
            .collect()
    }

    /// Collects every atom in the tree together with its path and byte span.
    pub fn atom_occurrences(&self) -> Vec<AtomOccurrence> {
        let mut occurrences = Vec::new();
        for (index, child) in self.node(NodeId::ROOT).children.iter().enumerate() {
            self.collect_atoms(*child, ExpressionPath::root_child(index), &mut occurrences);
        }
        occurrences
    }

    /// Rewrites matching atom occurrences while preserving the rest of the source text.
    ///
    /// # Examples
    ///
    /// ```
    /// use paredit_cli::sexpr::{SymbolName, SyntaxTree};
    ///
    /// let input = "(let ((value 1)) (+ value value))";
    /// let tree = SyntaxTree::parse(input).unwrap();
    /// let output = tree.rename_symbol(
    ///     input,
    ///     &SymbolName::new("value").unwrap(),
    ///     &SymbolName::new("count").unwrap(),
    /// );
    ///
    /// assert_eq!(output, "(let ((count 1)) (+ count count))");
    /// ```
    pub fn rename_symbol(&self, input: &str, from: &SymbolName, to: &SymbolName) -> String {
        let mut output = input.to_owned();
        let mut occurrences = self
            .atom_occurrences()
            .into_iter()
            .filter(|occurrence| common_lisp_symbol_name_eq(&occurrence.text, from.as_str()))
            .collect::<Vec<_>>();
        occurrences.sort_by_key(|occurrence| occurrence.span.start());
        for occurrence in occurrences.into_iter().rev() {
            output.replace_range(occurrence.span.as_range(), to.as_str());
        }
        output
    }

    /// Resolves a zero-based expression path into a non-root selection.
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

    /// Selects the smallest expression that contains the given byte offset.
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
        path: ExpressionPath,
        output: &mut Vec<AtomOccurrence>,
    ) {
        let node = self.node(node_id);
        if node
            .reader_prefixes
            .iter()
            .any(|prefix| prefix.is_opaque_reader_form())
        {
            return;
        }
        if node.kind == NodeKind::Atom {
            if let Some(text) = node.text.clone() {
                output.push(AtomOccurrence {
                    path,
                    span: node.span,
                    text,
                });
            }
            return;
        }
        for (index, child) in node.children.iter().enumerate() {
            self.collect_atoms(*child, path.child(index), output);
        }
    }

    fn atom_text(&self, node_id: NodeId) -> Option<&str> {
        let node = self.node(node_id);
        if node.kind != NodeKind::Atom || !node.reader_prefixes.is_empty() {
            return None;
        }
        node.text.as_deref()
    }

    pub(in crate::domain::sexpr) fn expression_view(&self, node_id: NodeId) -> ExpressionView {
        let node = self.node(node_id);
        ExpressionView {
            kind: match node.kind {
                NodeKind::Root => ExpressionKind::Root,
                NodeKind::List => ExpressionKind::List,
                NodeKind::Atom => ExpressionKind::Atom,
            },
            delimiter: node.delimiter,
            reader_prefixes: node.reader_prefixes.clone(),
            span: node.span,
            text: node.text.clone(),
            children: node
                .children
                .iter()
                .map(|child| self.expression_view(*child))
                .collect(),
        }
    }

    pub(in crate::domain::sexpr) fn node(&self, node_id: NodeId) -> &Node {
        &self.nodes[node_id.get()]
    }
}

impl<'a> Selection<'a> {
    /// Returns the original source text covered by this selection.
    pub fn text(self, input: &str) -> &str {
        self.span().slice(input)
    }

    pub(in crate::domain::sexpr) fn node(self) -> &'a Node {
        self.tree.node(self.node_id)
    }

    /// Returns the byte span of the selected expression.
    pub fn span(self) -> ByteSpan {
        self.node().span
    }

    /// Returns an immutable view of the selected expression subtree.
    pub fn view(self) -> ExpressionView {
        self.tree.expression_view(self.node_id)
    }

    /// Returns the enclosing list span when the parent node is a list.
    pub fn enclosing_list_span(self) -> Result<ByteSpan> {
        let parent_id = self
            .node()
            .parent
            .ok_or_else(|| anyhow!("selection has no enclosing list"))?;
        let parent = self.tree.node(parent_id);
        if parent.kind != NodeKind::List {
            anyhow::bail!("selection has no enclosing list");
        }
        Ok(parent.span)
    }
}
