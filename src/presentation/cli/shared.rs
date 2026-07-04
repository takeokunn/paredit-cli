use std::fs;
use std::io::{self, ErrorKind, Read};
use std::path::{Path as FsPath, PathBuf};

use anyhow::{Context, Result};

use super::{DialectArg, SourceInput, TargetArgs};
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{
    AtomOccurrence, ByteSpan, Delimiter, ExpressionKind, ExpressionView, Path, Selection,
    SymbolName, SyntaxTree,
};

pub(super) fn apply_byte_span_edits(
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

pub(super) fn unified_diff(path: &FsPath, before: &str, after: &str) -> String {
    let before_lines = split_diff_lines(before);
    let after_lines = split_diff_lines(after);
    let mut diff = String::new();

    diff.push_str("--- ");
    diff.push_str(&path.display().to_string());
    diff.push('\n');
    diff.push_str("+++ ");
    diff.push_str(&path.display().to_string());
    diff.push('\n');
    diff.push_str(&format!(
        "@@ -1,{} +1,{} @@\n",
        before_lines.len(),
        after_lines.len()
    ));

    for line in before_lines {
        diff.push('-');
        diff.push_str(line);
        if !line.ends_with('\n') {
            diff.push('\n');
        }
    }
    for line in after_lines {
        diff.push('+');
        diff.push_str(line);
        if !line.ends_with('\n') {
            diff.push('\n');
        }
    }

    diff
}

fn split_diff_lines(text: &str) -> Vec<&str> {
    if text.is_empty() {
        Vec::new()
    } else {
        text.split_inclusive('\n').collect()
    }
}

pub(super) fn stable_text_hash(text: &str) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("fnv1a64:{hash:016x}")
}

pub(super) fn bounded_preview(text: &str, max_bytes: usize) -> String {
    if text.len() <= max_bytes {
        return text.to_owned();
    }

    let mut end = max_bytes.min(text.len());
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}...", &text[..end])
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

pub(super) fn package_context_before_top_level(
    tree: &SyntaxTree,
    target_index: usize,
) -> Result<Option<String>> {
    let mut current_package = None;
    for index in 0..target_index {
        let path = Path::from_indexes(vec![index]);
        let view = tree.select_path(&path)?.view();
        if list_head(&view).is_some_and(|head| head.eq_ignore_ascii_case("in-package")) {
            if let Some(package_name) = atom_child(&view, 1) {
                current_package = Some(package_name.to_owned());
            }
        }
    }
    Ok(current_package)
}

pub(super) fn atom_text(view: &ExpressionView) -> Option<&str> {
    (view.kind == ExpressionKind::Atom)
        .then_some(view.text.as_deref())
        .flatten()
}

pub(super) fn atom_child(view: &ExpressionView, index: usize) -> Option<&str> {
    view.children.get(index).and_then(atom_text)
}

pub(super) fn list_head(view: &ExpressionView) -> Option<&str> {
    if view.kind != ExpressionKind::List || view.delimiter != Some(Delimiter::Paren) {
        return None;
    }

    atom_child(view, 0)
}

pub(super) fn matching_symbol_occurrences(
    tree: &SyntaxTree,
    symbol: &SymbolName,
) -> Vec<AtomOccurrence> {
    tree.atom_occurrences()
        .into_iter()
        .filter(|occurrence| occurrence.text == symbol.as_str())
        .collect()
}

pub(super) fn edit_target(
    args: TargetArgs,
    f: fn(&str, &SyntaxTree, Selection<'_>) -> Result<String>,
) -> Result<()> {
    let input = read_input(args.file)?;
    let tree = SyntaxTree::parse(&input.text)?;
    let selection = resolve_target(&tree, args.path.as_ref(), args.at)?;
    print!("{}", f(&input.text, &tree, selection)?);
    Ok(())
}

pub(super) fn resolve_target<'a>(
    tree: &'a SyntaxTree,
    path: Option<&Path>,
    at: Option<usize>,
) -> Result<Selection<'a>> {
    match (path, at) {
        (Some(path), None) => tree.select_path(path),
        (None, Some(offset)) => tree.select_at(offset),
        (None, None) => anyhow::bail!("target required: pass --path or --at"),
        (Some(_), Some(_)) => anyhow::bail!("pass only one of --path or --at"),
    }
}

pub(super) fn detect_dialect(input: &SourceInput, explicit: Option<DialectArg>) -> Dialect {
    Dialect::detect(input.file.as_deref(), explicit.map(Into::into))
}

pub(super) fn read_input(file: Option<PathBuf>) -> Result<SourceInput> {
    match file {
        Some(path) => {
            let text = fs::read_to_string(&path)
                .with_context(|| format!("failed to read {}", path.display()))?;
            Ok(SourceInput {
                text,
                file: Some(path),
            })
        }
        None => {
            let mut text = String::new();
            io::stdin()
                .read_to_string(&mut text)
                .context("failed to read stdin")?;
            Ok(SourceInput { text, file: None })
        }
    }
}

pub(super) fn read_file_or_empty(path: &PathBuf) -> Result<(SourceInput, bool)> {
    match fs::read_to_string(path) {
        Ok(text) => Ok((
            SourceInput {
                text,
                file: Some(path.clone()),
            },
            true,
        )),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok((
            SourceInput {
                text: String::new(),
                file: Some(path.clone()),
            },
            false,
        )),
        Err(error) => Err(error).with_context(|| format!("failed to read {}", path.display())),
    }
}
