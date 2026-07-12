use super::super::tree::ReaderPrefix;
use super::super::types::{Delimiter, is_symbol_boundary};
use super::{ParseError, Parser, PrefixToken};

impl Parser<'_> {
    pub(super) fn form(&mut self) -> std::result::Result<(), ParseError> {
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

    pub(super) fn skip_block_comment(&mut self) -> std::result::Result<(), ParseError> {
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
    pub(super) fn current_byte_is_feature_dispatch(&self) -> bool {
        self.current_byte() == b'#' && matches!(self.peek_byte(), Some(b'+') | Some(b'-'))
    }

    /// Matches Scheme/Common Lisp `#;` datum comments and Clojure `#_`
    /// discard forms. Both are two-byte dispatch macros that read and
    /// discard exactly one following form, so they share the same skip
    /// path and are recorded as comments rather than tree nodes.
    fn current_byte_is_reader_comment(&self) -> bool {
        self.current_byte() == b'#' && matches!(self.peek_byte(), Some(b';') | Some(b'_'))
    }

    pub(super) fn current_byte_is_block_comment(&self) -> bool {
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
}
