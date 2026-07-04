use super::tree::{NodeKind, SyntaxTree};
use super::types::NodeId;

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
