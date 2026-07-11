use super::*;

proptest! {
    #![proptest_config(cli_proptest_config(24))]

    #[test]
    fn pbt_cli_sort_package_exports_output_remains_parseable_and_ordered(
        mut symbols in prop::collection::vec("[a-z][a-z0-9-]{0,8}", 2..8),
    ) {
        symbols.sort();
        symbols.dedup();
        prop_assume!(symbols.len() >= 2);

        let dir = fresh_temp_dir("sort-package-exports-pbt");
        let package_file = dir.join("package.lisp");
        let reversed = symbols.iter().rev().map(|symbol| format!("#:{symbol}")).collect::<Vec<_>>();
        fs::write(
            &package_file,
            format!("(defpackage #:demo (:export {}))\n", reversed.join(" ")),
        )
        .expect("write package fixture");

        let mut cmd = paredit();
        cmd.arg("sort-package-exports")
            .arg("--file")
            .arg(&package_file)
            .arg("--write")
            .assert()
            .success();

        let rewritten = fs::read_to_string(&package_file).expect("read rewritten package");
        let expected = symbols.iter().map(|symbol| format!("#:{symbol}")).collect::<Vec<_>>();
        let expected_export = format!("(:export {})", expected.join(" "));
        prop_assert!(rewritten.contains(&expected_export));

        let mut check = paredit();
        check.arg("check")
            .arg("--file")
            .arg(&package_file)
            .assert()
            .success();
    }

    #[test]
    fn pbt_cli_sort_package_options_output_remains_parseable_and_ordered(
        mut option_indexes in prop::collection::vec(0usize..6, 2..6),
    ) {
        option_indexes.sort();
        option_indexes.dedup();
        prop_assume!(option_indexes.len() >= 2);

        let dir = fresh_temp_dir("sort-package-options-pbt");
        let package_file = dir.join("package.lisp");
        let reversed_options = option_indexes.iter().rev().map(|index| cli_option_fixture(*index)).collect::<Vec<_>>();
        fs::write(
            &package_file,
            format!("(defpackage #:demo {})\n", reversed_options.join(" ")),
        )
        .expect("write package fixture");

        let mut cmd = paredit();
        cmd.arg("sort-package-options")
            .arg("--file")
            .arg(&package_file)
            .arg("--write")
            .assert()
            .success();

        let rewritten = fs::read_to_string(&package_file).expect("read rewritten package");
        let expected_options = option_indexes.iter().map(|index| cli_option_fixture(*index)).collect::<Vec<_>>();
        assert_substrings_in_order(&rewritten, &expected_options);

        let mut check = paredit();
        check.arg("check")
            .arg("--file")
            .arg(&package_file)
            .assert()
            .success();
    }

    #[test]
    fn pbt_cli_merge_package_options_output_remains_parseable_and_deduplicated(
        mut symbols in prop::collection::vec("[a-z][a-z0-9-]{0,8}", 2..8),
    ) {
        symbols.sort();
        symbols.dedup();
        prop_assume!(symbols.len() >= 2);

        let dir = fresh_temp_dir("merge-package-options-pbt");
        let package_file = dir.join("package.lisp");
        let left = symbols.iter().map(|symbol| format!("#:{symbol}")).collect::<Vec<_>>();
        let right = symbols.iter().rev().map(|symbol| format!("#:{symbol}")).collect::<Vec<_>>();
        fs::write(
            &package_file,
            format!(
                "(defpackage #:demo (:export {}) (:export {}))\n",
                left.join(" "),
                right.join(" ")
            ),
        )
        .expect("write package fixture");

        let mut cmd = paredit();
        cmd.arg("merge-package-options")
            .arg("--file")
            .arg(&package_file)
            .arg("--write")
            .assert()
            .success();

        let rewritten = fs::read_to_string(&package_file).expect("read rewritten package");
        let expected_export = format!("(:export {})", left.join(" "));
        prop_assert!(rewritten.contains(&expected_export));

        let mut check = paredit();
        check.arg("check")
            .arg("--file")
            .arg(&package_file)
            .assert()
            .success();
    }
}
