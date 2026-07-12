use anyhow::Result;

use crate::domain::sexpr::ByteSpan;
use std::path::Path as FsPath;

const DIFF_CONTEXT_LINES: usize = 3;

#[derive(Clone, Copy)]
enum DiffOp {
    Equal(usize, usize),
    Delete(usize),
    Insert(usize),
}

pub(crate) fn apply_byte_span_edits(
    input: &str,
    mut edits: Vec<(ByteSpan, String)>,
) -> Result<String> {
    edits.sort_by_key(|(span, _)| span.start());
    ensure_non_overlapping_spans(edits.iter().map(|(span, _)| *span))?;

    let mut output = input.to_owned();
    for (span, replacement) in edits.into_iter().rev() {
        output.replace_range(span.as_range(), &replacement);
    }
    Ok(output)
}

pub(crate) fn unified_diff(path: &FsPath, before: &str, after: &str) -> String {
    let before_lines = split_diff_lines(before);
    let after_lines = split_diff_lines(after);
    let ops = diff_line_ops(&before_lines, &after_lines);
    let hunks = group_diff_hunks(&ops, DIFF_CONTEXT_LINES);
    if hunks.is_empty() {
        return String::new();
    }

    let mut diff = String::new();
    diff.push_str("--- ");
    diff.push_str(&path.display().to_string());
    diff.push('\n');
    diff.push_str("+++ ");
    diff.push_str(&path.display().to_string());
    diff.push('\n');

    for hunk in hunks {
        render_diff_hunk(&mut diff, &ops[hunk.clone()], &before_lines, &after_lines);
    }

    diff
}

pub(crate) fn stable_text_hash(text: &str) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("fnv1a64:{hash:016x}")
}

pub(crate) fn bounded_preview(text: &str, max_bytes: usize) -> String {
    if text.len() <= max_bytes {
        return text.to_owned();
    }

    let mut end = max_bytes.min(text.len());
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}...", &text[..end])
}

fn diff_line_ops(before: &[&str], after: &[&str]) -> Vec<DiffOp> {
    let n = before.len();
    let m = after.len();
    let mut lcs = vec![vec![0u32; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            lcs[i][j] = if before[i] == after[j] {
                lcs[i + 1][j + 1] + 1
            } else {
                lcs[i + 1][j].max(lcs[i][j + 1])
            };
        }
    }

    let mut ops = Vec::new();
    let (mut i, mut j) = (0, 0);
    while i < n && j < m {
        if before[i] == after[j] {
            ops.push(DiffOp::Equal(i, j));
            i += 1;
            j += 1;
        } else if lcs[i + 1][j] >= lcs[i][j + 1] {
            ops.push(DiffOp::Delete(i));
            i += 1;
        } else {
            ops.push(DiffOp::Insert(j));
            j += 1;
        }
    }
    while i < n {
        ops.push(DiffOp::Delete(i));
        i += 1;
    }
    while j < m {
        ops.push(DiffOp::Insert(j));
        j += 1;
    }
    ops
}

fn group_diff_hunks(ops: &[DiffOp], context: usize) -> Vec<std::ops::Range<usize>> {
    let changes = ops
        .iter()
        .enumerate()
        .filter(|(_, op)| !matches!(op, DiffOp::Equal(_, _)))
        .map(|(index, _)| index)
        .collect::<Vec<_>>();
    if changes.is_empty() {
        return Vec::new();
    }

    let mut hunks: Vec<std::ops::Range<usize>> = Vec::new();
    for &change in &changes {
        let start = change.saturating_sub(context);
        let end = (change + context + 1).min(ops.len());
        match hunks.last_mut() {
            Some(last) if start <= last.end => last.end = last.end.max(end),
            _ => hunks.push(start..end),
        }
    }
    hunks
}

fn render_diff_hunk(diff: &mut String, ops: &[DiffOp], before: &[&str], after: &[&str]) {
    let mut before_start = None;
    let mut after_start = None;
    let mut before_len = 0;
    let mut after_len = 0;
    for op in ops {
        match *op {
            DiffOp::Equal(bi, ai) => {
                before_start.get_or_insert(bi);
                after_start.get_or_insert(ai);
                before_len += 1;
                after_len += 1;
            }
            DiffOp::Delete(bi) => {
                before_start.get_or_insert(bi);
                before_len += 1;
            }
            DiffOp::Insert(ai) => {
                after_start.get_or_insert(ai);
                after_len += 1;
            }
        }
    }

    let before_header = hunk_header_start(before_start, before_len);
    let after_header = hunk_header_start(after_start, after_len);
    diff.push_str(&format!(
        "@@ -{before_header},{before_len} +{after_header},{after_len} @@\n"
    ));

    for op in ops {
        let (marker, line) = match *op {
            DiffOp::Equal(bi, _) => (' ', before[bi]),
            DiffOp::Delete(bi) => ('-', before[bi]),
            DiffOp::Insert(ai) => ('+', after[ai]),
        };
        diff.push(marker);
        diff.push_str(line);
        if !line.ends_with('\n') {
            diff.push('\n');
        }
    }
}

fn hunk_header_start(start: Option<usize>, len: usize) -> usize {
    match start {
        Some(index) => index + 1,
        None if len == 0 => 0,
        None => 1,
    }
}

fn split_diff_lines(text: &str) -> Vec<&str> {
    if text.is_empty() {
        Vec::new()
    } else {
        text.split_inclusive('\n').collect()
    }
}

fn ensure_non_overlapping_spans(spans: impl IntoIterator<Item = ByteSpan>) -> Result<()> {
    let mut previous_end = None;
    for span in spans {
        let start = span.start().get();
        let end = span.end().get();
        if let Some(previous_end) = previous_end {
            if start < previous_end {
                anyhow::bail!("refusing overlapping rewrite spans");
            }
        }
        previous_end = Some(end);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::unified_diff;
    use std::path::Path as FsPath;

    #[test]
    fn unified_diff_only_shows_changed_lines_with_context() {
        let before = "a\nb\nc\nd\ne\nf\ng\n";
        let after = "a\nb\nc\nD\ne\nf\ng\n";
        let diff = unified_diff(FsPath::new("x.lisp"), before, after);

        assert!(diff.contains("--- x.lisp\n"));
        assert!(diff.contains("+++ x.lisp\n"));
        assert!(diff.contains("@@ -1,7 +1,7 @@\n"));
        assert!(diff.contains("-d\n"));
        assert!(diff.contains("+D\n"));
        assert!(!diff.contains("-a\n"));
        assert!(!diff.contains("+a\n"));
        assert_eq!(
            diff.lines()
                .filter(|l| l.starts_with('+') && !l.starts_with("+++"))
                .count(),
            1
        );
        assert_eq!(
            diff.lines()
                .filter(|l| l.starts_with('-') && !l.starts_with("---"))
                .count(),
            1
        );
    }

    #[test]
    fn unified_diff_emits_separate_hunks_for_distant_changes() {
        let before = "1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n11\n12\n";
        let after = "1x\n2\n3\n4\n5\n6\n7\n8\n9\n10\n11\n12x\n";
        let diff = unified_diff(FsPath::new("f"), before, after);
        assert_eq!(diff.matches("@@ -").count(), 2);
    }

    #[test]
    fn unified_diff_empty_when_unchanged() {
        let text = "(defun a () 1)\n";
        assert!(unified_diff(FsPath::new("f"), text, text).is_empty());
    }
}
