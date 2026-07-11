use anyhow::{Context, Result};

mod merge;
mod slots;

use crate::domain::{
    common_lisp::CommonLispPackageDeclarationForm,
    dialect::Dialect,
    sexpr::{ByteSpan, Delimiter, ExpressionKind, ExpressionView, Path, SymbolName, SyntaxTree},
};

use super::syntax::{atom_text, is_package_head, package_atoms_match};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct OptionMergeEdit {
    pub(super) defpackage_path: String,
    pub(super) defpackage_span: ByteSpan,
    pub(super) package_name: String,
    pub(super) merges: Vec<OptionMerge>,
    pub(super) replacements: Vec<OptionReplacement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct OptionMerge {
    pub(super) head: String,
    pub(super) key: Option<String>,
    pub(super) kept_path: String,
    pub(super) kept_span: ByteSpan,
    pub(super) removed_paths: Vec<String>,
    pub(super) removed_spans: Vec<ByteSpan>,
    pub(super) old_atoms: Vec<String>,
    pub(super) new_atoms: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct OptionReplacement {
    pub(super) span: ByteSpan,
    pub(super) replacement: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::application::usecase::package) struct OptionSlot {
    pub(in crate::application::usecase::package) path: String,
    pub(in crate::application::usecase::package) span: ByteSpan,
    pub(in crate::application::usecase::package) head_text: String,
    pub(in crate::application::usecase::package) name: String,
    pub(in crate::application::usecase::package) key: Option<String>,
    pub(in crate::application::usecase::package) body_atoms: Vec<String>,
}

pub(super) fn defpackage_option_merge_edits(
    input: &str,
    tree: &SyntaxTree,
    dialect: Dialect,
    package: Option<&SymbolName>,
) -> Result<Vec<OptionMergeEdit>> {
    let mut edits = Vec::new();
    let mut matched_defpackages = 0usize;

    for index in 0..tree.root_children().len() {
        let path = Path::root_child(index);
        let view = tree.select_path(&path)?.view();
        collect_option_merge_edits(
            tree,
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

    let _ = input;
    Ok(edits)
}

fn collect_option_merge_edits(
    tree: &SyntaxTree,
    view: &ExpressionView,
    path: Path,
    dialect: Dialect,
    package: Option<&SymbolName>,
    matched_defpackages: &mut usize,
    edits: &mut Vec<OptionMergeEdit>,
) -> Result<()> {
    analyze_defpackage_options(
        tree,
        view,
        &path,
        dialect,
        package,
        matched_defpackages,
        edits,
    )?;

    for (index, child) in view.children.iter().enumerate() {
        collect_option_merge_edits(
            tree,
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

fn analyze_defpackage_options(
    tree: &SyntaxTree,
    view: &ExpressionView,
    path: &Path,
    dialect: Dialect,
    package: Option<&SymbolName>,
    matched_defpackages: &mut usize,
    edits: &mut Vec<OptionMergeEdit>,
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

    if view.children.len() <= 3 {
        return Ok(());
    }

    let slots = slots::collect_option_slots(view, path)?;
    let (merges, replacements) = merge::merge_slots(&slots, tree);
    if merges.is_empty() {
        return Ok(());
    }

    edits.push(OptionMergeEdit {
        defpackage_path: path.to_string(),
        defpackage_span: view.span,
        package_name: package_name.to_owned(),
        merges,
        replacements,
    });

    Ok(())
}
