use std::fs;
use std::path::Path as FsPath;

use anyhow::Result;

use crate::domain::sexpr::{ByteSpan, Path, SyntaxTree};

use super::super::MoveInsert;

pub(super) fn append_top_level_form(input: &str, form: &str) -> String {
    let mut output = input.trim_end().to_owned();
    if !output.is_empty() {
        output.push_str("\n\n");
    }
    output.push_str(form);
    output.push('\n');
    output
}

pub(super) fn same_file_path(left: &FsPath, right: &FsPath) -> bool {
    match (fs::canonicalize(left), fs::canonicalize(right)) {
        (Ok(left), Ok(right)) => left == right,
        _ => left == right,
    }
}

pub(super) fn top_level_path_index(path: &Path, command: &str) -> Result<usize> {
    match path.indexes() {
        [index] => Ok(index.get()),
        _ => anyhow::bail!("{command} requires a top-level path, for example --path 2"),
    }
}

pub(super) fn insert_top_level_form(
    input: &str,
    tree: &SyntaxTree,
    form: &str,
    insert: MoveInsert,
    anchor_path: Option<&Path>,
) -> Result<(String, Option<ByteSpan>)> {
    match insert {
        MoveInsert::Append => Ok((append_top_level_form(input, form), None)),
        MoveInsert::Before | MoveInsert::After => {
            let anchor_path = anchor_path.expect("validated by caller");
            let anchor_index = top_level_path_index(anchor_path, "move-form --anchor-path")?;
            if anchor_index >= tree.root_children().len() {
                anyhow::bail!("anchor top-level path {anchor_path} is out of range");
            }
            let anchor = tree.select_path(anchor_path)?;
            let anchor_span = anchor.span();
            let offset = match insert {
                MoveInsert::Before => anchor_span.start().get(),
                MoveInsert::After => anchor_span.end().get(),
                MoveInsert::Append => unreachable!("append handled above"),
            };
            let inserted = match insert {
                MoveInsert::Before => format!("{}\n\n", form.trim()),
                MoveInsert::After => format!("\n\n{}", form.trim()),
                MoveInsert::Append => unreachable!("append handled above"),
            };
            let mut output = String::with_capacity(input.len() + inserted.len());
            output.push_str(&input[..offset]);
            output.push_str(&inserted);
            output.push_str(&input[offset..]);
            Ok((output, Some(anchor_span)))
        }
    }
}
