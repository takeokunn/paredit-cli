use proptest::prelude::*;

use super::*;

fn report(input: &str) -> DependencyReport {
    let tree = SyntaxTree::parse(input).expect("parse fixture");
    build_dependency_report(&tree).expect("build dependency report")
}

fn contains_dependency(report: &DependencyReport, kind: DependencyKind, target: &str) -> bool {
    report
        .dependencies
        .iter()
        .any(|dependency| dependency.kind == kind && dependency.target == target)
}

#[test]
fn collects_runtime_asdf_and_qualified_symbol_dependencies() {
    let report = report(
        r#"(asdf:defsystem #:demo
  :depends-on (#:alexandria "cl-ppcre")
  :components ((:file "package") (:module "src" :components ((:file "core")))))
(require :swank)
(provide 'demo.core)
(load "extra.lisp")
(use-package #:alexandria)
(import 'uiop:ensure-directory-pathname)
(defun render ()
  (alexandria:when-let ((x 1))
    (uiop:ensure-directory-pathname x)))"#,
    );

    assert!(contains_dependency(
        &report,
        DependencyKind::AsdfDependsOn,
        "\"cl-ppcre\""
    ));
    assert!(contains_dependency(
        &report,
        DependencyKind::AsdfComponent,
        ":file \"package\""
    ));
    assert!(contains_dependency(
        &report,
        DependencyKind::Require,
        ":swank"
    ));
    assert!(contains_dependency(
        &report,
        DependencyKind::UsePackage,
        "#:alexandria"
    ));
    assert!(contains_dependency(
        &report,
        DependencyKind::QualifiedSymbol,
        "alexandria"
    ));
    assert!(contains_dependency(
        &report,
        DependencyKind::QualifiedSymbol,
        "uiop"
    ));
}

#[test]
fn includes_defpackage_dependencies_from_package_report() {
    let report = report(
        r#"(defpackage #:demo.core
  (:use #:cl #:alexandria)
  (:import-from #:uiop #:pathname-directory-pathname))"#,
    );

    assert!(contains_dependency(
        &report,
        DependencyKind::DefpackageUse,
        "#:alexandria"
    ));
    assert!(contains_dependency(
        &report,
        DependencyKind::DefpackageImportFrom,
        "#:uiop"
    ));
}

proptest! {
    #[test]
    fn pbt_collects_package_prefix_from_qualified_symbols(
        package in "[a-z][a-z0-9-]{0,12}",
        symbol in "[a-z][a-z0-9-]{0,12}",
        internal in any::<bool>(),
    ) {
        let separator = if internal { "::" } else { ":" };
        let input = format!("({package}{separator}{symbol} 1)");
        let report = report(&input);
        let matches = report
            .dependencies
            .iter()
            .filter(|dependency| {
                dependency.kind == DependencyKind::QualifiedSymbol
                    && dependency.target == package
                    && dependency.source.as_deref() == Some(format!("{package}{separator}{symbol}").as_str())
            })
            .count();

        prop_assert_eq!(matches, 1);
    }
}
