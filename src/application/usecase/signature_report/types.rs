use std::path::PathBuf;

use crate::application::usecase::call_report::CallReportItem;
use crate::domain::definition::DefinitionCategory;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{ByteSpan, Path, SyntaxTree};
pub use crate::domain::signature_report::{SignatureCallStatus, SignatureReportPolicy};

#[derive(Debug)]
pub struct SignatureReportSource {
    pub path: PathBuf,
    pub dialect: Dialect,
    pub tree: SyntaxTree,
}

#[derive(Debug)]
pub struct SignatureReportFile {
    pub path: PathBuf,
    pub dialect: Dialect,
    pub definitions: Vec<SignatureDefinitionItem>,
    pub calls: Vec<SignatureCallItem>,
}

#[derive(Debug, Clone)]
pub struct SignatureDefinitionItem {
    pub path: Path,
    pub span: ByteSpan,
    pub head: String,
    pub name: Option<String>,
    pub category: DefinitionCategory,
    pub parameter_count: Option<usize>,
    /// (minimum, maximum) call-argument arity; MAXIMUM is `None` when
    /// unbounded (`&rest`/`&body`). Drives call-site arity classification;
    /// PARAMETER_COUNT alone (a flat total across required, &optional, and
    /// &key slots) cannot tell a call omitting an optional/keyword argument
    /// apart from one genuinely missing a required argument.
    pub parameter_arity: Option<(usize, Option<usize>)>,
}

#[derive(Debug)]
pub struct SignatureCallItem {
    pub call: CallReportItem,
    /// (minimum, maximum) arity of the matched definition; MAXIMUM is `None`
    /// when unbounded. `None` overall when the call's definition is unknown
    /// or ambiguous.
    pub expected_parameter_arity: Option<(usize, Option<usize>)>,
    pub status: SignatureCallStatus,
}
