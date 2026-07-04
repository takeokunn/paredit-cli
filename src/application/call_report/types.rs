use crate::domain::definition::DefinitionCategory;
use crate::domain::sexpr::ByteSpan;

#[derive(Debug, Clone)]
pub struct CallReportItem {
    pub path: String,
    pub span: ByteSpan,
    pub head: String,
    pub argument_count: usize,
    pub category: Option<DefinitionCategory>,
    pub enclosing_definition: Option<String>,
}
