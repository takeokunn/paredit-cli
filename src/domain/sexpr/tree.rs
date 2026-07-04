use anyhow::{Result, anyhow};

use super::parser::{ParseError, Parser};
use super::types::{ByteOffset, ByteSpan, Delimiter, ExpressionPath, NodeId, SymbolName};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxTree {
    pub(in crate::domain::sexpr) nodes: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::domain::sexpr) struct Node {
    pub(in crate::domain::sexpr) kind: NodeKind,
    pub(in crate::domain::sexpr) delimiter: Option<Delimiter>,
    pub(in crate::domain::sexpr) parent: Option<NodeId>,
    pub(in crate::domain::sexpr) children: Vec<NodeId>,
    pub(in crate::domain::sexpr) span: ByteSpan,
    pub(in crate::domain::sexpr) open: Option<ByteOffset>,
    pub(in crate::domain::sexpr) close: Option<ByteOffset>,
    pub(in crate::domain::sexpr) text: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::domain::sexpr) enum NodeKind {
    Root,
    List,
    Atom,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpressionKind {
    Root,
    List,
    Atom,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpressionView {
    pub kind: ExpressionKind,
    pub delimiter: Option<Delimiter>,
    pub span: ByteSpan,
    pub text: Option<String>,
    pub children: Vec<ExpressionView>,
}

#[derive(Debug, Clone, Copy)]
pub struct Selection<'a> {
    pub(in crate::domain::sexpr) tree: &'a SyntaxTree,
    pub(in crate::domain::sexpr) node_id: NodeId,
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

    pub(in crate::domain::sexpr) fn expression_view(&self, node_id: NodeId) -> ExpressionView {
        let node = self.node(node_id);
        ExpressionView {
            kind: match node.kind {
                NodeKind::Root => ExpressionKind::Root,
                NodeKind::List => ExpressionKind::List,
                NodeKind::Atom => ExpressionKind::Atom,
            },
            delimiter: node.delimiter,
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
    pub fn text(self, input: &str) -> &str {
        self.span().slice(input)
    }

    pub(in crate::domain::sexpr) fn node(self) -> &'a Node {
        self.tree.node(self.node_id)
    }

    pub fn span(self) -> ByteSpan {
        self.node().span
    }

    pub fn view(self) -> ExpressionView {
        self.tree.expression_view(self.node_id)
    }

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
