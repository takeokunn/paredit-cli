use anyhow::Result;

use crate::application::package_report::analyze::{
    analyze_defpackage_form, analyze_in_package_form,
};
use crate::application::package_report::types::{
    InPackageReport, PackageDefinitionReport, PackageReport,
};
use crate::domain::sexpr::{ExpressionView, Path, SyntaxTree};

pub fn build_package_report(tree: &SyntaxTree) -> Result<PackageReport> {
    let mut defpackages = Vec::new();
    let mut in_packages = Vec::new();

    for index in 0..tree.root_children().len() {
        let path_indexes = vec![index];
        let path = Path::from_indexes(path_indexes.clone());
        let view = tree.select_path(&path)?.view();
        collect_package_forms_from_view(&view, path_indexes, &mut defpackages, &mut in_packages)?;
    }

    Ok(PackageReport {
        defpackages,
        in_packages,
    })
}

fn collect_package_forms_from_view(
    view: &ExpressionView,
    path_indexes: Vec<usize>,
    defpackages: &mut Vec<PackageDefinitionReport>,
    in_packages: &mut Vec<InPackageReport>,
) -> Result<()> {
    if let Some(defpackage) = analyze_defpackage_form(view, &path_indexes)? {
        defpackages.push(defpackage);
    }
    if let Some(in_package) = analyze_in_package_form(view, &path_indexes)? {
        in_packages.push(in_package);
    }

    for (index, child) in view.children.iter().enumerate() {
        let mut child_path = path_indexes.clone();
        child_path.push(index);
        collect_package_forms_from_view(child, child_path, defpackages, in_packages)?;
    }

    Ok(())
}
