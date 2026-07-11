use std::fmt;
use std::ops::Range;
use std::str::FromStr;

use anyhow::{Result, anyhow};

/// A byte offset into the original source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ByteOffset(usize);

impl ByteOffset {
    /// Creates an offset from a raw byte index.
    pub const fn new(value: usize) -> Self {
        Self(value)
    }

    /// Returns the raw byte index.
    pub const fn get(self) -> usize {
        self.0
    }
}

/// A half-open byte range `[start, end)` inside the original source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ByteSpan {
    start: ByteOffset,
    end: ByteOffset,
}

impl ByteSpan {
    /// Creates a span from byte offsets without revalidating ordering.
    pub const fn new(start: ByteOffset, end: ByteOffset) -> Self {
        Self { start, end }
    }

    /// Returns the inclusive start boundary as a byte offset.
    pub const fn start(&self) -> ByteOffset {
        self.start
    }

    /// Returns the exclusive end boundary as a byte offset.
    pub const fn end(&self) -> ByteOffset {
        self.end
    }

    /// Returns the span length in bytes, saturating at zero for invalid order.
    pub fn len(&self) -> usize {
        self.end.get().saturating_sub(self.start.get())
    }

    /// Returns `true` when the span covers no bytes.
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Returns `true` when `offset` lies inside the half-open range.
    pub fn contains(&self, offset: ByteOffset) -> bool {
        self.start.get() <= offset.get() && offset.get() < self.end.get()
    }

    /// Exposes the span as a Rust range over byte indexes.
    pub fn as_range(&self) -> Range<usize> {
        self.start.get()..self.end.get()
    }

    /// Borrows the substring covered by this byte span.
    pub fn slice<'a>(&self, input: &'a str) -> &'a str {
        &input[self.as_range()]
    }
}

/// A child position within one list node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChildIndex(usize);

impl ChildIndex {
    /// Creates a child index from its zero-based position.
    pub const fn new(value: usize) -> Self {
        Self(value)
    }

    /// Returns the zero-based child position.
    pub const fn get(self) -> usize {
        self.0
    }
}

/// A zero-based path from the virtual root to a nested expression.
///
/// # Examples
///
/// ```
/// use std::str::FromStr;
///
/// use paredit_cli::sexpr::ExpressionPath;
///
/// let path = ExpressionPath::from_str("0.2")?;
/// assert_eq!(path.to_raw_indexes(), vec![0, 2]);
/// assert_eq!(path.child(1).to_string(), "0.2.1");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExpressionPath(Vec<ChildIndex>);

/// Backwards-compatible alias for tree paths used by the CLI and API.
pub type Path = ExpressionPath;

impl ExpressionPath {
    /// Builds a path that points to one root-level child expression.
    pub fn root_child(index: usize) -> Self {
        Self::from_indexes(vec![index])
    }

    /// Builds a path from raw zero-based child indexes.
    pub fn from_indexes(indexes: Vec<usize>) -> Self {
        Self(indexes.into_iter().map(ChildIndex::new).collect())
    }

    /// Returns the typed child indexes that form this path.
    pub fn indexes(&self) -> &[ChildIndex] {
        &self.0
    }

    /// Clones this path into raw zero-based indexes.
    pub fn to_raw_indexes(&self) -> Vec<usize> {
        self.0.iter().map(|index| index.get()).collect()
    }

    /// Returns a new path extended by one child position.
    pub fn child(&self, index: usize) -> Self {
        let mut indexes = self.0.clone();
        indexes.push(ChildIndex::new(index));
        Self(indexes)
    }

    /// Returns the parent path, or `None` for the virtual root.
    pub fn parent(&self) -> Option<Self> {
        let mut indexes = self.0.clone();
        indexes.pop()?;
        Some(Self(indexes))
    }

    /// Returns a new path extended by a fixed list of child positions.
    pub fn descendant<const N: usize>(&self, indexes: [usize; N]) -> Self {
        let mut path = self.clone();
        for index in indexes {
            path = path.child(index);
        }
        path
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

/// A validated Lisp-family symbol name without reader delimiters or whitespace.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SymbolName(String);

impl SymbolName {
    /// Validates and stores a symbol name for rename and selection APIs.
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

    /// Returns the original symbol text.
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

/// The list delimiter used by a parsed expression.
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
