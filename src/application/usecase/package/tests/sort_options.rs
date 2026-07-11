use super::*;

#[test]
fn sorts_package_options_in_canonical_order_without_reformatting_bodies() {
    let input = "(defpackage #:demo\n  (:export #:z #:a)\n  (:import-from #:dep #:b #:a)\n  (:use #:cl)\n  (:local-nicknames (#:json #:jonathan)))\n";
    let plan = plan_sort_package_options(SortPackageOptionsRequest {
        input,
        dialect: Dialect::CommonLisp,
        package: Some(&SymbolName::new("demo").unwrap()),
        order: PackageOptionSortOrder::Canonical,
    })
    .unwrap();

    assert!(plan.changed);
    assert_eq!(plan.packages.len(), 1);
    assert_eq!(
        plan.packages[0].old_options,
        [
            ":export #:z",
            ":import-from #:dep",
            ":use #:cl",
            ":local-nicknames"
        ]
    );
    assert_eq!(
        plan.packages[0].new_options,
        [
            ":use #:cl",
            ":import-from #:dep",
            ":local-nicknames",
            ":export #:z"
        ]
    );
    assert!(plan.rewritten.contains("(:import-from #:dep #:b #:a)"));
    assert_ordered(
        &plan.rewritten,
        &[
            "(:use #:cl)",
            "(:import-from #:dep #:b #:a)",
            "(:local-nicknames (#:json #:jonathan))",
            "(:export #:z #:a)",
        ],
    );
    SyntaxTree::parse(&plan.rewritten).unwrap();
}

#[test]
fn sorts_package_options_by_name_when_requested() {
    let input =
        "(defpackage #:demo\n  (:use #:cl)\n  (:export #:main)\n  (:documentation \"demo\"))\n";
    let plan = plan_sort_package_options(SortPackageOptionsRequest {
        input,
        dialect: Dialect::CommonLisp,
        package: None,
        order: PackageOptionSortOrder::Name,
    })
    .unwrap();

    assert!(plan.changed);
    assert_ordered(
        &plan.rewritten,
        &[
            "(:documentation \"demo\")",
            "(:export #:main)",
            "(:use #:cl)",
        ],
    );
}

#[test]
fn sorted_package_options_are_idempotent() {
    let input = "(defpackage #:demo (:use #:cl) (:import-from #:dep #:x) (:export #:main))\n";
    let plan = plan_sort_package_options(SortPackageOptionsRequest {
        input,
        dialect: Dialect::CommonLisp,
        package: None,
        order: PackageOptionSortOrder::Canonical,
    })
    .unwrap();

    assert!(!plan.changed);
    assert_eq!(plan.rewritten, input);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn pbt_sort_package_options_output_remains_parseable_and_ordered(
        package in package_name_strategy(),
        mut option_indexes in prop::collection::vec(0usize..6, 2..6),
    ) {
        option_indexes.sort();
        option_indexes.dedup();
        prop_assume!(option_indexes.len() >= 2);
        let reversed_options = option_indexes.iter().rev().map(|index| option_fixture(*index)).collect::<Vec<_>>();
        let input = format!("(defpackage #:{package} {})\n", reversed_options.join(" "));
        let plan = plan_sort_package_options(SortPackageOptionsRequest {
            input: &input,
            dialect: Dialect::CommonLisp,
            package: None,
            order: PackageOptionSortOrder::Canonical,
        }).unwrap();
        let expected_options = option_indexes.iter().map(|index| option_label(*index)).collect::<Vec<_>>();

        SyntaxTree::parse(&plan.rewritten).unwrap();
        prop_assert_eq!(plan.packages.len(), 1);
        prop_assert_eq!(&plan.packages[0].new_options, &expected_options);
    }
}
