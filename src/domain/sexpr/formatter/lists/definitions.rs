use crate::domain::sexpr::formatter::Formatter;
use crate::domain::sexpr::tree::{NodeKind, SyntaxTree};
use crate::domain::sexpr::types::NodeId;

impl Formatter {
    pub(in crate::domain::sexpr::formatter) fn format_definition(
        &self,
        tree: &SyntaxTree,
        node_id: NodeId,
        depth: usize,
        output: &mut String,
    ) {
        let node = tree.node(node_id);
        let delimiter = self.list_delimiter(node);
        output.push(delimiter.open());

        for (position, child) in node.children.iter().enumerate() {
            match position {
                0 => self.format_node(tree, *child, depth + 1, output),
                1 | 2 => {
                    output.push(' ');
                    self.format_inline_or_node(tree, *child, depth + 1, output);
                }
                _ => {
                    output.push('\n');
                    output.push_str(&self.indent(depth + 1));
                    self.format_node(tree, *child, depth + 1, output);
                }
            }
        }

        output.push(delimiter.close());
    }

    pub(in crate::domain::sexpr::formatter) fn format_defmethod(
        &self,
        tree: &SyntaxTree,
        node_id: NodeId,
        depth: usize,
        output: &mut String,
    ) {
        let node = tree.node(node_id);
        let delimiter = self.list_delimiter(node);
        let lambda_list_position =
            node.children
                .iter()
                .enumerate()
                .skip(2)
                .find_map(|(position, child)| {
                    (tree.node(*child).kind == NodeKind::List).then_some(position)
                });
        output.push(delimiter.open());

        for (position, child) in node.children.iter().enumerate() {
            if position == 0 {
                self.format_node(tree, *child, depth + 1, output);
            } else if lambda_list_position.is_some_and(|lambda| position <= lambda) {
                output.push(' ');
                self.format_inline_or_node(tree, *child, depth + 1, output);
            } else {
                output.push('\n');
                output.push_str(&self.indent(depth + 1));
                self.format_node(tree, *child, depth + 1, output);
            }
        }

        output.push(delimiter.close());
    }

    pub(in crate::domain::sexpr::formatter) fn format_prefix_body(
        &self,
        tree: &SyntaxTree,
        node_id: NodeId,
        depth: usize,
        prefix_len: usize,
        output: &mut String,
    ) {
        let node = tree.node(node_id);
        let delimiter = self.list_delimiter(node);
        output.push(delimiter.open());

        for (position, child) in node.children.iter().enumerate() {
            if position <= prefix_len {
                if position > 0 {
                    output.push(' ');
                }
                self.format_inline_or_node(tree, *child, depth + 1, output);
            } else {
                output.push('\n');
                output.push_str(&self.indent(depth + 1));
                self.format_node(tree, *child, depth + 1, output);
            }
        }

        output.push(delimiter.close());
    }

    pub(in crate::domain::sexpr::formatter) fn format_head_body(
        &self,
        tree: &SyntaxTree,
        node_id: NodeId,
        depth: usize,
        output: &mut String,
    ) {
        let node = tree.node(node_id);
        let delimiter = self.list_delimiter(node);
        output.push(delimiter.open());

        for (position, child) in node.children.iter().enumerate() {
            if position == 0 {
                self.format_node(tree, *child, depth + 1, output);
            } else {
                output.push('\n');
                output.push_str(&self.indent(depth + 1));
                self.format_node(tree, *child, depth + 1, output);
            }
        }

        output.push(delimiter.close());
    }
}
