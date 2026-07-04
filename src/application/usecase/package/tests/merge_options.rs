use super::*;

#[test]
fn merges_duplicate_package_export_options() {
    let input = "(defpackage #:demo\n  (:use #:cl)\n  (:export #:b #:a)\n  (:export #:a #:c))\n";
    let plan = plan_merge_package_options(MergePackageOptionsRequest {
        input,
        package: Some(&SymbolName::new("demo").unwrap()),
    })
    .unwrap();

    assert!(plan.changed);
    assert_eq!(plan.merges.len(), 1);
    assert_eq!(plan.merges[0].head, ":export");
    assert_eq!(plan.merges[0].old_atoms, ["#:b", "#:a", "#:a", "#:c"]);
    assert_eq!(plan.merges[0].new_atoms, ["#:b", "#:a", "#:c"]);
    assert!(plan.rewritten.contains("(:export #:b #:a #:c)"));
    SyntaxTree::parse(&plan.rewritten).unwrap();
}

#[test]
fn merges_import_from_options_only_by_source_package() {
    let input = "(defpackage #:demo\n  (:import-from #:dep #:a)\n  (:import-from #:other #:b)\n  (:import-from #:dep #:a #:c))\n";
    let plan = plan_merge_package_options(MergePackageOptionsRequest {
        input,
        package: None,
    })
    .unwrap();

    assert!(plan.changed);
    assert_eq!(plan.merges.len(), 1);
    assert_eq!(plan.merges[0].head, ":import-from");
    assert_eq!(plan.merges[0].key.as_deref(), Some("dep"));
    assert_eq!(plan.merges[0].new_atoms, ["#:dep", "#:a", "#:c"]);
    assert!(plan.rewritten.contains("(:import-from #:dep #:a #:c)"));
    assert!(plan.rewritten.contains("(:import-from #:other #:b)"));
    SyntaxTree::parse(&plan.rewritten).unwrap();
}

#[test]
fn merged_package_options_are_idempotent() {
    let input = "(defpackage #:demo (:use #:cl) (:export #:a #:b))\n";
    let plan = plan_merge_package_options(MergePackageOptionsRequest {
        input,
        package: None,
    })
    .unwrap();

    assert!(!plan.changed);
    assert_eq!(plan.merges.len(), 0);
    assert_eq!(plan.rewritten, input);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn pbt_merge_package_options_output_remains_parseable_and_deduplicated(
        package in package_name_strategy(),
        mut symbols in prop::collection::vec(symbol_strategy(), 2..8),
    ) {
        symbols.sort();
        symbols.dedup();
        prop_assume!(symbols.len() >= 2);
        let left = symbols.iter().map(|symbol| format!("#:{symbol}")).collect::<Vec<_>>();
        let right = symbols.iter().rev().map(|symbol| format!("#:{symbol}")).collect::<Vec<_>>();
        let input = format!(
            "(defpackage #:{package} (:export {}) (:export {}))\n",
            left.join(" "),
            right.join(" ")
        );
        let plan = plan_merge_package_options(MergePackageOptionsRequest {
            input: &input,
            package: None,
        }).unwrap();
        let expected = left;

        SyntaxTree::parse(&plan.rewritten).unwrap();
        prop_assert!(plan.changed);
        prop_assert_eq!(plan.merges.len(), 1);
        prop_assert_eq!(&plan.merges[0].new_atoms, &expected);
    }
}
