use std::fmt;
use std::ops::Range;
use std::str::FromStr;

use anyhow::{Result, anyhow};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(usize);

impl NodeId {
    pub(in crate::domain::sexpr) const ROOT: Self = Self(0);

    pub(in crate::domain::sexpr) const fn new(value: usize) -> Self {
        Self(value)
    }

    pub(in crate::domain::sexpr) const fn get(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delimiter {
    Paren,
    Bracket,
    Brace,
}

impl Delimiter {
    pub(in crate::domain::sexpr) fn from_open(byte: u8) -> Option<Self> {
        match byte {
            b'(' => Some(Self::Paren),
            b'[' => Some(Self::Bracket),
            b'{' => Some(Self::Brace),
            _ => None,
        }
    }

    pub(in crate::domain::sexpr) fn from_close(byte: u8) -> Option<Self> {
        match byte {
            b')' => Some(Self::Paren),
            b']' => Some(Self::Bracket),
            b'}' => Some(Self::Brace),
            _ => None,
        }
    }

    pub(in crate::domain::sexpr) fn open(self) -> char {
        match self {
            Self::Paren => '(',
            Self::Bracket => '[',
            Self::Brace => '{',
        }
    }

    pub(in crate::domain::sexpr) fn close(self) -> char {
        match self {
            Self::Paren => ')',
            Self::Bracket => ']',
            Self::Brace => '}',
        }
    }
}

pub(in crate::domain::sexpr) fn is_symbol_boundary(byte: u8) -> bool {
    byte.is_ascii_whitespace() || matches!(byte, b'(' | b')' | b'[' | b']' | b'{' | b'}' | b';')
}
