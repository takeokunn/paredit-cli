use crate::domain::sexpr::ByteSpan;

use super::PackageRenameOccurrence;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SpanReplacement {
    pub(super) span: ByteSpan,
    pub(super) replacement: String,
}

pub(super) fn rewrite_package_occurrences(
    input: &str,
    occurrences: &[PackageRenameOccurrence],
) -> String {
    let mut rewritten = input.to_owned();
    let mut edits = occurrences.to_vec();
    edits.sort_by_key(|occurrence| occurrence.span.start());
    for occurrence in edits.into_iter().rev() {
        rewritten.replace_range(occurrence.span.as_range(), &occurrence.replacement);
    }
    rewritten
}

pub(super) fn replace_span(input: &str, span: ByteSpan, replacement: &str) -> String {
    let mut rewritten = input.to_owned();
    rewritten.replace_range(span.as_range(), replacement);
    rewritten
}

pub(super) fn rewrite_spans(input: &str, replacements: &[SpanReplacement]) -> String {
    let mut rewritten = input.to_owned();
    let mut edits = replacements.to_vec();
    edits.sort_by_key(|replacement| replacement.span.start());
    for replacement in edits.into_iter().rev() {
        rewritten.replace_range(replacement.span.as_range(), &replacement.replacement);
    }
    rewritten
}
