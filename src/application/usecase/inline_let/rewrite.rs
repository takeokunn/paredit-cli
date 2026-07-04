use crate::domain::sexpr::ByteSpan;

pub(super) fn replace_body_references(
    input: &str,
    body_span: ByteSpan,
    reference_spans: &[ByteSpan],
    replacement: &str,
) -> String {
    let body_start = body_span.start().get();
    let mut output = body_span.slice(input).to_owned();
    let mut spans = reference_spans.to_vec();
    spans.sort_by_key(|span| span.start());
    for span in spans.into_iter().rev() {
        let start = span.start().get() - body_start;
        let end = span.end().get() - body_start;
        output.replace_range(start..end, replacement);
    }
    output
}

pub(super) fn replace_span(input: &str, span: ByteSpan, replacement: &str) -> String {
    let mut output = String::with_capacity(input.len() - span.len() + replacement.len());
    output.push_str(&input[..span.start().get()]);
    output.push_str(replacement);
    output.push_str(&input[span.end().get()..]);
    output
}
