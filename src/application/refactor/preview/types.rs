#[derive(Debug)]
pub struct RefactorPreviewEdit {
    pub start: usize,
    pub end: usize,
    pub replacement: String,
}
