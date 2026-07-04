use std::path::PathBuf;

use crate::application::definition_report::DefinitionReportItem;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{ByteSpan, Path};

#[derive(Debug)]
pub(super) struct RemoveDefinitionPlan {
    pub(super) file: PathBuf,
    pub(super) dialect: Dialect,
    pub(super) path: Path,
    pub(super) span: ByteSpan,
    pub(super) definition: DefinitionReportItem,
    pub(super) definition_text: String,
    pub(super) rewritten: String,
    pub(super) changed: bool,
    pub(super) written: bool,
}
