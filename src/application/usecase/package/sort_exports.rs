use anyhow::{Context, Result};

use crate::domain::{
    common_lisp::CommonLispPackageDeclarationForm,
    dialect::Dialect,
    sexpr::{ByteSpan, Delimiter, ExpressionKind, ExpressionView, Path, SymbolName, SyntaxTree},
};

use super::syntax::{atom_text, is_package_head, package_atoms_match, package_option_name};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ExportSortEdit {
    pub(super) defpackage_path: String,
    pub(super) defpackage_span: ByteSpan,
    pub(super) package_name: String,
    pub(super) export_path: String,
    pub(super) export_span: ByteSpan,
    pub(super) old_symbols: Vec<String>,
    pub(super) new_symbols: Vec<String>,
    pub(super) replacements: Vec<ExportSymbolReplacement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ExportSymbolReplacement {
    pub(super) span: ByteSpan,
    pub(super) replacement: String,
}

pub(super) fn defpackage_export_sort_edits(
    tree: &SyntaxTree,
    dialect: Dialect,
    package: Option<&SymbolName>,
) -> Result<Vec<ExportSortEdit>> {
    let mut edits = Vec::new();
    let mut matched_defpackages = 0usize;

    for index in 0..tree.root_children().len() {
        let path = Path::root_child(index);
        let view = tree.select_path(&path)?.view();
        collect_export_sort_edits(
            &view,
            path.clone(),
            dialect,
            package,
            &mut matched_defpackages,
            &mut edits,
        )
        .with_context(|| format!("failed to inspect package form at {path}"))?;
    }

    if matched_defpackages == 0 {
        if let Some(target) = package {
            anyhow::bail!("no matching defpackage form found for {target}");
        }
    }

    Ok(edits)
}

fn collect_export_sort_edits(
    view: &ExpressionView,
    path: Path,
    dialect: Dialect,
    package: Option<&SymbolName>,
    matched_defpackages: &mut usize,
    edits: &mut Vec<ExportSortEdit>,
) -> Result<()> {
    analyze_defpackage_exports(view, &path, dialect, package, matched_defpackages, edits)?;

    for (index, child) in view.children.iter().enumerate() {
        collect_export_sort_edits(
            child,
            path.child(index),
            dialect,
            package,
            matched_defpackages,
            edits,
        )?;
    }

    Ok(())
}

fn analyze_defpackage_exports(
    view: &ExpressionView,
    path: &Path,
    dialect: Dialect,
    package: Option<&SymbolName>,
    matched_defpackages: &mut usize,
    edits: &mut Vec<ExportSortEdit>,
) -> Result<()> {
    if view.kind != ExpressionKind::List || view.delimiter != Some(Delimiter::Paren) {
        return Ok(());
    }
    if view.children.len() < 2 {
        return Ok(());
    }
    let Some(head) = atom_text(&view.children[0]) else {
        return Ok(());
    };
    if !is_package_head(dialect, head, CommonLispPackageDeclarationForm::Defpackage) {
        return Ok(());
    }

    let Some(package_name) = atom_text(&view.children[1]) else {
        return Ok(());
    };
    if package.is_some_and(|package| !package_atoms_match(package_name, package.as_str())) {
        return Ok(());
    }
    *matched_defpackages += 1;

    for (option_index, option) in view.children.iter().enumerate().skip(2) {
        if option.kind != ExpressionKind::List || option.children.is_empty() {
            continue;
        }
        let Some(option_head) = atom_text(&option.children[0]) else {
            continue;
        };
        if package_option_name(option_head) != "export" {
            continue;
        }

        edits.push(analyze_export_option(
            option,
            path,
            &path.child(option_index),
            package_name,
            view.span,
        )?);
    }

    Ok(())
}

fn analyze_export_option(
    option: &ExpressionView,
    defpackage_path: &Path,
    option_path: &Path,
    package_name: &str,
    defpackage_span: ByteSpan,
) -> Result<ExportSortEdit> {
    let mut symbol_slots = Vec::new();

    for child in option.children.iter().skip(1) {
        let Some(symbol) = atom_text(child) else {
            anyhow::bail!(
                "cannot sort :export option at {}; only atom symbol designators are supported",
                option_path
            );
        };
        symbol_slots.push((child.span, symbol.to_owned()));
    }

    let old_symbols = symbol_slots
        .iter()
        .map(|(_, symbol)| symbol.clone())
        .collect::<Vec<_>>();
    let mut new_symbols = old_symbols.clone();
    new_symbols.sort_by(|left, right| {
        let left_key = normalized_sort_key(left);
        let right_key = normalized_sort_key(right);
        left_key.cmp(&right_key).then_with(|| left.cmp(right))
    });

    let replacements = symbol_slots
        .into_iter()
        .zip(new_symbols.iter())
        .filter_map(|((span, old_symbol), new_symbol)| {
            (old_symbol != *new_symbol).then(|| ExportSymbolReplacement {
                span,
                replacement: new_symbol.clone(),
            })
        })
        .collect();

    Ok(ExportSortEdit {
        defpackage_path: defpackage_path.to_string(),
        defpackage_span,
        package_name: package_name.to_owned(),
        export_path: option_path.to_string(),
        export_span: option.span,
        old_symbols,
        new_symbols,
        replacements,
    })
}

fn normalized_sort_key(symbol: &str) -> String {
    symbol
        .strip_prefix("#:")
        .or_else(|| symbol.strip_prefix(':'))
        .unwrap_or(symbol)
        .to_ascii_lowercase()
}
