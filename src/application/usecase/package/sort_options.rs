use anyhow::{Context, Result};

mod ordering;
mod slots;

use crate::domain::sexpr::{
    ByteSpan, Delimiter, ExpressionKind, ExpressionView, Path, SymbolName, SyntaxTree,
};

use super::syntax::{atom_text, is_package_head, package_atoms_match};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageOptionSortOrder {
    Canonical,
    Name,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct OptionSortEdit {
    pub(super) defpackage_path: String,
    pub(super) defpackage_span: ByteSpan,
    pub(super) package_name: String,
    pub(super) old_options: Vec<String>,
    pub(super) new_options: Vec<String>,
    pub(super) replacements: Vec<OptionReplacement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct OptionReplacement {
    pub(super) span: ByteSpan,
    pub(super) replacement: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::application::usecase::package) struct OptionSlot {
    pub(in crate::application::usecase::package) span: ByteSpan,
    pub(in crate::application::usecase::package) text: String,
    pub(in crate::application::usecase::package) label: String,
    pub(in crate::application::usecase::package) sort_key: ordering::OptionSortKey,
}

pub(super) fn defpackage_option_sort_edits(
    input: &str,
    tree: &SyntaxTree,
    package: Option<&SymbolName>,
    order: PackageOptionSortOrder,
) -> Result<Vec<OptionSortEdit>> {
    let mut edits = Vec::new();
    let mut matched_defpackages = 0usize;

    for index in 0..tree.root_children().len() {
        let path_indexes = vec![index];
        let path = Path::from_indexes(path_indexes.clone());
        let view = tree.select_path(&path)?.view();
        collect_option_sort_edits(
            input,
            &view,
            path_indexes,
            package,
            order,
            &mut matched_defpackages,
            &mut edits,
        )
        .with_context(|| format!("failed to inspect package form at {path}"))?;
    }

    if matched_defpackages == 0
        && let Some(target) = package
    {
        anyhow::bail!("no matching defpackage form found for {target}");
    }

    Ok(edits)
}

fn collect_option_sort_edits(
    input: &str,
    view: &ExpressionView,
    path_indexes: Vec<usize>,
    package: Option<&SymbolName>,
    order: PackageOptionSortOrder,
    matched_defpackages: &mut usize,
    edits: &mut Vec<OptionSortEdit>,
) -> Result<()> {
    analyze_defpackage_options(
        input,
        view,
        &path_indexes,
        package,
        order,
        matched_defpackages,
        edits,
    )?;

    for (index, child) in view.children.iter().enumerate() {
        let mut child_path = path_indexes.clone();
        child_path.push(index);
        collect_option_sort_edits(
            input,
            child,
            child_path,
            package,
            order,
            matched_defpackages,
            edits,
        )?;
    }

    Ok(())
}

fn analyze_defpackage_options(
    input: &str,
    view: &ExpressionView,
    path_indexes: &[usize],
    package: Option<&SymbolName>,
    order: PackageOptionSortOrder,
    matched_defpackages: &mut usize,
    edits: &mut Vec<OptionSortEdit>,
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
    if !is_package_head(head, "defpackage") {
        return Ok(());
    }

    let Some(package_name) = atom_text(&view.children[1]) else {
        return Ok(());
    };
    if package.is_some_and(|package| !package_atoms_match(package_name, package.as_str())) {
        return Ok(());
    }
    *matched_defpackages += 1;

    if view.children.len() <= 3 {
        return Ok(());
    }

    let slots = slots::collect_option_slots(input, view, path_indexes, order)?;
    let (new_options, replacements) = ordering::sort_slots(&slots);
    let old_options = slots
        .iter()
        .map(|slot| slot.label.clone())
        .collect::<Vec<_>>();

    edits.push(OptionSortEdit {
        defpackage_path: Path::from_indexes(path_indexes.to_vec()).to_string(),
        defpackage_span: view.span,
        package_name: package_name.to_owned(),
        old_options,
        new_options,
        replacements,
    });

    Ok(())
}
