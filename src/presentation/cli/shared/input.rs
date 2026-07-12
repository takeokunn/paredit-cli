use std::collections::BTreeSet;
use std::fs;
use std::io::{self, ErrorKind, Read};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use super::super::{DialectArg, SourceInput, TargetArgs};
use crate::domain::common_lisp::common_lisp_symbol_reference_eq;
use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{
    AtomOccurrence, Delimiter, ExpressionKind, ExpressionPath, ExpressionView, Selection,
    SymbolName, SyntaxTree,
};
use crate::infrastructure::workspace::{WorkspaceDiscoveryOptions, discover_workspace_files};

pub(crate) fn package_context_before_top_level(
    tree: &SyntaxTree,
    target_index: usize,
) -> Result<Option<String>> {
    let mut current_package = None;
    for index in 0..target_index {
        let path = ExpressionPath::from_indexes(vec![index]);
        let view = tree.select_path(&path)?.view();
        if list_head(&view).is_some_and(|head| head.eq_ignore_ascii_case("in-package")) {
            if let Some(package_name) = atom_child(&view, 1) {
                current_package = Some(package_name.to_owned());
            }
        }
    }
    Ok(current_package)
}

pub(crate) fn atom_text(view: &ExpressionView) -> Option<&str> {
    (view.kind == ExpressionKind::Atom)
        .then_some(view.text.as_deref())
        .flatten()
}

pub(crate) fn atom_child(view: &ExpressionView, index: usize) -> Option<&str> {
    view.children.get(index).and_then(atom_text)
}

pub(crate) fn list_head(view: &ExpressionView) -> Option<&str> {
    if view.kind != ExpressionKind::List || view.delimiter != Some(Delimiter::Paren) {
        return None;
    }

    atom_child(view, 0)
}

pub(crate) fn matching_symbol_occurrences(
    tree: &SyntaxTree,
    symbol: &SymbolName,
) -> Vec<AtomOccurrence> {
    tree.atom_occurrences()
        .into_iter()
        // Bare quoted-symbol designators (`'foo`) are also included: they are
        // the standard idiom for referencing a symbol as data (e.g. `(error
        // 'foo ...)`, `(typep x 'foo)`), and a rename that skips them would
        // silently leave behind a reference to a definition that no longer
        // exists.
        .chain(tree.quoted_symbol_designator_occurrences())
        .filter(|occurrence| common_lisp_symbol_reference_eq(&occurrence.text, symbol.as_str()))
        .collect()
}

pub(crate) fn edit_target(
    args: TargetArgs,
    f: fn(&str, &SyntaxTree, Selection<'_>) -> Result<String>,
) -> Result<()> {
    let input = read_input(args.file)?;
    let tree = SyntaxTree::parse(&input.text)?;
    let selection = resolve_target(&tree, args.path.as_ref(), args.at)?;
    print!("{}", f(&input.text, &tree, selection)?);
    Ok(())
}

pub(crate) fn resolve_target<'a>(
    tree: &'a SyntaxTree,
    path: Option<&ExpressionPath>,
    at: Option<usize>,
) -> Result<Selection<'a>> {
    match (path, at) {
        (Some(path), None) => tree.select_path(path),
        (None, Some(offset)) => tree.select_at(offset),
        (None, None) => anyhow::bail!("target required: pass --path or --at"),
        (Some(_), Some(_)) => anyhow::bail!("pass only one of --path or --at"),
    }
}

pub(crate) fn detect_dialect(input: &SourceInput, explicit: Option<DialectArg>) -> Dialect {
    Dialect::detect(input.file.as_deref(), explicit.map(Into::into))
}

pub(crate) fn read_input(file: Option<PathBuf>) -> Result<SourceInput> {
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

pub(crate) fn expand_input_paths(paths: &[PathBuf]) -> Result<Vec<PathBuf>> {
    expand_input_paths_with_unknown(paths, false)
}

pub(crate) fn expand_input_paths_with_unknown(
    paths: &[PathBuf],
    include_unknown: bool,
) -> Result<Vec<PathBuf>> {
    let mut files = BTreeSet::new();

    for path in paths {
        expand_input_path(path.as_path(), include_unknown, &mut files)?;
    }

    Ok(files.into_iter().collect())
}

fn expand_input_path(
    path: &Path,
    include_unknown: bool,
    files: &mut BTreeSet<PathBuf>,
) -> Result<()> {
    if path.is_dir() {
        let discovery = discover_workspace_files(&WorkspaceDiscoveryOptions {
            roots: vec![path.to_path_buf()],
            include_unknown,
            include_hidden: false,
            include_generated: false,
            max_depth: None,
            exclude: Vec::new(),
        })
        .with_context(|| format!("failed to scan {}", path.display()))?;

        files.extend(discovery.files);
        return Ok(());
    }

    files.insert(path.to_path_buf());
    Ok(())
}

pub(crate) fn require_output_file(file: Option<&PathBuf>) -> Result<&PathBuf> {
    file.context("--write requires --file")
}

pub(crate) fn read_file_or_empty(path: &PathBuf) -> Result<(SourceInput, bool)> {
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

#[cfg(test)]
mod tests {
    use super::require_output_file;

    #[test]
    fn require_output_file_rejects_missing_file() {
        let error = require_output_file(None).unwrap_err();
        assert_eq!(error.to_string(), "--write requires --file");
    }
}
