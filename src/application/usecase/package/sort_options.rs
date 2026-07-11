use anyhow::{Context, Result};

mod ordering;
mod slots;

use crate::domain::{
    common_lisp::CommonLispPackageDeclarationForm,
    dialect::Dialect,
    sexpr::{ByteSpan, Delimiter, ExpressionKind, ExpressionView, Path, SymbolName, SyntaxTree},
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

/// `full_span`/`full_text` span from the newline that ends the previous
/// option's line up to this option's own end, so a leading `;;` comment (or
/// blank run) travels with the option below it when options are reordered.
/// The first option in a `defpackage` has no previous option to inherit
/// trivia from, so its slot starts right after the package name and
/// `has_leading_trivia` is `false`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::application::usecase::package) struct OptionSlot {
    pub(in crate::application::usecase::package) full_span: ByteSpan,
    pub(in crate::application::usecase::package) full_text: String,
    pub(in crate::application::usecase::package) has_leading_trivia: bool,
    pub(in crate::application::usecase::package) label: String,
    pub(in crate::application::usecase::package) sort_key: ordering::OptionSortKey,
}

pub(super) fn defpackage_option_sort_edits(
    input: &str,
    tree: &SyntaxTree,
    dialect: Dialect,
    package: Option<&SymbolName>,
    order: PackageOptionSortOrder,
) -> Result<Vec<OptionSortEdit>> {
    let mut traversal = OptionSortTraversal {
        input,
        dialect,
        package,
        order,
        matched_defpackages: 0,
        edits: Vec::new(),
    };

    for index in 0..tree.root_children().len() {
        let path = Path::root_child(index);
        let view = tree.select_path(&path)?.view();
        collect_option_sort_edits(&mut traversal, &view, path.clone())
            .with_context(|| format!("failed to inspect package form at {path}"))?;
    }

    if traversal.matched_defpackages == 0 {
        if let Some(target) = package {
            anyhow::bail!("no matching defpackage form found for {target}");
        }
    }

    Ok(traversal.edits)
}

struct OptionSortTraversal<'a> {
    input: &'a str,
    dialect: Dialect,
    package: Option<&'a SymbolName>,
    order: PackageOptionSortOrder,
    matched_defpackages: usize,
    edits: Vec<OptionSortEdit>,
}

fn collect_option_sort_edits(
    traversal: &mut OptionSortTraversal<'_>,
    view: &ExpressionView,
    path: Path,
) -> Result<()> {
    analyze_defpackage_options(traversal, view, &path)?;

    for (index, child) in view.children.iter().enumerate() {
        collect_option_sort_edits(traversal, child, path.child(index))?;
    }

    Ok(())
}

fn analyze_defpackage_options(
    traversal: &mut OptionSortTraversal<'_>,
    view: &ExpressionView,
    path: &Path,
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
    if !is_package_head(
        traversal.dialect,
        head,
        CommonLispPackageDeclarationForm::Defpackage,
    ) {
        return Ok(());
    }

    let Some(package_name) = atom_text(&view.children[1]) else {
        return Ok(());
    };
    if traversal
        .package
        .is_some_and(|package| !package_atoms_match(package_name, package.as_str()))
    {
        return Ok(());
    }
    traversal.matched_defpackages += 1;

    if view.children.len() <= 3 {
        return Ok(());
    }

    let slots = slots::collect_option_slots(traversal.input, view, path, traversal.order)?;
    let (new_options, replacements) = ordering::sort_slots(&slots);
    let old_options = slots
        .iter()
        .map(|slot| slot.label.clone())
        .collect::<Vec<_>>();

    traversal.edits.push(OptionSortEdit {
        defpackage_path: path.to_string(),
        defpackage_span: view.span,
        package_name: package_name.to_owned(),
        old_options,
        new_options,
        replacements,
    });

    Ok(())
}
