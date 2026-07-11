use anyhow::{Context, Result};

use crate::application::usecase::package_report::syntax::{
    atom_text, is_package_head, package_option_atoms, package_option_name,
};
use crate::application::usecase::package_report::types::{
    InPackageReport, PackageDefinitionReport, PackageImportReport,
};
use crate::domain::{
    common_lisp::CommonLispPackageDeclarationForm,
    dialect::Dialect,
    sexpr::{Delimiter, ExpressionKind, ExpressionView, Path},
};

pub(super) fn analyze_defpackage_form(
    view: &ExpressionView,
    dialect: Dialect,
    path: &Path,
) -> Result<Option<PackageDefinitionReport>> {
    if view.kind != ExpressionKind::List || view.delimiter != Some(Delimiter::Paren) {
        return Ok(None);
    }
    if view.children.len() < 2 {
        return Ok(None);
    }
    let Some(head) = atom_text(&view.children[0]) else {
        return Ok(None);
    };
    if !is_package_head(dialect, head, CommonLispPackageDeclarationForm::Defpackage) {
        return Ok(None);
    }

    let name = atom_text(&view.children[1])
        .context("defpackage package name must be an atom")?
        .to_owned();
    let mut nicknames = Vec::new();
    let mut uses = Vec::new();
    let mut exports = Vec::new();
    let mut imports = Vec::new();
    let mut option_count = 0;

    for option in view.children.iter().skip(2) {
        if option.kind != ExpressionKind::List || option.children.is_empty() {
            continue;
        }
        let Some(option_head) = atom_text(&option.children[0]) else {
            continue;
        };
        option_count += 1;
        match package_option_name(option_head).as_str() {
            "nicknames" => nicknames.extend(package_option_atoms(option).skip(1)),
            "use" => uses.extend(package_option_atoms(option).skip(1)),
            "export" => exports.extend(package_option_atoms(option).skip(1)),
            "import-from" => {
                let mut atoms = package_option_atoms(option).skip(1);
                if let Some(package) = atoms.next() {
                    imports.push(PackageImportReport {
                        package,
                        symbols: atoms.collect(),
                    });
                }
            }
            _ => {}
        }
    }

    Ok(Some(PackageDefinitionReport {
        path: path.to_string(),
        span: view.span,
        name,
        nicknames,
        uses,
        exports,
        imports,
        option_count,
    }))
}

pub(super) fn analyze_in_package_form(
    view: &ExpressionView,
    dialect: Dialect,
    path: &Path,
) -> Result<Option<InPackageReport>> {
    if view.kind != ExpressionKind::List || view.delimiter != Some(Delimiter::Paren) {
        return Ok(None);
    }
    if view.children.len() < 2 {
        return Ok(None);
    }
    let Some(head) = atom_text(&view.children[0]) else {
        return Ok(None);
    };
    if !is_package_head(dialect, head, CommonLispPackageDeclarationForm::InPackage) {
        return Ok(None);
    }

    let name = atom_text(&view.children[1])
        .context("in-package package name must be an atom")?
        .to_owned();

    Ok(Some(InPackageReport {
        path: path.to_string(),
        span: view.span,
        name,
    }))
}
