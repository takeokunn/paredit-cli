use thiserror::Error;

use super::tree::{Comment, Node, NodeKind, ReaderPrefix, SyntaxTree};
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
    #[error("unterminated block comment starting at byte {0}")]
    UnterminatedBlockComment(usize),
    #[error("unterminated multiple-escape symbol starting at byte {0}")]
    UnterminatedSymbol(usize),
}

pub(in crate::domain::sexpr) struct Parser<'a> {
    input: &'a str,
    bytes: &'a [u8],
    pos: ByteOffset,
    nodes: Vec<Node>,
    stack: Vec<NodeId>,
    comments: Vec<Comment>,
    /// Nesting depth of `#;` datum comments currently being skipped. While
    /// positive, inner trivia is folded into the enclosing datum comment span
    /// instead of being recorded as separate comments.
    suppress_depth: usize,
}

#[derive(Debug, Clone, Copy)]
struct PrefixToken {
    kind: ReaderPrefix,
    start: ByteOffset,
}

impl<'a> Parser<'a> {
    pub(in crate::domain::sexpr) fn new(input: &'a str) -> Self {
        let root = Node {
            kind: NodeKind::Root,
            delimiter: None,
            reader_prefixes: Vec::new(),
            parent: None,
            children: Vec::new(),
            span: ByteSpan::new(ByteOffset::new(0), ByteOffset::new(input.len())),
            open: None,
            close: None,
            text: None,
            source_text: None,
        };
        Self {
            input,
            bytes: input.as_bytes(),
            pos: ByteOffset::new(0),
            nodes: vec![root],
            stack: vec![NodeId::ROOT],
            comments: Vec::new(),
            suppress_depth: 0,
        }
    }

    pub(in crate::domain::sexpr) fn parse(
        &mut self,
    ) -> std::result::Result<SyntaxTree, ParseError> {
        while self.pos.get() < self.bytes.len() {
            self.skip_trivia()?;
            if self.pos.get() >= self.bytes.len() {
                break;
            }
            self.form()?;
        }
        if self.stack.len() > 1 {
            if let Some(open) = self
                .stack
                .last()
                .and_then(|node_id| self.nodes.get(node_id.get()))
                .and_then(|node| node.open)
            {
                return Err(ParseError::UnclosedList(open.get()));
            }
            return Err(ParseError::UnclosedList(self.pos.get()));
        }
        Ok(SyntaxTree {
            nodes: std::mem::take(&mut self.nodes),
            comments: std::mem::take(&mut self.comments),
            source: self.input.to_string(),
        })
    }

    fn skip_trivia(&mut self) -> std::result::Result<(), ParseError> {
        loop {
            while self.pos.get() < self.bytes.len() && self.current_byte().is_ascii_whitespace() {
                self.advance();
            }
            if self.pos.get() < self.bytes.len() && self.current_byte_is_block_comment() {
                let start = self.pos;
                self.skip_block_comment()?;
                self.record_comment(start, self.pos);
                continue;
            }
            if self.pos.get() < self.bytes.len() && self.current_byte() == b';' {
                let start = self.pos;
                while self.pos.get() < self.bytes.len() && self.current_byte() != b'\n' {
                    self.advance();
                }
                self.record_comment(start, self.pos);
                continue;
            }
            break;
        }
        Ok(())
    }

    /// Records the byte range `[start, end)` as a comment, unless we are inside a
    /// `#;` datum comment whose enclosing span already covers it.
    fn record_comment(&mut self, start: ByteOffset, end: ByteOffset) {
        if self.suppress_depth > 0 || end.get() <= start.get() {
            return;
        }
        let text = self.input[start.get()..end.get()].to_string();
        let own_line = self.is_line_start(start.get());
        self.comments.push(Comment {
            span: ByteSpan::new(start, end),
            text,
            own_line,
        });
    }

    /// Returns `true` when only whitespace precedes byte `start` on its line.
    fn is_line_start(&self, start: usize) -> bool {
        let mut index = start;
        while index > 0 {
            match self.bytes[index - 1] {
                b'\n' => return true,
                b' ' | b'\t' | b'\r' | b'\x0c' => index -= 1,
                _ => return false,
            }
        }
        true
    }

    fn open_list_with_prefixes(&mut self, prefixes: Vec<PrefixToken>) {
        let Some(&parent) = self.stack.last() else {
            debug_assert!(false, "parser stack unexpectedly empty when opening list");
            return;
        };
        let id = NodeId::new(self.nodes.len());
        let Some(delimiter) = Delimiter::from_open(self.current_byte()) else {
            debug_assert!(
                false,
                "open_list_with_prefixes called on non-opening delimiter"
            );
            return;
        };
        let start = prefixes
            .first()
            .map(|prefix| prefix.start)
            .unwrap_or(self.pos);
        self.nodes.push(Node {
            kind: NodeKind::List,
            delimiter: Some(delimiter),
            reader_prefixes: prefixes.into_iter().map(|prefix| prefix.kind).collect(),
            parent: Some(parent),
            children: Vec::new(),
            span: ByteSpan::new(start, ByteOffset::new(self.pos.get() + 1)),
            open: Some(self.pos),
            close: None,
            text: None,
            source_text: None,
        });
        self.nodes[parent.get()].children.push(id);
        self.stack.push(id);
        self.advance();
    }

    fn close_list(&mut self) -> std::result::Result<(), ParseError> {
        let Some(delimiter) = Delimiter::from_close(self.current_byte()) else {
            debug_assert!(false, "close_list called on non-closing delimiter");
            return Ok(());
        };
        if self.stack.len() == 1 {
            return Err(ParseError::UnexpectedClose {
                delimiter: delimiter.close(),
                position: self.pos.get(),
            });
        }
        let Some(&current) = self.stack.last() else {
            debug_assert!(false, "parser stack unexpectedly empty when closing list");
            return Err(ParseError::UnexpectedClose {
                delimiter: delimiter.close(),
                position: self.pos.get(),
            });
        };
        let expected = self.nodes[current.get()]
            .delimiter
            .map(Delimiter::close)
            .unwrap_or_else(|| {
                debug_assert!(false, "list node missing delimiter");
                delimiter.close()
            });
        if delimiter.close() != expected {
            return Err(ParseError::MismatchedClose {
                found: delimiter.close(),
                expected,
                position: self.pos.get(),
            });
        }
        let Some(id) = self.stack.pop() else {
            debug_assert!(false, "parser stack unexpectedly empty while popping list");
            return Err(ParseError::UnexpectedClose {
                delimiter: delimiter.close(),
                position: self.pos.get(),
            });
        };
        self.nodes[id.get()].span = ByteSpan::new(
            self.nodes[id.get()].span.start(),
            ByteOffset::new(self.pos.get() + 1),
        );
        self.nodes[id.get()].close = Some(self.pos);
        self.nodes[id.get()].source_text = Some(
            self.input[self.nodes[id.get()].span.start().get()..self.pos.get() + 1].to_string(),
        );
        self.advance();
        Ok(())
    }

    fn atom_string_with_prefixes(
        &mut self,
        prefixes: Vec<PrefixToken>,
    ) -> std::result::Result<(), ParseError> {
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
                self.push_atom(prefixes, start, self.pos);
                return Ok(());
            }
        }
        Err(ParseError::UnterminatedString(start.get()))
    }

    fn atom_with_prefixes(
        &mut self,
        prefixes: Vec<PrefixToken>,
    ) -> std::result::Result<(), ParseError> {
        let start = self.pos;
        if self.current_byte_is_feature_dispatch() {
            self.advance();
            self.advance();
            self.push_atom(prefixes, start, self.pos);
            return Ok(());
        }
        while self.pos.get() < self.bytes.len() {
            let byte = self.current_byte();
            if byte == b'\\' {
                self.consume_single_escape();
                continue;
            }
            if byte == b'|' {
                self.consume_multiple_escape()?;
                continue;
            }
            if is_symbol_boundary(byte) {
                break;
            }
            self.advance();
        }
        self.push_atom(prefixes, start, self.pos);
        Ok(())
    }

    /// Consumes a Lisp single-escape (`\`) and the following character literally.
    ///
    /// This keeps character literals such as `#\[`, `#\)`, and `#\Space`, as well
    /// as escaped symbol constituents like `\(`, from being split at what would
    /// otherwise be a delimiter or whitespace boundary.
    fn consume_single_escape(&mut self) {
        self.advance();
        if self.pos.get() < self.bytes.len() {
            self.advance();
        }
    }

    /// Consumes a Lisp multiple-escape (`|...|`) region and the following
    /// character literally, per CLHS 2.1.4.2.
    ///
    /// Every character inside the region, including whitespace and delimiters,
    /// is a literal symbol constituent rather than a token boundary, so `|Foo
    /// Bar|` scans as one atom instead of splitting at the space. A nested `\`
    /// still single-escapes the character that follows it.
    fn consume_multiple_escape(&mut self) -> std::result::Result<(), ParseError> {
        let start = self.pos;
        self.advance();
        while self.pos.get() < self.bytes.len() {
            let byte = self.current_byte();
            if byte == b'\\' {
                self.consume_single_escape();
                continue;
            }
            if byte == b'|' {
                self.advance();
                return Ok(());
            }
            self.advance();
        }
        Err(ParseError::UnterminatedSymbol(start.get()))
    }

    fn push_atom(&mut self, prefixes: Vec<PrefixToken>, start: ByteOffset, end: ByteOffset) {
        let Some(&parent) = self.stack.last() else {
            debug_assert!(false, "parser stack unexpectedly empty when pushing atom");
            return;
        };
        let id = NodeId::new(self.nodes.len());
        let span_start = prefixes.first().map(|prefix| prefix.start).unwrap_or(start);
        self.nodes.push(Node {
            kind: NodeKind::Atom,
            delimiter: None,
            reader_prefixes: prefixes.into_iter().map(|prefix| prefix.kind).collect(),
            parent: Some(parent),
            children: Vec::new(),
            span: ByteSpan::new(span_start, end),
            open: None,
            close: None,
            text: Some(self.input[span_start.get()..end.get()].to_string()),
            source_text: None,
        });
        self.nodes[parent.get()].children.push(id);
    }

    fn form(&mut self) -> std::result::Result<(), ParseError> {
        if self.current_byte_is_reader_comment() {
            self.skip_reader_comment()?;
            return Ok(());
        }
        let prefixes = self.consume_reader_prefixes()?;
        if self.pos.get() >= self.bytes.len() {
            self.push_atom(prefixes, self.pos, self.pos);
            return Ok(());
        }
        match self.current_byte() {
            byte if Delimiter::from_open(byte).is_some() => self.open_list_with_prefixes(prefixes),
            byte if Delimiter::from_close(byte).is_some() => self.close_list()?,
            b'"' => self.atom_string_with_prefixes(prefixes)?,
            _ => self.atom_with_prefixes(prefixes)?,
        }
        Ok(())
    }

    fn consume_reader_prefixes(&mut self) -> std::result::Result<Vec<PrefixToken>, ParseError> {
        let mut prefixes = Vec::new();
        while let Some(kind) = self.current_reader_prefix() {
            let start = self.pos;
            self.advance_reader_prefix(kind);
            prefixes.push(PrefixToken { kind, start });
            self.skip_trivia()?;
            if self.pos.get() >= self.bytes.len() {
                break;
            }
        }
        Ok(prefixes)
    }

    fn skip_reader_comment(&mut self) -> std::result::Result<(), ParseError> {
        let start = self.pos;
        self.advance();
        self.advance();
        self.suppress_depth += 1;
        let result = self.skip_trivia().and_then(|()| self.skip_form());
        self.suppress_depth -= 1;
        result?;
        // Only the outermost datum comment records; nested `#;`/`#_` are folded in.
        self.record_comment(start, self.pos);
        Ok(())
    }

    fn skip_form(&mut self) -> std::result::Result<(), ParseError> {
        if self.pos.get() >= self.bytes.len() {
            return Ok(());
        }
        if self.current_byte_is_reader_comment() {
            return self.skip_reader_comment();
        }

        while let Some(prefix) = self.current_reader_prefix() {
            self.advance_reader_prefix(prefix);
            self.skip_trivia()?;
            if self.pos.get() >= self.bytes.len() {
                return Ok(());
            }
            if self.current_byte_is_reader_comment() {
                return self.skip_reader_comment();
            }
        }

        if self.pos.get() >= self.bytes.len() {
            return Ok(());
        }

        match self.current_byte() {
            byte if Delimiter::from_open(byte).is_some() => self.skip_list()?,
            byte if Delimiter::from_close(byte).is_some() => {
                let Some(delimiter) = Delimiter::from_close(byte) else {
                    debug_assert!(false, "closing delimiter branch without delimiter");
                    return Ok(());
                };
                return Err(ParseError::UnexpectedClose {
                    delimiter: delimiter.close(),
                    position: self.pos.get(),
                });
            }
            b'"' => self.skip_string(),
            _ => self.skip_atom()?,
        }
        Ok(())
    }

    fn skip_list(&mut self) -> std::result::Result<(), ParseError> {
        let open_pos = self.pos;
        let open = self.current_byte();
        let Some(expected_close) = Delimiter::from_open(open).map(Delimiter::close) else {
            debug_assert!(false, "skip_list called on non-opening delimiter");
            return Ok(());
        };
        self.advance();
        loop {
            self.skip_trivia()?;
            if self.pos.get() >= self.bytes.len() {
                return Err(ParseError::UnclosedList(open_pos.get()));
            }
            if self.current_byte() == expected_close as u8 {
                self.advance();
                return Ok(());
            }
            self.skip_form()?;
        }
    }

    fn skip_string(&mut self) {
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
                return;
            }
        }
    }

    fn skip_block_comment(&mut self) -> std::result::Result<(), ParseError> {
        let start = self.pos;
        self.advance();
        self.advance();
        let mut depth = 1usize;
        while self.pos.get() < self.bytes.len() {
            if self.current_byte_is_block_comment() {
                depth += 1;
                self.advance();
                self.advance();
                continue;
            }
            if self.current_byte() == b'|' && self.peek_byte() == Some(b'#') {
                depth -= 1;
                self.advance();
                self.advance();
                if depth == 0 {
                    return Ok(());
                }
                continue;
            }
            self.advance();
        }
        Err(ParseError::UnterminatedBlockComment(start.get()))
    }

    fn skip_atom(&mut self) -> std::result::Result<(), ParseError> {
        if self.current_byte_is_feature_dispatch() {
            self.advance();
            self.advance();
            return Ok(());
        }
        while self.pos.get() < self.bytes.len() {
            let byte = self.current_byte();
            if byte == b'\\' {
                self.consume_single_escape();
                continue;
            }
            if byte == b'|' {
                self.consume_multiple_escape()?;
                continue;
            }
            if is_symbol_boundary(byte) {
                break;
            }
            self.advance();
        }
        Ok(())
    }

    /// `#+`/`#-` (CLHS 2.4.8/2.4.9 feature-conditional dispatch) must scan as
    /// their own fixed two-byte token, distinct from the feature expression
    /// that follows. Without this, `#+sbcl` glues into one opaque atom while
    /// `#+(and sbcl x86-64)` splits at the list delimiter, so equivalent
    /// feature conditionals produce inconsistent tree shapes: the guarded
    /// feature symbol is findable/renameable in one spelling but hidden
    /// inside an opaque token in the other.
    fn current_byte_is_feature_dispatch(&self) -> bool {
        self.current_byte() == b'#' && matches!(self.peek_byte(), Some(b'+') | Some(b'-'))
    }

    /// Matches Scheme/Common Lisp `#;` datum comments and Clojure `#_`
    /// discard forms. Both are two-byte dispatch macros that read and
    /// discard exactly one following form, so they share the same skip
    /// path and are recorded as comments rather than tree nodes.
    fn current_byte_is_reader_comment(&self) -> bool {
        self.current_byte() == b'#' && matches!(self.peek_byte(), Some(b';') | Some(b'_'))
    }

    fn current_byte_is_block_comment(&self) -> bool {
        self.current_byte() == b'#' && self.peek_byte() == Some(b'|')
    }

    fn current_reader_prefix(&self) -> Option<ReaderPrefix> {
        if self.pos.get() >= self.bytes.len() {
            return None;
        }
        match self.current_byte() {
            b'\'' => Some(ReaderPrefix::Quote),
            b'`' => Some(ReaderPrefix::Quasiquote),
            b',' if self.peek_byte() == Some(b'@') => Some(ReaderPrefix::UnquoteSplicing),
            b',' => Some(ReaderPrefix::Unquote),
            b'^' => Some(ReaderPrefix::Metadata),
            b'#' if self.peek_byte() == Some(b'.') => Some(ReaderPrefix::ReadEval),
            b'#' if self.peek_byte() == Some(b'\'') => Some(ReaderPrefix::Function),
            b'#' if self.peek_byte() == Some(b'?') && self.peek_byte_at(2) == Some(b'@') => {
                Some(ReaderPrefix::ReaderConditionalSplicing)
            }
            b'#' if self.peek_byte() == Some(b'?') => Some(ReaderPrefix::ReaderConditional),
            b'#' if Delimiter::from_open(self.peek_byte().unwrap_or(0)).is_some() => {
                Some(ReaderPrefix::HashLiteral)
            }
            _ => None,
        }
    }

    fn advance_reader_prefix(&mut self, prefix: ReaderPrefix) {
        let width = match prefix {
            ReaderPrefix::ReaderConditionalSplicing => 3,
            ReaderPrefix::UnquoteSplicing
            | ReaderPrefix::Function
            | ReaderPrefix::ReadEval
            | ReaderPrefix::ReaderConditional => 2,
            ReaderPrefix::Quote
            | ReaderPrefix::Quasiquote
            | ReaderPrefix::Unquote
            | ReaderPrefix::HashLiteral
            | ReaderPrefix::Metadata => 1,
        };
        for _ in 0..width {
            self.advance();
        }
    }

    fn current_byte(&self) -> u8 {
        self.bytes[self.pos.get()]
    }

    fn peek_byte(&self) -> Option<u8> {
        self.bytes.get(self.pos.get() + 1).copied()
    }

    fn peek_byte_at(&self, offset: usize) -> Option<u8> {
        self.bytes.get(self.pos.get() + offset).copied()
    }

    fn advance(&mut self) {
        self.pos = ByteOffset::new(self.pos.get() + 1);
    }
}
