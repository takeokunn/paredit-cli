use crate::domain::dialect::Dialect;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ReaderEscapeDiagnostic {
    code: &'static str,
    message: &'static str,
    suggestion: &'static str,
}

impl ReaderEscapeDiagnostic {
    pub(crate) fn code(&self) -> &'static str {
        self.code
    }

    pub(crate) fn message(&self) -> &'static str {
        self.message
    }

    pub(crate) fn suggestion(&self) -> &'static str {
        self.suggestion
    }
}

/// Finds a malformed `|\|` token that has consumed structural source text.
///
/// In a multiple-escaped symbol, the second `|` in `|\|` is escaped rather
/// than closing the symbol. The reader accepts that input when a later `|`
/// exists, which can silently absorb following forms into one atom.
pub(crate) fn common_lisp_reader_escape_diagnostics(
    input: &str,
    dialect: Dialect,
) -> Vec<ReaderEscapeDiagnostic> {
    if !matches!(dialect, Dialect::CommonLisp | Dialect::Unknown) {
        return Vec::new();
    }

    let bytes = input.as_bytes();
    let mut index = 0;
    let mut diagnostics = Vec::new();

    while index < bytes.len() {
        match bytes[index] {
            b';' => index = skip_line_comment(bytes, index + 1),
            b'"' => index = skip_string(bytes, index + 1),
            b'#' if bytes.get(index + 1) == Some(&b'|') => {
                index = skip_block_comment(bytes, index + 2);
            }
            b'#' if bytes.get(index + 1) == Some(&b'\\') => {
                index = skip_character_literal(bytes, index + 2);
            }
            b'\\' => index = (index + 2).min(bytes.len()),
            b'|' => {
                if let Some(end) = multiple_escape_end(bytes, index) {
                    if starts_with_escaped_pipe(bytes, index)
                        && swallows_structure(&bytes[index + 3..end])
                    {
                        diagnostics.push(ReaderEscapeDiagnostic {
                            code: "suspicious-reader-escape",
                            message: "`|\\|` escapes its apparent closing bar and absorbs following source into one symbol",
                            suggestion: "write `|\\\\|` for a backslash operator, or close the symbol before the following form",
                        });
                    }
                    index = end + 1;
                } else {
                    break;
                }
            }
            _ => index += 1,
        }
    }

    diagnostics
}

fn starts_with_escaped_pipe(bytes: &[u8], start: usize) -> bool {
    bytes.get(start + 1..start + 3) == Some(b"\\|")
}

fn swallows_structure(bytes: &[u8]) -> bool {
    bytes
        .iter()
        .any(|byte| matches!(byte, b'(' | b')' | b'\n' | b'\r'))
}

fn multiple_escape_end(bytes: &[u8], start: usize) -> Option<usize> {
    let mut index = start + 1;
    let mut escaped = false;

    while let Some(&byte) = bytes.get(index) {
        index += 1;
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if byte == b'|' {
            return Some(index - 1);
        }
    }

    None
}

fn skip_line_comment(bytes: &[u8], mut index: usize) -> usize {
    while let Some(&byte) = bytes.get(index) {
        index += 1;
        if byte == b'\n' {
            break;
        }
    }
    index
}

fn skip_string(bytes: &[u8], mut index: usize) -> usize {
    let mut escaped = false;
    while let Some(&byte) = bytes.get(index) {
        index += 1;
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if byte == b'"' {
            break;
        }
    }
    index
}

fn skip_block_comment(bytes: &[u8], mut index: usize) -> usize {
    let mut depth = 1;
    while index + 1 < bytes.len() {
        match (bytes[index], bytes[index + 1]) {
            (b'#', b'|') => {
                depth += 1;
                index += 2;
            }
            (b'|', b'#') => {
                depth -= 1;
                index += 2;
                if depth == 0 {
                    break;
                }
            }
            _ => index += 1,
        }
    }
    index
}

fn skip_character_literal(bytes: &[u8], mut index: usize) -> usize {
    while let Some(&byte) = bytes.get(index) {
        if byte.is_ascii_whitespace() || matches!(byte, b'(' | b')' | b'\"' | b';') {
            break;
        }
        index += 1;
    }
    index
}
