use anyhow::Result;

use crate::domain::dialect::Dialect;
use crate::domain::sexpr::{ByteSpan, Path, SymbolName, SyntaxTree};

use super::ExtractFunctionInsert;

pub(super) fn extracted_call(name: &SymbolName, params: &[String]) -> String {
    if params.is_empty() {
        format!("({})", name.as_str())
    } else {
        format!("({} {})", name.as_str(), params.join(" "))
    }
}

pub(super) fn extracted_definition(
    dialect: Dialect,
    name: &SymbolName,
    params: &[String],
    body: &str,
) -> String {
    let space_params = params.join(" ");
    match dialect {
        Dialect::Scheme if params.is_empty() => format!("(define ({}) {})", name.as_str(), body),
        Dialect::Scheme => format!("(define ({} {}) {})", name.as_str(), space_params, body),
        Dialect::Clojure | Dialect::Janet => {
            format!("(defn {} [{}] {})", name.as_str(), space_params, body)
        }
        Dialect::Fennel => format!("(fn {} [{}] {})", name.as_str(), space_params, body),
        Dialect::CommonLisp | Dialect::EmacsLisp | Dialect::Unknown => {
            format!("(defun {} ({}) {})", name.as_str(), space_params, body)
        }
    }
}

pub(super) fn replace_span(input: &str, span: ByteSpan, replacement: &str) -> String {
    let mut output = String::with_capacity(input.len() - span.len() + replacement.len());
    output.push_str(&input[..span.start().get()]);
    output.push_str(replacement);
    output.push_str(&input[span.end().get()..]);
    output
}

pub(super) fn insert_top_level_form(
    input: &str,
    tree: &SyntaxTree,
    form: &str,
    insert: ExtractFunctionInsert,
    anchor_path: Option<&Path>,
) -> Result<(String, Option<ByteSpan>)> {
    match insert {
        ExtractFunctionInsert::Append => Ok((append_top_level_definition(input, form), None)),
        ExtractFunctionInsert::Before | ExtractFunctionInsert::After => {
            let anchor_path = anchor_path
                .ok_or_else(|| anyhow::anyhow!("--insert before/after requires --anchor-path"))?;
            let anchor_index = top_level_path_index(anchor_path, "extract-function --anchor-path")?;
            if anchor_index >= tree.root_children().len() {
                anyhow::bail!("anchor top-level path {anchor_path} is out of range");
            }
            let anchor = tree.select_path(anchor_path)?;
            let anchor_span = anchor.span();
            let (offset, inserted) = match insert {
                ExtractFunctionInsert::Before => {
                    (anchor_span.start().get(), format!("{}\n\n", form.trim()))
                }
                ExtractFunctionInsert::After => {
                    (anchor_span.end().get(), format!("\n\n{}", form.trim()))
                }
                ExtractFunctionInsert::Append => {
                    return Ok((append_top_level_definition(input, form), None));
                }
            };
            let mut output = String::with_capacity(input.len() + inserted.len());
            output.push_str(&input[..offset]);
            output.push_str(&inserted);
            output.push_str(&input[offset..]);
            Ok((output, Some(anchor_span)))
        }
    }
}

fn append_top_level_definition(input: &str, definition: &str) -> String {
    if input.trim().is_empty() {
        format!("{}\n", definition.trim())
    } else {
        format!("{}\n\n{}\n", input.trim_end(), definition.trim())
    }
}

fn top_level_path_index(path: &Path, command: &str) -> Result<usize> {
    match path.indexes() {
        [index] => Ok(index.get()),
        _ => anyhow::bail!("{command} requires a top-level path, for example --path 2"),
    }
}
