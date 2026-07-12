use anyhow::{Result, anyhow};

use crate::domain::common_lisp::common_lisp_symbol_reference_eq;

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
    /// Exact comment text (`; ...`, `#| ... |#`, `#; <form>`, or `#_<form>`),
    /// trailing whitespace preserved as parsed.
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
    /// Byte offset from `span.start()` to where an atom's own symbol content
    /// begins, i.e. past its reader prefixes *and* any trivia (whitespace or
    /// comments) between the last prefix and the symbol. Reader prefixes are
    /// followed by `skip_trivia()` during parsing (`#' foo` is valid, if
    /// unusual, syntax), so this cannot be recovered later by summing each
    /// prefix's fixed source length — it must be recorded while parsing.
    /// Meaningless (`0`) for non-atom nodes.
    pub(in crate::domain::sexpr) symbol_offset: usize,
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
    /// A bare `#` immediately before an open delimiter: Common Lisp/Scheme
    /// vector literals (`#(1 2 3)`) and Clojure set (`#{1 2}`) or anonymous
    /// function (`#(+ % 1)`) literals. All three dialects glue `#` directly
    /// onto the following collection with no space, so this keeps the `#`
    /// attached to its list instead of scanning as a disconnected atom.
    HashLiteral,
    /// Clojure metadata sugar (`^{:doc "x"}`, `^:private`, `^String`)
    /// prefixing the map, keyword, or symbol that carries the metadata.
    Metadata,
    /// Clojure reader conditional (`#?(:clj a :cljs b)`).
    ReaderConditional,
    /// Clojure splicing reader conditional (`#?@(:clj [a] :cljs [b])`).
    ReaderConditionalSplicing,
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
            Self::HashLiteral => "#",
            Self::Metadata => "^",
            Self::ReaderConditional => "#?",
            Self::ReaderConditionalSplicing => "#?@",
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
    /// The expression span after its reader prefixes and intervening trivia.
    ///
    /// For lists this starts at the opening delimiter; for atoms it starts at
    /// the symbol content. Structural transformations can replace this span
    /// without detaching reader prefixes from their expression.
    pub content_span: ByteSpan,
    pub text: Option<String>,
    pub children: Vec<ExpressionView>,
    /// Byte offset from `span.start()` to where an atom's own symbol content
    /// begins, past its reader prefixes and any intervening trivia. `0` for
    /// non-atom nodes.
    pub symbol_offset: usize,
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

    /// Reports whether any comment discovered during parsing falls within
    /// `span`. Callers that rebuild source text from parsed atoms (rather
    /// than slicing it verbatim) can use this to detect when doing so would
    /// silently discard a comment, since comments live outside the node tree
    /// and are otherwise invisible to such callers.
    pub fn has_comment_in(&self, span: ByteSpan) -> bool {
        self.comments
            .iter()
            .any(|comment| comment.span.start() < span.end() && span.start() < comment.span.end())
    }

    /// Collects every atom in the tree together with its path and byte span.
    pub fn atom_occurrences(&self) -> Vec<AtomOccurrence> {
        let mut occurrences = Vec::new();
        let mut path_stack = Vec::new();
        for (index, child) in self.node(NodeId::ROOT).children.iter().enumerate() {
            path_stack.push(index);
            self.collect_atoms(*child, &mut path_stack, &mut occurrences);
            path_stack.pop();
        }
        occurrences
    }

    /// Counts the atoms `atom_occurrences` would report without materializing
    /// their paths and text. Callers that only need the total (e.g. workspace
    /// inventory reports) avoid one `String` and one path `Vec` per atom.
    pub fn atom_occurrence_count(&self) -> usize {
        fn count(tree: &SyntaxTree, node_id: NodeId) -> usize {
            let node = tree.node(node_id);
            if node
                .reader_prefixes
                .iter()
                .any(|prefix| prefix.is_opaque_reader_form())
            {
                return 0;
            }
            if node.kind == NodeKind::Atom {
                let is_quoted_literal = node.reader_prefixes.contains(&ReaderPrefix::Quote);
                return usize::from(
                    !is_quoted_literal
                        && node
                            .span
                            .slice(&tree.source)
                            .get(node.symbol_offset..)
                            .is_some(),
                );
            }
            node.children
                .iter()
                .map(|child| count(tree, *child))
                .sum()
        }
        self.node(NodeId::ROOT)
            .children
            .iter()
            .map(|child| count(self, *child))
            .sum()
    }

    /// Collects bare quoted-symbol designators (`'foo`, i.e. an atom whose own
    /// reader prefix is `'`), which `atom_occurrences` deliberately treats as
    /// inert data and excludes (see `does_not_rename_quoted_atom_occurrences`).
    ///
    /// That exclusion is right for `atom_occurrences`'s other consumers
    /// (unused-definition/impact/analysis reports, which have their own,
    /// more precise quote-aware reference collectors when they need one), but
    /// `'foo` is also the standard Common Lisp idiom for referencing a symbol
    /// in the value/type namespace as data -- e.g. `(error 'foo ...)`,
    /// `(typep x 'foo)`, `(make-instance 'foo)`. A blunt, tree-wide rename
    /// (`rename-symbol`, `refactor preview --mode symbol`) that skips these
    /// would silently leave behind references to a definition that no longer
    /// exists, so those two entry points additionally consult this method.
    ///
    /// Only a *bare* quoted atom counts: a quoted list such as `'(foo bar)`
    /// keeps its reader prefix on the list node, not on `foo`/`bar`, so those
    /// remain ordinary atoms already covered by `atom_occurrences` and are
    /// left untouched here.
    pub fn quoted_symbol_designator_occurrences(&self) -> Vec<AtomOccurrence> {
        let mut occurrences = Vec::new();
        let mut path_stack = Vec::new();
        for (index, child) in self.node(NodeId::ROOT).children.iter().enumerate() {
            path_stack.push(index);
            self.collect_quoted_symbol_designators(*child, &mut path_stack, &mut occurrences);
            path_stack.pop();
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
        let mut occurrences = self
            .atom_occurrences()
            .into_iter()
            .chain(self.quoted_symbol_designator_occurrences())
            .filter(|occurrence| common_lisp_symbol_reference_eq(&occurrence.text, from.as_str()))
            .collect::<Vec<_>>();
        occurrences.sort_by_key(|occurrence| occurrence.span.start());
        let mut output = String::with_capacity(input.len());
        let mut cursor = 0usize;
        for occurrence in occurrences {
            let range = occurrence.span.as_range();
            if range.start < cursor {
                continue;
            }
            output.push_str(&input[cursor..range.start]);
            output.push_str(to.as_str());
            cursor = range.end;
        }
        output.push_str(&input[cursor..]);
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

    // `path_stack` is pushed/popped in place rather than cloned at every
    // recursion level (as `ExpressionPath::child` would do): cloning the
    // whole path on the way down makes a deeply nested document (thousands
    // of levels) cost O(depth^2) allocation instead of O(depth), which is
    // slow enough to look hung. A full `ExpressionPath` is only built once
    // an atom is actually found.
    fn collect_atoms(
        &self,
        node_id: NodeId,
        path_stack: &mut Vec<usize>,
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
            let is_quoted_literal = node.reader_prefixes.contains(&ReaderPrefix::Quote);
            if let Some(symbol_text) = (!is_quoted_literal)
                .then(|| node.span.slice(&self.source))
                .and_then(|text| text.get(node.symbol_offset..))
            {
                let symbol_span = ByteSpan::new(
                    ByteOffset::new(node.span.start().get() + node.symbol_offset),
                    node.span.end(),
                );
                output.push(AtomOccurrence {
                    path: ExpressionPath::from_indexes(path_stack.clone()),
                    span: symbol_span,
                    text: symbol_text.to_string(),
                });
            }
            return;
        }
        for (index, child) in node.children.iter().enumerate() {
            path_stack.push(index);
            self.collect_atoms(*child, path_stack, output);
            path_stack.pop();
        }
    }

    fn collect_quoted_symbol_designators(
        &self,
        node_id: NodeId,
        path_stack: &mut Vec<usize>,
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
            if node.reader_prefixes.contains(&ReaderPrefix::Quote) {
                if let Some(symbol_text) = node
                    .span
                    .slice(&self.source)
                    .get(node.symbol_offset..)
                {
                    let symbol_span = ByteSpan::new(
                        ByteOffset::new(node.span.start().get() + node.symbol_offset),
                        node.span.end(),
                    );
                    output.push(AtomOccurrence {
                        path: ExpressionPath::from_indexes(path_stack.clone()),
                        span: symbol_span,
                        text: symbol_text.to_string(),
                    });
                }
            }
            return;
        }
        for (index, child) in node.children.iter().enumerate() {
            path_stack.push(index);
            self.collect_quoted_symbol_designators(*child, path_stack, output);
            path_stack.pop();
        }
    }

    fn atom_text(&self, node_id: NodeId) -> Option<&str> {
        let node = self.node(node_id);
        if node.kind != NodeKind::Atom || !node.reader_prefixes.is_empty() {
            return None;
        }
        Some(node.span.slice(&self.source))
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
            content_span: ByteSpan::new(
                match node.kind {
                    NodeKind::List => node.open.unwrap_or(node.span.start()),
                    NodeKind::Atom => ByteOffset::new(node.span.start().get() + node.symbol_offset),
                    NodeKind::Root => node.span.start(),
                },
                node.span.end(),
            ),
            text: (node.kind == NodeKind::Atom).then(|| node.span.slice(&self.source).to_string()),
            symbol_offset: node.symbol_offset,
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
