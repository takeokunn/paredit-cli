use super::tree::{NodeKind, SyntaxTree};
use super::types::NodeId;

const MAX_INLINE_WIDTH: usize = 80;

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
                if let Some(head) = self.head_text(tree, node_id) {
                    match self.style_for_head(head) {
                        ListStyle::Definition => {
                            self.format_definition(tree, node_id, depth, output);
                        }
                        ListStyle::Lambda => {
                            self.format_prefix_body(tree, node_id, depth, 2, output);
                        }
                        ListStyle::Binding => {
                            self.format_binding_form(tree, node_id, depth, output);
                        }
                        ListStyle::LocalFunctions => {
                            self.format_local_callable_form(tree, node_id, depth, head, output);
                        }
                        ListStyle::OneArgumentBody => {
                            self.format_prefix_body(tree, node_id, depth, 2, output);
                        }
                        ListStyle::HeadBody => {
                            self.format_head_body(tree, node_id, depth, output);
                        }
                        ListStyle::If => {
                            self.format_prefix_body(tree, node_id, depth, 2, output);
                        }
                        ListStyle::General => {
                            self.format_general_list(tree, node_id, depth, output);
                        }
                    }
                } else {
                    self.format_general_list(tree, node_id, depth, output);
                }
            }
        }
    }

    fn inline_list(&self, tree: &SyntaxTree, node_id: NodeId) -> Option<String> {
        let head = self.head_text(tree, node_id);
        if head.is_some_and(|head| self.style_for_head(head) != ListStyle::General) {
            return None;
        }
        self.compact_node(tree, node_id)
    }

    fn compact_node(&self, tree: &SyntaxTree, node_id: NodeId) -> Option<String> {
        let node = tree.node(node_id);
        match node.kind {
            NodeKind::Root => None,
            NodeKind::Atom => Some(
                node.text
                    .as_deref()
                    .expect("atom has source text")
                    .to_owned(),
            ),
            NodeKind::List => {
                if let Some(head) = self.head_text(tree, node_id)
                    && self.style_for_head(head) != ListStyle::General
                {
                    return None;
                }

                let delimiter = node.delimiter.expect("list has delimiter");
                let mut output = String::from(delimiter.open());
                for (position, child) in node.children.iter().enumerate() {
                    if position > 0 {
                        output.push(' ');
                    }
                    output.push_str(&self.compact_node(tree, *child)?);
                }
                output.push(delimiter.close());
                (output.len() <= MAX_INLINE_WIDTH).then_some(output)
            }
        }
    }

    fn format_definition(
        &self,
        tree: &SyntaxTree,
        node_id: NodeId,
        depth: usize,
        output: &mut String,
    ) {
        let node = tree.node(node_id);
        let delimiter = node.delimiter.expect("list has delimiter");
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

    fn format_binding_form(
        &self,
        tree: &SyntaxTree,
        node_id: NodeId,
        depth: usize,
        output: &mut String,
    ) {
        let node = tree.node(node_id);
        let delimiter = node.delimiter.expect("list has delimiter");
        let head = self
            .head_text(tree, node_id)
            .expect("binding form has head");
        output.push(delimiter.open());

        for (position, child) in node.children.iter().enumerate() {
            match position {
                0 => self.format_node(tree, *child, depth + 1, output),
                1 => {
                    output.push(' ');
                    self.format_sequence_list(
                        tree,
                        *child,
                        depth + 1,
                        depth * self.indent + head.len() + 3,
                        output,
                    );
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

    fn format_local_callable_form(
        &self,
        tree: &SyntaxTree,
        node_id: NodeId,
        depth: usize,
        head: &str,
        output: &mut String,
    ) {
        let node = tree.node(node_id);
        let delimiter = node.delimiter.expect("list has delimiter");
        output.push(delimiter.open());

        for (position, child) in node.children.iter().enumerate() {
            match position {
                0 => self.format_node(tree, *child, depth + 1, output),
                1 => {
                    output.push(' ');
                    self.format_sequence_list(
                        tree,
                        *child,
                        depth + 1,
                        depth * self.indent + head.len() + 3,
                        output,
                    );
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

    fn format_prefix_body(
        &self,
        tree: &SyntaxTree,
        node_id: NodeId,
        depth: usize,
        prefix_len: usize,
        output: &mut String,
    ) {
        let node = tree.node(node_id);
        let delimiter = node.delimiter.expect("list has delimiter");
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

    fn format_head_body(
        &self,
        tree: &SyntaxTree,
        node_id: NodeId,
        depth: usize,
        output: &mut String,
    ) {
        let node = tree.node(node_id);
        let delimiter = node.delimiter.expect("list has delimiter");
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

    fn format_general_list(
        &self,
        tree: &SyntaxTree,
        node_id: NodeId,
        depth: usize,
        output: &mut String,
    ) {
        let node = tree.node(node_id);
        let delimiter = node.delimiter.expect("list has delimiter");
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

    fn format_sequence_list(
        &self,
        tree: &SyntaxTree,
        node_id: NodeId,
        depth: usize,
        continuation_column: usize,
        output: &mut String,
    ) {
        let node = tree.node(node_id);
        if node.kind != NodeKind::List || node.children.is_empty() {
            self.format_inline_or_node(tree, node_id, depth, output);
            return;
        }

        let delimiter = node.delimiter.expect("list has delimiter");
        output.push(delimiter.open());
        for (position, child) in node.children.iter().enumerate() {
            if position > 0 {
                output.push('\n');
                output.push_str(&" ".repeat(continuation_column));
            }
            self.format_inline_or_node(tree, *child, depth + 1, output);
        }
        output.push(delimiter.close());
    }

    fn format_inline_or_node(
        &self,
        tree: &SyntaxTree,
        node_id: NodeId,
        depth: usize,
        output: &mut String,
    ) {
        if let Some(inline) = self.compact_node(tree, node_id) {
            output.push_str(&inline);
        } else {
            self.format_node(tree, node_id, depth, output);
        }
    }

    fn head_text<'a>(&self, tree: &'a SyntaxTree, node_id: NodeId) -> Option<&'a str> {
        let node = tree.node(node_id);
        let first = *node.children.first()?;
        let first = tree.node(first);
        (first.kind == NodeKind::Atom).then(|| first.text.as_deref().expect("atom has source text"))
    }

    fn style_for_head(&self, head: &str) -> ListStyle {
        match head.to_ascii_lowercase().as_str() {
            "defun"
            | "defmacro"
            | "defmethod"
            | "defgeneric"
            | "define-condition"
            | "define-compiler-macro"
            | "define-modify-macro"
            | "define-setf-expander"
            | "defsetf"
            | "defclass"
            | "defstruct"
            | "defparameter"
            | "defvar"
            | "defconstant" => ListStyle::Definition,
            "lambda" | "named-lambda" => ListStyle::Lambda,
            "let" | "let*" | "symbol-macrolet" | "handler-bind" | "restart-bind" => {
                ListStyle::Binding
            }
            "flet" | "labels" | "macrolet" | "compiler-macrolet" => ListStyle::LocalFunctions,
            "if" => ListStyle::If,
            "when"
            | "unless"
            | "dolist"
            | "dotimes"
            | "destructuring-bind"
            | "multiple-value-bind"
            | "with-open-file"
            | "with-slots"
            | "with-accessors" => ListStyle::OneArgumentBody,
            "progn" | "prog1" | "prog2" | "cond" | "case" | "ccase" | "ecase" | "typecase"
            | "ctypecase" | "etypecase" | "handler-case" | "restart-case" | "unwind-protect"
            | "block" | "catch" | "tagbody" | "loop" | "defpackage" | "declaim" | "declare"
            | "locally" | "proclaim" | "eval-when" => ListStyle::HeadBody,
            _ => ListStyle::General,
        }
    }

    fn indent(&self, depth: usize) -> String {
        " ".repeat(depth * self.indent)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ListStyle {
    Definition,
    Lambda,
    Binding,
    LocalFunctions,
    OneArgumentBody,
    HeadBody,
    If,
    General,
}
