use super::*;

#[test]
fn sorts_package_exports_without_moving_other_options() {
    let input =
        "(defpackage #:demo\n  (:use #:cl)\n  (:export #:z #:a #:m)\n  (:import-from #:x #:y))\n";
    let plan = plan_sort_package_exports(SortPackageExportsRequest {
        input,
        package: Some(&SymbolName::new("demo").unwrap()),
    })
    .unwrap();

    assert!(plan.changed);
    assert_eq!(plan.exports.len(), 1);
    assert_eq!(plan.exports[0].old_symbols, ["#:z", "#:a", "#:m"]);
    assert_eq!(plan.exports[0].new_symbols, ["#:a", "#:m", "#:z"]);
    assert!(plan.rewritten.contains("(:export #:a #:m #:z)"));
    assert!(plan.rewritten.contains("(:import-from #:x #:y)"));
    SyntaxTree::parse(&plan.rewritten).unwrap();
}

#[test]
fn sorted_package_exports_are_idempotent() {
    let input = "(defpackage #:demo (:export #:a #:b #:c))\n";
    let plan = plan_sort_package_exports(SortPackageExportsRequest {
        input,
        package: None,
    })
    .unwrap();

    assert!(!plan.changed);
    assert_eq!(plan.rewritten, input);
    assert_eq!(plan.exports[0].new_symbols, ["#:a", "#:b", "#:c"]);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    #[test]
    fn pbt_sort_package_exports_output_remains_parseable_and_ordered(
        package in package_name_strategy(),
        mut symbols in prop::collection::vec(symbol_strategy(), 2..8),
    ) {
        symbols.sort();
        symbols.dedup();
        prop_assume!(symbols.len() >= 2);
        let reversed = symbols.iter().rev().map(|symbol| format!("#:{symbol}")).collect::<Vec<_>>();
        let input = format!("(defpackage #:{package} (:export {}))\n", reversed.join(" "));
        let plan = plan_sort_package_exports(SortPackageExportsRequest {
            input: &input,
            package: None,
        }).unwrap();
        let expected = symbols.iter().map(|symbol| format!("#:{symbol}")).collect::<Vec<_>>();

        SyntaxTree::parse(&plan.rewritten).unwrap();
        prop_assert_eq!(plan.exports.len(), 1);
        prop_assert_eq!(&plan.exports[0].new_symbols, &expected);
    }
}
