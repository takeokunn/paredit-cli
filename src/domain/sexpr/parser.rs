use thiserror::Error;

use super::tree::{Node, NodeKind, SyntaxTree};
use super::types::{ByteOffset, ByteSpan, Delimiter, NodeId, is_symbol_boundary};

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

pub(in crate::domain::sexpr) struct Parser<'a> {
    input: &'a str,
    bytes: &'a [u8],
    pos: ByteOffset,
    nodes: Vec<Node>,
    stack: Vec<NodeId>,
}

impl<'a> Parser<'a> {
    pub(in crate::domain::sexpr) fn new(input: &'a str) -> Self {
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

    pub(in crate::domain::sexpr) fn parse(
        &mut self,
    ) -> std::result::Result<SyntaxTree, ParseError> {
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
