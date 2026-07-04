use std::path::PathBuf;

use crate::domain::definition::DefinitionCategory;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{ByteSpan, Path};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDefinitionsStrategy {
    Name,
    KindThenName,
}

impl SortDefinitionsStrategy {
    pub fn label(self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::KindThenName => "kind-then-name",
        }
    }
}

#[derive(Debug)]
pub struct SortDefinitionsRequest<'a> {
    pub file: PathBuf,
    pub input: &'a str,
    pub dialect: Dialect,
    pub strategy: SortDefinitionsStrategy,
    pub write: bool,
}

#[derive(Debug)]
pub struct SortDefinitionsPlan {
    pub file: PathBuf,
    pub dialect: Dialect,
    pub strategy: SortDefinitionsStrategy,
    pub items: Vec<SortDefinitionsItem>,
    pub rewritten: String,
    pub changed: bool,
    pub written: bool,
}

#[derive(Debug, Clone)]
pub struct SortDefinitionsItem {
    pub old_path: Path,
    pub new_path: Path,
    pub span: ByteSpan,
    pub head: String,
    pub name: Option<String>,
    pub category: DefinitionCategory,
    pub source_index: usize,
    pub target_index: usize,
}

pub(super) struct DefinitionBlock {
    pub(super) start: usize,
    pub(super) end: usize,
    pub(super) entries: Vec<DefinitionEntry>,
    pub(super) separators: Vec<String>,
}

pub(super) struct DefinitionEntry {
    pub(super) item: SortDefinitionsItem,
    pub(super) form_text: String,
}

pub(super) struct RawDefinition {
    pub(super) path: Path,
    pub(super) span: ByteSpan,
    pub(super) head: String,
    pub(super) name: Option<String>,
    pub(super) category: DefinitionCategory,
    pub(super) source_index: usize,
}

pub(super) struct BlockReplacement {
    pub(super) start: usize,
    pub(super) end: usize,
    pub(super) text: String,
}
