use crate::domain::sexpr::ByteSpan;

#[derive(Debug)]
pub struct RefactorPreviewEdit {
    span: ByteSpan,
    replacement: String,
}

impl RefactorPreviewEdit {
    pub fn new(span: ByteSpan, replacement: String) -> Self {
        Self { span, replacement }
    }

    pub const fn span(&self) -> ByteSpan {
        self.span
    }

    pub fn start(&self) -> usize {
        self.span.start().get()
    }

    pub fn end(&self) -> usize {
        self.span.end().get()
    }

    pub fn replacement(&self) -> &str {
        &self.replacement
    }
}
